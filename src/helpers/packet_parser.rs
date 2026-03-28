use std::net::Ipv4Addr;
use crate::enums::{TransportProtocol, IcmpType, IpProtocol};

pub fn extract_icmp_identifier_seq(packet: &[u8]) -> Option<(Ipv4Addr, u16, u16)> {
    if packet.len() < 28 {
        return None; // Need at least IP header (20) + ICMP header (8)
    }
    let ip_header_len = ((packet[0] & 0x0f) * 4) as usize;
    if packet.len() < ip_header_len + 8 {
        return None;
    }
    
    let src_ip = Ipv4Addr::new(
        packet[12],
        packet[13],
        packet[14],
        packet[15],
    );
    
    let icmp_type = packet[ip_header_len];
    if icmp_type != IcmpType::EchoReply.to_u8() {
        return None;
    }
    let identifier = u16::from_be_bytes([packet[ip_header_len + 4], packet[ip_header_len + 5]]);
    let sequence = u16::from_be_bytes([packet[ip_header_len + 6], packet[ip_header_len + 7]]);
    Some((src_ip, identifier, sequence))
}

pub fn extract_udp_source_port_from_icmp_error(packet: &[u8], local_ip: Ipv4Addr) -> Option<(Ipv4Addr, u16)> {
    if packet.len() < 48 {
        return None; 
    }
    let ip_header_len = ((packet[0] & 0x0f) * 4) as usize;
    if packet.len() < ip_header_len + 8 + 20 {
        return None;
    }
    
    let src_ip = Ipv4Addr::new(
        packet[12],
        packet[13],
        packet[14],
        packet[15],
    );
    
    let icmp_type = packet[ip_header_len];
    if icmp_type != IcmpType::DestinationUnreachable.to_u8() && icmp_type != IcmpType::TimeExceeded.to_u8() {
        return None;
    }
    let embedded_ip_start = ip_header_len + 8;
    if packet.len() < embedded_ip_start + 20 {
        return None;
    }
    
    // Calculate embedded IP header length 
    let embedded_ip_header_len = ((packet[embedded_ip_start] & 0x0f) * 4) as usize;
    if packet.len() < embedded_ip_start + embedded_ip_header_len + 8 {
        return None;
    }
    
    // Check if the embedded IP header's protocol is UDP
    let embedded_protocol = packet[embedded_ip_start + 9];
    if embedded_protocol != IpProtocol::UDP.to_u8() {
        return None;
    }
    
    // Check if the embedded IP header's source address matches our local IP
    let embedded_src = Ipv4Addr::new(
        packet[embedded_ip_start + 12],
        packet[embedded_ip_start + 13],
        packet[embedded_ip_start + 14],
        packet[embedded_ip_start + 15],
    );
    if embedded_src != local_ip {
        return None;
    }
    // UDP header starts after embedded IP header
    let udp_header_start = embedded_ip_start + embedded_ip_header_len;
    let src_port = u16::from_be_bytes([packet[udp_header_start], packet[udp_header_start + 1]]);
    Some((src_ip, src_port))
}

