/// ICMP Types and Codes as per RFC 792
/// https://tools.ietf.org/html/rfc792
/// https://www.geeksforgeeks.org/computer-networks/types-of-icmp-internet-control-message-protocol-messages/
#[derive(Clone, Debug)]
pub enum IcmpType {
    EchoReply = 0,
    DestinationUnreachable = 3,
    Redirect = 5,
    EchoRequest = 8,
    RouterAdvertisement = 9,
    RouterSolicitation = 10,
    TimeExceeded = 11,
    ParameterProblem = 12,
    Timestamp = 13,
    TimestampReply = 14,
}
impl IcmpType {
    pub fn to_u8(&self) -> u8 {
        match self {
            IcmpType::EchoReply => 0,
            IcmpType::DestinationUnreachable => 3,
            IcmpType::Redirect => 5,
            IcmpType::EchoRequest => 8,
            IcmpType::RouterAdvertisement => 9,
            IcmpType::RouterSolicitation => 10,
            IcmpType::TimeExceeded => 11,
            IcmpType::ParameterProblem => 12,
            IcmpType::Timestamp => 13,
            IcmpType::TimestampReply => 14,
        }
    }
}
#[derive(Clone, Debug)]
pub enum IcmpCode {
    None,
    DestinationUnreachable(DestinationUnreachableCode),
    TimeExceeded(TimeExceededCode),
    Redirect(RedirectCode),
    ParameterProblem(ParameterProblemCode),
    Raw(u8),
}
impl IcmpCode {
    pub fn to_u8(&self) -> u8 {
        match self {
            IcmpCode::None => 0,
            IcmpCode::Raw(n) => *n,
            IcmpCode::DestinationUnreachable(code) => match code {
                DestinationUnreachableCode::NetUnreachable => 0,
                DestinationUnreachableCode::HostUnreachable => 1,
                DestinationUnreachableCode::ProtocolUnreachable => 2,
                DestinationUnreachableCode::PortUnreachable => 3,
                DestinationUnreachableCode::FragmentationNeeded => 4,
                DestinationUnreachableCode::SourceRouteFailed => 5,
            },
            IcmpCode::TimeExceeded(code) => match code {
                TimeExceededCode::TtlExceeded => 0,
                TimeExceededCode::FragmentReassemblyTimeExceeded => 1,
            },
            IcmpCode::Redirect(code) => match code {
                RedirectCode::Network => 0,
                RedirectCode::Host => 1,
                RedirectCode::TosNetwork => 2,
                RedirectCode::TosHost => 3,
            },
            IcmpCode::ParameterProblem(code) => match code {
                ParameterProblemCode::PointerIndicatesError => 0,
                ParameterProblemCode::MissingRequiredOption => 1,
                ParameterProblemCode::BadLength => 2,
            },
        }
    }

}
#[derive(Clone, Debug)]
pub enum DestinationUnreachableCode {
    NetUnreachable = 0,
    HostUnreachable = 1,
    ProtocolUnreachable = 2,
    PortUnreachable = 3,
    FragmentationNeeded = 4,
    SourceRouteFailed = 5,
}
#[derive(Clone, Debug)]
pub enum TimeExceededCode {
    TtlExceeded = 0,
    FragmentReassemblyTimeExceeded = 1,
}
#[derive(Clone, Debug)]
pub enum RedirectCode {
    Network = 0,
    Host = 1,
    TosNetwork = 2,
    TosHost = 3,
}
#[derive(Clone, Debug)]
pub enum ParameterProblemCode {
    PointerIndicatesError = 0,
    MissingRequiredOption = 1,
    BadLength = 2,
}

#[derive(Debug)]
pub enum IpProtocol {
    ICMP = 1,
    TCP = 6,
    UDP = 17,
}
impl IpProtocol {
    pub fn to_u8(&self) -> u8 {
        match self {
            IpProtocol::ICMP => 1,
            IpProtocol::TCP => 6,
            IpProtocol::UDP => 17,
        }
    }
}

#[derive(Debug)]
pub enum IpFlags {
    Reserved = 0,
    DontFragment = 2,
    MoreFragments = 4,
}
impl IpFlags {
    pub fn to_u8(&self) -> u8 {
        match self {
            IpFlags::Reserved => 0,
            IpFlags::DontFragment => 2,
            IpFlags::MoreFragments => 4,
        }
    }
}

pub enum TransportProtocol {
    ICMP,  // Although ICMP is not a transport layer protocol, it's included here for completeness
    TCP,
    UDP,
}
