#!/usr/bin/env bash
set -euo pipefail

API="${API:-http://127.0.0.1:3000}"

echo "== Free /analyze =="
curl -s "$API/analyze" \
  -H 'content-type: application/json' \
  -d @examples/unlimited_approval_unknown_spender.json | jq

echo
echo "== Paid /x402/analyze without payment =="
curl -i "$API/x402/analyze" \
  -H 'content-type: application/json' \
  -d @examples/unlimited_approval_unknown_spender.json
