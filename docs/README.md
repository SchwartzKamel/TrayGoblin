# TrayGoblin documentation

**Audience:** Operators and Agents

TrayGoblin is a native Windows tray for concurrent pi/Copilot sessions on Windows
and local WSL: **Working**, **Idle**, **Attention needed**, attributed notifications
and a persistent session list. Pi input/final errors alert; Copilot persisted
errors only (input unavailable); completion is quiet. Implementation is unpublished,
not real Windows/WSL delivery or performance evidence.

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
- [Configuration](operator/configuration.md) — legacy settings, aggregate source configuration,
  explicit probe loading, accepted ranges, and the restart rule.
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
| [`specs/CONSTITUTION.md`](../specs/CONSTITUTION.md) | Non-negotiable principles: privacy, performance, no elevation, no CI dependency |
| [`specs/CONTEXT.md`](../specs/CONTEXT.md) | Canonical vocabulary for states and session terms |
| [`specs/tray-status.md`](../specs/tray-status.md) | Executable acceptance criteria and their verifying commands |
| [`specs/adr/0001-native-rust-tray.md`](../specs/adr/0001-native-rust-tray.md) | Accepted architecture decision |
| [`IMPLEMENTATION_PLAN.md`](../IMPLEMENTATION_PLAN.md) | Historical completed MVP tasks |
| [Attention follow-up plan](../.wgm/IMPLEMENTATION_PLAN.md) | Sole authority for the current attention candidate, exact gate results, review/audit verdicts and acceptance status |
| [Attention spec](../.wgm/specs/session-attention.md) | Follow-up behavior, defaults, limitations, and exact validation boundaries |
| [Attention glossary](../.wgm/specs/CONTEXT.md) | Follow-up source, instance, and episode vocabulary |
| [`AGENTS.md`](../AGENTS.md) | Protected historical MVP command sheet; [Development](agent/development.md#active-follow-up-routing) clarifies active routing |

Completion claims concern the historical MVP only. The
[M3 recovery history](architecture.md#m3-recovery-checkpoint) retains rejected-candidate
dissent AND the later compiled proof/fix/independent acceptance, not an active blocker.
The current native wiring is cross-compiled; M5 provides a continuous offline
pi-host→Rust implementation demo and standalone package resources, not release readiness.
M1–M5 are accepted at implementation tier and unpublished; M6 actual
Windows/WSL/performance evidence remains BLOCKED. Optional read-only Actions test
ZIPs are not Releases; signed-in downloads and the local fallback are described
in [Installation](operator/installation.md#1-obtain-the-archive). CI is not a release
prerequisite or M6 proof. Consult the active plan above
for current candidate/gates/verdicts, not duplicated status or historical reports.
The [standalone setup source](../assets/attention/SETUP.md) ships as resources/SETUP.md;
the [pi integration](../integrations/pi/README.md) explains explicit persistent opt-in.
Attention audit history/dispositions remain in the
[attention audit index](../.wgm/docs/audit/README.md); RED is not relabeled GREEN.

Validate this documentation set with:

```bash
bash scripts/check-docs.sh
```
