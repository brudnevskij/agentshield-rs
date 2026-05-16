#!/usr/bin/env bash
set -euo pipefail

CONFIG_PATH="${CONFIG_PATH:-config/facilitator.base-sepolia.json}"

if [ -f ".env" ]; then
  set -a
  source .env
  set +a
fi

if [ -z "${FACILITATOR_PRIVATE_KEY:-}" ]; then
  echo "error: FACILITATOR_PRIVATE_KEY is not set"
  echo "create .env from .env.example and use a fresh Base Sepolia test wallet"
  exit 1
fi

echo "Starting x402 facilitator"
echo "Config: $CONFIG_PATH"
echo "URL:    http://127.0.0.1:8080"
echo

exec x402-facilitator --config "$CONFIG_PATH"
