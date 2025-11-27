pub trait TransportHeader{
    fn to_byte_array(&self, payload: &[u8]) -> Vec<u8>;
}