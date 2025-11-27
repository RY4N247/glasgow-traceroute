//! UDP Header Structure
//! https://www.geeksforgeeks.org/user-datagram-protocol-udp/

use crate::headers::transport_header::TransportHeader;

pub struct UdpHeader {
    source_port: u16,
    destination_port: u16,
    length: u16,
    checksum: u16,
}
impl TransportHeader for UdpHeader {
    fn to_byte_array(&self, payload: &[u8]) -> Vec<u8> {
        let mut buf:Vec<u8> = Vec::with_capacity(8 + payload.len());

        buf.extend_from_slice(&self.source_port.to_be_bytes());
        buf.extend_from_slice(&self.destination_port.to_be_bytes());
        let length = 8 + payload.len();
        buf.extend_from_slice(&(length as u16).to_be_bytes());
        buf.extend_from_slice(&self.checksum.to_be_bytes());
        buf.extend_from_slice(payload);

        buf
    }
}
pub struct UdpHeaderBuilder {
    source_port: u16,
    destination_port: u16,
    length: u16,
    checksum: u16,
}
impl UdpHeaderBuilder {
    pub fn new() -> Self {
        Self {
            // default values
            source_port: 0,
            destination_port: 0,
            length: 0,
            checksum: 0,
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
    pub fn length(mut self, length: u16) -> Self {
        self.length = length;
        self
    }
    pub fn checksum(mut self, checksum: u16) -> Self {
        self.checksum = checksum;
        self
    }
    pub fn build(self) -> UdpHeader {
        UdpHeader {
            source_port: self.source_port,
            destination_port: self.destination_port,
            length: self.length,
            checksum: self.checksum,
        }
    }
}