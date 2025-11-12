use crate::enums::ProbeType;
use crate::probes::icmp::IcmpBuilder;
use crate::probes::tcp::TcpBuilder;
use crate::probes::udp::UdpBuilder;
use crate::probes::probe::Probe;

pub struct ProbeFactory;

impl ProbeFactory {
    //For creating custom probes call builder directly
    pub fn create_default_probe(probe_type: ProbeType) -> Box<dyn Probe> { //default probe creation
       match probe_type {
           ProbeType::Icmp => Box::new(IcmpBuilder::new().build()),
           ProbeType::Tcp => Box::new(TcpBuilder::new().build()),
           ProbeType::Udp => Box::new(UdpBuilder::new().build()),
       }
   }
}