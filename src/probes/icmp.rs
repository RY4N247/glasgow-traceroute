//! ICMP Probe Implementation
//!
//! This module defines the ICMP probe and its builder, implementing the Probe trait.
use std::net::{Ipv4Addr, SocketAddr};
use socket2::*;
use crate::probes::probe::Probe;
use crate::enums::IcmpType::EchoRequest;
use packet::icmp::checksum;
use crate::network::socket_config::SocketConfig;
use std::mem::MaybeUninit;

pub struct Icmp {
    /// ICMP Type (e.g., Echo Request, Echo Reply)
    pub icmp_type: u8,
    ///Provides further detail about the type, e.g., the reason for a Destination Unreachable.
    pub code: u8,
    /// Checksum (one’s-complement sum of all 16-bit words in the ICMP message)
    pub checksum: u16,
    /// Identifier (used to match requests and replies)
    pub identifier: u16,
    /// Sequence Number (used to match requests and replies)
    pub sequence: u16,
    /// Payload (data carried by the ICMP message)
    pub payload: Vec<u8>,
    /// Destination IP address
    pub destination: Ipv4Addr,
}

impl Probe for Icmp {
    fn to_byte_array(&self) -> Vec<u8> {
        let mut buf:Vec<u8> = Vec::with_capacity(8 + self.payload.len());
        buf.push(self.icmp_type);
        buf.push(self.code);

        // set checksum placeholder
        buf.extend_from_slice(&self.checksum.to_be_bytes());

        // append identifier, sequence number and payload
        buf.extend_from_slice(&self.identifier.to_be_bytes());
        buf.extend_from_slice(&self.sequence.to_be_bytes());
        buf.extend_from_slice(&self.payload);

        let checksum = checksum(&buf);

        buf[2] = (checksum >> 8) as u8;
        buf[3] = (checksum & 0xff) as u8;

        buf
    }

    fn get_socket_config(&self) -> SocketConfig {
        SocketConfig {
            domain: Domain::IPV4,
            sock_type: Type::RAW,
            protocol: Some(Protocol::ICMPV4),
        }
    }

    fn send(&mut self, socket : &Socket) {
        println!("Sending ICMP packet to {}", self.destination);
        let addr = SocketAddr::from((self.destination, 0));
        let packet = self.to_byte_array();
        println!("Packet bytes: {:?}", packet);
        socket.send_to(&packet, &addr.into()).unwrap();
        self.sequence = self.sequence.wrapping_add(1);
    }
    fn receive(&self, socket: &Socket) {
        println!("Receiving ICMP packet...");

        let mut buf: [MaybeUninit<u8>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };

        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                let packet: &[u8] = unsafe {
                    std::slice::from_raw_parts(buf.as_ptr() as *const u8, n)
                };
                println!("Received {} bytes from {:?}", n, addr);
                println!("Packet bytes: {:?}", packet);

                // Create an Icmp instance from the received packet for further processing
                let received_icmp_bytes = &packet[20..28];

                let response = IcmpBuilder::new()
                    .icmp_type(received_icmp_bytes[0])
                    .code(received_icmp_bytes[1])
                    .checksum(u16::from_be_bytes([received_icmp_bytes[2], received_icmp_bytes[3]]))
                    .identifier(u16::from_be_bytes([received_icmp_bytes[4], received_icmp_bytes[5]]))
                    .sequence(u16::from_be_bytes([received_icmp_bytes[6], received_icmp_bytes[7]]))
                    .payload(packet[28..n].to_vec())
                    .build();

                if response.validate_response() {

                } else {

                }

            }
            Err(e) => {
                eprintln!("recv_from failed: {}", e);
            }
        }
    }

    fn validate_response(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct IcmpBuilder {
    icmp_type:u8,
    code:u8,
    checksum:u16,
    identifier:u16,
    sequence:u16,
    payload:Vec<u8>,
    destination: Ipv4Addr,
}

impl IcmpBuilder {
    pub fn new() -> Self {
        IcmpBuilder {
            icmp_type: EchoRequest as u8,
            code: 0,
            checksum: 0,
            identifier: std::process::id() as u16,
            sequence: 0,
            payload: Vec::new(),
            destination: Ipv4Addr::new(8,8,8,8), // default to Google DNS
        }
    }
    pub fn icmp_type(mut self, icmp_type: u8) -> Self {
        self.icmp_type = icmp_type;
        self
    }
    pub fn code(mut self, code: u8) -> Self {
        self.code = code;
        self
    }
    pub fn checksum(mut self, checksum: u16) -> Self {
        self.checksum = checksum;
        self
    }
    pub fn identifier(mut self, identifier: u16) -> Self {
        self.identifier = identifier;
        self
    }
    pub fn sequence(mut self, sequence: u16) -> Self {
        self.sequence = sequence;
        self
    }
    pub fn payload(mut self, payload: Vec<u8>) -> Self {
        self.payload = payload;
        self
    }
    pub fn destination(mut self, destination: Ipv4Addr) -> Self {
        self.destination = destination;
        self
    }
    pub fn build(self) -> Icmp {
        Icmp {
            icmp_type: self.icmp_type,
            code: self.code,
            checksum: self.checksum,
            identifier: self.identifier,
            sequence: self.sequence,
            payload: self.payload,
            destination: self.destination,
        }
    }

}