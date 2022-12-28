use std::net::Ipv4Addr;

use pnet_datalink::{MacAddr, NetworkInterface};

use crate::{
    arp::{run_arp_scan, ArpResult},
    iphone_sync::check_iphone_sync_port,
    oui::OuiDatabase,
};

#[derive(Debug, PartialEq, Eq)]
pub enum DetectionResult {
    NotApple,
    AppleByMac,
    AppleByOpenPort,
}

fn detect_apple(database: &OuiDatabase, arp_result: &ArpResult) -> DetectionResult {
    let mut result = DetectionResult::NotApple;

    let org = database.get_org(&arp_result.mac);
    if let Some(org) = org {
        if org.starts_with("Apple") {
            result = DetectionResult::AppleByMac;
        }
    }

    if result == DetectionResult::NotApple {
        log::info!("running port check for {}", arp_result.ip);
        if check_iphone_sync_port(arp_result.ip) {
            result = DetectionResult::AppleByOpenPort;
        }
    }

    result
}

pub struct SearchResult {
    pub ip: Ipv4Addr,
    pub mac: MacAddr,
    pub detection_result: DetectionResult,
}

pub fn search_for_apples(
    interface: &NetworkInterface,
    database: &OuiDatabase,
) -> Vec<SearchResult> {
    let arp_results = run_arp_scan(&interface);
    let mut results = Vec::new();

    for arp_result in arp_results {
        let detection_result = detect_apple(&database, &arp_result);

        results.push(SearchResult {
            ip: arp_result.ip,
            mac: arp_result.mac,
            detection_result,
        });
    }

    results
}
