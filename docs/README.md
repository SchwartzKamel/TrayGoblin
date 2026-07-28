# TrayGoblin documentation

**Audience:** Operators and Agents

TrayGoblin is a native Windows notification-area companion for GitHub Copilot CLI. It reads
Copilot's local session state read-only and reports **Working**, **Idle**, or
**Attention needed** together with the model, repository, and last completed turn duration.

This page is the map of the documentation. Two audiences are used everywhere:

| Audience | Means | Typical actions |
|---|---|---|
| **Operators** | People who install, run, configure, and troubleshoot TrayGoblin on their own Windows machine | Install, read the tray state, edit `config.json`, collect a content-free diagnostic, uninstall |
| **Agents** | People and automated agents who change this repository: code, tests, docs, packaging, and releases | Build, run deterministic tests, cross-build the Windows executable, package, tag, publish, record plan status |

Every page states its audience in its header. Where the same topic has both an operator and an
agent responsibility, the page says who does what.

## Start here

| I want to… | Read |
|---|---|
| Install TrayGoblin on Windows | [Installation](operator/installation.md) |
| See what the tray shows on first launch | [First run](operator/first-run.md) |
| Understand Working / Idle / Attention needed | [Status reference](operator/status-reference.md) |
| Change the polling interval or session folder | [Configuration](operator/configuration.md) |
| Know exactly what is (and is not) read | [Privacy](operator/privacy.md) |
| Fix a missing icon, a stuck state, or a Copilot format change | [Troubleshooting](operator/troubleshooting.md) |
| Build and change the code | [Development](agent/development.md) |
| Run the deterministic and manual checks | [Testing](agent/testing.md) |
| Know who does what for a release | [Release responsibilities](agent/release-responsibilities.md) |
| Reproduce the no-CI release end to end | [Manual release](manual-release.md) |
| Write or review docs in this repository | [Documentation standards](agent/documentation-standards.md) |

## Operator guides

- [Installation](operator/installation.md) — obtain or build the portable ZIP, verify the
  checksum, install per-user without elevation, and uninstall.
- [First run](operator/first-run.md) — what the tray icon, tooltip, and menu do the first time
  TrayGoblin starts.
- [Status reference](operator/status-reference.md) — how each state is derived, what the tooltip
  lines mean, and how fast the tray reacts.
- [Configuration](operator/configuration.md) — `config.json` location, both supported keys,
  accepted ranges, and the restart rule.
- [Privacy](operator/privacy.md) — the content-free contract, what is read, and how to confirm it
  yourself.
- [Troubleshooting](operator/troubleshooting.md) — symptom-first fixes, including Copilot session
  format changes.

## Agent guides

- [Development](agent/development.md) — repository layout, toolchain, module boundaries, and the
  rules a change must not break.
- [Testing](agent/testing.md) — the deterministic Linux/host suite versus the manual Windows
  interaction and performance checks.
- [Release responsibilities](agent/release-responsibilities.md) — the split between agent-owned
  preparation and operator-owned Windows verification and publishing.
- [Documentation standards](agent/documentation-standards.md) — audience labels, link rules, and
  what `bash scripts/check-docs.sh` enforces.

## Reference

- [Architecture](architecture.md) — process model, module map, polling and state machine,
  privacy boundary, and failure handling.
- [Manual release](manual-release.md) — the reproducible, CI-free procedure from a clean tagged
  commit to a published preview, including abort and rollback conditions.
- [MVP](MVP.md) — product scope and the accepted deviation from the original Electron sketch.
- [Documentation audit log](audit/README.md) — review paper trail.

## Authoritative sources outside `docs/`

| Source | Authority |
|---|---|
| [`specs/CONSTITUTION.md`](../specs/CONSTITUTION.md) | Non-negotiable principles: privacy, performance, no elevation, no CI |
| [`specs/CONTEXT.md`](../specs/CONTEXT.md) | Canonical vocabulary for states and session terms |
| [`specs/tray-status.md`](../specs/tray-status.md) | Executable acceptance criteria and their verifying commands |
| [`specs/adr/0001-native-rust-tray.md`](../specs/adr/0001-native-rust-tray.md) | Accepted architecture decision |
| [`IMPLEMENTATION_PLAN.md`](../IMPLEMENTATION_PLAN.md) | Ordered task list and current task status |
| [`AGENTS.md`](../AGENTS.md) | Lean build and validation command sheet |

Validate this documentation set with:

```bash
bash scripts/check-docs.sh
```
