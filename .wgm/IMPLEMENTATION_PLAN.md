# Session attention implementation plan

Full track · Ralph-full (fresh headless Copilot invocations) · one task per iteration.
This follow-up plan governs the delta in `specs/session-attention.md`. The root
`IMPLEMENTATION_PLAN.md` remains the historical completed MVP plan.

## Goal and boundaries
An attention radar for many pi/Copilot sessions on native Windows and local WSL.
Pi input + terminal-error alerts; Copilot persisted-error alerts only. Normal
completion stays quiet. Native notifications + persistent session menu; no daemon,
network collector, prompt access, dashboard, or automatic terminal focus.
User approved the opt-in pi extension and Copilot input-detection limitation.

## Operating contract
- Root `AGENTS.md` and `specs/CONSTITUTION.md` still govern; never edit AGENTS.md.
- Read `.wgm/specs/CONTEXT.md` and memories before each task. Search before building.
- Implementers do not read either scenario directory. Parent orchestrator owns
  holdout judging, separate fresh spec and quality reviewers, rugged gates, and
  commits. Implementers leave work `in_progress`, never self-certify done.
- Every new test explains the behavior it protects. Do not weaken existing tests.
- Pin pi host validation to installed `@earendil-works/pi-coding-agent` 1.4.0;
  use the actual SDK loader/runner offline, isolated test HOME and synthetic data.
- Exact backpressure must exit 0 and both fresh non-author reviews PASS before
  `done`. Bind reviews to an immutable candidate tree/commit; preserve reservations.
- Scope evidence honestly: source policy, real pi host, Windows compilation, and
  actual Windows/WSL interaction are distinct tiers. M6 is an operator boundary and
  must remain blocked/unverified on Linux; never invent Windows measurements.
- Keep packaging version 0.1.0; this is unreleased follow-up work, not publication.
- Durable plans/specs/scenarios/audits are tracked; runtime logs/memories remain
  ignored. Runtime logs live under `.wgm/` or an external run directory.

## Tasks

### M1 — Establish the real pi status signal
- **objective:** Ship the smallest content-free pi publisher and prove its real-host lifecycle before building consumers.
- **files/areas:** `integrations/pi/tray-goblin.js`, `integrations/pi/package.json`, `integrations/pi/README.md`, `tests/pi-status.test.mjs`, `tests/pi-host.test.mjs`, `scripts/test-pi-host.mjs` if needed.
- **validation:** `node --test tests/pi-status.test.mjs tests/pi-host.test.mjs`
- **acceptance:** R1–R3. Fixed v1 JSON schema; native/WSL default `~/.traygoblin/pi`, `TRAY_GOBLIN_STATUS_DIR` override; actual pi 1.4.0 loader/runner with delayed/resolved/rejected blocking UI, final error vs success/abort/recovery, serialized atomic writes, 2s heartbeat and ordered idempotent cleanup with injected write/rename/delete failure, pending-write shutdown, replacement ownership protection, and content-free failure reporting. No sensitive getter/history access. No global user config changes or model/network calls.
- **status:** pending
- **notes:** Protocol consumer tasks depend on the exact tested schema this task documents. Metadata/episode fields may be named by the implementer but must be explicit, bounded, and allow-listed in both languages.

### M2 — Monitor concurrent Windows/WSL sources safely
- **objective:** Add source configuration, typed pi parsing, and independent Copilot session state while preserving legacy behavior.
- **files/areas:** `src/config.rs`, `src/monitor.rs`, `src/events.rs`, `src/attention.rs`, `src/lib.rs`, `src/bin/tray-goblin-probe.rs`, `tests/attention_monitor.rs`, `tests/fixtures/attention/`, `docs/operator/configuration.md`.
- **validation:** `cargo test --workspace`
- **acceptance:** R4–R5, R7 parser/monitor portion, R9 compatibility. Bound 16 sources, 64 live sessions/source, 16 KiB pi files, 10s stale heartbeat; independent state/offsets and generation identity; malformed/partial/stale/future input safely degrades. No historical Copilot error replay; only new session.error episodes, not failed tools. Preserve original six-field probe default; add `--all-sessions` using configured sources with explicit capability/diagnostic data. Test missing config, legacy sessionRoot overriding implicit Copilot only, explicit sources replacing both implicit roots, explicit empty sources, duplicate IDs and invalid kinds. Default absent pi root is normal. WSL distro is attribution only; explicit UNC reads may wake/keep a distro running. Do not promise atomic no-wake behavior or add per-tick distro subprocesses. Document the config/public probe delta.
- **status:** pending

