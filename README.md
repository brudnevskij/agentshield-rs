# AgentShield

Before your agent signs, AgentShield scores the risk.

AgentShield is a pre-signing transaction risk analyzer for autonomous blockchain agents. It decodes unsigned transactions, checks known and unknown addresses, applies deterministic rule-based scoring, and returns a structured `allow` / `review` / `reject` recommendation.

AgentShield can run as:

- a normal HTTP API
- an MCP tool for agent runtimes
- an x402-paid service

Selling line:

```text
x402 handles the payment. AgentShield handles the judgment.
```

## What it does

AgentShield currently supports:

- Native token transfer detection
- ERC-20 `approve(address,uint256)` decoding
- ERC-20 `transfer(address,uint256)` decoding
- Unlimited approval detection
- Unknown calldata detection
- Malformed calldata fallback
- Hardcoded known/trusted address registry
- Rule-based risk scoring
- Structured JSON risk reports
- MCP integration
- x402-paid endpoint with local facilitator

## Architecture

```text
Agent / client
  -> AgentShield HTTP API or MCP tool
  -> decoder
  -> address registry
  -> risk engine
  -> JSON risk report
```

x402 flow:

```text
Agent / x402 client
  -> POST /x402/analyze
  -> AgentShield x402 middleware
  -> x402 facilitator
  -> payment verification / settlement
  -> risk analysis
  -> JSON risk report
```

## Endpoints

```text
GET  /health
POST /analyze
POST /x402/analyze
```

`/analyze` is the normal free endpoint.

`/x402/analyze` is protected by x402 payment middleware.

## Configuration

Create `.env`:

```env
# AgentShield x402 server config
X402_FACILITATOR_URL=http://127.0.0.1:8080
X402_PAY_TO=0xYOUR_RECEIVER_ADDRESS

# Facilitator settlement wallet
FACILITATOR_PRIVATE_KEY=0xYOUR_FACILITATOR_PRIVATE_KEY

# Client wallet used by the x402 demo client / paid MCP server
X402_CLIENT_PRIVATE_KEY=0xYOUR_CLIENT_PRIVATE_KEY

# Optional
AGENTSHIELD_API_URL=http://127.0.0.1:3000/analyze
AGENTSHIELD_X402_API_URL=http://127.0.0.1:3000/x402/analyze
```

Use test wallets only. Do not use real private keys.

## Run AgentShield

```sh
cargo run
```

Health check:

```sh
curl http://127.0.0.1:3000/health
```

Expected:

```text
ok
```

## Analyze a transaction

```sh
curl -s http://127.0.0.1:3000/analyze \
  -H "content-type: application/json" \
  -d @examples/unlimited_approval_unknown_spender.json | jq
```

## Run demo examples

```sh
./scripts/run_examples.sh
```

The demo examples show:

1. Safe native transfer
2. ERC-20 transfer to unknown recipient
3. Unknown contract call
4. Unlimited ERC-20 approval to unknown spender

## x402 facilitator

AgentShield uses a local x402 facilitator for payment verification and settlement.

Config file:

```text
config/facilitator.base-sepolia.json
```

Working Base Sepolia config:

```json
{
  "port": 8080,
  "host": "127.0.0.1",
  "chains": {
    "eip155:84532": {
      "_comment": "Base Sepolia",
      "eip1559": true,
      "flashblocks": true,
      "signers": [
        "$FACILITATOR_PRIVATE_KEY"
      ],
      "rpc": [
        {
          "http": "https://sepolia.base.org",
          "rate_limit": 50
        }
      ]
    }
  },
  "schemes": [
    {
      "id": "v1-eip155-exact",
      "chains": "eip155:*"
    },
    {
      "id": "v2-eip155-exact",
      "chains": "eip155:*"
    },
    {
      "id": "v2-eip155-upto",
      "chains": "eip155:*"
    }
  ]
}
```

Run facilitator:

```sh
./scripts/run_facilitator.sh
```

