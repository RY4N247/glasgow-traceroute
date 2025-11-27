use crate::headers::ipv4_header::Ipv4Header;
use crate::headers::transport_header::TransportHeader;
pub struct Ipv4Probe{
    pub ip_header: Ipv4Header,
    pub transport_header: Box<dyn TransportHeader>,
    pub payload: Vec<u8>,
}
impl Ipv4Probe {
    pub fn new(ip_header: Ipv4Header, transport_header: Box<dyn TransportHeader>, payload: Vec<u8>) -> Self {
        Self {
            ip_header,
            transport_header,
            payload,
        }
    }
    pub fn to_byte_array(&mut self) -> Vec<u8> {
        let transport_bytes = self.transport_header.to_byte_array(&self.payload);

        self.ip_header.set_total_length_from_payload(transport_bytes.len());

        let mut bytes = self.ip_header.to_byte_array();

        bytes.extend_from_slice(&transport_bytes);

        bytes
    }
}
