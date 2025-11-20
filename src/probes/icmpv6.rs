// Icmpv6 concrete class and builder

use socket2::Socket;
use crate::network::socket_config::SocketConfig;
use crate::probes::probe::Probe;

pub struct Icmpv6 {

}

impl Icmpv6 {

}

impl Probe for Icmpv6 {
    fn to_byte_array(&self) -> Vec<u8> {
        todo!()
    }

    fn get_socket_config(&self) -> SocketConfig {
        todo!()
    }


    fn send(&mut self, socket: &Socket) {
        todo!()
    }

    fn receive(&self, socket: &Socket) {
        todo!()
    }

    fn validate_response(&self) -> bool {
        todo!()
    }
}

pub struct Icmpv6Builder {

}

impl Icmpv6Builder {
    pub fn new() -> Self {
        Icmpv6Builder {

        }
    }

    pub fn build(&self) -> Icmpv6 {
        Icmpv6 {

        }
    }

}