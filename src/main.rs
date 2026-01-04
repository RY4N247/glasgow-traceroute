// File: `src/main.rs`
use std::net::Ipv4Addr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration};
use std::{thread};
use clap::Parser;
use glasgow_traceroute::applications::ping::Ping;
use glasgow_traceroute::applications::traceroute::Traceroute;
use glasgow_traceroute::enums::{Tool, TransportProtocol};
use glasgow_traceroute::helpers::packet_parser;

#[derive(Parser, Debug)]
#[command(name = "glasgow-traceroute")]
struct Args {
    #[arg(value_enum)]
    tool: Tool,

    #[arg(value_enum)]
    probe_type: TransportProtocol,

    destination: String,

    #[arg(long)]
    port: Option<u16>,
}

fn main() {
    let args = Args::parse();

    if matches!(args.probe_type, TransportProtocol::Udp) && args.port.is_none() {
        panic!("UDP probes require --port");
    }

    // Ctrl-C handler
    let running = Arc::new(AtomicBool::new(true));
    {
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
    }

    match args.tool {
        Tool::Ping => {
            println!("PING {}:", args.destination);

            let dest: Ipv4Addr = args
                .destination
                .parse()
                .expect("Invalid IPv4 address");

            // timeout_ms, payload_size
            let timeout_ms = 1000;
            let payload_size = 36;

            let mut ping = Ping::new(args.probe_type.clone(), dest, timeout_ms, payload_size, args.port);

            let mut packets_sent: u64 = 0;
            let mut packets_received: u64 = 0;
            let mut rtts_ms: Vec<f64> = Vec::new();

            while running.load(Ordering::SeqCst) {
                packets_sent += 1;
                match ping.send_ping() {
                    Ok(res) => {
                        let rtt_ms = res
                            .rtt
                            .map(|d| d.as_secs_f64() * 1000.0)
                            .unwrap();

                        if args.probe_type == TransportProtocol::Icmp {
                            // Use packet_parser helper to extract sequence number properly
                            let seq = packet_parser::extract_icmp_identifier_seq(&res.raw_packet)
                                .map(|(_src_ip, _identifier, sequence)| sequence)
                                .unwrap_or(0);
                            
                            println!(
                                "Sent ICMP echo request, received ICMP echo reply: {} bytes from {}: icmp_seq={} time={:.3} ms",
                                res.bytes_received,
                                res.destination,
                                seq,
                                rtt_ms
                            );
                        } else if args.probe_type == TransportProtocol::Udp {
                            println!(
                                "Sent UDP request, received ICMP reply: {} bytes from {}: time={:.3} ms",
                                res.bytes_received,
                                res.destination,
                                rtt_ms
                            );
                        }
                        packets_received += 1;
                        rtts_ms.push(rtt_ms);
                    }
                    Err(_e) => {
                        println!("Request timeout for {}", args.destination);
                    }
                }

                thread::sleep(Duration::from_secs(1));
            }

            // Summary
            println!("\n--- {} ping statistics ---", args.destination);
            println!(
                "{} packets transmitted, {} packets received, {:.1}% packet loss",
                packets_sent,
                packets_received,
                if packets_sent > 0 {
                    ((packets_sent - packets_received) as f64 / packets_sent as f64) * 100.0
                } else {
                    0.0
                }
            );

            if !rtts_ms.is_empty() {
                let mut min = f64::INFINITY;
                let mut max = f64::NEG_INFINITY;
                let mut sum = 0.0;
                
                for rtt in &rtts_ms {
                    if *rtt < min { min = *rtt; }
                    if *rtt > max { max = *rtt; }
                    sum += rtt;
                }
                
                let avg = sum / rtts_ms.len() as f64;
                
                let mut variance_sum = 0.0;
                for rtt in &rtts_ms {
                    let diff = rtt - avg;
                    variance_sum += diff * diff;
                }
                let stddev = (variance_sum / rtts_ms.len() as f64).sqrt();

                println!(
                    "round-trip min/avg/max/stddev = {:.3}/{:.3}/{:.3}/{:.3} ms",
                    min, avg, max, stddev
                );
            }
        }

        Tool::Traceroute => {
            let dest: Ipv4Addr = args
                .destination
                .parse()
                .expect("Invalid IPv4 address");

            let timeout_ms = 2000;
            let payload_size = 36;
            let max_ttl = 30;

            println!("traceroute to {} ({}), {} hops max", args.destination, dest, max_ttl);

            let mut traceroute = Traceroute::new(args.probe_type.clone(), dest, timeout_ms, payload_size, max_ttl);
            let results = traceroute.trace_route();

            for hop in results {
                match hop.address {
                    Some(addr) => {
                        let rtt_str = hop.rtt
                            .map(|d| format!("{:.3} ms", d.as_secs_f64() * 1000.0))
                            .unwrap_or_else(|| "*".to_string());
                        println!("{:2}  {}  {}", hop.ttl, addr, rtt_str);
                    }
                    None => {
                        println!("{:2}  *", hop.ttl);
                    }
                }
            }
        }
    }
}