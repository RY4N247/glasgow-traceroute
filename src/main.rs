use glasgow_traceroute::enums::{ProbeType, Tool};
use glasgow_traceroute::probes::probe_factory::ProbeFactory;
use socket2::Socket;
use clap::Parser;

/// Define command line arguments structure
#[derive(Parser, Debug)]
struct Args {
    tool : Tool,
    probe_type: ProbeType
}

// ------------------------------------
//            Main Program
// ------------------------------------
fn main() {
    let args = Args::parse();

    match args.tool {
        Tool::Ping => {
            println!("Ping tool selected");
            let mut probe = ProbeFactory::create_default_probe(args.probe_type);
            let socket_config = probe.get_socket_config();
            let socket = Socket::new(
                socket_config.domain,
                socket_config.sock_type,
                socket_config.protocol
            ).unwrap();

            loop {
                probe.send(&socket);
                probe.receive(&socket);
            }
        }

        Tool::Traceroute => {
            println!("Traceroute tool selected");
        }
    }
}
