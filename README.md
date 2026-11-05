<!--
  README.md
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
-->

# aiqiezi-cli

Agent-ready command-line client and shared Rust client library for
[aiqiezi-server](https://github.com/gocpicnic/aiqiezi-server), the self-hosted
photo/video archive backend.

## Why iroh

The target server is a self-hosted machine behind NAT without a static public
IP, public domain, port forwarding, or TLS certificate.

The CLI therefore uses:

- iroh JSON for metadata and control;
- iroh-blobs for media transfer;
- the aiqiezi-server EndpointId for connection setup.

No HTTP endpoint is required.

## Status

First CLI milestone implemented:

- local client daemon;
- cross-platform local socket;
- persistent client key and EndpointId;
- aiqiezi-server EndpointId stored in a config file;
- server-only iroh firewall;
- `start`, `stop`, `status`, `whoami`, `set-server`, `login`,
  `change-password`, and `server-info`.

The GUI is still deferred.

## Built for agents

The CLI is designed to be driven by scripts and AI agents:

- command-first local daemon;
- shared `Request` / `Response` JSON envelope;
- `--json` output;
- `--password-stdin` for non-interactive login and password changes;
- local socket JSON control channel;
- `SKILL.md` for agent usage.

An MCP bridge to the local socket is future work. The daemon currently starts
in the foreground; [SKILL.md](crates/aiqiezi-cli/SKILL.md) documents
platform-specific background startup.

## Architecture

```text
aiqiezi CLI
    |
    | local socket: JSON Lines
    v
aiqiezi client daemon
    |
    | persistent client key + iroh endpoint
    | firewall: configured aiqiezi-server EndpointId only
    v
aiqiezi-server
```

The local daemon runs in the foreground. Run `aiqiezi start` in one terminal and
the other commands in another terminal.

## Platform support

Tier 1 targets are x86_64 Windows, x86_64 macOS, and x86_64 Linux. The CLI is
currently built from source; prebuilt binaries and installers are deferred.

ARM64 targets, including Apple Silicon, are not yet supported. Source builds
may work on those hosts, but they are untested and outside the support matrix.

Local IPC uses:

- Unix domain socket on Linux and macOS;
- named pipe on Windows.

## Client enrollment

Get the aiqiezi-server EndpointId first:

```bash
# On the server host
aiqiezi-server whoami
```

Configure the client and start its local daemon:

```bash
aiqiezi set-server <server-endpoint-id>
aiqiezi start
```

In another terminal, perform the first login:

```bash
aiqiezi login
aiqiezi change-password
aiqiezi server-info
```

The first login returns `password_change_required` and exits nonzero. Run
`aiqiezi change-password` to continue. Changing it enrolls the client's
persistent iroh EndpointId in the server whitelist. After that, the bootstrap
password is ignored and the whitelist authorizes the EndpointId.

`aiqiezi-server whoami` prints the **aiqiezi-server** EndpointId.
`aiqiezi whoami` prints the **client** EndpointId.

Login and `change-password` must use the persistent client key. The server
enrolls the EndpointId of the connection, not a temporary key.

Stop the daemon with:

```bash
aiqiezi stop
```

The client never stores the server password. It stores only its own iroh secret
key.

## Commands

```bash
aiqiezi start
aiqiezi stop
aiqiezi status
aiqiezi whoami
aiqiezi set-server <endpoint-id>
aiqiezi login [--password-stdin]
aiqiezi change-password [--password-stdin]
aiqiezi server-info
```

`--json` is a global flag. `whoami --json` prints a JSON object; socket
commands already print the JSON response envelope.

For AI agents, `login --password-stdin` reads one password line and
`change-password --password-stdin` reads old password, new password, and
confirmation, one per line. See [SKILL.md](crates/aiqiezi-cli/SKILL.md).

Future commands will add folders, assets, search, upload, and download.

## Configuration

Environment variables:

```text
AIQIEZI_DATA_DIR       # default: ./data
AIQIEZI_RUNTIME_DIR    # default: <AIQIEZI_DATA_DIR>/run
AIQIEZI_LOG_LEVEL      # default: info
```

The aiqiezi-server EndpointId is stored in:

```text
<AIQIEZI_DATA_DIR>/config/client.json
```

Example:

```json
{
  "server_id": "<server-endpoint-id>"
}
```

The config file is mode `0600`. The client secret key lives under:

```text
<AIQIEZI_DATA_DIR>/key_data/secret_key.bin
```

Back up this key. Losing it changes the client EndpointId, and the new
EndpointId must be enrolled on the server.

The local socket lives under:

```text
<AIQIEZI_RUNTIME_DIR>/client.sock
```

The runtime directory is mode `0700`; the Unix socket is mode `0600`.

There is no endpoint ticket in the client configuration.

## Security

- The client daemon accepts iroh connections only from the configured server
  EndpointId.
- Outbound connections are checked against the same EndpointId.
- There is no local-IP bypass.
- The local socket is the only local control channel.
- Passwords are prompted without echo and are never stored.
- Agent mode reads passwords from stdin; never pass them in argv or the
  environment.
- The client stores only its own iroh secret key and the aiqiezi-server EndpointId.

## Machine-readable output

Socket commands print the JSON response envelope on stdout. `whoami` prints
only the client EndpointId, or a JSON object with `--json`. `--json` is a global
flag.

`login --password-stdin` reads one line. `change-password --password-stdin`
reads three lines: old password, new password, and confirmation.

Rules:

- stdout contains data only;
- stderr contains errors, warnings, and progress;
- exit code is nonzero on failure, including `password_change_required`;
- future batch commands may use NDJSON.

## Development

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p aiqiezi-cli -- --help
cargo run -p aiqiezi-cli -- start
```

Or use the Makefile:

```bash
make build
make run ARGS="start"
make test
make clippy
make ci
make version
```

`make build` copies the release binary to `dist/aiqiezi`
(`dist/aiqiezi.exe` on Windows).

Optional target builds:

```bash
make build TARGET=x86_64-unknown-linux-musl
```

Target builds are copied under `dist/<target>/aiqiezi`.

Cross-build the Windows MSVC executable on Linux:

```bash
make build-windows
```

Output:

```text
dist/x86_64-pc-windows-msvc/aiqiezi.exe
```

This requires `cargo-xwin`, the `x86_64-pc-windows-msvc` Rust target,
`llvm-tools`, `clang-cl`, and `lld-link`. The executable is built with the
static CRT so Windows does not need the VC++ runtime installed.

`aiqiezi-proto` is consumed through a local path dependency during development:

```text
../../../aiqiezi-server/crates/aiqiezi-proto
```

The release dependency strategy is not decided yet. Do not copy the protocol
types into this repository.

## Documentation

- [AGENTS.md](AGENTS.md): implementation rules for AI coding agents
- [crates/aiqiezi-cli/SKILL.md](crates/aiqiezi-cli/SKILL.md): agent-facing
  command and safety reference
- `../aiqiezi-server/README.md`: system architecture and aiqiezi-server setup
