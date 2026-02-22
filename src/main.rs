// File: `src/main.rs`
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration};
use std::{thread};
use clap::Parser;
use glasgow_traceroute::applications::mda::Mda;
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

    #[arg(long)]
    topology: Option<String>,
}

fn main() {
    let args = Args::parse();

    // Only require --port for Ping with UDP, not for Traceroute
    if matches!(args.tool, Tool::Ping) && matches!(args.probe_type, TransportProtocol::Udp) && args.port.is_none() {
        panic!("UDP ping requires --port");
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
            
            // Get host IP from traceroute instance
            let host_ip = traceroute.source_address();
            let results = traceroute.trace_route();

            // Collect IP addresses from traceroute results
            let mut traceroute_ips: Vec<String> = Vec::new();
            
            // Add host IP at the beginning
            traceroute_ips.push(host_ip.to_string());
            
            for hop in &results {
                match hop.address {
                    Some(addr) => {
                        let rtt_str = hop.rtt
                            .map(|d| format!("{:.3} ms", d.as_secs_f64() * 1000.0))
                            .unwrap_or_else(|| "*".to_string());
                        println!("{:2}  {}  {}", hop.ttl, addr, rtt_str);
                        traceroute_ips.push(addr.to_string());
                    }
                    None => {
                        println!("{:2}  *", hop.ttl);
                    }
                }
            }

            if let Some(ref topology_path) = args.topology {
                let root = env!("CARGO_MANIFEST_DIR");
                let venv_python = PathBuf::from(root).join(".venv/bin/python3");

                // Check if venv exists and has dependencies
                let needs_setup = !venv_python.exists() || 
                    Command::new(&venv_python)
                        .arg("-c").arg("import yaml, networkx, phart")
                        .output()
                        .map(|o| !o.status.success())
                        .unwrap_or(true);
                
                if needs_setup {
                    println!("Failed to print topology: ");
                    println!("    - Python venv not found or missing dependencies.");
                    println!("    - Please run on the host (not in mininet): src/pycall/setup_py_venv.sh");
                    println!("    - Or run: python3 -m venv .venv && .venv/bin/pip install -r src/pycall/requirements.txt");
                    return;
                }

                // Convert IP addresses to JSON
                let ips_json = serde_json::to_string(&traceroute_ips).unwrap_or_else(|_| "[]".to_string());
                
                // Run script
                let output = Command::new(&venv_python)
                    .arg(PathBuf::from(root).join("src/pycall/print_topology.py"))
                    .arg(topology_path)
                    .arg(&ips_json)
                    .output()
                    .expect("Failed to run script");
                
                print!("{}", String::from_utf8_lossy(&output.stdout));
                if !output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
                
            }
        }

        Tool::Mda => {
            if args.probe_type != TransportProtocol::Udp {
                panic!("MDA only supports UDP. Use: mda udp <destination>");
            }

            let dest: Ipv4Addr = args
                .destination
                .parse()
                .expect("Invalid IPv4 address");

            let timeout_ms = 2000;
            let payload_size = 36;
            let max_ttl = 30;

            println!("mda traceroute to {} ({}), {} hops max (UDP)", args.destination, dest, max_ttl);

            let mda = Mda::new(dest, timeout_ms, payload_size);
            let paths = mda.multipath_traceroute(1, max_ttl);

            for (i, path) in paths.iter().enumerate() {
                println!("Path {}:", i + 1);
                for (ttl, addr) in path.iter().enumerate() {
                    println!("  {:2}  {}", ttl + 1, addr);
                }
                if i < paths.len() - 1 {
                    println!();
                }
            }
        }
    }
}
