use clap::ValueEnum;
// ------------------------------------
//         Common Enums
// ------------------------------------
#[derive(ValueEnum, Clone, Debug)]
pub enum Tool {
    Ping,
    Traceroute
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ProbeType {
    Icmp,
    Tcp,
    Udp
}

#[derive(ValueEnum, Clone, Debug)]
pub enum IpVersion {
    V4,
    V6
}
// ------------------------------------
//              ICMP Types
// ------------------------------------
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


// ------------------------------------
//              TCP Types
// ------------------------------------


// ------------------------------------
//              UDP Types
// ------------------------------------

