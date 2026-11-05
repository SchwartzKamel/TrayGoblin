# Development

**Audience:** Agents

How to build, run, and change TrayGoblin. Read
[Architecture](../architecture.md) first for the module boundaries this page assumes.

## Toolchain

| Requirement | Why |
|---|---|
| Stable Rust (see `rust-toolchain.toml`, edition 2024, MSRV 1.85) | Nightly and beta are rejected by `scripts/validate-toolchain.sh` |
| `x86_64-pc-windows-gnu` target | The Windows executable is cross-built, not built on Windows |
| MinGW-w64 x86-64 GCC (`x86_64-w64-mingw32-gcc`) | Linker for that target, wired up in `.cargo/config.toml` |
| `file` | Verifies the produced binary really is a PE32+ x86-64 image |
| `zip`, `unzip`, `sha256sum` | Only needed for packaging |
| PowerShell 7 (`pwsh`) | Installer controls and Windows field/resource collectors |
| Node + installed pi1.4.0, Podman + cached Debian bookworm-slim | Actual offline host journey and separate CLI-only OCI check (no pulling) |

Install the target with `rustup target add x86_64-pc-windows-gnu`. If the linker is missing, the
validation script fails with an installation hint rather than a link error.

## Offline gate prerequisites

Follow the shipped [pi package/cache provisioning and check recipe](../../integrations/pi/README.md#offline-gate-prerequisites).
It checks the actual npm pi1.4.0 manifest/CLI help, package placement or absolute
`PI_CODING_AGENT_PACKAGE` override, and cached `docker.io/library/debian:bookworm-slim`.
Online preparation is a SEPARATELY authorized operator prerequisite, NEVER a test
or offline-gate step. Installed host and cached image must already exist.
Under isolated HOME select verified public Podman root/runroot explicitly, not
credential/user-config mounts. The gate never pulls or enables network.

## First commands

```bash
bash scripts/validate-toolchain.sh
```

This runs the workspace tests, cross-builds `tray-goblin.exe`, and asserts the artifact is a PE32+
x86-64 executable. It proves the Rust/cross-build environment, not pi-host,
OCI or real Windows runtime readiness.

Inspect behaviour without Windows using the diagnostic probe:

```bash
cargo run --bin tray-goblin-probe -- --session-root tests/fixtures/live-session
```

`tray-goblin` itself cannot start on a non-Windows host; it prints that message and points at the
probe instead. This is deliberate, not a gap.

## Active follow-up routing

Root `IMPLEMENTATION_PLAN.md`, `specs/tray-status.md` and `specs/CONTEXT.md` describe the historical
MVP. Active attention work uses the [follow-up plan](../../.wgm/IMPLEMENTATION_PLAN.md),
[attention spec](../../.wgm/specs/session-attention.md) and
[glossary](../../.wgm/specs/CONTEXT.md), still under the root constitution.
Protected root `AGENTS.md` retains historical forward-contract/T6 wording; architecture and
manual-release docs now exist. Use this routing clarification without changing that protected file.

M1–M5 are accepted at implementation/compilation tier and remain unpublished.
The follow-up plan above is the sole
authority for the current candidate, exact gate results, review/audit verdicts
and next acceptance action; this guide does not duplicate transient review state.
[M3 recovery history](../architecture.md#m3-recovery-checkpoint) retains the rejected
candidate and dissent, then compiled proof/fix and later independent acceptance;
do not resurrect it as a current blocker. M6 actual Windows/WSL and 50 MB/5%
evidence remains BLOCKED. The [Testing inventory](testing.md#attention-checkpoint-gates)
separates each evidence tier. Historical MVP completion does not certify attention.

## Optional CI test builds

`.github/workflows/windows-test-build.yml` uses Ubuntu 24.04, public apt tools,
stable Rust and the existing `scripts/validate-toolchain.sh` / `scripts/package-release.sh`
commands on push to main or manual dispatch. Its token is contents:read only; it
uploads test ZIP/checksum files for 14 days, never creates Releases, tags or deployments.
It runs synthetic Rust tests under isolated HOME/XDG while retaining public compiler
caches; it is Windows-target compilation/package, not real pi-host/model/Windows/WSL
or performance evidence. Local manual builds, the full attention demo and clean-tag
release procedure remain authoritative and independent; no CI prerequisite.

Offline CI policy/verifier controls (PyYAML and actionlint are development tools,
not runtime dependencies):

```bash
actionlint .github/workflows/windows-test-build.yml
python3 scripts/test-ci-workflow.py
bash scripts/verify-ci-build.sh --self-test
```

After a separately authorized push, CI2 verifies the **actual** exact-SHA run and
download (requires authenticated `gh`, Python 3 and GNU `timeout`):

```bash
bash scripts/verify-ci-build.sh --repo SchwartzKamel/TrayGoblin --commit "$(git rev-parse HEAD)" --out-root .wgm/ci-downloads
```

The verifier bounds discovery/watch/download to 120/1500/120 seconds (kill-after 5),
retains a new owned child on failure, and reports run/artifact URLs, source SHA and
digests only after ZIP/checksum/PE/clean unpublished manifest verification. No
caller folder is deleted. Offline stubs are not GitHub boundary evidence.
See [Installation](../operator/installation.md#1-obtain-the-archive) for signed-in downloads.

## Repository layout

| Path | Contents |
|---|---|
| `src/` | Library modules plus `main.rs`; see the module map in [Architecture](../architecture.md) |
| `src/bin/tray-goblin-probe.rs` | Content-free diagnostic binary |
| `tests/monitor.rs` | Integration tests driving the monitor over fixture session folders |
| `tests/fixtures/parser/` | Event and workspace fixtures for parser boundaries |
| `tests/fixtures/live-session/` | Two sessions, one active, used for the tier-1 journey |
| `tests/fixtures/degraded-session/` | Future-format and failure fixtures for the tier-2 journey |
| `assets/` | Plain-text pixel maps rasterized into tray icons |
| `scripts/` | Toolchain validation, installer test, packaging, performance measurement, docs check |
| `specs/` | Constitution, glossary, acceptance spec, and the architecture decision |
| `docs/` | The documentation set indexed by [`docs/README.md`](../README.md) |

## Rules a change must not break

1. **Content-free parsing.** Never add a field for prompt text, assistant content, tool arguments,
   tool results, tokens, credentials, or file contents to any struct.
   `events::tests::does_not_model_sensitive_fields` is the deterministic contract.
2. **Forward compatibility.** Unknown event types, unknown fields, and malformed lines must stay
   non-fatal. Copilot's session format is internal and will change.
3. **Platform separation.** Parsing, monitoring, icon rendering, action selection, and app
   decisions stay platform-neutral; only `src/tray.rs` may use Win32 APIs.
4. **Typed, content-free core/UI errors.** No silent fallbacks; GUI startup/action messages must
   not contain a path, session identifier or operating-system message. The existing diagnostic
   probe's config-error surface is separate; see [Privacy](../operator/privacy.md).
5. **No elevation, no CI dependency.** Installation, startup registration, and normal use must work
   without administrator rights, and no validation or release step may require a hosted pipeline.
6. **Performance shape.** Keep the default cadence at one second, keep reads incremental, and do
   not add a runtime that threatens the 50 MB working-set budget.
7. **Canonical vocabulary.** Use the code and UI names from
   [`specs/CONTEXT.md`](../../specs/CONTEXT.md) for the MVP: `Generating` in code, **Working** in the UI;
   `Error` in code, **Attention needed** in the UI. Use the attention glossary for the additive model.

## Holdout rule

`scenarios/` (MVP) and `.wgm/scenarios/` (attention) contain holdout user journeys. Do not read
either while implementing; they are reserved for authorized validation and review.

## Working on the tray shell

The Win32 shell cannot be exercised on Linux. Keep decisions in `src/app.rs`, which is fully
host-testable, and keep `src/tray.rs` limited to translating decisions into Win32 calls. When a
change does touch `src/tray.rs`, it needs the manual Windows checks in [Testing](testing.md).

Cross-compilation checks that still work on Linux:

```bash
cargo build --release --target x86_64-pc-windows-gnu --bin tray-goblin
cargo clippy --all-targets --target x86_64-pc-windows-gnu -- -D warnings
```

## Working on icons

Icons are 32×32 text pixel maps in `assets/`: `#` is fill, `+` is outline, `.` is transparent.
Each state must keep a distinct silhouette as well as a distinct color, so state is never signalled
by color alone.

## Working on configuration

`src/config.rs` owns the accepted range and the parse errors. Any new key must ignore unknown
values, keep an explicit typed error for invalid input, and be documented in
[Configuration](../operator/configuration.md) and in the settings template in `src/actions.rs`,
which is kept in sync by a test.

## Before you hand work over

```bash
cargo test --workspace
cargo clippy --all-targets -- -D warnings
bash scripts/validate-toolchain.sh
bash scripts/check-docs.sh
```

Record exact commands/results in the task's governing plan: root `IMPLEMENTATION_PLAN.md` for
historical MVP tasks, `.wgm/IMPLEMENTATION_PLAN.md` for attention work. A task's exact gate must
exit zero, but passing checks do not override independent acceptance gates or
prove Windows delivery. Authors do not audit/accept themselves. The parent owns
candidate freezing, independent reviews/audit and acceptance; use the active plan
for current candidate/gates/verdicts.

## Related

- [Testing](testing.md)
- [Release responsibilities](release-responsibilities.md)
- [Documentation standards](documentation-standards.md)
- [Architecture](../architecture.md)
