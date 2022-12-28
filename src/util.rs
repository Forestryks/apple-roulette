use pnet_datalink::NetworkInterface;

pub fn get_default_interface() -> NetworkInterface {
    pnet_datalink::interfaces()
        .into_iter()
        .find(|e| {
            if !e.is_up() || e.is_loopback() || e.mac.is_none() {
                return false;
            }

            e.ips.iter().any(|i| i.is_ipv4())
        })
        .expect("cannot find default interface")
}
