# AgentShield

Before your agent signs, AgentShield scores the risk.

AgentShield is a pre-signing transaction risk analyzer for autonomous blockchain agents. It decodes unsigned transactions, checks known and unknown addresses, applies deterministic rule-based scoring, and returns a structured allow/review/reject recommendation.

AgentShield is designed as a small paid safety service for agent builders. The core service works as a normal HTTP API, and can later be exposed through MCP and placed behind an x402 payment gateway.

## Selling line

Before your agent signs, AgentShield scores the risk.

Or:

x402 handles the payment. AgentShield handles the judgment.

## Why AgentShield?

Autonomous agents are starting to use wallets, APIs, and paid services. But wallet-enabled agents should not blindly sign blockchain transactions.

A malicious tool, prompt injection, bad route, fake contract, or unsafe approval can cause an agent to sign something dangerous.

AgentShield acts as a pre-signing safety layer:

- Decode the unsigned transaction
- Identify what the transaction is trying to do
- Check known and unknown addresses
- Apply deterministic risk rules
- Return a clear recommendation: allow, review, or reject

## Current MVP

The current MVP is a deterministic transaction-risk linter for agents.

It supports:

- Native token transfer detection
- ERC-20 approve(address,uint256) decoding
- ERC-20 transfer(address,uint256) decoding
- Unknown calldata detection
- Malformed calldata fallback
- Hardcoded known/trusted address registry
- Rule-based risk scoring
- Structured JSON risk reports
- Integration tests
- Demo example payloads
- Demo script

## Current architecture

HTTP request
  -> decoder
  -> address registry
  -> risk engine
  -> JSON report

Current source structure:

src/
  lib.rs
  main.rs
  handlers.rs
  types.rs
  decoder.rs
  registry.rs
  risk.rs

## Run the service

Start the Rust API:

cargo run

By default, AgentShield runs on:

http://localhost:3000

Health check:

curl http://localhost:3000/health

Expected response:

ok

## Analyze a transaction

Example:

```sh
curl -X POST http://localhost:3000/analyze \
  -H "Content-Type: application/json" \
  -d @examples/unlimited_approval_unknown_spender.json | jq
```

## Run demo examples

Run all curated demo examples:

```sh
./scripts/run_examples.sh
```

The script demonstrates:

1. Safe native transfer
2. ERC-20 transfer to unknown recipient
3. Unlimited ERC-20 approval to unknown spender
4. Unknown contract call

## API

### POST /analyze

Request body:

```json
{
  "chain_id": 1,
  "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
  "value": "0",
  "data": "0x095ea7b30000000000000000000000001111111111111111111111111111111111111111ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
}
```

Fields:

- chain_id: blockchain chain ID
- from: sender address
- to: target address
- value: native token amount as string
- data: transaction calldata

Response body:

```json
{
  "risk_score": 95,
  "risk_level": "critical",
  "recommendation": "reject",
  "summary": "Unlimited ERC-20 approval to unknown spender",
  "decoded_action": {
    "type": "erc20_approve",
    "token": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
    "spender": "0x1111111111111111111111111111111111111111",
    "amount": "unlimited"
  },
  "findings": [
    {
      "code": "ERC20_APPROVAL",
      "severity": "high",
      "score_impact": 20,
      "title": "ERC-20 approval detected",
      "description": "The transaction grants another address permission to spend tokens."
    },
    {
      "code": "UNLIMITED_APPROVAL",
      "severity": "critical",
      "score_impact": 50,
      "title": "Unlimited token approval",
      "description": "The spender can move all current and future token balance."
    },
    {
      "code": "UNKNOWN_SPENDER",
      "severity": "high",
      "score_impact": 25,
      "title": "Unknown spender",
      "description": "The spender is not in the trusted address registry."
    }
  ]
}
```
## Risk levels

0-20: low
21-50: medium
51-80: high
81-100: critical

## Recommendations

allow:
  The transaction looks safe enough for the agent to sign automatically.

review:
  The transaction is not clearly malicious, but it should require additional checks or human review.

reject:
  The transaction is dangerous enough that the agent should not sign it by default.

## Example scoring rules

Native transfer: +10
Unknown recipient: +20
ERC-20 approval: +20
Unlimited approval: +50
Unknown spender: +25
ERC-20 transfer: +20
Unknown calldata: +40
Unknown contract: +25
Trusted spender: -15
Trusted recipient: -15
Trusted native recipient: -10

The final score is clamped to the range 0-100.

