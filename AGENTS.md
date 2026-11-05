<!--
  AGENTS.md
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
-->

# AGENTS.md

## AI assistant rules

- Always communicate in English.
- Explain code, errors, and design decisions in English.
- Ask clarification questions in English.
- Read this file and the shared client library API before changing behavior.
- Never commit secrets, client keys, media, or runtime data.
- Keep CLI and GUI behavior consistent by sharing `aiqiezi-client`.

## Project context

`aiqiezi-cli` is the command-line client and shared Rust client library for the
aiqiezi single-backend photo/video archive. See [README.md](README.md) and the
server repository for the system architecture.

Mandatory constraints:

- iroh JSON for metadata/control;
- iroh-blobs for media transfer;
- no HTTP client;
- no direct PostgreSQL access;
- no direct server filesystem access;
- client identity is a persistent iroh EndpointId;
- initial enrollment uses the server bootstrap password;
- after the first password change, authorization is the server-side EndpointId
  whitelist;
- no session-token flow;
- the shared client library must not depend on CLI or GUI frameworks.

## Platform support

Tier 1 platforms are x86_64 Windows, x86_64 macOS, and x86_64 Linux. ARM64
targets are deferred. The CLI is built from source; no prebuilt binaries or
installers are provided yet.

Rules:

- `aiqiezi-client` must not use Unix-only APIs, hardcoded `/` paths, Unix
  domain sockets, signals, or shell commands for normal operation.
- Use platform-native data directories for configuration, cache, and key
  storage.
- Abstract secret-key permissions: `0600`/`0700` on Unix and an appropriate
  ACL or user-profile protection on Windows.
- Keep stdout/stderr behavior and JSON schemas identical across platforms.
- Document and test any platform-specific code path before adding it.

## Repository layout

```text
aiqiezi-cli/
├── Cargo.toml                 # workspace
├── crates/
│   ├── aiqiezi-client/        # shared iroh client library
│   └── aiqiezi-cli/           # CLI binary
└── AGENTS.md
```

## Shared client library

`aiqiezi-client` owns:

- iroh endpoint setup and shutdown;
- relay/DNS configuration;
- persistent client secret key and stable EndpointId;
- JSON request/response framing;
- protocol request/response types;
- endpoint ticket/EndpointId handling;
- iroh-blobs upload/download;
- pagination and cursor helpers;
- typed errors shared by CLI and GUI.

The library must not:

- access PostgreSQL;
- read the server filesystem;
- print CLI output;
- read terminal input;
- depend on egui or clap.

## CLI responsibilities

`aiqiezi-cli` owns:

- command parsing;
- human-readable formatting;
- machine-readable output;
- exit codes;
- configuration and client-key loading;
- agent skill documentation.

## Output contract

- stdout is for data only.
- stderr is for errors, warnings, and progress.
- `--json` prints one valid JSON document.
- Batch commands may use NDJSON/JSON Lines.
- Never mix progress logs into stdout.
- Exit code 0 on success; nonzero on failure.
- Interactive prompts are not allowed in agent mode.

## Client enrollment

The first login uses the server's `AIQIEZI_BOOTSTRAP_PASSWORD`:

```bash
# On the client
aiqiezi login
aiqiezi change-password
aiqiezi whoami
```

The server returns `password_change_required` until the bootstrap password is
changed. The password change enrolls the client EndpointId in the server
whitelist. After that, the bootstrap value is ignored and the whitelist
authorizes the EndpointId.

Login and `change-password` must use the client's persistent iroh secret key.
Do not create a temporary endpoint for these commands: the server enrolls the
connection EndpointId.

A disabled or removed EndpointId is rejected at connection time.

Server administrators can also list or manage enrolled clients:

```bash
# On the server host
aiqiezi-server client-list
aiqiezi-server client-disable <endpoint-id>
```

## AI-agent usage

The CLI is a first-class AI-agent interface.

Commands should be:

- idempotent where possible;
- safe by default;
- non-interactive once the client key is configured;
- well documented with JSON schemas;
- explicit about destructive operations.

Destructive commands require an explicit confirmation flag such as `--yes` or
`--force`.

A `SKILL.md` in the CLI crate must document commands, JSON output, enrollment,
examples, and safety rules.

## Protocol rules

- Use the shared `aiqiezi-proto` types from the server repository.
- Do not introduce a second wire format.
- Version protocol changes.
- Use cursor pagination for large result sets.
- Keep media bytes on iroh-blobs; keep JSON messages small.
- Never add an HTTP fallback without an explicit architecture decision.

## Data rules

- The CLI never writes directly to PostgreSQL.
- The CLI never writes server files directly.
- Uploads and downloads are hash-verified.
- Large media must stream; do not load entire files into memory.
- Temporary files must be cleaned up on failure.

## Security rules

- Never commit client keys, relay tokens, or runtime credentials.
- Store the client secret key with restrictive permissions.
- Prompt for passwords without echo; never put them in argv, environment, or
  logs.
- Never store the server password; store only the client secret key.
- Redact secrets in errors and logs.
- Do not log full endpoint tickets.
- Treat EndpointId as identity and the secret key as the credential.
- Do not bypass the server whitelist for local development.
- A stolen client key grants that client's access until its EndpointId is
  disabled or removed on the server.

## Configuration

Planned variables:

```text
AIQIEZI_SERVER_ID
AIQIEZI_ENDPOINT_TICKET
AIQIEZI_RELAY_URL
AIQIEZI_RELAY_TOKEN
AIQIEZI_DATA_DIR
```

There is no `AIQIEZI_TOKEN`; authorization is by EndpointId.

## Commands

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p aiqiezi-cli
```

## Source file rules

Every new `.rs`, `.sql`, or `.md` source file must start with the project
license header, using the file path relative to the repository root:

```rust
/*
  relative/path.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/
```

For Markdown use an HTML comment with the same content.

## Testing rules

- Unit-test JSON framing and protocol encoding.
- Test pagination and cursor handling.
- Test client-key and endpoint-ticket redaction.
- Test password redaction and authentication error mapping.
- Test error-to-exit-code mapping.
- Test machine-readable output schemas.
- Integration tests must not require production data or secrets.

## Agent workflow

1. Read this file and the relevant protocol types.
2. Keep CLI and GUI behavior aligned through `aiqiezi-client`.
3. Make output contracts explicit before implementation.
4. Run formatting, tests, and checks.
5. Document compatibility or migration impact.
