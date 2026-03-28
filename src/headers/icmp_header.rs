use crate::enums::{IcmpCode, IcmpType};
use packet::icmp::checksum;


#[derive(Debug)]
pub struct IcmpHeader {
    pub icmp_type: IcmpType,
    pub code: IcmpCode,
    pub checksum: u16,
    pub identifier: u16,
    pub sequence_number: u16,
}

impl IcmpHeader {
    pub fn to_byte_array(&self, payload: &[u8]) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::with_capacity(8 + payload.len());

        buf.push(self.icmp_type.to_u8());
        buf.push(self.code.to_u8());
        buf.extend_from_slice(&[0, 0]);
        buf.extend_from_slice(&self.identifier.to_be_bytes());
        buf.extend_from_slice(&self.sequence_number.to_be_bytes());
        buf.extend_from_slice(payload);

        let checksum = checksum(&buf);
        buf[2] = (checksum >> 8) as u8;
        buf[3] = (checksum & 0xff) as u8;

        buf
    }

    pub fn increment_sequence_number(&mut self) {
        self.sequence_number = self.sequence_number.wrapping_add(1);
        // Paris Traceroute: "For ICMP Echo probes, Paris traceroute varies the
        // Sequence Number field, as does classic traceroute, but also varies the
        // Identifier field, so as to keep constant the value for the Checksum field."
        // — Augustin et al., "Avoiding Traceroute Anomalies with Paris Traceroute", IMC 2006, Section 2.2
        self.identifier = self.identifier.wrapping_sub(1);
    }
}

pub struct IcmpHeaderBuilder {
    icmp_type: IcmpType,
    code: IcmpCode,
    #[allow(dead_code)] // computed dynamically in to_byte_array()
    checksum: u16,
    identifier: u16,
    sequence_number: u16,
}

impl IcmpHeaderBuilder {
    pub fn new() -> Self {
        IcmpHeaderBuilder {
            icmp_type: IcmpType::EchoRequest,
            code: IcmpCode::None,
            checksum: 0, // computed dynamically
            identifier: std::process::id() as u16,
            sequence_number: 0,
        }
    }

    pub fn icmp_type(mut self, icmp_type: IcmpType) -> Self {
        self.icmp_type = icmp_type;
        self
    }

    pub fn code(mut self, code: IcmpCode) -> Self {
        self.code = code;
        self
    }

    pub fn identifier(mut self, id: u16) -> Self {
        self.identifier = id;
        self
    }

    pub fn sequence_number(mut self, seq: u16) -> Self {
        self.sequence_number = seq;
        self
    }

    pub fn build(self) -> IcmpHeader {
        IcmpHeader {
            icmp_type: self.icmp_type,
            code: self.code,
            checksum: 0,
            identifier: self.identifier,
            sequence_number: self.sequence_number,
        }
    }
}