### M3 — Create the attention failure/recovery validation signal
- **objective:** Turn source snapshots into bounded deduplicated notification requests without letting slow WSL I/O block Windows sessions/UI.
- **files/areas:** `src/attention.rs`, `src/attention_worker.rs`, `src/lib.rs`, `tests/attention_controller.rs`, `docs/architecture.md`.
- **validation:** `cargo test --test attention_monitor --test attention_controller && cargo test --workspace`
- **acceptance:** R6–R7. Stable episode identity; no repeat alerts for persistent attention; successful completion/abort/recovered tool errors quiet; same-directory sessions remain distinct. Coalesce bursts into one request/tick; acknowledge only successful adapter submission. Per-source fixed worker ownership, bounded latest-result channels, stale/late generations rejected, UI drain/stop below 250ms in controlled delay tests and other-source progress within 1s; ten stall/recovery cycles without worker growth, stop during blocked read, and late publication rejected. No in-process source replacement; retain blocked ownership until exit/process termination. Notification failure: at most 3 attempts/episode, >=5s apart, no obsolete delivery, exhaustion retains visible attention, Refresh explicitly rearms. No WSL liveness cache/subprocess: read explicit paths only and document UNC wake behavior. Errors persist through Idle and notification acknowledgement, clear on new provider work; input clears on dialog end; instance removal/replacement drops old pending work. Fresh state/clear deadline2P+500ms; staleness10s+2P via injected clocks. 64 mixed-session stress uses real controller/worker entry points. This strengthened validation-signal task was spawned by the initial UNKNOWN rugged plan verdict.
- **status:** pending

### M4 — Connect native Windows notifications and the session menu
- **objective:** Wire the tested attention policy into the Windows tray and compile the real Win32 adapter.
- **files/areas:** `src/tray.rs`, `src/app.rs`, `src/actions.rs`, `src/icon.rs`, `tests/attention_controller.rs`, `docs/operator/status-reference.md`, `docs/operator/first-run.md`.
- **validation:** `cargo test --workspace && cargo build --release --target x86_64-pc-windows-gnu --bin tray-goblin`
- **acceptance:** R8 implementation tier. UI polls latest worker results without filesystem I/O; bounded content-free NIF_INFO notification requests and attention-first persistent session entries; aggregate tooltip/counts and distinct identity; Copilot input capability and source diagnostics visible. Notification submission failure retains pending attention. Explorer icon restoration doesn't replay acknowledged alerts. Preserve existing actions. Windows compilation is not notification delivery evidence: M6 owns that check explicitly.
- **status:** pending

### M5 — Package, document, and run the offline attention demo
- **objective:** Make the integration installable and prove the smallest real-host-to-consumer demo with clear Windows/WSL instructions.
- **files/areas:** `scripts/demo-attention.sh`, `scripts/test-attention-windows.ps1`, `scripts/package-release.sh`, `scripts/check-docs.sh`, `scripts/measure-performance.ps1`, `integrations/pi/`, `README.md`, `docs/README.md`, `docs/operator/{installation,configuration,privacy,troubleshooting,status-reference}.md`, `docs/agent/{development,testing,release-responsibilities}.md`, `docs/architecture.md`, `docs/manual-release.md`, `docs/MVP.md`.
- **validation:** `bash scripts/demo-attention.sh --container podman && bash scripts/demo.sh 0.1.0 && bash scripts/check-docs.sh`
- **acceptance:** R9–R10. Exact package allow-list includes the pi extension/docs; installer remains opt-in about pi configuration. Published Windows + WSL source examples, extension install/remove, capability limitation, no-completion-alert policy, notification suppression, stopped distro/stale publisher troubleshooting. Demo uses shipped extension and Rust entry points, verifies privacy and mixed-session isolation, builds Windows artifact and package, preserves original MVP gate. New Windows field script fails explicitly off-Windows and records actual operator observations rather than manufacturing a pass. Exact OCI gate is `bash scripts/demo-attention.sh --container podman`: non-root/network-disabled Debian bookworm-slim, read-only host-release probe and fixtures, actual JSON asserted; no secrets or pi/Windows proof inferred. Field script parameters/evidence follow spec Failure/recovery contract; performance -Path rejects an existing mismatched process and reports measured PID/path/hash. Final five-role docs audit is recorded.
- **status:** pending

