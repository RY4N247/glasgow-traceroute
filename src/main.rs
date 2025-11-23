use std::net::IpAddr;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use socket2::Socket;
use clap::Parser;
use glasgow_traceroute::enums::{ProbeType, Tool};
use glasgow_traceroute::probes::probe_factory::ProbeFactory;
use std::time::{Duration, Instant};
use std::thread;

/// CLI arguments
#[derive(Parser, Debug)]
#[command(name = "glasgow-traceroute")]
#[command(about = "Paris traceroute and ping tool in Rust", long_about = None)]
struct Args {
    #[arg(value_enum)]
    tool: Tool,

    #[arg(value_enum)]
    probe_type: ProbeType,

    /// Destination IP address
    destination: String,

    /// Port (required for TCP/UDP)
    #[arg(long)]
    port: Option<u16>,
}

fn main() {
    let args = Args::parse();

    let destination: IpAddr = args.destination.parse().expect("Invalid IP address");

    // Validate IP versions
    match args.probe_type {
        ProbeType::Icmp | ProbeType::Tcp | ProbeType::Udp => {
            if !matches!(destination, IpAddr::V4(_)) {
                panic!("ICMP, TCP, and UDP probes require an IPv4 address");
            }
        }
        ProbeType::Icmpv6 => {
            if !matches!(destination, IpAddr::V6(_)) {
                panic!("ICMPv6 probe requires an IPv6 address");
            }
        }
    }

    // TCP/UDP require a port
    if matches!(args.probe_type, ProbeType::Tcp | ProbeType::Udp) && args.port.is_none() {
        panic!("TCP and UDP probes require --port");
    }

    // Ctrl-C
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    match args.tool {
        Tool::Ping => {
            println!("-------- PING ---------");

            let mut probe = ProbeFactory::create_default_probe(args.probe_type);

            probe.set_destination(args.destination.clone());
            if let Some(port) = args.port {
                probe.set_port(port);
            }

            let socket_config = probe.get_socket_config();
            let socket = Socket::new(
                socket_config.domain,
                socket_config.sock_type,
                socket_config.protocol,
            ).unwrap();
            //prevents blocking indefinitely
            socket.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

            //stats accumulators
            let mut packets_sent: u64 = 0;
            let mut packets_received: u64 = 0;


            //ping loop
            while running.load(Ordering::SeqCst) {

                packets_sent += 1;

                let start_t = Instant::now();
                probe.send(&socket);
                let ok = probe.receive(&socket);
                let rtt = start_t.elapsed().as_millis();

                println!("  └── time={} ms", rtt);

                if ok {
                    packets_received += 1;
                }

                thread::sleep(Duration::from_secs(1));
            }


            println!("\n-------- {} STATISTICS ---------", args.destination);

            let loss = ((packets_sent - packets_received) as f64 / packets_sent as f64) * 100.0;

            println!("{} packets transmitted, {} received, {:.1}% packet loss",
                     packets_sent, packets_received, loss);

        }

        Tool::Traceroute => {
            println!("Traceroute tool selected");
        }
    }
}
