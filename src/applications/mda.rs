//! # MDA application module
//! Implements the MDA traceroute algorithm as described in the paper "Multipath Discovery Algorithm" using a dublin-traceroute style stopping point. 
//! 
//! ## Overview
//! This module builds UDP probe packets encapsulated within IPv4 headers using the [`crate::headers`] module.
//! 
//! ## Design
//! Mda results are returned via the `HopResult` struct, which exposes results
//! in an iterable format enabling flexible client usage. The `Mda` struct
//! encapsulates all necessary state and methods.
//! ### Example
//! Note: Running this example may require elevated privileges due to raw socket usage.
//! - On `Linux` systems, please see the run_raw.sh script in the repository root for guidance.
//! - On `macOS` systems, running sudo cargo run will suffice.
//! - On `Windows` systems, please use WSL 2 as this crate does not currently support native Windows functionality.
//! ```
//!
use std::collections::{BTreeSet, HashMap, HashSet};
use std::mem::MaybeUninit;
use crate::enums::{ByteOrderMode, IpProtocol};
use crate::headers::ipv4_header::{Ipv4Header, Ipv4HeaderBuilder};
use crate::headers::udp_header::UdpHeaderBuilder;
use crate::helpers::packet_parser;
use socket2::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use local_ip_address::local_ip;
use std::time::Duration;

const UDP_INITIAL_DEST_PORT: u16 = 33434;
const UDP_INITIAL_SRC_PORT: u16 = 49152;


/// # HopResult Struct
/// Represents the result of a MDA hop.
/// # Fields
/// - `ttl`: The time to live value of the packet.
/// - `address`: The address of the hop.
/// - `rtt`: The round-trip time of the hop.
pub struct HopResult {
    pub ttl: u8,
    pub address: Option<Ipv4Addr>,
    pub rtt: Option<Duration>,
}

/// # Mda Struct
/// Represents a MDA instance.
/// # Fields
/// - `destination`: The destination IPv4 address.
/// - `socket`: The raw socket used for sending and receiving packets.
/// - `ipv4_header`: The IPv4 header.
/// - `payload_size`: The size of the payload.
/// - `max_probes`: The maximum number of probes to send.
pub struct Mda {
    destination: Ipv4Addr,
    socket: Socket,
    ipv4_header: Ipv4Header,
    payload_size: usize,
    max_probes: usize,

}

