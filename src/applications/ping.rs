use std::mem::MaybeUninit;
use crate::enums::TransportProtocol;
use crate::headers::icmp_header::IcmpHeaderBuilder;
use crate::headers::ipv4_header::{Ipv4Header, Ipv4HeaderBuilder};
use crate::headers::transport_header::TransportHeader;
use crate::headers::udp_header::UdpHeaderBuilder;
use socket2::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use local_ip_address::local_ip;
use std::time::{Duration, Instant};

pub struct Ping {
    destination: Ipv4Addr,
    timeout_ms: u64,
    payload_size: usize,
    socket: Socket,
    ipv4_header: Ipv4Header,
    transport_header: Box<dyn TransportHeader>,
}

pub struct PingResult {
    pub destination: Ipv4Addr,
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub rtt: Option<Duration>,
    pub raw_packet: Vec<u8>,
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

    pub fn send_ping(&mut self) -> Result<PingResult, std::io::Error> {
        let payload: Vec<u8> = vec![0u8; self.payload_size];
        let transport_bytes = self.transport_header.to_byte_array(&payload);
        let bytes = self.ipv4_header.build_packet(&transport_bytes);

        let sockaddr = SocketAddr::from((self.destination, 0));

        let start = Instant::now();
        let bytes_sent = self
            .socket
            .send_to(&bytes, &sockaddr.into())?;

        let mut buf: [MaybeUninit<u8>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
        match self.socket.recv_from(&mut buf) {
            Ok((n, _addr)) => {
                let rtt = start.elapsed();
                let packet: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) };
                let raw_packet = packet.to_vec();

                // Build the result and return it for the caller to inspect/parse
                Ok(PingResult {
                    destination: self.destination,
                    bytes_sent,
                    bytes_received: n,
                    rtt: Some(rtt),
                    raw_packet,
                })
            }
            Err(e) => Err(e),
        }
    }
}
