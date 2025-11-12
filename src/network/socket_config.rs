use socket2::{Domain, Type, Protocol};

pub struct SocketConfig {
    pub domain: Domain,
    pub sock_type: Type,
    pub protocol: Option<Protocol>,
}