impl Mda {
    /// Creates MDA instance. Uses UDP probes (source port = flow id) for L4 multipath hash compatibility.
    /// 
    pub fn new(destination: Ipv4Addr, timeout_ms: u64, payload_size: usize, max_probes: usize) -> Self {
        let user_local_ip: Ipv4Addr = match local_ip().expect("Failed to get local IP") {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => panic!("IPv6 not supported"),
        };
        
        let ipv4_header = Ipv4HeaderBuilder::new()
            .source_address(user_local_ip)
            .destination_address(destination)
            .protocol(IpProtocol::UDP)
            .build();

        let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))
            .expect("Failed to create socket");
        socket.set_header_included_v4(true).expect("Failed to set header included");
        socket.set_read_timeout(Some(Duration::from_millis(timeout_ms))).expect("Failed to set read timeout");

        Self {
            destination,
            socket,
            ipv4_header,
            payload_size,
            max_probes: std::cmp::max(1, max_probes),
        }
    }

    /// Returns the maximum number of probes to send.
    fn probes_to_send(&self) -> usize {
        self.max_probes
    }
    
    /// Selects a flow by returning an integer in the range 10,000 and 65,535 given a flow identifier k
    fn select_flow(k: usize) -> u16 {
        return 10000 + ((k % (65535 - 10000 + 1)) as u16)
    }

    /// Sends a probe to the destination and returns the source address of the response.
    /// 
    /// The method sends a probe to the destination with the given TTL and flow identifier.
    /// The method then waits for a response and returns the source address of the response.
    /// Returns the source address of the response or None if the operation fails or times out.
    fn send_probe(&self, h: u8, phi: u16) -> Option<Ipv4Addr> {
        let payload: Vec<u8> = vec![0u8; self.payload_size];
        let local_ip = self.ipv4_header.source_address;

        // Build IP header with TTL = h
        let mut ip_header = Ipv4HeaderBuilder::new()
            .source_address(local_ip)
            .destination_address(self.destination)
            .protocol(IpProtocol::UDP)
            .build();
        ip_header.time_to_live = h;

        // φ = source port (flow id), adjust dest port to keep checksum constant
        let transport_bytes = UdpHeaderBuilder::new()
            .source_port(phi)
            .destination_port(UDP_INITIAL_DEST_PORT.wrapping_add(UDP_INITIAL_SRC_PORT.wrapping_sub(phi)))
            .build()
            .to_byte_array(&payload);

        // Compute checksum
        let parser_packet = ip_header.build_packet(&transport_bytes, Some(ByteOrderMode::Network));
        let ip_pkt = match packet::ip::Packet::new(parser_packet.as_slice()) {
            Ok(p) => p,
            Err(_) => return None,
        };
        let mut tbytes = transport_bytes.clone();
        let checksum = packet::udp::checksum(&ip_pkt, &tbytes);
        tbytes[6..8].copy_from_slice(&checksum.to_be_bytes());

        // Send packet (will retransmit on timeout)
        let pkt = ip_header.build_packet(&tbytes, None);
        let expected_destination_port = UDP_INITIAL_DEST_PORT.wrapping_add(UDP_INITIAL_SRC_PORT.wrapping_sub(phi));
        let mut buf: [MaybeUninit<u8>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut consecutive_timeouts = 0u32;

        loop {
            if self.socket.send_to(&pkt, &SocketAddr::from((self.destination, 0)).into()).is_err() {
                return None;
            }

            // Receive loop: match response or retransmit on timeout
            loop {
                match self.socket.recv_from(&mut buf) {
                    Ok((n, _)) => {
                        let recv = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) };
                        if let Some((src_ip, src_port, dst_port)) =
                            packet_parser::extract_udp_ports_from_icmp_error(recv, local_ip)
                        {
                            if src_port == phi && dst_port == expected_destination_port {
                                return Some(src_ip);  // s = responding interface
                            }
                        }
                    }
                    Err(_) => {
                        consecutive_timeouts += 1;
                        if consecutive_timeouts >= 2 {
                            return None;  // No response in 2 consecutive attempts (permanent non-responsiveness)
                        }
                        break;  // Retransmit
                    }
                }
            }
        }
    }


    /// Algorithm 2 of the MDA paper
    /// 
    /// 
    pub fn next_hops(&self, hop: u8, flows_to_r: &Vec<u16>, is_source: bool) 
        -> (HashSet<Ipv4Addr>, HashMap<Ipv4Addr, Vec<u16>>)  // Returns (Ŝr, F_{h,s} for each s)
    {
        let mut k: usize = 0;  // k ← 0
        let mut nexthop_interfaces_of_r: HashSet<Ipv4Addr> = HashSet::new();  // Ŝr ← ∅
        let mut flows_to_nexthops: HashMap<Ipv4Addr, Vec<u16>> = HashMap::new();  // F_{h,s}
        
        loop {  // repeat
            let n_guess = nexthop_interfaces_of_r.len() + 1;

            // n ← |Ŝr| + 1
            let max = self.probes_to_send();  // fixed probe budget per discovery round
 
            for i in (k + 1)..=max {

                let flow_id = if i <= flows_to_r.len() {
                    flows_to_r[i - 1]  // Reuse flow that we know reached r (paper uses 1-based k)
                } else if is_source {
                    Self::select_flow(i)  // Source can generate any flow
                } else {
                    // No more known flows for this interface - stop probing
                    k = i;
                    break;
                };

                if let Some(successor) = self.send_probe(hop, flow_id) {  
                    nexthop_interfaces_of_r.insert(successor);  // Ŝr ← Ŝr ∪ {s} // typo in paper saying Ŝr ← Ŝr ∪ {r}
                    flows_to_nexthops.entry(successor).or_default().push(flow_id);  // F_{h,s} ← F_{h,s} ∪ {φ}
                }

                k = i;

            }
                     
            if nexthop_interfaces_of_r.len() < n_guess { 
 
                break;
            }
        }

        (nexthop_interfaces_of_r, flows_to_nexthops)  // return Ŝr and F_{h,s}
    }


    // Algorithm 1: F_{h,r} and F_{h-1,r} are not initialised in the paper but are necessary to track flows interface to interface across hops
    pub fn multipath_traceroute(&self, hmin: u8, hmax: u8) -> Vec<Vec<Ipv4Addr>> {
        let mut previous_interfaces: HashSet<Ipv4Addr> = HashSet::new();
        let fake_interface = Ipv4Addr::new(0, 0, 0, 0);
        previous_interfaces.insert(fake_interface);  //Rhmin−1 ←{0}

        let mut flows_at_hop_h_minus_1: HashMap<Ipv4Addr, BTreeSet<u16>> = HashMap::new(); // F_{h-1,r} ← ∅
        let mut flow_to_path: HashMap<u16, Vec<Ipv4Addr>> = HashMap::new(); // F_{h,s} ← ∅

        for h in hmin..=hmax.saturating_add(1) {

            let mut interfaces_at_hop_h: HashSet<Ipv4Addr> = HashSet::new(); // Rh ← ∅
            let mut flows_at_hop_h: HashMap<Ipv4Addr, BTreeSet<u16>> = HashMap::new(); //F_{h,r} ← ∅

            for r in &previous_interfaces {
                // Get flows that reached r (F_{h-1,r})                     
                let flows_to_r: Vec<u16> = flows_at_hop_h_minus_1.get(r).cloned().unwrap_or_default().into_iter().collect();
                let is_source = *r == fake_interface;  // fake interface can generate any flow

                let (nexthop_interfaces_of_r, flows_to_nexthops) = self.next_hops(h, &flows_to_r, is_source); 

                for (s, flows) in flows_to_nexthops {
                    for f in &flows {
                        flow_to_path.entry(*f).or_default().push(s);
                    }
                    flows_at_hop_h.entry(s).or_default().extend(flows);
                }


                interfaces_at_hop_h.extend(&nexthop_interfaces_of_r);
     
            }

            flows_at_hop_h_minus_1 = flows_at_hop_h;
            if interfaces_at_hop_h.is_empty() {
  
                break;
            }
            if interfaces_at_hop_h.contains(&self.destination) {

                break;
            }
            previous_interfaces = interfaces_at_hop_h;

        }

        // Keep unique paths that reach destination, flows that dont reach the destination are discarded
        let mut paths = Vec::new();
        let mut seen = HashSet::new();
        for p in flow_to_path.into_values() {
            // If the path reaches the destination and we haven't seen it before, add it to  paths
            if p.last() == Some(&self.destination) && seen.insert(p.clone()) {
                paths.push(p);
            }
        }
        paths.sort();

        paths


        
    }
        
}