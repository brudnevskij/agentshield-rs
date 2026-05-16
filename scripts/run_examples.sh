#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:3000}"

if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required but not installed."
  exit 1
fi

echo "========================================"
echo " AgentShield demo examples"
echo " API: $BASE_URL"
echo "========================================"

run_example() {
  local title="$1"
  local file="$2"
  local expected="$3"

  echo
  echo "----------------------------------------"
  echo "$title"
  echo "File: $file"
  echo "Expected: $expected"
  echo "----------------------------------------"

  response="$(
    curl -sS -X POST "$BASE_URL/analyze" \
      -H "Content-Type: application/json" \
      -d @"$file"
  )"

  echo "$response" | jq -r '
    "Risk: \(.risk_score)/100 [\(.risk_level)]",
    "Recommendation: \(.recommendation)",
    "Summary: \(.summary)",
    "",
    "Decoded action:",
    (.decoded_action | tojson),
    "",
    "Findings:",
    (.findings[] | "  - \(.code): \(.title) (\(.severity), impact \(.score_impact))")
  '
}

run_example \
  "1. Safe native transfer" \
  "examples/safe_native_transfer.json" \
  "low risk / allow"

run_example \
  "2. ERC-20 transfer to unknown recipient" \
  "examples/erc20_transfer_unknown_recipient.json" \
  "medium risk / review"

run_example \
  "3. Unlimited approval to unknown spender" \
  "examples/unlimited_approval_unknown_spender.json" \
  "critical risk / reject"

run_example \
  "4. Unknown contract call" \
  "examples/unknown_contract_call.json" \
  "high risk / review"

echo
echo "================================--------"
echo " Demo finished"
echo "========================================"
