---
name: aiqiezi
description: Use the aiqiezi CLI to inspect and manage an aiqiezi photo/video archive through a local client daemon.
allowed-tools: Bash(aiqiezi:*) Bash(printf:*) Bash(nohup:*)
---

<!--
  crates/aiqiezi-cli/SKILL.md
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
-->

# aiqiezi CLI

Use this skill when the user asks to inspect or manage an aiqiezi archive from a
terminal or agent environment.

Install it by copying this file to your agent's skills directory, for example:

```text
.opencode/skills/aiqiezi/SKILL.md
```

## Prerequisites

- `aiqiezi` is installed and on `PATH`.
- The user has the server EndpointId.
- The local client daemon is running, or the user asks you to start it.
- Enrollment requires the server bootstrap password from the user.

The client uses its persistent iroh key as its identity. There is no session
token and no endpoint ticket.

## Safety rules

- Never put passwords in command arguments, environment variables, logs, or
  chat output.
- Read passwords only through `--password-stdin`.
- Do not ask for or print the client secret key.
- Do not edit `config/client.json` or `key_data/secret_key.bin` directly.
- Use `aiqiezi set-server` to change the server EndpointId.
- Do not run `aiqiezi change-password` unless the user explicitly asks.
- Do not run `aiqiezi stop` unless the user asks.
- Do not start the daemon in the foreground from an agent; it blocks.
- Treat the client secret key as a credential. A stolen key keeps that
  EndpointId's access until the server administrator disables it.
- The server EndpointId is public; the client key is secret.

## Daemon lifecycle

Check whether the daemon is already running:

```bash
aiqiezi status --json
```

If it is not running, start it in the background.

Linux/macOS:

```bash
nohup aiqiezi start >>"${AIQIEZI_DATA_DIR:-./data}/client.log" 2>&1 &
```

Windows PowerShell:

```powershell
Start-Process aiqiezi -ArgumentList start -WindowStyle Hidden
```

Then poll `aiqiezi status --json` until it succeeds. Do not run `aiqiezi start`
without backgrounding it.

## Configuration

The server EndpointId is stored in:

```text
<AIQIEZI_DATA_DIR>/config/client.json
```

Set it with:

```bash
aiqiezi set-server <server-endpoint-id> --json
```

Do not edit the file manually. There is no endpoint ticket.

## First enrollment

The first login uses the server bootstrap password and must be followed by a
password change.

```bash
# One password line on stdin
printf '%s\n' '<bootstrap-password>' | aiqiezi login --password-stdin
```

This is expected to exit nonzero with `password_change_required`; it does not
enroll the client yet.

```bash
# Three lines: old password, new password, confirmation
printf '%s\n%s\n%s\n' \
  '<bootstrap-password>' '<new-password>' '<new-password>' \
  | aiqiezi change-password --password-stdin
```

The password change enrolls the client EndpointId in the server whitelist.

Later logins:

```bash
printf '%s\n' '<password>' | aiqiezi login --password-stdin
```

## Commands

```text
aiqiezi start
aiqiezi stop
aiqiezi status
aiqiezi whoami
aiqiezi set-server <endpoint-id>
aiqiezi login [--password-stdin]
aiqiezi change-password [--password-stdin]
aiqiezi server-info
```

`--json` is a global flag accepted by every command. Socket commands already
print the JSON response envelope.

## JSON output

Most commands print the shared response envelope:

```json
{
  "version": 1,
  "request_id": "cli-123",
  "ok": true,
  "data": {}
}
```

Errors use:

```json
{
  "version": 1,
  "request_id": "cli-123",
  "ok": false,
  "error": {
    "code": "invalid_credentials",
    "message": "invalid password"
  }
}
```

`aiqiezi whoami --json` prints:

```json
{"endpoint_id":"<client-endpoint-id>"}
```

`aiqiezi status --json` returns data containing:

```text
client_id      client EndpointId
server_id      configured server EndpointId, or null
uptime_secs    daemon uptime
version        CLI version and build date
socket         local socket path
```

## Error codes

```text
password_change_required   bootstrap password must be changed
invalid_credentials        wrong password
weak_password              new password does not meet policy
password_unchanged         new password equals old password
rate_limited               too many failed attempts; retry later
auth_unavailable           server EndpointId is not configured
invalid_request            missing or malformed parameters
unknown_command            unsupported local command
internal                   backend or local operation failed
```

Exit code `0` means the command completed successfully. Exit code `1` means the
command failed, including `password_change_required` on first login.

## Common tasks

Show client identity:

```bash
aiqiezi whoami
aiqiezi whoami --json
```

Show server configuration and daemon state:

```bash
aiqiezi status --json
```

Request backend information after enrollment:

```bash
aiqiezi server-info --json
```

Change the server EndpointId:

```bash
aiqiezi set-server <server-endpoint-id> --json
```

## Troubleshooting

- `client daemon is not running`: start the daemon in the background and poll
  `aiqiezi status`.
- `client key not found`: start the daemon once to create the persistent key.
- `auth_unavailable`: run `aiqiezi set-server <server-endpoint-id>`.
- `password_change_required`: run `aiqiezi change-password --password-stdin`.
- `rate_limited`: wait for the reported retry period.
- `invalid_credentials`: verify the password with the user; do not print it.
- Backend connection failures: confirm the server EndpointId and that the
  client is whitelisted on the server.

## Current limitations

Only these commands exist today. Folder, asset, search, upload, and download
commands are future work; do not invent them.

The daemon currently starts in the foreground; use the background examples
above for agent use.
