# Glasgow Traceroute

**Glasgow Traceroute** is a Rust CLI (and library) for active IPv4 network measurement using **raw sockets**. It provides **ping**, **Paris traceroute** (keeping flow-identifying fields constant across TTL steps to avoid load-balancing artefacts), and **MDA-style** multipath discovery (varying the flow identifier to explore multiple valid paths).

## Dissertation

For full design details, implementation discussion, and evaluation, read the full dissertation:

[Read the full dissertation](./Ryan_Bramwell_Glasgow_Traceroute.pdf)

### Usage and Documentation
Run the following commands for usage and implementation details:

```bash
cargo run -- --help
cargo doc --open
```

**Note (raw sockets)**:
- **Linux**: run `./run_raw.sh` once. It builds the project and uses `sudo setcap cap_net_raw+ep target/debug/glasgow-traceroute`.
- **macOS**: run with `sudo` (e.g. `sudo cargo run -- <args>`).

## Documentation

This project includes several guides to help you get started:

### 1. [Load Balanced Network Example](./example/README.md)
   **Start here** - Complete guide to setting up and running the load balanced network example using Mininet and FRR. This demonstrates Paris Traceroute's ability to trace paths through networks with multiple equal-cost paths.

### 2. [Custom Topologies](./example/load_balanced_network_a/README.md)
   Learn how to create your own network topologies for testing and experimentation with glasgow-traceroute.

### 3. [ASCII Topology Visualization](./src/pycall/README.md)
   Set up the optional Python environment to visualize traceroute paths as ASCII art on your network topology diagrams.

## Tests

```bash
cargo test
```

## Author

Ryan Bramwell <ryan.bramwell.2022@uni.strath.ac.uk>
