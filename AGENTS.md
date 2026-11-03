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
- authorization is the server-side EndpointId whitelist;
- no login, password, or session-token flow;
- the shared client library must not depend on CLI or GUI frameworks.

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

```bash
# On the client
aiqiezi whoami

# On the server host
aiqiezi-server client-add <endpoint-id> --label "laptop"
```

There is no login, password, or session-token flow. A disabled or removed
EndpointId is rejected at connection time.

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
- Test error-to-exit-code mapping.
- Test machine-readable output schemas.
- Integration tests must not require production data or secrets.

## Agent workflow

1. Read this file and the relevant protocol types.
2. Keep CLI and GUI behavior aligned through `aiqiezi-client`.
3. Make output contracts explicit before implementation.
4. Run formatting, tests, and checks.
5. Document compatibility or migration impact.
