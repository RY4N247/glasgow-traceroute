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

// Table II from paper: PROBESTOSEND(n, α) for 95% confidence
// n = number of interfaces expected, value = number of probes k to send
// ie if you have seen n interfaces at a hop send k probes so that you can be 95% confident that you have discovered all interfaces
const STOPPING_POINTS_95: [usize; 18] = [
//  n=0  1   2   3   4   5   6   7   8   9  10  11  12  13  14  15  16  17
    0, 0, 7, 11, 16, 21, 27, 33, 38, 44, 51, 57, 63, 70, 76, 83, 90, 96
];


pub struct HopResult {
    pub ttl: u8,
    pub address: Option<Ipv4Addr>,
    pub rtt: Option<Duration>,
}

/// A single path from source to destination (sequence of interface IPs at each hop)
pub type Path = Vec<Ipv4Addr>;

pub struct Mda {
    destination: Ipv4Addr,
    socket: Socket,
    ipv4_header: Ipv4Header,
    payload_size: usize,

}

impl Mda {
    /// Creates MDA instance. Uses UDP probes (source port = flow id) for L4 multipath hash compatibility.
    pub fn new(destination: Ipv4Addr, timeout_ms: u64, payload_size: usize) -> Self {
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
        }
    }

    // Uses table lookup to determine the number of probes to send based on the number of interfaces observes so far
    pub fn probes_to_send(n: usize) -> usize {
        if n < 2 { 
            1 // Initial probes when expecting 1 interface
        } else if n < STOPPING_POINTS_95.len() { 
            STOPPING_POINTS_95[n] 
        } else { 
            STOPPING_POINTS_95[17] // any more than 17 just do the max
        }
    }
    
    // Hash function that returns a random integer in the range 10,000 and 65,535 given a flow identifier k
    fn select_flow(k: usize) -> u16 {
        return 10000 + ((k % (65535 - 10000 + 1)) as u16)
    }
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


    // Algorithm 2:
    pub fn next_hops(&self, hop: u8, flows_to_r: &Vec<u16>, is_source: bool) 
        -> (HashSet<Ipv4Addr>, HashMap<Ipv4Addr, Vec<u16>>)  // Returns (Ŝr, F_{h,s} for each s)
    {
        println!("  Running Algorithm 2: next_hops");

        
        let mut k: usize = 0;  // k ← 0
        let mut nexthop_interfaces_of_r: HashSet<Ipv4Addr> = HashSet::new();  // Ŝr ← ∅
        let mut flows_to_nexthops: HashMap<Ipv4Addr, Vec<u16>> = HashMap::new();  // F_{h,s}
        
        loop {  // repeat
            let n_guess = nexthop_interfaces_of_r.len() + 1;
            println!("      N guess: {:?}", n_guess);
            // n ← |Ŝr| + 1
            let max = Self::probes_to_send(n_guess);  // max ← PROBESTOSEND(n, α)
            println!("      Max: {:?}", max);
            for i in (k + 1)..=max {
                println!("      Sending probe {}/{}", i, max);
                let flow_id = if i <= flows_to_r.len() {
                    flows_to_r[i - 1]  // Reuse flow that we know reached r (paper uses 1-based k)
                } else if is_source {
                    Self::select_flow(i)  // Source can generate any flow
                } else {
                    // No more known flows for this interface - stop probing
                    k = i;
                    break;
                };
                println!("      Flow id: {:?}", flow_id);
                if let Some(successor) = self.send_probe(hop, flow_id) {  
                    nexthop_interfaces_of_r.insert(successor);  // Ŝr ← Ŝr ∪ {s} // typo in paper saying Ŝr ← Ŝr ∪ {r}
                    flows_to_nexthops.entry(successor).or_default().push(flow_id);  // F_{h,s} ← F_{h,s} ∪ {φ}
                }
                println!("      nexthop_interfaces_of_r: {:?}", nexthop_interfaces_of_r);
                println!("      flows_to_nexthops: {:?}", flows_to_nexthops);
                println!("      k before update: {:?}", k);
                k = i;
                println!("      k: {:?}", k);
            }
                     
            if nexthop_interfaces_of_r.len() < n_guess { 
                println!("      |Ŝr| < n_guess, stopping. |Ŝr|: {:?}, n_guess: {:?}", nexthop_interfaces_of_r.len(), n_guess); // until |Ŝr| < n
                break;
            }
        }

        (nexthop_interfaces_of_r, flows_to_nexthops)  // return Ŝr and F_{h,s}
    }


    // Algorithm 1: F_{h,r} and F_{h-1,r} are not initialised in the paper but are necessary to track flows interface to interface across hops
    pub fn multipath_traceroute(&self, hmin: u8, hmax: u8) -> Vec<Path> {
        let mut previous_interfaces: HashSet<Ipv4Addr> = HashSet::new();
        let fake_interface = Ipv4Addr::new(0, 0, 0, 0);
        previous_interfaces.insert(fake_interface);  //Rhmin−1 ←{0}

        let mut flows_at_hop_h_minus_1: HashMap<Ipv4Addr, BTreeSet<u16>> = HashMap::new(); // F_{h-1,r} ← ∅
        let mut flow_to_path: HashMap<u16, Vec<Ipv4Addr>> = HashMap::new(); // F_{h,s} ← ∅

        for h in hmin..=hmax.saturating_add(1) {
            println!("Hop h={}", h);
            println!("  Previous interfaces: {:?}", previous_interfaces);
            println!("  Flows at hop h-1: {:?}", flows_at_hop_h_minus_1);
            println!("  Flows to path: {:?}", flow_to_path); 
            let mut interfaces_at_hop_h: HashSet<Ipv4Addr> = HashSet::new(); // Rh ← ∅
            let mut flows_at_hop_h: HashMap<Ipv4Addr, BTreeSet<u16>> = HashMap::new(); //F_{h,r} ← ∅

            for r in &previous_interfaces {
                // Get flows that reached r (F_{h-1,r})                     
                let flows_to_r: Vec<u16> = flows_at_hop_h_minus_1.get(r).cloned().unwrap_or_default().into_iter().collect();
                let is_source = *r == fake_interface;  // fake interface can generate any flow
                println!("  Flows to r: {:?}", flows_to_r);
                println!("  Is source: {:?}", is_source);
                // Get next hops and flows to next hops using Algorithm 2
                println!("  Sending probes to next hops, flows to r: {:?}, is source: {:?}, hop: {:?}", flows_to_r, is_source, h);

                let (nexthop_interfaces_of_r, flows_to_nexthops) = self.next_hops(h, &flows_to_r, is_source); 
                println!("  Nexthop interfaces of r: {:?}", nexthop_interfaces_of_r);
                println!("  Flows to nexthops: {:?}", flows_to_nexthops);
                for (s, flows) in flows_to_nexthops {
                    for f in &flows {
                        flow_to_path.entry(*f).or_default().push(s);
                    }
                    flows_at_hop_h.entry(s).or_default().extend(flows);
                }
                println!("  Flows at hop h: {:?}", flows_at_hop_h);

                interfaces_at_hop_h.extend(&nexthop_interfaces_of_r);
                println!("  Interfaces at hop h: {:?}", interfaces_at_hop_h);
            }

            flows_at_hop_h_minus_1 = flows_at_hop_h;
            if interfaces_at_hop_h.is_empty() {
                println!("  Interfaces at hop h are empty, stopping");
                break;
            }
            if interfaces_at_hop_h.contains(&self.destination) {
                println!("  Interfaces at hop h contain destination, stopping");
                break;
            }
            previous_interfaces = interfaces_at_hop_h;
            println!("  Previous interfaces: {:?}", previous_interfaces);
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
        println!("  Paths: {:?}", paths);
        paths


        
    }
        
}