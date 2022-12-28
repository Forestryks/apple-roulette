use std::{
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(2);

pub fn check_iphone_sync_port(ip: Ipv4Addr) -> bool {
    let addr = SocketAddr::new(IpAddr::V4(ip), 62078);

    let _stream = match TcpStream::connect_timeout(&addr, CONNECTION_TIMEOUT) {
        Ok(stream) => stream,
        Err(err) => match err.kind() {
            ErrorKind::ConnectionRefused | ErrorKind::TimedOut => return false,
            k => panic!("unhandled error while connecting: {}", k),
        },
    };

    true
}
