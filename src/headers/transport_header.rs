use packet::ip::Packet;
use crate::headers::ipv4_header::Ipv4Header;

pub trait TransportHeader {
    fn to_byte_array(&self, payload: &[u8]) -> Vec<u8>;
    fn apply_ip_context(&self, ip_packet: &packet::ip::Packet<&[u8]>, transport_bytes: & mut [u8]){}
}