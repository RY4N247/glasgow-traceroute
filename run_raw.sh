#!/usr/bin/env bash
#Simple Linux build script to compile the Rust project and set the CAP_NET_RAW capability on the resulting binary.
set -e
BIN_NAME="glasgow-traceroute"
BIN_PATH="target/debug/$BIN_NAME"

echo "[*] Building project..."
cargo build

if [ ! -f "$BIN_PATH" ]; then
    echo "[!] Binary not found at $BIN_PATH"
    exit 1
fi

echo "[*] Granting CAP_NET_RAW to $BIN_PATH"
sudo setcap cap_net_raw+ep "$BIN_PATH"

echo "[*] Capabilities set:"
getcap "$BIN_PATH"

echo
echo "Done"
echo "Run the program with:"
echo "    ./$BIN_PATH"