//! UDP Header Structure
//! https://www.geeksforgeeks.org/user-datagram-protocol-udp/

use crate::headers::transport_header::TransportHeader;

pub struct UdpHeader {
    source_port: u16,
    destination_port: u16,
    #[allow(dead_code)] // computed dynamically in to_byte_array()
    length: u16,
    #[allow(dead_code)] // computed dynamically in apply_ip_context()
    checksum: u16,
}
impl TransportHeader for UdpHeader {
    fn to_byte_array(&self, payload: &[u8]) -> Vec<u8> {
        let mut buf:Vec<u8> = Vec::with_capacity(8 + payload.len());

        buf.extend_from_slice(&self.source_port.to_be_bytes());
        buf.extend_from_slice(&self.destination_port.to_be_bytes());
        let length = 8 + payload.len();
        buf.extend_from_slice(&(length as u16).to_be_bytes());
        buf.extend_from_slice(&0u16.to_be_bytes());
        buf.extend_from_slice(payload);

        buf
    }
    fn apply_ip_context(&self, ip_packet: &packet::ip::Packet<&[u8]>, transport_bytes: &mut [u8]) {
        let checksum = packet::udp::checksum(ip_packet, transport_bytes);
        transport_bytes[6..8].copy_from_slice(&checksum.to_be_bytes());
    }
}
pub struct UdpHeaderBuilder {
    source_port: u16,
    destination_port: u16,
}
impl UdpHeaderBuilder {
    pub fn new() -> Self {
        Self {
            source_port: 0,
            destination_port: 0,
        }
    }
    pub fn source_port(mut self, source_port: u16) -> Self {
        self.source_port = source_port;
        self
    }
    pub fn destination_port(mut self, destination_port: u16) -> Self {
        self.destination_port = destination_port;
        self
    }
    pub fn build(self) -> UdpHeader {
        UdpHeader {
            source_port: self.source_port,
            destination_port: self.destination_port,
            length: 0,   // computed dynamically
            checksum: 0, // computed dynamically
        }
    }
}