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
#              a
#             / \          
#           l2   e
#          /  \ / \
#         /    b   \
# h1-s1-l1          g-s2-h2  
#         \    c   /
#          \  / \ /
#           l3   f
#             \ /
#              d
class NetworkTopo(Topo):
    def build(self, **_opts):
        #add routers - ip= must match the FIRST interface that will be created
        l1 = self.addHost('l1', cls = LoadBalancer, ip='10.1.0.1/24')     
        l2 = self.addHost('l2', cls = LoadBalancer, ip='10.100.0.2/24')
        l3 = self.addHost('l3', cls = LoadBalancer, ip='10.101.0.2/24')
        a = self.addHost('a', cls = LoadBalancer, ip='10.102.0.2/24')    
        b = self.addHost('b', cls = LoadBalancer, ip='10.103.0.2/24')    
        c = self.addHost('c', cls = LoadBalancer, ip='10.104.0.2/24')    
        d = self.addHost('d', cls = LoadBalancer, ip='10.105.0.2/24')    
        e = self.addHost('e', cls = LoadBalancer, ip='10.106.0.2/24')      
        f = self.addHost('f', cls = LoadBalancer, ip='10.108.0.2/24')    
        g = self.addHost('g', cls = LoadBalancer, ip='10.12.0.1/24')   
      
        #add switch
        s1 = self.addSwitch('s1')
        s2 = self.addSwitch('s2')

        #connect switches to routers l1 and g
        self.addLink(s1, l1, intfName2='l1-eth0', params2={'ip': '10.1.0.1/24'})
        self.addLink(s2, g, intfName2='g-eth1', params2={'ip': '10.12.0.1/24'})

        #add router-router links

        # self.addLink(l1, l2, intfName1='l1-eth1', intfName2='l2-eth0',
        #      params1={'ip': '10.100.0.1/24'}, params2={'ip': '10.100.0.2/24'},
        #      bw=100)
        # self.addLink(l1, l3, intfName1='l1-eth2', intfName2='l3-eth0', 
        #      params1={'ip': '10.101.0.1/24'}, params2={'ip': '10.101.0.2/24'},
        #      bw=100)

        # #l1 -> l2
        self.addLink(l1,
                     l2,
                     intfName1='l1-eth2',
                     intfName2='l2-eth0',
                     params1={'ip': '10.100.0.1/24'},
                     params2={'ip': '10.100.0.2/24'})

        #l1 -> l3
        self.addLink(l1,
                     l3,
                     intfName1='l1-eth1',
                     intfName2='l3-eth0',
                     params1={'ip': '10.101.0.1/24'},
                     params2={'ip': '10.101.0.2/24'})
        #l2 -> a
        self.addLink(l2,
                     a,
                     intfName1='l2-eth2',
                     intfName2='a-eth0',
                     params1={'ip': '10.102.0.1/24'},
                     params2={'ip': '10.102.0.2/24'})
                     
        #l2 -> b
        self.addLink(l2,
                     b,
                     intfName1='l2-eth1',
                     intfName2='b-eth0',
                     params1={'ip': '10.103.0.1/24'},
                     params2={'ip': '10.103.0.2/24'})            

        #l3 -> c
        self.addLink(l3,
                     c,
                     intfName1='l3-eth2',
                     intfName2='c-eth0',
                     params1={'ip': '10.104.0.1/24'},
                     params2={'ip': '10.104.0.2/24'})   
        
        #l3 -> d
        self.addLink(l3,
                     d,
                     intfName1='l3-eth1',
                     intfName2='d-eth0',
                     params1={'ip': '10.105.0.1/24'},
                     params2={'ip': '10.105.0.2/24'})  

        #a -> e
        self.addLink(a,
                     e,
                     intfName1='a-eth1',
                     intfName2='e-eth0',
                     params1={'ip': '10.106.0.1/24'},
                     params2={'ip': '10.106.0.2/24'})  
        
        #b -> e
        self.addLink(b,
                     e,
                     intfName1='b-eth1',
                     intfName2='e-eth2',
                     params1={'ip': '10.107.0.1/24'},
                     params2={'ip': '10.107.0.2/24'})  
                     
        #c -> f
        self.addLink(c,
                     f,
                     intfName1='c-eth1',
                     intfName2='f-eth0',
                     params1={'ip': '10.108.0.1/24'},
                     params2={'ip': '10.108.0.2/24'}) 

        #d -> f
        self.addLink(d,
                     f,
                     intfName1='d-eth1',
                     intfName2='f-eth2',
                     params1={'ip': '10.109.0.1/24'},
                     params2={'ip': '10.109.0.2/24'}) 

        #e -> g
        self.addLink(e,
                     g,
                     intfName1='e-eth1',
                     intfName2='g-eth0',
                     params1={'ip': '10.110.0.1/24'},
                     params2={'ip': '10.110.0.2/24'}) 
        #f -> g
        self.addLink(f,
                     g,
                     intfName1='f-eth1',
                     intfName2='g-eth2', 
                     params1={'ip': '10.111.0.1/24'},
                     params2={'ip': '10.111.0.2/24'}) 

        #add hosts with a default route
        h1 = self.addHost(name='h1', ip='10.1.0.251/24', defaultRoute='via 10.1.0.1')
        h2 = self.addHost(name='h2', ip='10.12.0.251/24', defaultRoute='via 10.12.0.1')

        #add host-switch links
        self.addLink(h1, s1)
        self.addLink(h2, s2)

        
         
def run():
    topo = NetworkTopo()
    net = Mininet(topo=topo)

    routers = ['l1', 'l2', 'l3', 'a', 'b', 'c', 'd', 'e', 'f', 'g']
    for r in routers:
        info(net[r].cmd("/usr/lib/frr/frrinit.sh start '{}'".format(r)))
    info(net['l1'].cmd("sysctl -w net.ipv4.fib_multipath_hash_policy=3"))
    info(net['l1'].cmd("sysctl -w net.ipv4.fib_multipath_hash_fields=0x0133"))
    info(net['l2'].cmd("sysctl -w net.ipv4.fib_multipath_hash_policy=3"))
    info(net['l2'].cmd("sysctl -w net.ipv4.fib_multipath_hash_fields=0x0037"))
    info(net['l3'].cmd("sysctl -w net.ipv4.fib_multipath_hash_policy=3"))
    info(net['l3'].cmd("sysctl -w net.ipv4.fib_multipath_hash_fields=0x0037"))
    info(net['e'].cmd("sysctl -w net.ipv4.fib_multipath_hash_policy=3"))
    info(net['e'].cmd("sysctl -w net.ipv4.fib_multipath_hash_fields=0x0027"))
    info(net['f'].cmd("sysctl -w net.ipv4.fib_multipath_hash_policy=3"))
    info(net['f'].cmd("sysctl -w net.ipv4.fib_multipath_hash_fields=0x0036"))
    info(net['g'].cmd("sysctl -w net.ipv4.fib_multipath_hash_policy=3"))
    info(net['g'].cmd("sysctl -w net.ipv4.fib_multipath_hash_fields=0x0133"))

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

