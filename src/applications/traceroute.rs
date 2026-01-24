//! # Traceroute application module
//! Implements paris-traceroute functionality using raw IPv4 sockets to send ICMP Echo Requests or UDP packets. Paris Traceroute is a variant of tracerroute
//! where fields are kept constant to avoid anomalies in load balanced networks. To test this please see the example directory.
//!
//! ## Overview
//! This module builds ICMP and UDP probe packets encapsulated
//! within IPv4 headers using the [`crate::headers`] module.
//!
//! ## Design
//! Traceroute results are returned via the `HopResult` struct, which exposes results
//! in an iterable format enabling flexible client usage. The `Traceroute` struct
//! encapsulates all necessary state and methods.
//! ### Example
//! Note: Running this example may require elevated privileges due to raw socket usage.
//! - On `Linux` systems, please see the run_raw.sh script in the repository root for guidance.
//! - On `macOS` systems, running sudo cargo run will suffice.
//! - On `Windows` systems, please use WSL 2 as this crate does not currently support native Windows functionality.
//! ```
//! use std::net::Ipv4Addr;
//! use glasgow_traceroute::applications::traceroute::Traceroute;
//! use glasgow_traceroute::enums::TransportProtocol;
//!
//! fn main() {
//!     let destination = Ipv4Addr::new(8, 8, 8, 8);
//!     let mut traceroute = Traceroute::new(
//!         TransportProtocol::Icmp,
//!         destination,
//!         1000,  // timeout_ms
//!         32,    // payload_size
//!         32,    // max_ttl
//!     );
//!
//!     let results = traceroute.trace_route();
//!     for result in results {
//!         println!("TTL: {}, Address: {:?}, RTT: {:?}", result.ttl, result.address, result.rtt);
//!     }
//! }
//! ```
use std::collections::HashSet;
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

// initial values for ICMP and UDP packets
const ICMP_INITIAL_ID: u16 = 1000; 
const UDP_INITIAL_DEST_PORT: u16 = 33434; 
const UDP_INITIAL_SRC_PORT: u16 = 49152; 

/// # HopResult Struct
/// Represents the result of a traceroute hop.
/// # Fields
/// - `ttl`: The time to live value of the packet.
/// - `address`: The address of the hop.
/// - `rtt`: The round-trip time of the hop.
pub struct HopResult {
    pub ttl: u8,
    pub address: Option<Ipv4Addr>,
    pub rtt: Option<Duration>,
}
pub struct MultipathOUTPUT {
    pub r: u32,
    pub pi_r: bool,
    pub f_h_1_r: Vec<u32>,
}

