#!/usr/bin/env bash
multipass launch 22.04 \
  --name example-vm \
  --memory 2G \
  --disk 10G \
  --cloud-init cloud-init.yaml
multipass shell example-vm
1