pub fn extract_udp_ports_from_icmp_error(packet: &[u8], local_ip: Ipv4Addr) -> Option<(Ipv4Addr, u16, u16)> {
    if packet.len() < 48 {
        return None; 
    }
    let ip_header_len = ((packet[0] & 0x0f) * 4) as usize;
    if packet.len() < ip_header_len + 8 + 20 {
        return None;
    }
    
    let src_ip = Ipv4Addr::new(
        packet[12],
        packet[13],
        packet[14],
        packet[15],
    );
    
    let icmp_type = packet[ip_header_len];
    if icmp_type != IcmpType::DestinationUnreachable.to_u8() && icmp_type != IcmpType::TimeExceeded.to_u8() {
        return None;
    }
    let embedded_ip_start = ip_header_len + 8;
    if packet.len() < embedded_ip_start + 20 {
        return None;
    }
    
    // Calculate embedded IP header length 
    let embedded_ip_header_len = ((packet[embedded_ip_start] & 0x0f) * 4) as usize;
    if packet.len() < embedded_ip_start + embedded_ip_header_len + 8 {
        return None;
    }
    
    // Check if the embedded IP header's protocol is UDP
    let embedded_protocol = packet[embedded_ip_start + 9];
    if embedded_protocol != IpProtocol::UDP.to_u8() {
        return None;
    }
    
    // Check if the embedded IP header's source address matches our local IP
    let embedded_src = Ipv4Addr::new(
        packet[embedded_ip_start + 12],
        packet[embedded_ip_start + 13],
        packet[embedded_ip_start + 14],
        packet[embedded_ip_start + 15],
    );
    if embedded_src != local_ip {
        return None;
    }
    // UDP header starts after embedded IP header
    let udp_header_start = embedded_ip_start + embedded_ip_header_len;
    let src_port = u16::from_be_bytes([packet[udp_header_start], packet[udp_header_start + 1]]);
    let dst_port = u16::from_be_bytes([packet[udp_header_start + 2], packet[udp_header_start + 3]]);
    Some((src_ip, src_port, dst_port))
}

pub fn extract_icmp_identifier_seq_from_icmp_error(packet: &[u8]) -> Option<(Ipv4Addr, u16, u16)> {
    const IP_HEADER_LEN: usize = 20;
    const ICMP_HEADER_LEN: usize = 8;
    const MIN_LEN: usize = IP_HEADER_LEN + ICMP_HEADER_LEN + IP_HEADER_LEN + ICMP_HEADER_LEN;
    
    if packet.len() < MIN_LEN {
        return None;
    }
    
    // Check ICMP type is Time Exceeded or Destination Unreachable
    let icmp_type = packet[IP_HEADER_LEN];
    if icmp_type != IcmpType::TimeExceeded.to_u8() && icmp_type != IcmpType::DestinationUnreachable.to_u8() {
        return None;
    }
    
    // Router that sent the error (source IP from outer IP header, bytes 12-15)
    let src_ip = Ipv4Addr::new(packet[12], packet[13], packet[14], packet[15]);
    
    // Embedded ICMP starts at: IP(20) + ICMP_error(8) + embedded_IP(20) = 48
    // Identifier at offset 4, Sequence at offset 6 within ICMP header
    const EMBEDDED_ICMP_START: usize = IP_HEADER_LEN + ICMP_HEADER_LEN + IP_HEADER_LEN;
    let identifier = u16::from_be_bytes([packet[EMBEDDED_ICMP_START + 4], packet[EMBEDDED_ICMP_START + 5]]);
    let sequence = u16::from_be_bytes([packet[EMBEDDED_ICMP_START + 6], packet[EMBEDDED_ICMP_START + 7]]);
    
    Some((src_ip, identifier, sequence))
}

pub fn extract_source_ip(packet: &[u8]) -> Option<Ipv4Addr> {
    if packet.len() < 20 {
        return None;
    }
    Some(Ipv4Addr::new(packet[12], packet[13], packet[14], packet[15]))
}

