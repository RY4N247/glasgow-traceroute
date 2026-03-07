#!/usr/bin/env bash
multipass launch 22.04 \
  --name example-vm \
  --cpus 2 \
  --memory 2G \
  --disk 20G \
  --cloud-init cloud-init.yaml
multipass shell example-vm

