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
aiqiezi-server photo/video archive. See [README.md](README.md) and the
server repository for the system architecture.

Mandatory constraints:

- iroh JSON for metadata/control;
- iroh-blobs for media transfer;
- no HTTP client;
- no direct PostgreSQL access;
- no direct server filesystem access;
- client identity is a persistent iroh EndpointId;
- initial enrollment uses the aiqiezi-server bootstrap password;
- after the first password change, authorization is the server-side EndpointId
  whitelist;
- no session-token flow;
- the CLI runs a local client daemon controlled over a local socket;
- the client daemon accepts iroh connections only from the configured server
  EndpointId;
- the shared client library must not depend on CLI or GUI frameworks.

## Platform support

Tier 1 platforms are x86_64 Windows, x86_64 macOS, and x86_64 Linux. ARM64
targets are deferred. The CLI is built from source; no prebuilt binaries or
installers are provided yet.

Rules:

- `aiqiezi-client` must not use hardcoded `/` paths, raw Unix-only APIs,
  signals, or shell commands outside the local IPC abstraction.
- Local IPC must use a Unix domain socket on Linux/macOS and a named pipe on
  Windows behind one shared abstraction.
- Use a configurable data directory; the current default is `./data`.
  Platform-native default directories are deferred.
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
- client config file with the aiqiezi-server EndpointId;
- local socket client and cross-platform IPC abstraction;
- server-only iroh firewall;
- aiqiezi-server JSON request/response framing;
- protocol request/response types;
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
- local daemon lifecycle and instance lock;
- local socket server and command dispatcher;
- human-readable formatting;
- machine-readable output;
- exit codes;
- configuration and client-key loading;
- agent skill documentation.

## Client daemon and local socket

- `aiqiezi start` runs the daemon in the foreground.
- `aiqiezi stop` sends a shutdown command over the local socket.
- The local socket path is `<AIQIEZI_RUNTIME_DIR>/client.sock`.
- Linux/macOS use a Unix socket; Windows uses a named pipe.
- Runtime directory mode is `0700`; Unix socket mode is `0600`.
- Local control uses the shared `aiqiezi-proto::Request` / `Response` envelope
  with JSON Lines, one request per connection.
- The local daemon forwards aiqiezi-server commands through `aiqiezi-client`.
- Process commands such as `start`, `stop`, `status`, `whoami`, and
  `set-server` do not use the aiqiezi-server wire protocol.

## Output contract

- stdout is for data only.
- stderr is for errors, warnings, and progress.
- Socket commands print the JSON response envelope.
- `--json` is a global flag; `whoami --json` prints a JSON object.
- `login --password-stdin` reads one password line.
- `change-password --password-stdin` reads old password, new password, and
  confirmation, one per line.
- Batch commands may use NDJSON/JSON Lines.
- Never mix progress logs into stdout.
- Exit code 0 on success; nonzero on failure.
- Interactive prompts are not allowed in agent mode.
- The first login returning `password_change_required` exits nonzero.

## Client enrollment

Get the aiqiezi-server EndpointId first:

```bash
# On the server host
aiqiezi-server whoami
```

Configure the aiqiezi-server EndpointId and start the local daemon:

```bash
# On the client
aiqiezi set-server <server-endpoint-id>
aiqiezi start
```

The first login uses the server's `AIQIEZI_BOOTSTRAP_PASSWORD`:

```bash
# In another terminal
aiqiezi login
aiqiezi change-password
aiqiezi whoami
```

`aiqiezi-server whoami` prints the aiqiezi-server EndpointId; `aiqiezi whoami`
prints the client EndpointId.

aiqiezi-server returns `password_change_required` until the bootstrap password
is changed. The first login exits nonzero in that state. The password change
enrolls the client EndpointId in the aiqiezi-server whitelist. After that, the
bootstrap value is ignored and the whitelist authorizes the EndpointId.

Login and `change-password` must use the client's persistent iroh secret key.
Do not create a temporary endpoint for these commands: aiqiezi-server enrolls
the connection EndpointId.

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

`crates/aiqiezi-cli/SKILL.md` is the agent-facing reference. It documents
commands, JSON output, non-interactive password input, enrollment, examples,
error codes, and safety rules. Keep it in sync with CLI behavior.

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
- Agent mode accepts passwords only through `--password-stdin`.
- Never store the server password; store only the client secret key.
- Redact secrets in errors and logs.
- Treat the client EndpointId as identity and the client secret key as the
  credential.
- The client daemon accepts iroh connections only from the configured server
  EndpointId; do not add a local-IP bypass.
- Restrict the local socket to the current user.
- Do not bypass the aiqiezi-server whitelist for local development.
- A stolen client key grants that client's access until its EndpointId is
  disabled or removed on the server.

## Configuration

Environment variables:

```text
AIQIEZI_DATA_DIR       # default: ./data
AIQIEZI_RUNTIME_DIR    # default: <AIQIEZI_DATA_DIR>/run
AIQIEZI_LOG_LEVEL
```

The aiqiezi-server EndpointId is stored in:

```text
<AIQIEZI_DATA_DIR>/config/client.json
```

```json
{
  "server_id": "<server-endpoint-id>"
}
```

There is no endpoint ticket. There is no `AIQIEZI_TOKEN`; authorization is by
EndpointId.

## Commands

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p aiqiezi-cli -- --help
cargo run -p aiqiezi-cli -- start
```

Makefile shortcuts:

```bash
make build
make run ARGS="start"
make test
make clippy
make ci
make version
```

`make build` copies the release binary to `dist/aiqiezi`
(`dist/aiqiezi.exe` on Windows). Target builds are copied under
`dist/<target>/aiqiezi`.

Cross-target builds use `TARGET`, for example:

```bash
make build TARGET=x86_64-unknown-linux-musl
```

Windows MSVC cross-builds use cargo-xwin:

```bash
make build-windows
```

Output:

```text
dist/x86_64-pc-windows-msvc/aiqiezi.exe
```

`make build TARGET=x86_64-pc-windows-msvc` also uses cargo-xwin. The Windows
build requires `cargo-xwin`, the MSVC Rust target, `llvm-tools`, `clang-cl`,
and `lld-link`, and uses the static CRT.

## CI and release rules

- CI runs formatting and clippy on Linux plus workspace tests on Linux,
  Windows, and macOS.
- Pushing a `v*` tag builds the Linux musl, macOS x86_64, and Windows MSVC
  x86_64 binaries and publishes them as GitHub Release assets.
- Windows release builds use the static CRT.
- Release artifacts include `LICENSE` and checksums.
- No installers, signing, or notarization are part of the normal workflow.
- Pin third-party Actions to immutable commit SHAs.
- Keep workflow `permissions` minimal; only the release job gets
  `contents: write`.

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
- Test client-key and config redaction.
- Test local socket framing and permissions.
- Test the server-only iroh firewall.
- Test password stdin parsing and redaction.
- Test password redaction and authentication error mapping.
- Test error-to-exit-code mapping.
- Test machine-readable output schemas.
- Keep `SKILL.md` examples aligned with tests.
- Integration tests must not require production data or secrets.

## Agent workflow

1. Read this file and the relevant protocol types.
2. Keep CLI and GUI behavior aligned through `aiqiezi-client`.
3. Make output contracts explicit before implementation.
4. Run formatting, tests, and checks.
5. Document compatibility or migration impact.
