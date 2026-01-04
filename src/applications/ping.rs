use std::mem::MaybeUninit;
use crate::enums::{TransportProtocol, IpProtocol};
use crate::headers::icmp_header::IcmpHeaderBuilder;
use crate::headers::ipv4_header::{Ipv4Header, Ipv4HeaderBuilder};
use crate::headers::transport_header::TransportHeader;
use crate::headers::udp_header::UdpHeaderBuilder;
use crate::helpers::packet_parser;
use socket2::*;
use std::net::{Ipv4Addr, SocketAddr};
use local_ip_address::local_ip;
use std::time::{Duration, Instant};
use rand::Rng;

pub struct Ping {
    destination: Ipv4Addr,
    payload_size: usize,
    socket: Socket,
    ipv4_header: Ipv4Header,
    transport_header: Box<dyn TransportHeader>,
    transport_type: TransportProtocol,
    icmp_identifier: Option<u16>,  
    udp_source_port: Option<u16>,  
}

pub struct PingResult {
    pub destination: Ipv4Addr,
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub rtt: Option<Duration>,
    pub raw_packet: Vec<u8>,
}

impl Ping {
    pub fn new(transport_type: TransportProtocol, destination: Ipv4Addr, timeout_ms: u64, payload_size: usize, port: Option<u16>) -> Self {
        let socket_protocol;
        let ip_protocol;
        let transport_header: Box<dyn TransportHeader>;
        let (icmp_identifier, udp_source_port);
        
        match transport_type {
           TransportProtocol::Icmp => {
               socket_protocol = Some(Protocol::ICMPV4);
               ip_protocol = IpProtocol::ICMP;
               let identifier = std::process::id() as u16;
               icmp_identifier = Some(identifier);
               udp_source_port = None;
               transport_header =
                   Box::new(IcmpHeaderBuilder::new()
                       .identifier(identifier)
                       .build()
               );

           }
           TransportProtocol::Udp => {
               socket_protocol = Some(Protocol::ICMPV4);
               ip_protocol = crate::enums::IpProtocol::UDP;
               let dest_port = port.unwrap_or(33434);
               let src_port = rand::rng().random_range(49152..65535);
               icmp_identifier = None;
               udp_source_port = Some(src_port);
               transport_header = Box::new(UdpHeaderBuilder::new()
                   .source_port(src_port)
                   .destination_port(dest_port)
                   .build()
               );
           }
        }
        let user_local_ip: Ipv4Addr = match local_ip().expect("Failed to get local IP") {
            std::net::IpAddr::V4(ip) => ip,
            std::net::IpAddr::V6(_) => panic!("IPv6 not supported"),
        };
        let ipv4_header = Ipv4HeaderBuilder::new()
            .source_address(user_local_ip)
            .destination_address(destination)
            .protocol(ip_protocol)
            .build();

        let socket = Socket::new(Domain::IPV4, Type::RAW, socket_protocol).expect("Failed to create socket");
        socket.set_header_included_v4(true).expect("Failed to set header included");
        socket.set_read_timeout(Some(Duration::from_millis(timeout_ms))).expect("Failed to set read timeout");

        Self {
            destination,
            payload_size,
            socket,
            ipv4_header,
            transport_header,
            transport_type,
            icmp_identifier,
            udp_source_port,
        }
    }


    pub fn send_ping(&mut self) -> Result<PingResult, std::io::Error> {
        let payload: Vec<u8> = vec![0u8; self.payload_size];
        self.transport_header.increment_sequence_number();
        let mut transport_bytes = self.transport_header.to_byte_array(&payload);
        
        // Extract expected identifier and sequence number for ICMP (from bytes 4-5 and 6-7)
        let (expected_id_opt, expected_seq_opt) = match self.transport_type {
            TransportProtocol::Icmp => {
                let id = u16::from_be_bytes([transport_bytes[4], transport_bytes[5]]);
                let seq = u16::from_be_bytes([transport_bytes[6], transport_bytes[7]]);
                (Some(id), Some(seq))
            }
            _ => (None, None), // Not used for UDP
        };
        
        let local_ip = self.ipv4_header.source_address;
        
        // Build packet with Network byte order to create IP packet structure for checksum calculation
        let parser_packet = self.ipv4_header.build_packet(&transport_bytes, Some(crate::enums::ByteOrderMode::Network));

        let temp_ip_packet = packet::ip::Packet::new(parser_packet.as_slice()).unwrap();

        // Apply IP context 
        self.transport_header.apply_ip_context(&temp_ip_packet, &mut transport_bytes);

        // Build final packet with platform-appropriate byte order for sending
        let bytes = self.ipv4_header.build_packet(&transport_bytes, None);

        let sockaddr = SocketAddr::from((self.destination, 0));

        let start = Instant::now();
        let bytes_sent = self
            .socket
            .send_to(&bytes, &sockaddr.into())?;

        let mut buf: [MaybeUninit<u8>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((n, _addr)) => {
                    let packet: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) };
                    if packet_parser::packet_matches(
                        packet,
                        &self.transport_type,
                        self.destination,
                        expected_id_opt,
                        expected_seq_opt,
                        self.udp_source_port,
                        local_ip,
                    ) {
                        let rtt = start.elapsed();
                        let raw_packet = packet.to_vec();

                        return Ok(PingResult {
                            destination: self.destination,
                            bytes_sent,
                            bytes_received: n,
                            rtt: Some(rtt),
                            raw_packet,
                        });
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}
