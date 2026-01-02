#!/usr/bin/env bash
multipass launch 22.04 \
  --name example-vm2 \
  --cpus 4 \
  --memory 8G \
  --disk 32G \
  --cloud-init cloud-init.yaml
multipass shell example-vm2

