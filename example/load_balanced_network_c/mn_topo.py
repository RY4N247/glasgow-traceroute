#!/usr/bin/python3
from mininet.topo import Topo
from mininet.net import Mininet
from mininet.node import Node
from mininet.log import setLogLevel, info
from mininet.cli import CLI

class LoadBalancer(Node):
    def config(self, **params):
        super(LoadBalancer, self).config(**params)

    def terminate(self):
        super(LoadBalancer, self).terminate()
                                            


# Custom
#             a---c    e---g
#            /     \  /     \
#   h1--s1--l1      l2       i--s2--h2
#            \     /  \     /
#             b---d    f---h

class NetworkTopo(Topo):
    def build(self, **_opts):

        l1 = self.addHost('l1', cls = LoadBalancer, ip='10.1.0.1/24')      
        a = self.addHost('a', cls = LoadBalancer, ip='10.100.0.2/24')    
        b = self.addHost('b', cls = LoadBalancer, ip='10.101.0.2/24')    
        c = self.addHost('c', cls = LoadBalancer, ip='10.102.0.2/24')    
        d = self.addHost('d', cls = LoadBalancer, ip='10.103.0.2/24')  
        l2 = self.addHost('l2', cls = LoadBalancer, ip='10.104.0.2/24')    
        e = self.addHost('e', cls = LoadBalancer, ip='10.106.0.2/24')
        f = self.addHost('f', cls = LoadBalancer, ip='10.107.0.2/24')       
        g = self.addHost('g', cls = LoadBalancer, ip='10.108.0.2/24')    
        h = self.addHost('h', cls = LoadBalancer, ip='10.109.0.2/24')    
        i = self.addHost('i', cls = LoadBalancer, ip='10.110.0.2/24')    


        #add switch
        s1 = self.addSwitch('s1')
        s2 = self.addSwitch('s2')

        #connect s1 to l1
        self.addLink(s1, l1, intfName2='l1-eth0', params2={'ip': '10.1.0.1/24'})


        #add router-router links first (so i gets i-eth0, i-eth1 before i-eth2)

        #l1-1 -> a0
        self.addLink(l1, 
                     a, 
                     intfName1='l1-eth1',
                     intfName2='a-eth0',
                     params1={'ip': '10.100.0.1/24'},
                     params2={'ip': '10.100.0.2/24'})

        #l1-2 -> b0
        self.addLink(l1, 
                     b, 
                     intfName1='l1-eth2',
                     intfName2='b-eth0',
                     params1={'ip': '10.101.0.1/24'},
                     params2={'ip': '10.101.0.2/24'})
        #a1 -> c0
        self.addLink(a,
                     c,
                     intfName1='a-eth1',
                     intfName2='c-eth0',
                     params1={'ip': '10.102.0.1/24'},
                     params2={'ip': '10.102.0.2/24'})
        
        #b1 -> d0
        self.addLink(b,
                     d, 
                     intfName1='b-eth1',
                     intfName2='d-eth0',
                     params1={'ip': '10.103.0.1/24'},
                     params2={'ip': '10.103.0.2/24'})
        #c1 -> l2-0
        self.addLink(c,
                     l2, 
                     intfName1='c-eth1',
                     intfName2='l2-eth0',
                     params1={'ip': '10.104.0.1/24'},
                     params2={'ip': '10.104.0.2/24'})

        #d1 -> l2-3
        self.addLink(d,
                     l2, 
                     intfName1='d-eth1',
                     intfName2='l2-eth3',
                     params1={'ip': '10.105.0.1/24'},
                     params2={'ip': '10.105.0.2/24'})
        
        
        #l2-1 -> e0
        self.addLink(l2,
                     e, 
                     intfName1='l2-eth1',
                     intfName2='e-eth0',
                     params1={'ip': '10.106.0.1/24'},
                     params2={'ip': '10.106.0.2/24'})

        #l2-2 -> f0
        self.addLink(l2,
                     f, 
                     intfName1='l2-eth2',
                     intfName2='f-eth0',
                     params1={'ip': '10.107.0.1/24'},
                     params2={'ip': '10.107.0.2/24'})

        #e1 -> g0
        self.addLink(e,
                     g,
                     intfName1='e-eth1',
                     intfName2='g-eth0',
                     params1={'ip': '10.108.0.1/24'},
                     params2={'ip': '10.108.0.2/24'})

        #f1 -> h0
        self.addLink(f,
                     h,
                     intfName1='f-eth1',
                     intfName2='h-eth0',
                     params1={'ip': '10.109.0.1/24'},
                     params2={'ip': '10.109.0.2/24'})

        #g1 -> i0
        self.addLink(g,
                     i,
                     intfName1='g-eth1',
                     intfName2='i-eth0',
                     params1={'ip': '10.110.0.1/24'},
                     params2={'ip': '10.110.0.2/24'})

        #h1 -> i1
        self.addLink(h,
                     i,
                     intfName1='h-eth1',
                     intfName2='i-eth1',
                     params1={'ip': '10.111.0.1/24'},
                     params2={'ip': '10.111.0.2/24'})

        #s2 to i and h2 (add last so i-eth2 is created after router interfaces)
        self.addLink(s2, i, intfName2='i-eth2', params2={'ip': '10.12.0.1/24'})

        #add hosts with a default route
        h1 = self.addHost(name='h1', ip='10.1.0.251/24', defaultRoute='via 10.1.0.1')
        h2 = self.addHost(name='h2', ip='10.12.0.251/24', defaultRoute='via 10.12.0.1')

        #add host-switch links (add s2 links together so switch bridges correctly)
        self.addLink(h1, s1)
        self.addLink(h2, s2)

        
         
def run():
    topo = NetworkTopo()
    net = Mininet(topo=topo)
    
    routers = ['l1', 'l2', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i']
    for r in routers:
        info(net[r].cmd("/usr/lib/frr/frrinit.sh start '{}'".format(r)))
    info(net['l1'].cmd("sysctl -w net.ipv4.fib_multipath_hash_policy=3"))
    info(net['l1'].cmd("sysctl -w net.ipv4.fib_multipath_hash_fields=0x0133"))
    info(net['l2'].cmd("sysctl -w net.ipv4.fib_multipath_hash_policy=3"))
    info(net['l2'].cmd("sysctl -w net.ipv4.fib_multipath_hash_fields=0x0037"))
    info(net['i'].cmd("sysctl -w net.ipv4.fib_multipath_hash_policy=3"))
    info(net['i'].cmd("sysctl -w net.ipv4.fib_multipath_hash_fields=0x0133"))

    for r in routers:
        info(net[r].cmd("sysctl -w net.ipv4.icmp_errors_use_inbound_ifaddr=1"))

    net.start()
    CLI(net)

    for r in routers:
        info(net[r].cmd("/usr/lib/frr/frrinit.sh stop '{}'".format(r)))

    net.stop()

if __name__ == '__main__':
    setLogLevel('info')
    run()
