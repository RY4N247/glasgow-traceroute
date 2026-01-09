#!/bin/bash
cd "$(dirname "$0")/../.."
rm -rf .venv 2>/dev/null || sudo rm -rf .venv 2>/dev/null || true
