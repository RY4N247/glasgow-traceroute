use std::net::Ipv4Addr;
use glasgow_traceroute::applications::ping::Ping;
use glasgow_traceroute::enums::TransportProtocol::{ICMP, UDP};

fn main() {
    let mut ping = Ping::new(UDP,Ipv4Addr::new(8, 8, 8, 8),1000,1);
    ping.send_ping();
}
