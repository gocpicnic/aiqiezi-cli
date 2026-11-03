# aiqiezi-cli

Command-line client for the aiqiezi single-backend photo and video archive.

## Why iroh

The target server is a self-hosted machine behind NAT without a static public
IP, public domain, port forwarding, or TLS certificate.

The CLI therefore uses:

- iroh JSON for metadata and control;
- iroh-blobs for media transfer;
- the server EndpointId or endpoint ticket for connection setup.

No HTTP endpoint is required.

## Status

Early design phase. The workspace and client library are not implemented yet.

Planned workspace:

```text
aiqiezi-cli/
├── Cargo.toml
├── crates/
│   ├── aiqiezi-client/        # shared iroh client library
│   └── aiqiezi-cli/           # CLI binary
└── AGENTS.md
```

`aiqiezi-client` is shared with the future egui GUI. The CLI is a thin wrapper
around that library.

## Client enrollment

There is no login, password, or session-token flow.

The client has a persistent iroh secret key and a stable EndpointId:

```bash
aiqiezi whoami
```

The server administrator authorizes that EndpointId:

```bash
aiqiezi-server client-add <endpoint-id> --label "laptop"
```

To revoke access, disable or remove the EndpointId on the server:

```bash
aiqiezi-server client-disable <endpoint-id>
```

## Planned commands

```bash
aiqiezi whoami
aiqiezi server-info
aiqiezi folder list --parent <id>
aiqiezi folder tree --path /photos
aiqiezi asset list --cursor <cursor>
aiqiezi asset get <id>
aiqiezi asset search "summer"
aiqiezi upload ./photo.jpg --parent <id>
aiqiezi download <content-hash> --output ./photo.jpg
```

## Machine-readable output

The CLI is designed for scripts and AI agents:

```bash
aiqiezi asset list --json
aiqiezi asset search "summer" --json
aiqiezi folder tree --path /photos --json
```

Rules:

- stdout contains data only;
- stderr contains errors, warnings, and progress;
- JSON output uses stable schemas;
- batch commands may use NDJSON;
- exit code is nonzero on failure;
- no interactive prompts in agent mode.

A future `SKILL.md` will document agent usage, output schemas, enrollment, and
safe destructive operations.

## Configuration

Planned environment variables:

```text
AIQIEZI_SERVER_ID
AIQIEZI_ENDPOINT_TICKET
AIQIEZI_RELAY_URL
AIQIEZI_RELAY_TOKEN
AIQIEZI_DATA_DIR
```

The client secret key lives under:

```text
AIQIEZI_DATA_DIR/key_data/secret_key.bin
```

Back up this key. Losing it changes the client EndpointId, and the new
EndpointId must be enrolled on the server.

## Development

Planned commands:

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p aiqiezi-cli
```

## Documentation

- [AGENTS.md](AGENTS.md): implementation rules for AI coding agents
- `../aiqiezi-server/README.md`: system architecture and backend setup
- `../aiqiezi-server/docs/protocol.md`: iroh JSON protocol
