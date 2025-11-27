use std::mem::MaybeUninit;
use crate::enums::TransportProtocol;
use crate::headers::icmp_header::IcmpHeaderBuilder;
use crate::headers::ipv4_header::{Ipv4Header, Ipv4HeaderBuilder};
use crate::headers::transport_header::TransportHeader;
use crate::headers::udp_header::UdpHeaderBuilder;
use socket2::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use local_ip_address::local_ip;
pub struct Ping {
    destination: Ipv4Addr,
    timeout_ms: u64,
    payload_size: usize,
    socket: Socket,
    ipv4_header: Ipv4Header,
    transport_header: Box<dyn TransportHeader>,
}

impl Ping{
    pub fn new(transport_type: TransportProtocol, destination: Ipv4Addr, timeout_ms: u64, payload_size: usize) -> Self {
        let socket_protocol;
        let ip_protocol;
        let transport_header: Box<dyn TransportHeader>;
        match transport_type {
           TransportProtocol::ICMP => {
               socket_protocol = Some(Protocol::ICMPV4);
               ip_protocol = crate::enums::IpProtocol::ICMP;
               transport_header =
                   Box::new(IcmpHeaderBuilder::new().build()
               );

           }
           TransportProtocol::UDP => {
               socket_protocol = Some(Protocol::UDP);
               ip_protocol = crate::enums::IpProtocol::UDP;
               transport_header = Box::new(UdpHeaderBuilder::new()
                   .source_port(34254)
                   .destination_port(34254)
                   .build()
               );
           }
           _ => {
               panic!("Unsupported transport protocol for Ping");
           }
        }
        let user_local_ip:Ipv4Addr = local_ip().unwrap().to_string().parse().expect("Invalid Ip address");
        println!("user ip {}", user_local_ip);
        let ipv4_header = Ipv4HeaderBuilder::new()
            .source_address(user_local_ip)
            .destination_address(destination)
            .protocol(ip_protocol)
            .build();

        let socket = Socket::new(Domain::IPV4, Type::RAW, socket_protocol).expect("Failed to create socket");
        socket.set_header_included_v4(true).expect("Failed to set header included");
        socket.set_read_timeout(Some(std::time::Duration::from_millis(timeout_ms))).expect("Failed to set read timeout");

        Self {
            destination,
            timeout_ms,
            payload_size,
            socket,
            ipv4_header,
            transport_header
        }
    }

    pub fn send_ping(&mut self) {
        let payload: Vec<u8> = vec![0u8; self.payload_size];
        let transport_bytes = self.transport_header.to_byte_array(&payload);
        println!("Transport Bytes: {:?}",transport_bytes);
        println!("Transport Bytes Length {:?}", transport_bytes.len());

        self.ipv4_header.set_total_length_from_payload(transport_bytes.len());
        println!("ipv4 header length {}", self.ipv4_header.total_length);


        let mut bytes = self.ipv4_header.to_byte_array();

        bytes.extend_from_slice(&transport_bytes);
        println!("sending: {:?}",bytes.len());
        println!("is header included?{:?}",self.socket.header_included_v4());


        let sockaddr = SocketAddr::from((self.destination, 0));
        self.socket
            .send_to(&bytes, &sockaddr.into())
            .expect("Failed to send ping");

        let mut buf: [MaybeUninit<u8>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
        match self.socket.recv_from(&mut buf) {
            Ok((n, _addr)) => {
                let packet: &[u8] = unsafe {
                    std::slice::from_raw_parts(buf.as_ptr() as *const u8, n)
                };
                // n = total packet size (IP + ICMP)
                let icmp_len = n - 20; // subtract IP header
                println!("{} bytes from {}", icmp_len, self.destination);

            }
            Err(e) => {
                print!("recv_from failed: {}", e);
                println!("\x1b[31m ✗ \x1b[0m");
            }
        }


    }
}
