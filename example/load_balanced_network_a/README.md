# Custom Topologies
## Creating Your Own Topologies
- To create your own custom topologies follow this guide by James Wanderer. https://medium.com/@jmwanderer/fun-with-routing-protocols-8a0677aab2fc
- There is no need to follow the vm setup detailed https://github.com/jmwanderer/mininet_frr as running `bash run_network.sh` installs everything including the repo itself which can be found by `cd /home/ubuntu/git/mininet_frr`
- Be aware of the `sysctl` setting: `sysctl -w net.ipv4.icmp_errors_use_inbound_ifaddr=1` when working with multi-interface topologies to avoid ICMP errors being sourced from unexpected interface addresses.