//! # Transport Header Module
//! implements the transport header for the transport protocol.

use crate::headers::icmp_header::IcmpHeader;
use crate::headers::udp_header::UdpHeader;

/// # TransportHeader Enum
/// Represents a transport header.
/// # Fields
/// - `Icmp`: The ICMP header.
/// - `Udp`: The UDP header.
pub enum TransportHeader {
    Icmp(IcmpHeader),
    Udp(UdpHeader),
}

impl TransportHeader {
    /// Converts the tranpsort header and payload into a byte array.
    ///
    /// The method dispatches to the appropriate header to convert the header and payload into a byte array.
    pub fn to_byte_array(&self, payload: &[u8]) -> Vec<u8> {
        match self {
            TransportHeader::Icmp(h) => h.to_byte_array(payload),
            TransportHeader::Udp(h) => h.to_byte_array(payload),
        }
    }

    pub fn apply_ip_context(
        &self,
        ip_packet: &packet::ip::Packet<&[u8]>,
        transport_bytes: &mut [u8],
    ) {
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
