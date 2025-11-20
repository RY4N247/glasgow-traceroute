use socket2::{Socket};
use crate::network::socket_config::SocketConfig;

//Common interface for each protocol
pub trait Probe {
    fn to_byte_array(&self) -> Vec<u8>;
    fn get_socket_config(&self) -> SocketConfig;
    fn send(&mut self, socket: &Socket);
    fn receive(&self, socket: &Socket);
    fn validate_response(&self) -> bool;

}
