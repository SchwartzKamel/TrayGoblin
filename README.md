# TrayGoblin

**Audience:** Operators and Agents

TrayGoblin is a native Windows notification-area companion for concurrent pi and
GitHub Copilot CLI sessions on Windows and local WSL. It shows **Working**, **Idle**
or **Attention needed** and a persistent, attention-first session list. Pi alerts
for all explicit blocking dialogs and final run errors; Copilot alerts only for
new persisted session errors (input detection is unavailable). Completion is quiet.

- Native Rust process targeting Windows 10/11 x86-64 — no Electron, no Node.js sidecar, no service
- One-second default polling; attention/clear target within two intervals plus 500 ms
- Content-free by construction: prompts, responses, tool arguments, tool results, tokens, and
  credentials are never modeled, stored, logged, or displayed; temporary raw-byte scanning is
  distinct from allow-listed field deserialization (see [Privacy](docs/operator/privacy.md))
- Per-user PowerShell install and startup registration, with no administrator privileges
- Manual local validation, packaging, tagging, and GitHub release publishing — no CI service

## Project status

The deterministic MVP implementation is complete: monitor, tray shell, installer, packaging,
performance tooling, Operator/Agent documentation, and the end-to-end release-candidate demo are
implemented and validated. See [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md) for authoritative
historical MVP task state.

**Attention implementation, UNPUBLISHED:** M1–M4 are accepted at implementation/
compilation tier (M4 `d94c85c`). M5 packaging/setup/continuous offline demo is
in_progress and unaccepted pending actual disposition. The
[active plan](.wgm/IMPLEMENTATION_PLAN.md) is the sole authority for the current
candidate, exact gate results, review/audit verdicts and next acceptance action.
The [attention spec](.wgm/specs/session-attention.md) and
[glossary](.wgm/specs/CONTEXT.md) govern behavior and terminology; root MVP artifacts
are history.
The historical M3 blocker was reproduced, repaired and independently accepted;
its [dissent/history](docs/architecture.md#m3-recovery-checkpoint) is retained.
Actual Windows/WSL interaction and 50 MB/5% evidence remain **M6 BLOCKED**.
No Linux cross-build, offline pi host, OCI CLI or structural docs check proves
Windows notification delivery, performance or release readiness.

No release has been published from this repository yet. Until one is, build the portable archive
yourself with `bash scripts/package-release.sh 0.1.0`. Releases are previews: Windows x86-64 only,
per-user, unsigned, and without auto-update. Historical MVP agent-owned deterministic work is complete; the
[Windows interaction checklist](docs/agent/testing.md#manual-windows-checklist) must pass before a
preview is published, and `scripts/measure-performance.ps1` must pass before promoting a preview
build to stable. See [Release responsibilities](docs/agent/release-responsibilities.md).

## Documentation

Start at the [documentation index](docs/README.md). It is organized around two audiences:
**Operators**, who install and run TrayGoblin, and **Agents**, who change this repository.

**Operators**

| Guide | Purpose |
|---|---|
| [Installation](docs/operator/installation.md) | Obtain or build the ZIP, verify the checksum, install per-user, uninstall |
| [First run](docs/operator/first-run.md) | What the icon, tooltip, and menu do on first launch |
| [Status reference](docs/operator/status-reference.md) | How Working, Idle, and Attention needed are derived |
| [Configuration](docs/operator/configuration.md) | `config.json` location, keys, ranges, and the restart rule |
| [Privacy](docs/operator/privacy.md) | What is read, what is never read, and how to confirm it |
| [Troubleshooting](docs/operator/troubleshooting.md) | Missing icon, stuck state, Copilot format changes |

**Agents**

| Guide | Purpose |
|---|---|
| [Development](docs/agent/development.md) | Toolchain, layout, and the rules a change must not break |
| [Testing](docs/agent/testing.md) | Deterministic host checks versus manual Windows checks |
| [Release responsibilities](docs/agent/release-responsibilities.md) | Ownership split and release gates |
| [Documentation standards](docs/agent/documentation-standards.md) | Audience labels, link rules, and what the docs checker enforces |

**Reference:** [Architecture](docs/architecture.md) ·
[Manual release](docs/manual-release.md) · [MVP](docs/MVP.md)

## Quick start

**Operators** — on Windows, extract the release archive and run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
```

Then read [First run](docs/operator/first-run.md). The archive includes standalone
`resources/SETUP.md`, pi integration and Windows collectors. The installer persists
these below the install root; explicitly `pi install` that installed package to opt
in. Never point pi at an extraction folder you plan to delete; installation does
not change pi settings automatically.

**Agents** — on Linux, WSL, or another host with stable Rust and the MinGW-w64 x86-64 toolchain
([setup instructions](docs/agent/development.md#toolchain)):

```bash
bash scripts/validate-toolchain.sh
cargo run --bin tray-goblin-probe -- --session-root tests/fixtures/live-session
```

The first command runs the workspace tests and cross-builds the Windows executable; the second
prints a content-free status snapshot from a fixture session. The tray itself only runs on Windows.

## Validation

```bash
cargo test --workspace
bash scripts/validate-toolchain.sh
bash scripts/check-docs.sh
bash scripts/demo.sh 0.1.0
bash scripts/demo-attention.sh --container podman
```

The attention demo keeps two installed pi1.4 runtimes alive together through the
shipped Rust dispatcher, with external machine-readable evidence. Its cached
Debian OCI check is CLI-only, nonroot/network-none/read-only. Set an external
`TRAY_GOBLIN_EVIDENCE_DIR` to retain exact journey/engine commands and digests.
Windows interaction is collected INTERACTIVELY by `scripts/test-attention-windows.ps1`;
`scripts/measure-performance.ps1 -Path` binds the same exact executable/PID/hash.
Both reject Linux. See [Testing](docs/agent/testing.md).

## Repository map

| Path | Purpose |
|---|---|
| `src/` | Platform-neutral parser, monitor, and decisions, plus the Windows-only tray shell |
| `src/bin/tray-goblin-probe.rs` | Content-free diagnostic probe |
| `tests/` | Integration tests and content-free session fixtures |
| `scripts/` | Toolchain validation, installer test, packaging, performance measurement, docs check |
| `install.ps1`, `uninstall.ps1` | Per-user installation and removal |
| `docs/` | Operator and agent documentation, indexed by [`docs/README.md`](docs/README.md) |
| `specs/` | Constitution, glossary, acceptance spec, and architecture decision |
| `scenarios/` | Holdout user journeys used only during validation and review |
| `IMPLEMENTATION_PLAN.md` | Historical MVP tasks; active attention status lives in `.wgm/IMPLEMENTATION_PLAN.md` |
| `AGENTS.md` | Protected historical MVP command sheet; see Development for active routing |

## License

[MIT](LICENSE)
