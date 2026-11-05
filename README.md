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

## Platform support

Tier 1 targets are x86_64 Windows, x86_64 macOS, and x86_64 Linux. The CLI is
currently built from source; prebuilt binaries and installers are deferred.

ARM64 targets, including Apple Silicon, are not yet supported. Source builds
may work on those hosts, but they are untested and outside the support matrix.

## Client enrollment

The first login uses the bootstrap password configured on the server:

```bash
aiqiezi login
aiqiezi change-password
```

The server returns `password_change_required` until the bootstrap password is
changed. Changing it enrolls the client's persistent iroh EndpointId in the
server whitelist. After that, the bootstrap password is ignored and the
whitelist authorizes the EndpointId.

Login and `change-password` must use the persistent client key. The server
enrolls the EndpointId of the connection, not a temporary key.

The client has a persistent iroh secret key and a stable EndpointId:

```bash
aiqiezi whoami
```

To revoke access, disable or remove the EndpointId on the server:

```bash
aiqiezi-server client-disable <endpoint-id>
```

The client never stores the server password. It stores only its own iroh secret
key.

## Planned commands

```bash
aiqiezi login
aiqiezi change-password
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

`AIQIEZI_BOOTSTRAP_PASSWORD` is server-side only. Clients prompt for the
password interactively and never store it.

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