/// # Traceroute Struct
/// Represents a traceroute operation.
/// # Fields
/// - `destination`: The target IPv4 address to traceroute.
/// - `socket`: The raw socket used for sending and receiving packets.
/// - `ipv4_header`: Encapsulates ICMP/UDP packets providing network layer functionality.
/// - `transport_header`: The transport layer header (technically ICMP is a network layer protocol but included to unify probe construction).
/// - `transport_type`: The transport protocol used (ICMP or UDP).
/// - `payload_size`: The size of the payload in bytes.
/// - `max_ttl`: The maximum time to live value.
pub struct Traceroute {
    destination: Ipv4Addr,
    socket: Socket,
    ipv4_header: Ipv4Header,
    transport_header: Box<dyn TransportHeader>,
    transport_type: TransportProtocol,
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
                        .identifier(ICMP_INITIAL_ID)
                        .build()
                );
            }
            TransportProtocol::Udp => {
                socket_protocol = Some(Protocol::ICMPV4);
                ip_protocol = IpProtocol::UDP;
                transport_header = Box::new(
                    UdpHeaderBuilder::new()
                        .source_port(UDP_INITIAL_SRC_PORT)
                        .destination_port(UDP_INITIAL_DEST_PORT)
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

        let socket = Socket::new(Domain::IPV4, Type::RAW, socket_protocol)
            .expect("Failed to create socket");
        socket.set_header_included_v4(true).expect("Failed to set header included");
        socket.set_read_timeout(Some(Duration::from_millis(timeout_ms))).expect("Failed to set read timeout");

        Self {
            destination,
            socket,
            ipv4_header,
            transport_header,
            transport_type,
            payload_size,
            max_ttl,
        }
    }
    
    pub fn source_address(&self) -> Ipv4Addr {
        self.ipv4_header.source_address
    }
    
    pub fn trace_route(&mut self) -> Vec<HopResult> {
        let mut results: Vec<HopResult> = Vec::new();
        let payload: Vec<u8> = vec![0u8; self.payload_size];
        let local_ip = self.ipv4_header.source_address;

        for ttl in 1..=self.max_ttl {
            self.ipv4_header.time_to_live = ttl;
            self.transport_header.increment_sequence_number();
            
            let transport_bytes = self.transport_header.to_byte_array(&payload);
            
            // Build packet with Network byte order to create IP packet structure for checksum calculation
            let parser_packet = self.ipv4_header.build_packet(&transport_bytes, Some(crate::enums::ByteOrderMode::Network));
            let temp_ip_packet = packet::ip::Packet::new(parser_packet.as_slice()).unwrap();
            
            // Apply IP context for checksum calculation
            let mut transport_bytes_mut = transport_bytes.clone();
            self.transport_header.apply_ip_context(&temp_ip_packet, &mut transport_bytes_mut);
            
            // Build final packet with platform-appropriate byte order for sending
            let packet = self.ipv4_header.build_packet(&transport_bytes_mut, None);
            let sockaddr = SocketAddr::from((self.destination, 0));

            let start = Instant::now();
            if self.socket.send_to(&packet, &sockaddr.into()).is_err() {
                results.push(HopResult { ttl, address: None, rtt: None });
                continue;
            }

            let mut buf: [MaybeUninit<u8>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
            
            loop {
                match self.socket.recv_from(&mut buf) {
                    Ok((n, _)) => {
                        let recv_packet: &[u8] = unsafe { 
                            std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) 
                        };
                        let rtt = start.elapsed();
                        
                        match self.transport_type {
                            TransportProtocol::Udp => {
                                // Expected values: destination port increments, source port decrements
                                let expected_src_port = UDP_INITIAL_SRC_PORT.wrapping_sub(ttl as u16);
                                let expected_dst_port = UDP_INITIAL_DEST_PORT.wrapping_add(ttl as u16);
                                
                                // Check for ICMP Time Exceeded or Destination Unreachable with UDP
                                if let Some((src_ip, recv_src_port, recv_dst_port)) = 
                                    packet_parser::extract_udp_ports_from_icmp_error(recv_packet, local_ip) 
                                {
                                    if recv_src_port == expected_src_port && recv_dst_port == expected_dst_port {
                                        results.push(HopResult { ttl, address: Some(src_ip), rtt: Some(rtt) });
                                     
                                        if src_ip == self.destination {
                                            return results;
                                        }
                                        break;
                                    }
                                }
                            }
                            TransportProtocol::Icmp => {
                                // Expected values: sequence increments, identifier decrements
                                let expected_seq = ttl as u16;
                                let expected_id = ICMP_INITIAL_ID.wrapping_sub(ttl as u16);
                                
                                // Check for ICMP Time Exceeded or Destination Unreachable with ICMP
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



    // MDA IMPLEMENTATION     
    fn next_hops(_r: u32, _h: u8, _alpha: f64) -> HashSet<u32> {
        // TODO: Implement this
        HashSet::new()
    }
    fn per_packet(_r: u32, _h: u8) -> bool {
        // TODO: Implement this
        false
    }
    fn output(x: MultipathOUTPUT) {
        // TODO: Implement this
        println!("r: {}, pi_r: {}, f_h_1_r: {:?}", x.r, x.pi_r, x.f_h_1_r);
    }
    // Symbol    Meaning
    // ----------  ------------------------------------------------------------
    // r, s     responding interface or successor of an interface
    // h        hop (TTL value)
    // α        degree of confidence
    // ^Rh      set of interfaces discovered at distance h from the source
    // ^Sr      set of nexthop interfaces of r
    // φ        flow identifier
    // Fh,r     set of flows traversing r at hop h
    // πr       Boolean indicating if r belongs to a per-packet load balancer
    pub fn multipath_traceroute(&mut self, hmin: u8, hmax: u8, alpha_all: f64) {
        let alpha_nexthops = alpha_all;
        let mut r_prev: HashSet<u32> = [0].into_iter().collect();

        for h in hmin..=hmax.saturating_add(1) {
            let mut r_h: HashSet<u32> = HashSet::new();
            for &r in &r_prev {
                let s_r = Self::next_hops(r, h, alpha_nexthops);
                r_h.extend(&s_r);
                if s_r.len() > 1 {
                    let pi_r = Self::per_packet(r, h);
                    let x = MultipathOUTPUT { r, pi_r, f_h_1_r: vec![0] };
                    Self::output(x);
                }
            }
            // Update R_{h-1} for next h
            r_prev = r_h;
        }
    }








    
}