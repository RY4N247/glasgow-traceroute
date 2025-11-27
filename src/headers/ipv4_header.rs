//!

use std::net::Ipv4Addr;
use packet::ip::v4::checksum;
use crate::enums::IpFlags::DontFragment;
use crate::enums::{IpFlags, IpProtocol};
use crate::enums::IpProtocol::ICMP;

/// https://networklessons.com/ip-routing/ipv4-packet-header
#[derive(Debug)]
pub struct Ipv4Header {
    pub version : u8,
    pub internet_header_length: u8,
    pub type_of_service: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags: IpFlags,
    pub fragment_offset: u16,
    pub time_to_live: u8,
    pub protocol: IpProtocol,
    pub header_checksum: u16,
    pub source_address: Ipv4Addr,
    pub destination_address:Ipv4Addr
}

impl Ipv4Header {
    pub fn to_byte_array(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::with_capacity(20);

        let version_ihl = (self.version << 4) | (self.internet_header_length & 0x0F);
        buf.push(version_ihl);

        buf.push(self.type_of_service);

        buf.extend_from_slice(&self.total_length.to_be_bytes());

        buf.extend_from_slice(&self.identification.to_be_bytes());

        let flags_and_offset = ((self.flags.to_u8() as u16) << 13) | (self.fragment_offset & 0x1FFF);
        buf.extend_from_slice(&flags_and_offset.to_be_bytes());

        buf.push(self.time_to_live);

        buf.push(self.protocol.to_u8());

        buf.extend_from_slice(&[0, 0]);

        buf.extend_from_slice(&self.source_address.octets());

        buf.extend_from_slice(&self.destination_address.octets());

        let checksum = checksum(&buf[0..20]);

        buf[10] = (checksum >> 8) as u8;
        buf[11] = (checksum & 0xFF) as u8;

        buf
    }
    pub fn set_total_length_from_payload(&mut self, payload_length: usize) {
        self.total_length = (self.internet_header_length as usize * 4 + payload_length) as u16;
    }
}
pub struct Ipv4HeaderBuilder {
    version : u8,
    internet_header_length: u8,
    type_of_service: u8,
    total_length: u16,
    identification: u16,
    flags: IpFlags,
    fragment_offset: u16,
    time_to_live: u8,
    protocol: IpProtocol,
    source_address: Ipv4Addr,
    destination_address:Ipv4Addr
}
impl Ipv4HeaderBuilder {
    pub fn new() -> Self {
        Ipv4HeaderBuilder {
            version: 4,
            internet_header_length: 5,
            type_of_service: 0,
            total_length: 0,
            identification: 1,
            flags: DontFragment, 
            fragment_offset: 0,
            time_to_live: 64,
            protocol: ICMP, 
            source_address: Ipv4Addr::new(0,0,0,0),
            destination_address: Ipv4Addr::new(0,0,0,0)
        }
    }

    pub fn source_address(mut self, src: Ipv4Addr) -> Self {
        self.source_address = src;
        self
    }

    pub fn destination_address(mut self, dst: Ipv4Addr) -> Self {
        self.destination_address = dst;
        self
    }

    pub fn ttl(mut self, ttl: u8) -> Self {
        self.time_to_live = ttl;
        self
    }

    pub fn protocol(mut self, protocol: IpProtocol) -> Self {
        self.protocol = protocol;
        self
    }

    pub fn build(self) -> Ipv4Header {
        Ipv4Header {
            version: self.version,
            internet_header_length: self.internet_header_length,
            type_of_service: self.type_of_service,
            total_length: self.total_length,
            identification: self.identification,
            flags: self.flags,
            fragment_offset: self.fragment_offset,
            time_to_live: self.time_to_live,
            protocol: self.protocol,
            header_checksum: 0,
            source_address: self.source_address,
            destination_address: self.destination_address
        }
    }
}

