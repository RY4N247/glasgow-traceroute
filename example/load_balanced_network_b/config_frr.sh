#!/bin/bash 

for NODE in l1 l2 l3 a b c d e f g
do
    sudo install -m 775 -o frr -g frr -d /var/log/frr/${NODE}
    sudo install -m 775 -o frr -g frrvty -d /etc/frr/${NODE}
    sudo install -m 640 -o frr -g frrvty ${NODE}/vtysh.conf \
	    /etc/frr/${NODE}/vtysh.conf
    sudo install -m 640 -o frr -g frr ${NODE}/frr.conf \
	    /etc/frr/${NODE}/frr.conf
    sudo install -m 640 -o frr -g frr ${NODE}/daemons  \
	    /etc/frr/${NODE}/daemons
done    
