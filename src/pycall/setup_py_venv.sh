#!/bin/bash
cd "$(dirname "$0")/../.."
rm -rf .venv 2>/dev/null || sudo rm -rf .venv 2>/dev/null || true
python3 -m venv .venv
.venv/bin/pip install -r src/pycall/requirements.txt