Test facilitator:

```sh
curl -s http://127.0.0.1:8080/health
curl -s http://127.0.0.1:8080/supported | jq
```

## Test x402 endpoint

Start facilitator:

```sh
./scripts/run_facilitator.sh
```

Start AgentShield:

```sh
cargo run
```

Call paid endpoint without payment:

```sh
curl -i http://127.0.0.1:3000/x402/analyze \
  -H "content-type: application/json" \
  -d @examples/unlimited_approval_unknown_spender.json
```

Expected:

```text
HTTP/1.1 402 Payment Required
```

Run the paid x402 client demo:

```sh
cargo run --bin x402-client-demo
```

Expected success:

```text
Status: 200 OK
```

followed by an AgentShield risk report.

## MCP

AgentShield provides two MCP binaries:

```text
agentshield-mcp        -> calls /analyze
agentshield-mcp-x402   -> calls /x402/analyze and pays through x402
```

Build them:

```sh
cargo build --bin agentshield-mcp
cargo build --bin agentshield-mcp-x402
```

Example MCP config:

```toml
[mcp_servers.agentshield]
command = "/absolute/path/to/agentshield-rs/target/debug/agentshield-mcp"
args = []
startup_timeout_sec = 10
tool_timeout_sec = 10

[mcp_servers.agentshield.env]
AGENTSHIELD_API_URL = "http://127.0.0.1:3000/analyze"


[mcp_servers.agentshield_x402]
command = "/absolute/path/to/agentshield-rs/target/debug/agentshield-mcp-x402"
args = []
startup_timeout_sec = 10
tool_timeout_sec = 30

[mcp_servers.agentshield_x402.env]
AGENTSHIELD_X402_API_URL = "http://127.0.0.1:3000/x402/analyze"
X402_CLIENT_PRIVATE_KEY = "0xYOUR_CLIENT_PRIVATE_KEY"
```

The free MCP tool exposes:

```text
analyze_transaction_risk
```

The paid x402 MCP tool exposes:

```text
analyze_transaction_risk_paid
```

## Example MCP prompt

```text
Call the AgentShield x402 MCP tool analyze_transaction_risk_paid now.

Scenario:
An autonomous DeFi agent is preparing to sign a token approval transaction. The agent cannot rely on its own guess. Before signing, it must pay for an AgentShield analysis using x402 and follow the returned recommendation.

Input:
chain_id: 1
from: 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
to: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48
value: 0
data: 0x095ea7b30000000000000000000000001111111111111111111111111111111111111111ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff

After the tool returns, answer in this exact format:

Payment:
Transaction type:
Risk score:
Risk level:
Recommendation:
Should the agent sign?:
Main reason:
Agent decision:

Important:
First call analyze_transaction_risk_paid.
Do not infer the result yourself.
If AgentShield recommends reject, the agent must not sign.
```

## API

### POST `/analyze`

Request:

```json
{
  "chain_id": 1,
  "from": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
  "value": "0",
  "data": "0x095ea7b30000000000000000000000001111111111111111111111111111111111111111ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
}
```

Response:

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

```text
0-20    low
21-50   medium
51-80   high
81-100  critical
```

## Recommendations

```text
allow   -> safe enough for automatic signing
review  -> needs more checks or human review
reject  -> dangerous; agent should not sign by default
```

## Example scoring rules

```text
Native transfer             +10
Unknown recipient           +20
ERC-20 approval             +20
Unlimited approval          +50
Unknown spender             +25
ERC-20 transfer             +20
Unknown calldata            +40
Unknown contract            +25
Trusted spender             -15
Trusted recipient           -15
Trusted native recipient    -10
```

Final score is clamped to `0..100`.

## Tests

```sh
cargo test
```

## Demo story

```text
Codex / agent is not trusted to guess transaction safety.
Before signing, it must call AgentShield.
If using the paid endpoint, it first pays through x402.
Then AgentShield returns a deterministic risk judgment.
```
