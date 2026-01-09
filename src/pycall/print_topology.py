import sys
import json
import yaml
import networkx as nx
from phart import ASCIIRenderer


GREEN = "\033[92m"
RESET  = "\033[0m"


def print_topology(yaml_file_path, traceroute_ips):
    with open(yaml_file_path, 'r') as f:
        topo = yaml.safe_load(f)

    G = nx.DiGraph()
    for node in topo['nodes']:
        G.add_node(node)

    for link in topo['links']:
        G.add_edge(link[0], link[1])

    # Map IP addresses to node names
    ip_to_node = {}
    for node_name, node_ip in topo['nodes'].items():
        if node_ip:
            # Extract IP from "IP/prefix" format
            ip = node_ip.split('/')[0]
            ip_to_node[ip] = node_name

    # Map traceroute IPs to node names
    nodes_with_ips = []
    for ip in traceroute_ips:
        if ip in ip_to_node:
            node_name = ip_to_node[ip]
            nodes_with_ips.append(node_name)
    
    # Find the complete path including switches
    # If we have at least 2 nodes, find the shortest path between them
    if len(nodes_with_ips) >= 2:
        try:
            # Find shortest path from first to last node
            complete_path = nx.shortest_path(G, nodes_with_ips[0], nodes_with_ips[-1])
            node_path = complete_path
        except (nx.NetworkXNoPath, nx.NodeNotFound):
            # Fallback to just the nodes with IPs if path not found
            node_path = nodes_with_ips
    else:
        node_path = nodes_with_ips

    print(" ")
    print(" ")
    print("===TOPOLOGY===")
    print("")
    
    # Print all nodes and their IPs, highlighting ones in the path
    node_path_set = set(node_path)
    for node_name, node_ip in sorted(topo['nodes'].items()):
        if node_name in node_path_set:
            # Highlight nodes in the path in green
            if node_ip:
                ip = node_ip.split('/')[0]
                print(f"{GREEN}{node_name}{RESET}: {ip}")
            else:
                print(f"{GREEN}{node_name}{RESET}: null")
        else:
            # Regular nodes not in path
            if node_ip:
                ip = node_ip.split('/')[0]
                print(f"{node_name}: {ip}")
            else:
                print(f"{node_name}: null")

    renderer = ASCIIRenderer(G)
    ascii_graph = renderer.render()

    # Highlight nodes in the traceroute path
    for node in node_path:
        ascii_graph = ascii_graph.replace(
            node, 
            f"{GREEN}{node}{RESET}"
        )
    print(" ") 
    print(ascii_graph)


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python print_topology.py <topology.yaml> <traceroute_ips_json>", file=sys.stderr)
        sys.exit(1)

    yaml_file_path = sys.argv[1]
    traceroute_ips_json = sys.argv[2]
    
    # Parse JSON array of IP addresses
    traceroute_ips = json.loads(traceroute_ips_json)
    print_topology(yaml_file_path, traceroute_ips)
