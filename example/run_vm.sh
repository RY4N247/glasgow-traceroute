#!/usr/bin/env bash
multipass launch 22.04 \
  --name example-vm \
  --cpus 4 \
  --memory 8G \
  --disk 32G \
  --cloud-init cloud-init.yaml
multipass shell example-vm

