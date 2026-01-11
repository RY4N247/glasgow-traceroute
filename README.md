# Glasgow Traceroute

A network traceroute tool written in Rust that implements **Paris Traceroute** functionality using raw IPv4 sockets. Paris Traceroute is a variant of traceroute where fields are kept constant to avoid anomalies in load balanced networks.

## Future Work
- Support for IPv6 [ ]
- Support for TCP probes [ ]
- MDA Traceroute implementation [ ]
- Multi-threaded probing for faster results [ ]

### Prerequisites
- Rust 
- Linux, macOS, or WSL 2 (Windows not natively supported)
- Elevated privileges for raw socket usage

### Usage and Documentation
Run `cargo run -- --help` for usage or `cargo doc --open` for further implementation details.

**Note**: On Linux systems, see `run_raw.sh` in the repository root for guidance on setting up raw socket permissions.

## Documentation

This project includes several guides to help you get started:

### 1. [Load Balanced Network Example](./example/README.md)
   **Start here** - Complete guide to setting up and running the load balanced network example using Mininet and FRR. This demonstrates Paris Traceroute's ability to trace paths through networks with multiple equal-cost paths.

### 2. [Custom Topologies](./example/load_balanced_network/README.md)
   Learn how to create your own network topologies for testing and experimentation with glasgow-traceroute.

### 3. [ASCII Topology Visualization](./src/pycall/README.md)
   Set up the optional Python environment to visualize traceroute paths as ASCII art on your network topology diagrams.

## How It Works

Glasgow Traceroute implements Paris Traceroute by:
- Keeping probe packet fields constant (source port, destination port, etc.) across TTL increments
- Using raw sockets to construct and send custom IPv4 packets
- Parsing ICMP error messages to identify intermediate hops
- Maintaining state to match responses to sent probes

This approach ensures that probes follow the same path through load-balanced networks, providing accurate route discovery.

## Author

Ryan Bramwell <ryan.bramwell.2022@uni.strath.ac.uk>
