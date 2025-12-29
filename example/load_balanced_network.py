from mininet.node import Node
from mininet.topo import Topo

class Router(Node):
    def config(self, **params):
        super(Router, self).config(**params)

    def terminate(self):
        super(Router, self).terminate()


class NetworkTopo(Topo):
    def build(self, **_opts):
        pass
