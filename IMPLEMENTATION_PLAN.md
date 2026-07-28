# Implementation plan

> Shared state for the Full-track Ralph loop. One task advances per iteration.

**Current status:** T1–TZ are complete. The deterministic MVP release candidate is assembled; real Windows interaction and performance measurement remain operator release gates.

## Convergence
- **Satisfaction threshold:** 95 — the mean holdout step score after deterministic checks pass; every tier-1 scenario must score at least 95 before tier 2 is judged, and no individual scenario may score below 90
- **Stratified order:** tier 1, then tier 2
- **Scenarios:** `scenarios/` (Validate/Review only; never read during Implement)
- **Release target:** `v0.1.0`, Windows x86-64 portable ZIP, manually built and published

## Now

### T1 — Establish a deterministic Rust and Windows cross-build signal
- **files/areas:** `Cargo.toml`, `rust-toolchain.toml`, `.cargo/config.toml`, `src/lib.rs`, `src/main.rs`, `scripts/validate-toolchain.sh`
- **validation:** `bash scripts/validate-toolchain.sh`
- **acceptance:** stable Rust runs a host-side test and produces a Windows x86-64 PE executable without CI; missing tools fail with actionable output
- **scenarios/tier:** enables all scenarios
- **status:** done
- **notes:** 2026-07-28 — `bash scripts/validate-toolchain.sh` exited 0: one host test passed and `target/x86_64-pc-windows-gnu/release/tray-goblin.exe` was verified as a PE32+ x86-64 executable. A PATH-isolated check also proved a missing MinGW linker fails with an installation hint. Tier-1 satisfaction is 0 (baseline; no regression): `active-turn-magic-moment` cannot run until T2–T3 create the fixture parser and `tray-goblin-probe`; dominant diagnostic `missing_bin_target`. Tier 2 was not opened because tier 1 has not converged.

### T2 — Parse content-free Copilot session metadata and events
- **files/areas:** `src/events.rs`, `src/session.rs`, `src/status.rs`, `tests/fixtures/parser/`
- **validation:** `cargo test --lib`
- **acceptance:** typed parsing extracts only state, timestamps, model, success, repository, and directory; unknown future event types, extra fields, and malformed lines are non-fatal; sensitive fields are never modeled; create all parser fixtures named by this task
- **scenarios/tier:** advances `active-turn-magic-moment` tier 1 and `degraded-session-state` tier 2
- **status:** done
- **notes:** 2026-07-28 — `cargo test --lib` exited 0 with 10 tests. Typed parsers now retain only event kind, RFC 3339 timestamp, model, success, repository, and active directory; synthetic fixtures prove unknown event types, extra fields, malformed lines, nested workspace metadata, and sensitive-field exclusion remain non-fatal and content-free. Tier-1 `active-turn-magic-moment` satisfaction improved 0→16.7 with no regression; the expected `missing_bin_target` diagnostic remains because T3 owns snapshot assembly, the live-session fixture, and `tray-goblin-probe`. Tier 2 was not opened because tier 1 has not converged.

### T3 — Build the polling monitor, configuration, and diagnostic probe
- **files/areas:** `src/monitor.rs`, `src/config.rs`, `src/bin/tray-goblin-probe.rs`, `tests/monitor.rs`, `tests/fixtures/live-session/`, `tests/fixtures/degraded-session/`
- **validation:** `cargo test --lib && cargo test --test monitor`
- **acceptance:** newest active session is selected, event offsets are cached, Working/Idle/Attention needed transitions are correct, 500–10,000 ms configuration is enforced, the degraded fixture proves future-format compatibility, and the probe emits content-free JSON
- **scenarios/tier:** completes deterministic behavior behind both scenarios
- **status:** done
- **notes:** 2026-07-28 — `cargo test --lib && cargo test --test monitor` exited 0 with 32 tests; `cargo clippy --all-targets -- -D warnings` and `bash scripts/validate-toolchain.sh` also exited 0. The monitor selects the most recently modified active lock, caches complete JSONL offsets without losing partial writes, rebuilds state when sessions reactivate, enforces 500–10,000 ms polling, and emits an explicit content-free probe schema. Review added regressions for partial lines, timestamp-less turns, session reactivation, and stale duration removal. Tier-1 and tier-2 holdouts both scored 100 after aligning Attention needed to remain visible until the next turn starts.

### T4 — Implement the native Windows tray shell and actions
- **files/areas:** `src/app.rs`, `src/actions.rs`, `src/icon.rs`, `src/main.rs`, `assets/`
- **validation:** `cargo test --workspace && cargo build --release --target x86_64-pc-windows-gnu --bin tray-goblin`
- **acceptance:** Windows build has Idle/Working/Attention needed icons and tooltip, a 1-second timer, immediate refresh, Open in VS Code, View Copilot logs, Open settings, and Quit; action failures remain content-free
- **scenarios/tier:** exposes tier-1 behavior through the tray
- **status:** done
- **notes:** 2026-07-28 — `cargo test --workspace && cargo build --release --target x86_64-pc-windows-gnu --bin tray-goblin` exited 0 with 73 tests and a PE32+ Windows GUI executable. Host and Windows-target clippy also exited 0. The native Win32 shell now provides distinct state silhouettes and labels, bounded tooltips, configured polling with immediate refresh, the five required actions, notification-area retry after Explorer startup/restart, explicit content-free configuration/startup errors, and safe launch failure reporting. Both holdout tiers remained at 100; live Windows interaction remains the documented manual validation.

