# FGP Fly.io Daemon

Fast Fly.io operations via FGP daemon. Manage apps, machines, and deployments without MCP cold-start overhead.

## Installation

```bash
git clone https://github.com/fast-gateway-protocol/fly.git
cd fgp-fly
cargo build --release
```

**Requirements:**
- Rust 1.70+
- Fly.io API token (`FLY_API_TOKEN` env var)

## Quick Start

```bash
# Set your Fly.io token
export FLY_API_TOKEN="fo1_xxxxx"

# Start the daemon
./target/release/fgp-fly start

# List your apps
fgp call fly.apps

# Get app status
fgp call fly.status '{"app": "my-app"}'

# List machines
fgp call fly.machines '{"app": "my-app"}'

# Stop daemon
./target/release/fgp-fly stop
```

## Available Methods

| Method | Params | Description |
|--------|--------|-------------|
| `fly.apps` | `limit` (default: 25) | List all Fly.io apps |
| `fly.status` | `app` (required) | Get status for a specific app |
| `fly.machines` | `app` (required) | List machines for an app |
| `fly.user` | - | Get current user info |

## FGP Protocol

Socket: `~/.fgp/services/fly/daemon.sock`

**Request:**
```json
{"id": "uuid", "v": 1, "method": "fly.apps", "params": {"limit": 10}}
```

**Response:**
```json
{"id": "uuid", "ok": true, "result": {"apps": [...], "count": 5}}
```

## Why FGP?

| Operation | FGP Daemon | MCP stdio | Speedup |
|-----------|------------|-----------|---------|
| List apps | ~200ms | ~2,500ms | **12x** |
| App status | ~150ms | ~2,400ms | **16x** |

FGP keeps the API connection warm, eliminating cold-start overhead.

## Troubleshooting

### Invalid API Token

**Symptom:** Requests fail with 401 or "unauthorized"

**Solutions:**
1. Verify token is set: `echo $FLY_API_TOKEN`
2. Check token is valid: `curl -H "Authorization: Bearer $FLY_API_TOKEN" https://api.fly.io/graphql`
3. Generate new token at https://fly.io/user/personal_access_tokens

### App Not Found

**Symptom:** "App not found" error for existing app

**Check:**
1. App name is correct (case-sensitive)
2. Token has access to the organization owning the app
3. Try listing apps first: `fgp call fly.apps`

### Rate Limiting

**Symptom:** 429 errors or slow responses

**Solutions:**
1. Fly.io has rate limits on API calls
2. Add delays between bulk operations
3. Cache results when possible

### Empty Machine List

**Symptom:** `fly.machines` returns empty for deployed app

**Check:**
1. App uses Fly Machines (not Nomad/legacy)
2. Machines exist: `fly machines list -a <app-name>`
3. Token has correct permissions

### Connection Refused

**Symptom:** "Connection refused" when calling daemon

**Solution:**
```bash
# Check daemon is running
pgrep -f fgp-fly

# Restart daemon
./target/release/fgp-fly stop
export FLY_API_TOKEN="fo1_xxxxx"
./target/release/fgp-fly start

# Verify socket exists
ls ~/.fgp/services/fly/daemon.sock
```

### GraphQL Errors

**Symptom:** Errors mentioning GraphQL or query failures

**Check:**
1. Fly.io API status: https://status.fly.io
2. Token permissions (some queries need org admin)
3. Try simpler query first: `fly.user`

## License

MIT
