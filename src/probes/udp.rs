// Udp concrete class and builder

use socket2::Socket;
use crate::network::socket_config::SocketConfig;
use crate::probes::probe::Probe;

pub struct Udp {

}

impl Udp {

}

impl Probe for Udp {
    fn to_byte_array(&self) -> Vec<u8> {
        todo!()
    }

    fn get_socket_config(&self) -> SocketConfig {
        todo!()
    }


    fn send(&mut self, socket: &Socket) {
        todo!()
    }

    fn receive(&self, socket: &Socket) -> bool {
        todo!()
    }

}

pub struct UdpBuilder {

}

impl UdpBuilder {
    pub fn new() -> Self {
        UdpBuilder {

        }
    }

    pub fn build(&self) -> Udp {
        Udp {

        }
    }

}