### M6 — Obtain real Windows/WSL interaction and resource evidence
- **objective:** Validate actual notification/menu delivery and recovery on the operator's Windows+WSL environment before claiming the feature release-ready.
- **files/areas:** `scripts/test-attention-windows.ps1`, `scripts/measure-performance.ps1`, `.wgm/rugged/`, this plan.
- **validation:** `pwsh -NoProfile -File scripts/test-attention-windows.ps1 -ExecutablePath target/x86_64-pc-windows-gnu/release/tray-goblin.exe -EvidencePath .wgm/windows-attention.json && pwsh -NoProfile -File scripts/measure-performance.ps1 -Path target/x86_64-pc-windows-gnu/release/tray-goblin.exe -DurationSeconds 30 -JsonPath .wgm/windows-performance.json`
- **acceptance:** R11–R12. Real Windows-origin observations: native+WSL pi blocked dialogs/error, Copilot persisted error, quiet normal completion, burst/dedupe, suppressed notification retains menu attention, WSL shutdown/recovery, pi reload/crash, Explorer restart, clean quit, 50 MB/5% budget. Field procedure asserts attention/clear <=2P+500ms, quiet completion over3P, WSL shutdown immediately after a positive source read, native monitoring continuing over3P, actual UNC wake behavior recorded (not prohibited), source removal/restart disabling further reads, recovery<=2P+500ms after successful file access, and the remaining spec Failure/recovery cases. Record UTC/build SHA/executable hash/PID/path/Windows+WSL versions/source kinds/load and real per-case observations. Performance report must match candidate hash. No Linux proxy or mocked adapter can satisfy this task.
- **status:** blocked
- **notes:** Development host is Linux; no Windows/WSL runtime is available here. Operator action required after M5. This is a standing release-readiness blocker, not a reason to pretend offline core/compilation evidence is live Windows proof.

## Review and validation design
- Real pi lifecycle: M1 command, Node 25.2.1 / pi 1.4.0, isolated HOME; exact state/episode fields and cleanup; missing hook, payload access, stale resources, failed publication fail loudly.
- Core/stress: M2/M3 commands on Linux Rust and Windows-target compilation; 64 concurrent sessions, one failed/stalled source, partial/stale input, reactivation/replacement; recovery preserves other sources and fresh episodes.
- Windows adapter: M4 command proves policy seam and FFI compilation, not OS delivery. Live delivery/recovery and performance are separately owed at M6.
- End-to-end/demo: M5 exact command; tier-1 real pi→Rust attention, tier-2 mixed providers, tier-3 lifecycle degradation; satisfaction threshold 95 after deterministic checks, each tier before the next.
- Plan consistency: R1–R3→M1; R4–R5→M2; R6–R7→M2/M3; R8→M4; R9–R10→M5; R11–R12→M6. Every task has objective, areas, exact command, acceptance, and status.

## Plan gates
- Rugged plan current verdict: **RUGGED**, 2026-11-03, session-attention follow-up plan, discoverable rugged companion through a fresh headless context; report `.wgm/rugged/plan-current.md`. No new task spawned on final rerun. Superseded history: 2026-11-03 companion plan review UNKNOWN; created/strengthened validation-signal task M3 and exact M1/M2/M5/M6 failure contracts. Initial report retained in `.wgm/rugged/plan-initial.md`. The open issues were ambiguous default activation, unmeasured worker/cleanup/retry boundaries, WSL behavior, OCI invocation, and candidate-bound field/performance evidence. Baseline audit F1–F4 then clarified provider error clearing, default Source vocabulary and navigation; the unrealizable atomic no-wake promise was removed in favor of explicit configured UNC reads with documented wake behavior, no added liveness subprocess/cache, and actual Windows-origin shutdown observations.
- Full-track baseline docs audit: completed 2026-11-03, four independent fresh Copilot persona contexts plus writer (same tooling/model-family, context/lens independence only), `.wgm/docs/audit/2026-11-03T2148Z_attention-baseline.md`; structural checker exit0. AMBER F1–F4 dispositions in its index; final rugged review confirms no blocking planning gap.
- Consistency: PASS; requirement→task map above covers all deltas; tiered holdouts cover demo/concurrency/recovery; historical MVP acceptance remains unchanged and is not confused with follow-up readiness. Spec-quality14/14.
- Preflight readiness: **98/100** (goal20, observable success20, scenario coverage18, backpressure20, scope20); real Windows field evidence remains explicitly blocked M6. Planning baseline committed before starting M1; verify `git status --short` is empty.

## Current handoff
Grill, consistency, rugged plan, and Full baseline docs audit gates are complete.
Start M1 with the bundled skill loop runner; parent orchestrator retains reviews,
holdout judging and commits. External run directory is
`/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103`; `run-one.sh` invokes a
fresh implementer for one task, `review-one.sh` invokes fresh rugged/spec/quality
contexts. Logs/metrics stay there. Never open holdouts as an implementer.
