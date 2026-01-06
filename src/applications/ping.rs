//! # Ping application module
//! Implements ping functionality using raw IPv4 sockets to send ICMP Echo Requests or UDP packets.
//!
//! ## Overview
//! This module builds ICMP and UDP probe packets encapsulated
//! within IPv4 headers using the [`crate::headers`] module.
//!
//! ## Design
//! Ping results are returned via the `PingResult` struct, which exposes results
//! in an iterable format enabling flexible client usage. The `Ping` struct
//! encapsulates all necessary state and methods.
//! ### Example
//! Note: Running this example may require elevated privileges due to raw socket usage.
//! - On `Linux` systems, please see the run_raw.sh script in the repository root for guidance.
//! - On `macOS` systems, running sudo cargo run will suffice.
//! - On `Windows` systems, please use WSL 2 as this crate does not currently support native Windows functionality.
//! ```
//! use std::net::Ipv4Addr;
//! use glasgow_traceroute::applications::ping::Ping;
//! use glasgow_traceroute::enums::TransportProtocol;
//!
//! fn main() {
//!     let destination = Ipv4Addr::new(8, 8, 8, 8);
//!     let mut ping = Ping::new(
//!         TransportProtocol::Icmp,
//!         destination,
//!         1000,  // timeout_ms
//!         32,    // payload_size
//!         None,  // port (not needed for ICMP)
//!     );
//!
//!     match ping.send_ping() {
//!         Ok(result) => {
//!             println!("Ping successful!");
//!             println!("Destination: {}", result.destination);
//!             println!("Bytes sent: {}", result.bytes_sent);
//!             println!("Bytes received: {}", result.bytes_received);
//!             if let Some(rtt) = result.rtt {
//!                 println!("RTT: {:?}", rtt);
//!             }
//!         }
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```
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

/// # Ping Struct
/// Represents a Ping operation using either ICMP or UDP transport protocols.
///
/// # Fields
/// - `destination`: The target IPv4 address to ping.
/// - `payload_size`: Size of the payload in bytes.
/// - `socket`: The raw socket used for sending and receiving packets.
/// - `ipv4_header`: Encapsulates ICMP/UDP packets providing network layer functionality.
/// - `transport_header`: The transport layer header (technically ICMP is a network layer protocol but included to unify probe construction).
/// - `transport_type`: The transport protocol used (ICMP or UDP).
/// - `icmp_identifier`: Optional identifier for ICMP packets.
/// - `udp_source_port`: Optional source port for UDP packets.
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
/// # PingResult Struct
/// Represents the result of a Ping to a destination.
/// # Fields
/// - `destination`: The target IPv4 address that was pinged.
/// - `bytes_sent`: Number of bytes sent in the ping request.
/// - `bytes_received`: Number of bytes received in the icmp reply or udp response.
/// - `rtt`: Round-trip time for the ping.
/// - `raw_packet`: The raw bytes of the received packet.
pub struct PingResult {
    pub destination: Ipv4Addr,
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub rtt: Option<Duration>,
    pub raw_packet: Vec<u8>,
}

impl Ping {
    /// Creates a new `Ping` probe and initialises its internal probing state.
    ///
    /// The IPv4 and transport-layer headers are constructed according to
    /// the selected [`crate::enums::TransportProtocol`], establishing the
    /// context required to send probes.
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

    /// Sends a ping request to the destination and waits for a response.
    ///
    /// Constructs a probe packet by incrementing the transport header sequence
    /// number, building the complete IPv4 packet with transport-layer headers,
    /// and sending it to the configured destination. The method then waits for
    /// a matching response packet, calculating the round-trip time upon receipt.
    /// Returns a [`PingResult`] containing the response details or an error if
    /// the operation fails or times out.
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
        // Send the packet to the destination
        let bytes_sent = self
            .socket
            .send_to(&bytes, &sockaddr.into())?;

        // Unsafe buffer as uninitialised memory is required for the recv_from method.
        let mut buf: [MaybeUninit<u8>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };

        // Loop until a matching response packet is received
        loop {
            // Receive a packet from the socket
            match self.socket.recv_from(&mut buf) {
                Ok((n, _addr)) => {
                    let packet: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) };

                    // Check if the packet matches using the packet_parser helper function
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
                    // Return an error if the socket fails to receive a packet
                    return Err(e);
                }
            }
        }
    }
}
