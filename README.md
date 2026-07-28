# TrayGoblin

**Audience:** Operators and Agents

TrayGoblin is a lightweight Windows notification-area companion for GitHub Copilot CLI. It observes
Copilot's local session state read-only and shows whether the active turn is **Working**, **Idle**,
or **Attention needed**, together with the model, repository, and last completed turn duration.

- Native Rust process targeting Windows 10/11 x86-64 — no Electron, no Node.js sidecar, no service
- One-second default polling with a two-second state-update expectation
- Content-free by construction: prompts, responses, tool arguments, tool results, tokens, and
  credentials are never parsed, stored, logged, or displayed
- Per-user PowerShell install and startup registration, with no administrator privileges
- Manual local validation, packaging, tagging, and GitHub release publishing — no CI service

## Project status

The monitor, tray shell, installer, packaging, performance tooling, and documentation are
implemented and validated; the final end-to-end demo assembly remains. See
[`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md) for authoritative task state.

No release has been published from this repository yet. Until one is, build the portable archive
yourself with `bash scripts/package-release.sh 0.1.0`. Releases are previews: Windows x86-64 only,
per-user, unsigned, and without auto-update. The Windows performance measurement
(`scripts/measure-performance.ps1`) must pass before promoting a preview build to stable.

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

Then read [First run](docs/operator/first-run.md).

**Agents** — on Linux, WSL, or another host with stable Rust and the MinGW-w64 x86-64 toolchain:

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
```

Windows interaction and the 50 MB / 5% resource budgets are verified manually with
`scripts/measure-performance.ps1`; see [Testing](docs/agent/testing.md).

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
| `IMPLEMENTATION_PLAN.md` | Ordered build tasks and current status |
| `AGENTS.md` | Lean operational build and validation guide |

## License

[MIT](LICENSE)
