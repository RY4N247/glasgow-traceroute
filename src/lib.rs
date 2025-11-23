//! # Glasgow Traceroute CLI
//!
//! Paris traceroute and ping tool implemented in Rust.
//!
//! ## Usage
//!
//! Run the CLI with the desired tool, probe type, destination, and optional port:
//!
//! ```bash
//! glasgow-traceroute <TOOL> <PROBE_TYPE> <DESTINATION> [--port <PORT>]
//! ```
//!
//! ### Tools
//!
//! - `ping` — Send ICMP/TCP/UDP probes to a destination.
//! - `traceroute` — Perform a Paris traceroute to a destination.
//!
//! ### Probe Types
//!
//! - `icmp` — ICMPv4
//! - `icmpv6` — ICMPv6
//! - `tcp` — TCP probe
//! - `udp` — UDP probe
//!
//! ### Ping Example
//!
//! Ping an IPv4 host using ICMP:
//!
//! ```bash
//! glasgow-traceroute ping icmp 8.8.8.8
//! ```
//!
//! Ping an IPv6 host using ICMPv6:
//!
//! ```bash
//! glasgow-traceroute ping icmpv6 2001:4860:4860::8888
//! ```
//!
//! TCP ping on port 80:
//!
//! ```bash
//! glasgow-traceroute ping tcp 192.168.1.1 --port 80
//! ```
//!
//! UDP ping on port 53:
//!
//! ```bash
//! glasgow-traceroute ping udp 192.168.1.1 --port 53
//! ```
//! ### Notes
//!
//! - ICMP, TCP, and UDP probes require an **IPv4 address**.
//! - ICMPv6 probes require an **IPv6 address**.
//! - TCP and UDP probes require the `--port` argument.

pub mod enums;
pub mod probes;
pub mod network;
