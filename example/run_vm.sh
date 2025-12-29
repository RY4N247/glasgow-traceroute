#!/usr/bin/env bash
multipass launch 22.04 \
  --name example-vm1 \
  --memory 2G \
  --disk 10G \
  --cloud-init cloud-init.yaml
multipass shell example-vm1

