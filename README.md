# FGP Fly.io Daemon

Fast Fly.io operations via FGP daemon. Manage apps, machines, and deployments without MCP cold-start overhead.

## Installation

```bash
git clone https://github.com/wolfiesch/fgp-fly.git
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

## License

MIT
