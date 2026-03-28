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

    # Map IP addresses to node names (a node may have one IP or a list, e.g. router with multiple interfaces)
    ip_to_node = {}
    for node_name, node_ip in topo['nodes'].items():
        if node_ip is None:
            continue
        items = node_ip if isinstance(node_ip, list) else [node_ip]
        for item in items:
            raw = item if isinstance(item, str) else str(item)
            ip = raw.split('/')[0]
            ip_to_node[ip] = node_name

    # Map traceroute IPs to node names
    nodes_with_ips = []
    for ip in traceroute_ips:
        if ip in ip_to_node:
            node_name = ip_to_node[ip]
            nodes_with_ips.append(node_name)
    
    # Highlight: exact path returned by traceroute (nodes_with_ips) plus always s1 and s2 if present
    nodes_to_highlight = list(nodes_with_ips)
    for sw in ("s1", "s2"):
        if sw in topo["nodes"] and sw not in nodes_to_highlight:
            nodes_to_highlight.append(sw)

    print(" ")
    print(" ")
    print("===TOPOLOGY===")
    print("")
    # Explicit path: order of nodes as seen by traceroute (IPs mapped to node names)
    if nodes_with_ips:
        print("Path: " + " -> ".join(nodes_with_ips))
        print("")
    
    def first_ip(node_ip):
        if node_ip is None:
            return None
        item = node_ip[0] if isinstance(node_ip, list) else node_ip
        raw = item if isinstance(item, str) else str(item)
        return raw.split('/')[0]

    # Print all nodes and their IPs, highlighting path + s1/s2
    node_path_set = set(nodes_to_highlight)
    for node_name, node_ip in sorted(topo['nodes'].items()):
        ip = first_ip(node_ip)
        if node_name in node_path_set:
            print(f"{GREEN}{node_name}{RESET}: {ip}" if ip else f"{GREEN}{node_name}{RESET}: null")
        else:
            print(f"{node_name}: {ip}" if ip else f"{node_name}: null")

    renderer = ASCIIRenderer(G)
    ascii_graph = renderer.render()

    # Highlight path + s1/s2 in diagram; replace longer names first to avoid partial matches
    for node in sorted(nodes_to_highlight, key=len, reverse=True):
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
