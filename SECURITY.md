<!--
  SECURITY.md
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
-->

# Security Policy

## Supported versions

The project is in early development. Security fixes are applied to the current
`main` branch and the latest release, if a release exists.

## Reporting a vulnerability

Please do not open a public GitHub issue for security vulnerabilities.

Use GitHub's private vulnerability reporting:

<https://github.com/gocpicnic/aiqiezi-cli/security/advisories/new>

Include:

- affected version or commit;
- a description of the issue;
- reproduction steps or a proof of concept;
- impact assessment;
- any suggested fix.

## Scope

Security-sensitive areas include:

- local client daemon socket permissions and local IPC handling;
- persistent client secret-key storage and permissions;
- the server EndpointId-only iroh firewall;
- password prompting, stdin handling, and password redaction;
- logs and error messages that could expose keys or credentials;
- release binaries, checksums, and dependency supply chain;
- accidental secret or runtime-data commits.

The CLI never stores the aiqiezi-server password. Authorization after enrollment
is based on the client's persistent iroh EndpointId and the server whitelist.

## Disclosure

Please allow time for triage and a fix before public disclosure. Confirmed
vulnerabilities may be disclosed through a GitHub Security Advisory.
