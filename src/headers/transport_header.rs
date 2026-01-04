pub trait TransportHeader {
    fn to_byte_array(&self, payload: &[u8]) -> Vec<u8>;
    fn apply_ip_context(&self, _ip_packet: &packet::ip::Packet<&[u8]>, _transport_bytes: &mut [u8]) {}
    fn increment_sequence_number(&mut self) {}
}