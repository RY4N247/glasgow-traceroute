# Load Balanced Network Example
## Instructions:
1. Ensure multipass is installed on your system. For instructions see https://canonical.com/multipass/install. Note: Windows is not currently supported, including when using Multipass.  
2. Once multipass is installed, run `bash run_vm.sh` from within `glasgow-traceroute/example` to create the virtual machine. 

```bash
bash run_vm.sh
```

3. After the script completes its initialisation you will be within the multipass shell. 
4. From here, `cd /home/ubuntu/git/glasgow-traceroute/example` and run `bash install_frr.sh`. This will install everything needed for the load balanced network example that was otherwise not included in `cloud-init.yaml`. Please be patient and answer `y` to any prompts or `enter` to any outdated messages.

```bash
cd /home/ubuntu/git/glasgow-traceroute/example
bash install_frr.sh
```

5. After installation completes, we can compile glasgow-traceroute to use within mininet. `cd /home/ubuntu/git/glasgow-traceroute` and run `bash run_raw.sh` to build glasgow-traceroute with the required permissions.

```bash
cd /home/ubuntu/git/glasgow-traceroute
bash run_raw.sh
```

6. Then back to `example` and choose one topology directory: `load_balanced_network_a`, `load_balanced_network_b`, or `load_balanced_network_c`. `cd` into your chosen directory and run `bash run_network.sh`. This will set up the selected load balanced network topology using FRR. Note: it may take a few minutes for ospfd to find its neighbors and establish adjacencies. 

```bash
cd /home/ubuntu/git/glasgow-traceroute/example/load_balanced_network_a
bash run_network.sh
```

7. You will now be within mininet. From the mininet prompt `mininet>`, run `pingall` to verify connectivity between all hosts. If successful, you should see:

```bash
mininet> pingall
```

```text
*** Ping: testing ping reachability
a  -> b c d e h1 h2 l
b  -> a c d e h1 h2 l
c  -> a b d e h1 h2 l
d  -> a b c e h1 h2 l
e  -> a b c d h1 h2 l
h1 -> a b c d e h2 l
h2 -> a b c d e h1 l
l  -> a b c d e h1 h2
*** Results: 0% dropped (56/56 received)
```
8. (OPTIONAL) - To observe an ASCII representation of the traceroute paths taken follow the README.md guide found in `glasgow-traceroute/src/pycall/README.md` to install the required python virtual environment.
9. We can now run glasgow-traceroute using `./target/debug/glasgow-traceroute`. An example is shown below, tracing from host `h1` to host `h2`:

### Example
Speed measurements may vary:

```bash
mininet> h1 /home/ubuntu/git/glasgow-traceroute/target/debug/glasgow-traceroute traceroute icmp h2
```

```text
traceroute to 10.6.0.251 (10.6.0.251), 30 hops max
 1  10.1.0.1  1.902 ms
 2  10.100.0.2  0.377 ms
 3  10.102.0.2  0.303 ms
 4  10.104.0.2  0.317 ms
 5  10.6.0.251  0.785 ms
```

Only one Mininet instance (`load_balanced_network_a`, `load_balanced_network_b`, or `load_balanced_network_c`) can run at a time. Exit cleanly from each before starting another:

```bash
mininet> exit
```
