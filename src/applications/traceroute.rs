use std::mem::MaybeUninit;
use crate::enums::{TransportProtocol, IpProtocol};
use crate::headers::icmp_header::IcmpHeaderBuilder;
use crate::headers::ipv4_header::{Ipv4Header, Ipv4HeaderBuilder};
use crate::headers::transport_header::TransportHeader;
use crate::helpers::packet_parser;
use socket2::*;
use std::net::{Ipv4Addr, SocketAddr};
use local_ip_address::local_ip;
use std::time::{Duration, Instant};


const PARIS_ICMP_INITIAL_ID: u16 = 1000;

pub struct HopResult {
    pub ttl: u8,
    pub address: Option<Ipv4Addr>,
    pub rtt: Option<Duration>,
}

pub struct Traceroute {
    destination: Ipv4Addr,
    socket: Socket,
    ipv4_header: Ipv4Header,
    transport_header: Box<dyn TransportHeader>,
    payload_size: usize,
    max_ttl: u8,
}

impl Traceroute{
    pub fn new(transport_type: TransportProtocol, destination: Ipv4Addr, timeout_ms: u64, payload_size: usize, max_ttl: u8) -> Self {
        let socket_protocol;
        let ip_protocol;
        let transport_header: Box<dyn TransportHeader>;
        
        match transport_type {
            TransportProtocol::Icmp => {
                socket_protocol = Some(Protocol::ICMPV4);
                ip_protocol = IpProtocol::ICMP;
                transport_header = Box::new(
                    IcmpHeaderBuilder::new()
                        .identifier(PARIS_ICMP_INITIAL_ID)
                        .build()
                );
            }
            _ => {
                panic!("UDP traceroute not yet implemented");
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

        let socket = Socket::new(Domain::IPV4, Type::RAW, socket_protocol)
            .expect("Failed to create socket");
        socket.set_header_included_v4(true).expect("Failed to set header included");
        socket.set_read_timeout(Some(Duration::from_millis(timeout_ms))).expect("Failed to set read timeout");

        Self {
            destination,
            socket,
            ipv4_header,
            transport_header,
            payload_size,
            max_ttl,
        }
    }
    pub fn trace_route(&mut self) -> Vec<HopResult> {
        let mut results: Vec<HopResult> = Vec::new();
        let payload: Vec<u8> = vec![0u8; self.payload_size];

        for ttl in 1..=self.max_ttl {
            self.ipv4_header.time_to_live = ttl;
            self.transport_header.increment_sequence_number();
            
            let transport_bytes = self.transport_header.to_byte_array(&payload);
            let packet = self.ipv4_header.build_packet(&transport_bytes, None);
            let sockaddr = SocketAddr::from((self.destination, 0));

            let start = Instant::now();
            if self.socket.send_to(&packet, &sockaddr.into()).is_err() {
                results.push(HopResult { ttl, address: None, rtt: None });
                continue;
            }

            let mut buf: [MaybeUninit<u8>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
            
            // Expected values: sequence increments from 1, identifier decrements from initial
            let expected_seq = ttl as u16;
            let expected_id = PARIS_ICMP_INITIAL_ID.wrapping_sub(ttl as u16);
            
            loop {
                match self.socket.recv_from(&mut buf) {
                    Ok((n, _)) => {
                        let recv_packet: &[u8] = unsafe { 
                            std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) 
                        };
                        let rtt = start.elapsed();
                        
                        // Check for ICMP Time Exceeded or Destination Unreachable
                        if let Some((src_ip, recv_id, recv_seq)) = 
                            packet_parser::extract_icmp_identifier_seq_from_icmp_error(recv_packet) 
                        {
                            if recv_id == expected_id && recv_seq == expected_seq {
                                results.push(HopResult { ttl, address: Some(src_ip), rtt: Some(rtt) });
                             
                                if src_ip == self.destination {
                                    return results;
                                }
                                break;
                            }
                        }
                        
                        // Check for ICMP Echo Reply 
                        if let Some((src_ip, recv_id, recv_seq)) = 
                            packet_parser::extract_icmp_identifier_seq(recv_packet) 
                        {
                            if src_ip == self.destination && recv_id == expected_id && recv_seq == expected_seq {
                                results.push(HopResult { ttl, address: Some(src_ip), rtt: Some(rtt) });
                                return results;
                            }
                        }
                    }
                    Err(_) => {
                        results.push(HopResult { ttl, address: None, rtt: None });
                        break;
                    }
                }
            }
        }
        results
    }
}