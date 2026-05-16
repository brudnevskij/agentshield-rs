#!/usr/bin/env bash
set -euo pipefail

echo "Installing x402-facilitator from x402-rs..."

cargo install \
  --git https://github.com/x402-rs/x402-rs \
   x402-facilitator \
  --features chain-eip155 \
  --locked

echo
echo "Installed:"
command -v x402-facilitator
