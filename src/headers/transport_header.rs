use crate::headers::icmp_header::IcmpHeader;
use crate::headers::udp_header::UdpHeader;

pub enum TransportHeader {
    Icmp(IcmpHeader),
    Udp(UdpHeader),
}

impl TransportHeader {
    pub fn to_byte_array(&self, payload: &[u8]) -> Vec<u8> {
        match self {
            TransportHeader::Icmp(h) => h.to_byte_array(payload),
            TransportHeader::Udp(h) => h.to_byte_array(payload),
        }
    }

    pub fn apply_ip_context(&self, ip_packet: &packet::ip::Packet<&[u8]>, transport_bytes: &mut [u8]) {
        match self {
            // ICMP does not need to apply IP context
            TransportHeader::Icmp(_) => {}
            TransportHeader::Udp(h) => h.apply_ip_context(ip_packet, transport_bytes),
        }
    }

    pub fn increment_sequence_number(&mut self) {
        match self {
            TransportHeader::Icmp(h) => h.increment_sequence_number(),
            // UDP does not need to increment sequence number
            TransportHeader::Udp(_) => {}
        }
    }
}