### T5 — Add per-user installation and deterministic release packaging
- **files/areas:** `install.ps1`, `uninstall.ps1`, `scripts/test-installer.ps1`, `scripts/package-release.sh`, `scripts/measure-performance.ps1`
- **validation:** `pwsh -NoProfile -File scripts/test-installer.ps1 && bash scripts/package-release.sh 0.1.0`
- **acceptance:** scripts require no elevation, install below LocalAppData, create/remove a Startup shortcut, preserve user configuration on uninstall by default, and produce ZIP plus SHA-256 files
- **scenarios/tier:** makes the demo installable
- **status:** done
- **notes:** 2026-07-28 — `pwsh -NoProfile -File scripts/test-installer.ps1 && bash scripts/package-release.sh 0.1.0` exited 0 with 19 checks passed and one Windows-only shortcut check explicitly skipped on Linux. Installation defaults below LocalAppData, uses an ownership marker for interruption-safe repair/removal, scopes process termination to the exact installed executable, removes Startup registration before folder work, preserves configuration by default, and removes only `config.json` on opt-in. Packaging produced a reproducible Windows x86-64 ZIP and verified SHA-256 file; `shellcheck scripts/package-release.sh` also exited 0. The performance script deterministically gates 50 MB working set and 5% normalized idle CPU; Windows measurement remains required before stable promotion.

### T6 — Document usage, privacy, architecture, and manual release operation
- **files/areas:** `README.md`, `docs/MVP.md`, `docs/architecture.md`, `docs/manual-release.md`, `AGENTS.md`, `scripts/check-docs.sh`
- **validation:** `bash scripts/check-docs.sh`
- **acceptance:** `docs/architecture.md`, `docs/manual-release.md`, and `scripts/check-docs.sh` all exist and are validated; a new user can install, interpret every state, configure polling, troubleshoot Copilot format changes, and reproduce the no-CI release; docs state that Windows performance must pass before promoting preview stability
- **scenarios/tier:** documents the complete user journey
- **status:** done
- **notes:** 2026-07-28 — `bash scripts/check-docs.sh` exited 0 with 327 checks and two explicit skips for the planned TZ demo script and unavailable host `pwsh`; `cargo test --workspace` also exited 0. The documentation now has a shared index, six Operator guides, four Agent guides, architecture and reproducible manual-release references, refreshed README/MVP content, and an offline checker for audiences, links, commands, versions, task status, and shell syntax. Release instructions distinguish same-executable package determinism from cross-machine build reproducibility and include release-notes creation, abort, and rollback procedures. Root `AGENTS.md` was intentionally not edited because WGM artifact-safety requires explicit operator approval.

### TZ — Run the end-to-end demo and assemble the release candidate
- **files/areas:** `scripts/demo.sh`, `tests/fixtures/live-session/`, `tests/fixtures/degraded-session/`, `dist/`
- **validation:** `bash scripts/demo.sh 0.1.0`
- **acceptance:** host tests pass; tier-1 and tier-2 fixture journeys produce expected content-free snapshots; the Windows release build is a PE executable; package contents and checksum are correct
- **scenarios/tier:** validates both holdout journeys in tier order
- **status:** done
- **notes:** 2026-07-28 — `bash scripts/demo.sh 0.1.0` exited 0. The single command ran 73 workspace tests, verified the Windows PE32+ x86-64 build, judged the live and degraded fixture journeys in tier order against the six-field content-free probe allow-list, rebuilt the reproducible ZIP, asserted exact archive entries and packaged-executable identity, and verified the SHA-256 file. `bash scripts/check-docs.sh` and `shellcheck scripts/demo.sh` also exited 0. The release candidate remains in `dist/`; Windows tray interaction, Startup behavior, and the 50 MB / 5% performance gate remain manual Operator checks.

## Later
- Add an official Copilot plugin or OpenTelemetry provider if the CLI publishes a stable real-time contract.
- Add ARM64, signing, MSI/MSIX, auto-update, token visualization, and additional AI CLI providers.

## Done

- **T1 — Establish a deterministic Rust and Windows cross-build signal** (2026-07-28): stable host test and local MinGW PE verification are green.
- **T2 — Parse content-free Copilot session metadata and events** (2026-07-28): parser and privacy fixture tests are green; tier-1 scoring now awaits T3's monitor and probe.
- **T3 — Build the polling monitor, configuration, and diagnostic probe** (2026-07-28): deterministic monitor/probe tests, both holdout tiers, clippy, and the Windows cross-build are green.
- **T4 — Implement the native Windows tray shell and actions** (2026-07-28): host tests, Windows cross-build/clippy, accessible state rendering, actions, timer scheduling, and safe error paths are green.
- **T5 — Add per-user installation and deterministic release packaging** (2026-07-28): installer safety checks, reproducible ZIP/checksum packaging, and performance gate scripts are green.
- **T6 — Document usage, privacy, architecture, and manual release operation** (2026-07-28): the Operator/Agent documentation set and deterministic offline docs checker are green.
- **TZ — Run the end-to-end demo and assemble the release candidate** (2026-07-28): the full deterministic demo produced and verified the Windows ZIP and checksum release candidate.
