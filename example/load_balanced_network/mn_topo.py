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
                                            


# Figure 1 from https://dl.acm.org/doi/pdf/10.1145/1177080.1177100
#            a---c
#           /     \
#   h1--s1--l      e--s2--h2
#           \     /
#            b---d

class NetworkTopo(Topo):
    def build(self, **_opts):
        #add routers - ip= must match the FIRST interface that will be created
        l = self.addHost('l', cls = LoadBalancer, ip='10.1.0.1/24')      # l-eth0 to s1
        a = self.addHost('a', cls = LoadBalancer, ip='10.100.0.2/24')    # a-eth0 to l
        b = self.addHost('b', cls = LoadBalancer, ip='10.101.0.2/24')    # b-eth0 to l
        c = self.addHost('c', cls = LoadBalancer, ip='10.102.0.2/24')    # c-eth0 to a
        d = self.addHost('d', cls = LoadBalancer, ip='10.103.0.2/24')    # d-eth0 to b
        e = self.addHost('e', cls = LoadBalancer, ip='10.6.0.1/24')      # e-eth2 to s2

        #add switch
        s1 = self.addSwitch('s1')
        s2 = self.addSwitch('s2')

        #connect switches to routers l and e
        self.addLink(s1, l, intfName2='l-eth0', params2={'ip': '10.1.0.1/24'})
        self.addLink(s2, e, intfName2='e-eth2', params2={'ip': '10.6.0.1/24'})

        #add router-router links

        #l1 -> a0
        self.addLink(l, 
                     a, 
                     intfName1='l-eth1',
                     intfName2='a-eth0',
                     params1={'ip': '10.100.0.1/24'},
                     params2={'ip': '10.100.0.2/24'})
        #l2 -> b0
        self.addLink(l, 
                     b, 
                     intfName1='l-eth2',
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
        #c1 -> e0
        self.addLink(c,
                     e, 
                     intfName1='c-eth1',
                     intfName2='e-eth0',
                     params1={'ip': '10.104.0.1/24'},
                     params2={'ip': '10.104.0.2/24'})
        
        #d1 -> e1
        self.addLink(d,
                     e, 
                     intfName1='d-eth1',
                     intfName2='e-eth1',
                     params1={'ip': '10.105.0.1/24'},
                     params2={'ip': '10.105.0.2/24'})
        
                     

        #add hosts with a default route
        h1 = self.addHost(name='h1', ip='10.1.0.251/24', defaultRoute='via 10.1.0.1')
        h2 = self.addHost(name='h2', ip='10.6.0.251/24', defaultRoute='via 10.6.0.1')

        #add host-switch links
        self.addLink(h1, s1)
        self.addLink(h2, s2)

        
         
def run():
    topo = NetworkTopo()
    net = Mininet(topo=topo)
    
    # frr for all routers
    #TODO: change to for loop once working 
    info(net['l'].cmd("/usr/lib/frr/frrinit.sh start 'l'"))
    info(net['l'].cmd("sysctl -w net.ipv4.fib_multipath_hash_policy=1")) # enable layer 4 hashing
    info(net['a'].cmd("/usr/lib/frr/frrinit.sh start 'a'"))
    info(net['b'].cmd("/usr/lib/frr/frrinit.sh start 'b'"))
    info(net['c'].cmd("/usr/lib/frr/frrinit.sh start 'c'"))
    info(net['d'].cmd("/usr/lib/frr/frrinit.sh start 'd'"))
    info(net['e'].cmd("/usr/lib/frr/frrinit.sh start 'e'"))
    info(net['e'].cmd("sysctl -w net.ipv4.fib_multipath_hash_policy=1")) # enable layer 4 hashing

    net.start()
    CLI(net)

    info(net['l'].cmd("/usr/lib/frr/frrinit.sh stop 'l'"))
    info(net['a'].cmd("/usr/lib/frr/frrinit.sh stop 'a'"))
    info(net['b'].cmd("/usr/lib/frr/frrinit.sh stop 'b'"))
    info(net['c'].cmd("/usr/lib/frr/frrinit.sh stop 'c'"))
    info(net['d'].cmd("/usr/lib/frr/frrinit.sh stop 'd'"))
    info(net['e'].cmd("/usr/lib/frr/frrinit.sh stop 'e'"))


    net.stop()

if __name__ == '__main__':
    setLogLevel('info')
    run()

