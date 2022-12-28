use std::{
    io::ErrorKind,
    net::Ipv4Addr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use ipnetwork::{IpNetwork, Ipv4Network};
use pnet::packet::{
    arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket},
    ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
    Packet,
};
use pnet_datalink::{Channel, Config, DataLinkReceiver, DataLinkSender, MacAddr, NetworkInterface};

const SOCKET_READ_TIMEOUT: Duration = Duration::from_millis(30);
const SLEEP_BETWEEN_ARP_REQUESTS: Duration = Duration::from_millis(5);
const WAIT_FOR_LAST_ARP_RESPONSES: Duration = Duration::from_secs(2);

fn make_socket(
    interface: &NetworkInterface,
) -> (Box<dyn DataLinkSender>, Box<dyn DataLinkReceiver>) {
    let mut config = Config::default();
    config.read_timeout = Some(SOCKET_READ_TIMEOUT);
    let channel = pnet_datalink::channel(interface, config).expect("cannot create socket");
    let (sender, receiver) = match channel {
        Channel::Ethernet(sender, receiver) => (sender, receiver),
        _ => {
            panic!("Expected Ethernet channel, but got something else");
        }
    };
    (sender, receiver)
}

fn get_interface_ipv4_network(interface: &NetworkInterface) -> Ipv4Network {
    interface
        .ips
        .iter()
        .find_map(|i| match i {
            IpNetwork::V4(v4) => Some(v4.clone()),
            IpNetwork::V6(_) => None,
        })
        .expect("cannot find ipv4 network for default interface")
}

fn build_packet(source_mac: MacAddr, source_ip: Ipv4Addr, target_ip: Ipv4Addr) -> Vec<u8> {
    let target_mac = MacAddr::broadcast();

    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet =
        MutableEthernetPacket::new(&mut ethernet_buffer).expect("cannot allocate ethernet packet");

    ethernet_packet.set_source(source_mac);
    ethernet_packet.set_destination(target_mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet =
        MutableArpPacket::new(&mut arp_buffer).expect("cannot allocate ARP packet");

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(source_mac);
    arp_packet.set_sender_proto_addr(source_ip);
    arp_packet.set_target_hw_addr(target_mac);
    arp_packet.set_target_proto_addr(target_ip);

    ethernet_packet.set_payload(arp_packet.packet());

    ethernet_packet.packet().to_vec()
}

#[derive(Debug)]
pub struct ArpResult {
    pub ip: Ipv4Addr,
    pub mac: MacAddr,
}

fn recv_single_packet(receiver: &mut dyn DataLinkReceiver) -> Option<&[u8]> {
    let packet = match receiver.next() {
        Ok(ok) => ok,
        Err(error) => {
            if error.kind() == ErrorKind::TimedOut {
                return None;
            }

            panic!("cannot receive packet: {}", error);
        }
    };

    Some(packet)
}

fn get_single_arp_response<'a>(receiver: &'a mut dyn DataLinkReceiver) -> Option<ArpResult> {
    let packet_buffer = recv_single_packet(receiver)?;

    let ethernet_packet = EthernetPacket::new(packet_buffer)?;
    if ethernet_packet.get_ethertype() != EtherTypes::Arp {
        return None;
    }

    // let arp_packet = ArpPacket::new(&arp_buffer[MutableEthernetPacket::minimum_packet_size()..]);
    let arp_packet = ArpPacket::new(ethernet_packet.payload())?;

    Some(ArpResult {
        ip: arp_packet.get_sender_proto_addr(),
        mac: arp_packet.get_sender_hw_addr(),
    })
}

fn read_arp_responses(
    receiver: &mut dyn DataLinkReceiver,
    termination_flag: Arc<AtomicBool>,
) -> Vec<ArpResult> {
    let mut results = Vec::new();
    while !termination_flag.load(Ordering::Relaxed) {
        if let Some(arp_result) = get_single_arp_response(receiver) {
            results.push(arp_result);
        }
    }

    results.sort_by_key(|a| a.ip);
    results.dedup_by_key(|a| a.ip);

    results
}

fn send_arp_request(
    sender: &mut dyn DataLinkSender,
    source_mac: MacAddr,
    source_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
) {
    let packet = build_packet(source_mac, source_ip, target_ip);
    sender.send_to(&packet, None);
}

pub fn run_arp_scan(interface: &NetworkInterface) -> Vec<ArpResult> {
    log::info!("running arp scan on {}", interface.name);
    let (mut sender, mut receiver) = make_socket(interface);

    let ipv4_network = get_interface_ipv4_network(interface);
    log::info!("{} ips to scan", ipv4_network.size());

    let source_mac = interface.mac.expect("cannot get MAC of default interface");
    let source_ip = ipv4_network.ip();

    let termination_flag = Arc::new(AtomicBool::new(false));
    let termination_flag_clone = termination_flag.clone();
    let responses_thread =
        std::thread::spawn(move || read_arp_responses(receiver.as_mut(), termination_flag_clone));

    // to allow thread initialize
    std::thread::sleep(Duration::from_millis(100));

    for target_ip in ipv4_network.iter() {
        send_arp_request(sender.as_mut(), source_mac, source_ip, target_ip);
        std::thread::sleep(SLEEP_BETWEEN_ARP_REQUESTS);
    }

    std::thread::sleep(WAIT_FOR_LAST_ARP_RESPONSES);
    termination_flag.store(true, Ordering::Relaxed);

    let results = responses_thread.join().expect("join() failed");
    log::info!("arp scan completed, {} addresses found", results.len());

    results
}
