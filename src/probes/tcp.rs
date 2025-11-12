// Tcp concrete class and builder

use socket2::Socket;
use crate::network::socket_config::SocketConfig;
use crate::probes::probe::Probe;

pub struct Tcp {

}

impl Tcp {

}

impl Probe for Tcp {
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

    fn summary(&self) {
        todo!()
    }
}

pub struct TcpBuilder {

}

impl TcpBuilder {
    pub fn new() -> Self {
        TcpBuilder {

        }
    }

    pub fn build(&self) -> Tcp {
        Tcp {

        }
    }

}