pub fn packet_matches(
    packet: &[u8],
    transport_type: &TransportProtocol,
    expected_destination: Ipv4Addr,
    expected_icmp_identifier: Option<u16>,
    expected_icmp_sequence: Option<u16>,
    expected_udp_source_port: Option<u16>,
    local_ip: Ipv4Addr,
) -> bool {
    match transport_type {
        TransportProtocol::Icmp => {
            if packet.len() < 20 {
                return false;
            }
            let src_ip = match extract_source_ip(packet) {
                Some(ip) => ip,
                None => return false,
            };
            if src_ip != expected_destination {
                return false;
            }
            
            if let Some(identifier) = expected_icmp_identifier {
                if let Some(seq) = expected_icmp_sequence {
                    if let Some((_recv_src_ip, recv_id, recv_seq)) = extract_icmp_identifier_seq(packet) {
                        return recv_id == identifier && recv_seq == seq;
                    }
                    return false; // Not a valid Echo Reply
                }
            }
            false
        }
        TransportProtocol::Udp => {
            if let Some(expected_src_port) = expected_udp_source_port {
                if let Some((recv_src_ip, recv_src_port)) = extract_udp_source_port_from_icmp_error(packet, local_ip) {
                    return recv_src_ip == expected_destination && recv_src_port == expected_src_port;
                }
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip_header(src_ip: Ipv4Addr) -> [u8; 20] {
        let octets = src_ip.octets();
        let mut h = [0u8; 20];
        h[0] = 0x45; // version 4, IHL 5
        h[12..16].copy_from_slice(&octets);
        h
    }

    #[test]
    fn extract_icmp_identifier_seq_valid() {
        let mut packet = vec![0u8; 28];
        packet[0..20].copy_from_slice(&ip_header(Ipv4Addr::new(192, 168, 1, 1)));
        packet[20] = IcmpType::EchoReply.to_u8();
        packet[24..26].copy_from_slice(&1234u16.to_be_bytes());
        packet[26..28].copy_from_slice(&5678u16.to_be_bytes());

        let result = extract_icmp_identifier_seq(&packet).unwrap();
        assert_eq!(result.0, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(result.1, 1234);
        assert_eq!(result.2, 5678);
    }

    #[test]
    fn extract_icmp_identifier_seq_too_short() {
        assert!(extract_icmp_identifier_seq(&[0u8; 20]).is_none());
    }

    #[test]
    fn extract_icmp_identifier_seq_wrong_type() {
        let mut packet = vec![0u8; 28];
        packet[0..20].copy_from_slice(&ip_header(Ipv4Addr::new(192, 168, 1, 1)));
        packet[20] = IcmpType::EchoRequest.to_u8();
        assert!(extract_icmp_identifier_seq(&packet).is_none());
    }

    #[test]
    fn extract_udp_source_port_from_icmp_error_valid() {
        let mut packet = vec![0u8; 56]; // outer IP(20) + ICMP(8) + embedded IP(20) + UDP(8)
        packet[0..20].copy_from_slice(&ip_header(Ipv4Addr::new(10, 0, 0, 1)));
        packet[20] = IcmpType::TimeExceeded.to_u8();
        packet[28] = 0x45; // embedded IP IHL
        packet[37] = IpProtocol::UDP.to_u8();
        packet[40..44].copy_from_slice(&Ipv4Addr::new(192, 168, 1, 100).octets()); // embedded src
        packet[48..50].copy_from_slice(&33456u16.to_be_bytes()); // UDP src port at offset 48

        let local = Ipv4Addr::new(192, 168, 1, 100);
        let result = extract_udp_source_port_from_icmp_error(&packet, local).unwrap();
        assert_eq!(result.0, Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(result.1, 33456);
    }

    #[test]
    fn extract_udp_source_port_from_icmp_error_wrong_local_ip() {
        let mut packet = vec![0u8; 56];
        packet[0..20].copy_from_slice(&ip_header(Ipv4Addr::new(10, 0, 0, 1)));
        packet[20] = IcmpType::DestinationUnreachable.to_u8();
        packet[28] = 0x45;
        packet[37] = IpProtocol::UDP.to_u8();
        packet[40..44].copy_from_slice(&Ipv4Addr::new(192, 168, 1, 100).octets());
        packet[48..50].copy_from_slice(&33456u16.to_be_bytes());

        let wrong_local = Ipv4Addr::new(192, 168, 1, 99);
        assert!(extract_udp_source_port_from_icmp_error(&packet, wrong_local).is_none());
    }

    #[test]
    fn extract_udp_ports_from_icmp_error_valid() {
        let mut packet = vec![0u8; 56];
        packet[0..20].copy_from_slice(&ip_header(Ipv4Addr::new(10, 0, 0, 1)));
        packet[20] = IcmpType::TimeExceeded.to_u8();
        packet[28] = 0x45;
        packet[37] = IpProtocol::UDP.to_u8();
        packet[40..44].copy_from_slice(&Ipv4Addr::new(192, 168, 1, 100).octets());
        packet[48..50].copy_from_slice(&33456u16.to_be_bytes());
        packet[50..52].copy_from_slice(&443u16.to_be_bytes());

        let local = Ipv4Addr::new(192, 168, 1, 100);
        let result = extract_udp_ports_from_icmp_error(&packet, local).unwrap();
        assert_eq!(result.0, Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(result.1, 33456);
        assert_eq!(result.2, 443);
    }

    #[test]
    fn extract_icmp_identifier_seq_from_icmp_error_valid() {
        let mut packet = vec![0u8; 56];
        packet[0..20].copy_from_slice(&ip_header(Ipv4Addr::new(10, 0, 0, 1)));
        packet[20] = IcmpType::TimeExceeded.to_u8();
        // Embedded ICMP starts at 48; identifier at +4, sequence at +6
        packet[52..54].copy_from_slice(&1111u16.to_be_bytes());
        packet[54..56].copy_from_slice(&2222u16.to_be_bytes());

        let result = extract_icmp_identifier_seq_from_icmp_error(&packet).unwrap();
        assert_eq!(result.0, Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(result.1, 1111);
        assert_eq!(result.2, 2222);
    }

    #[test]
    fn extract_icmp_identifier_seq_from_icmp_error_too_short() {
        assert!(extract_icmp_identifier_seq_from_icmp_error(&[0u8; 47]).is_none());
    }

    #[test]
    fn extract_source_ip_valid() {
        let mut packet = vec![0u8; 20];
        packet[12..16].copy_from_slice(&Ipv4Addr::new(1, 2, 3, 4).octets());
        assert_eq!(extract_source_ip(&packet), Some(Ipv4Addr::new(1, 2, 3, 4)));
    }

    #[test]
    fn extract_source_ip_too_short() {
        assert!(extract_source_ip(&[0u8; 19]).is_none());
    }

    #[test]
    fn packet_matches_icmp() {
        let mut packet = vec![0u8; 28];
        packet[0..20].copy_from_slice(&ip_header(Ipv4Addr::new(192, 168, 1, 1)));
        packet[20] = IcmpType::EchoReply.to_u8();
        packet[24..26].copy_from_slice(&100u16.to_be_bytes());
        packet[26..28].copy_from_slice(&200u16.to_be_bytes());

        assert!(packet_matches(
            &packet,
            &TransportProtocol::Icmp,
            Ipv4Addr::new(192, 168, 1, 1),
            Some(100),
            Some(200),
            None,
            Ipv4Addr::UNSPECIFIED,
        ));
    }

    #[test]
    fn packet_matches_icmp_wrong_identifier() {
        let mut packet = vec![0u8; 28];
        packet[0..20].copy_from_slice(&ip_header(Ipv4Addr::new(192, 168, 1, 1)));
        packet[20] = IcmpType::EchoReply.to_u8();
        packet[24..26].copy_from_slice(&100u16.to_be_bytes());
        packet[26..28].copy_from_slice(&200u16.to_be_bytes());

        assert!(!packet_matches(
            &packet,
            &TransportProtocol::Icmp,
            Ipv4Addr::new(192, 168, 1, 1),
            Some(999), // wrong identifier
            Some(200),
            None,
            Ipv4Addr::UNSPECIFIED,
        ));
    }

    #[test]
    fn packet_matches_udp() {
        let mut packet = vec![0u8; 56];
        packet[0..20].copy_from_slice(&ip_header(Ipv4Addr::new(10, 0, 0, 1)));
        packet[20] = IcmpType::TimeExceeded.to_u8();
        packet[28] = 0x45;
        packet[37] = IpProtocol::UDP.to_u8();
        packet[40..44].copy_from_slice(&Ipv4Addr::new(192, 168, 1, 100).octets());
        packet[48..50].copy_from_slice(&55555u16.to_be_bytes());

        let local = Ipv4Addr::new(192, 168, 1, 100);
        assert!(packet_matches(
            &packet,
            &TransportProtocol::Udp,
            Ipv4Addr::new(10, 0, 0, 1),
            None,
            None,
            Some(55555),
            local,
        ));
    }
}

