# Session attention implementation plan

Full track · Ralph-full, fresh headless context per task · one task per iteration.
This follow-up governs `.wgm/specs/session-attention.md`; the root plan remains
historical completed MVP state. Root AGENTS.md and specs/CONSTITUTION.md govern.

## Goal and approved boundaries
An attention radar for many pi/Copilot sessions on native Windows and local WSL.
Pi: every explicit blocking dialog (including manual/settings/compaction dialogs)
and final run errors. Copilot: new persisted session.error only; passive input
request detection is unavailable. Successful completion and ordinary Idle stay
quiet. One Windows tray, explicit roots, local files, no daemon/network service,
conversation access, dashboard, or automatic terminal focus. User approved the pi
extension, Copilot limitation, and all-dialog policy (2026-11-03).

## Operating contract
- Never edit existing AGENTS.md. Read glossary and lean memories; search/reuse first.
- Implementers do not open either scenario directory. Parent owns independent
  judging, immutable-candidate rugged/spec/quality reviews, and final task commits.
  Authors leave in_progress, never self-certify done. Both fresh non-author review
  passes must PASS and exactly one current rugged verdict must be RUGGED.
- Exact backpressure must exit0. Every new test explains its purpose. Only tests
  asserting idle dialogs stay invisible may change under explicit user approval;
  replace them with manual-dialog attention/restoration assertions. Do not weaken
  privacy, startup recovery, ordinary-idle, completion, or event-order protections.
- Pi host is installed @earendil-works/pi-coding-agent1.4.0, Node25.2.1. Use real
  loader/runner offline, synthetic data and scrubbed/isolated HOME; no user config,
  conversation, auth state or model/network calls. Grant CLI reads of the installed
  SDK via --add-dir; tool approval alone does not grant external path access.
- Preserve evidence tiers: policy/core, real offline pi host, Windows compilation,
  and actual Windows/WSL field interaction. M6 is blocked on this Linux host.
- Version0.1.0 remains an unpublished build. Do not push/tag/publish releases.
- Runtime logs/helpers: /home/lafiamafia/.cache/traygoblin-wgm/attention-20261103.
  run-one.sh delegates one author task; check-one.sh freezes a candidate, runs exact
  gate and fresh rugged/spec/quality roles. Parent MUST inspect report verdicts;
  process exit0 only means a report exists. judge-one.sh is validator-only.

## Tasks

### M1 — Establish the real pi status signal
- **objective:** Complete the content-free pi publisher and actual-host failure/recovery signal before consumers.
- **files/areas:** `integrations/pi/{tray-goblin.js,package.json,README.md}`, `tests/pi-status.test.mjs`, `tests/pi-host.test.mjs`, `scripts/test-pi-host.mjs`.
- **validation:** `node --test tests/pi-status.test.mjs tests/pi-host.test.mjs`
- **acceptance:** R1–R3. Fixed bounded protocol v1; default ~/.traygoblin/pi and TRAY_GOBLIN_STATUS_DIR override; atomic serialized writes,2s heartbeat, ordered cleanup/ownership, fixed content-free diagnostics. Every explicit dialog overlays an independently observed Idle/Working/Error state; close/reject restores that state and the SAME underlying error identity. Never infer run state from ctx.isIdle. UI end-before-start must not resurrect closed attention; hooks may be delayed by upstream observers, so latency begins at delivered events. Episode IDs are uniquely allocated, but restoring an underlying error may restore an older ID. Final errors alert; successful completion/abort/recovered tool failures do not. Use actual pi1.4 loader/runner/UI wrapper, including manual/compaction dialogs, error preservation, earlier async UI observers, fast-end, filesystem failures, startup heartbeat recovery, pending-write shutdown, replacement ownership and sensitive-field traps.
- **status:** pending
- **resume:** M1.R3 is approved, no further interview. Replace generic-busy inference with all-dialog overlay, add real-host counterexamples for standalone compaction and dispatch inversion. Prove new regression fails against c67522bd185e26ce18a7ddfa7a5a628b4cff0790 and passes fixed. Keep unaffected existing tests; change only idle-dialog assumptions explicitly authorized above. Document the event-delivery boundary honestly, not private SDK patching or extension-load-order requirements.

### M2 — Monitor concurrent Windows/WSL sources safely
- **objective:** Add source config, typed pi parsing and independent Copilot state while preserving legacy probe behavior.
- **files/areas:** `src/config.rs`, `src/monitor.rs`, `src/events.rs`, `src/attention.rs`, `src/lib.rs`, `src/bin/tray-goblin-probe.rs`, `tests/attention_monitor.rs`, `tests/fixtures/attention/`, `docs/operator/configuration.md`.
- **validation:** `cargo test --workspace`
- **acceptance:** R4–R5,R7 parsing,R9 compatibility. Bound16 sources,64 live sessions/source,16KiB pi file,10s stale heartbeat; independent identities/offsets. Validate malformed/partial/stale/future input safely. No historical Copilot error replay or failed-tool-only alerts. Default six-field probe unchanged; --all-sessions emits configured sources, capabilities and diagnostics. Missing config uses implicit native Copilot+pi; sessionRoot overrides Copilot only; explicit sources replaces both defaults, empty disables; duplicate IDs/unknown kinds fail. Missing implicit pi root is normal. Explicit UNC WSL reads may wake/keep a distro running; wsl_distro is attribution, not discovery/launch. Consume actual documented M1 schema; restored old error IDs are not stale generations.
- **status:** pending

### M3 — Build the attention controller and source-isolation validation signal
- **objective:** Produce bounded deduplicated requests while slow WSL I/O cannot block UI or native sources.
- **files/areas:** `src/attention.rs`, `src/attention_worker.rs`, `src/lib.rs`, `tests/attention_controller.rs`, `tests/attention_monitor.rs`, `docs/architecture.md`.
- **validation:** `cargo test --test attention_monitor --test attention_controller && cargo test --workspace`
- **acceptance:** R6–R7. Distinct same-directory identities; stable episodes, no re-alert when the SAME old error returns after a manual dialog. Completion/ordinary Idle/abort/recovered tools quiet. Coalesce bursts to one request/tick; acknowledge successful native submission only. At most3 attempts/episode,>=5s apart; exhaustion stays visible, Refresh rearms; clear/obsolete pending requests dropped. Fixed owned worker/source, bounded latest snapshots, no replacement workers. Controlled delayed-reader tests: UI drain/stop<250ms, another source progresses<1s,10 stall/recovery cycles without worker growth, stop during blocked read and late publication rejected. Retain ownership until worker exit/process termination; never join blocked I/O on UI. Config immutable until restart. Fresh state/clear <=2P+500ms after delivered hook; stale after10s+2P via injected clocks.64 mixed-session stress drives actual controller/worker entry points.
- **status:** pending

### M4 — Connect native Windows notifications and session menu
- **objective:** Wire the tested attention engine into the real Win32 shell and compile the adapter.
- **files/areas:** `src/tray.rs`, `src/app.rs`, `src/actions.rs`, `src/icon.rs`, `tests/attention_controller.rs`, `docs/operator/status-reference.md`, `docs/operator/first-run.md`.
- **validation:** `cargo test --workspace && cargo build --release --target x86_64-pc-windows-gnu --bin tray-goblin`
- **acceptance:** R8 implementation tier. UI drains workers without filesystem I/O. Bounded content-free NIF_INFO requests; attention-first persistent session entries, counts/tooltip, unique identity, Copilot capability and source diagnostics. Failed submission retains pending attention. Explorer restoration does not replay acknowledged episodes. Preserve existing actions. Compilation/policy is NOT Windows delivery evidence; M6 owns that field gate.
- **status:** pending

### M5 — Package, document and run offline attention demo
- **objective:** Make the integration installable and prove shipped-publisher→Rust demo with explicit Windows/WSL setup.
- **files/areas:** `scripts/{demo-attention.sh,test-attention-windows.ps1,package-release.sh,check-docs.sh,measure-performance.ps1}`, `integrations/pi/`, `README.md`, `docs/README.md`, `docs/operator/{installation,configuration,privacy,troubleshooting,status-reference}.md`, `docs/agent/{development,testing,release-responsibilities}.md`, `docs/{architecture,manual-release,MVP}.md`.
- **validation:** `bash scripts/demo-attention.sh --container podman && bash scripts/demo.sh 0.1.0 && bash scripts/check-docs.sh`
- **acceptance:** R9–R10. Exact package allow-list includes pi integration/docs; installing pi remains opt-in. Actual shipped extension→Rust demo, identity/privacy/mixed-session checks, Windows cross-build/package and preserved MVP demo. Exact Windows+WSL examples, install/remove, all-dialog/no-completion policy, Copilot limitation, stale/unavailable/UNC wake behavior, notification suppression. Field script rejects non-Windows and collects actual observations, never synthetic PASS. OCI CLI-only validation: non-root/network-disabled Debian bookworm-slim, read-only host-release probe+fixtures, assert actual JSON; no secrets or pi/Windows proof inferred. Performance -Path binds PID/path/hash and rejects mismatched existing processes. Final five-role docs audit recorded.
- **status:** pending

### M6 — Obtain real Windows/WSL interaction and resource evidence
- **objective:** Prove actual delivery, menu/recovery and resource budgets on the operator's target host before release readiness.
- **files/areas:** `scripts/test-attention-windows.ps1`, `scripts/measure-performance.ps1`, `.wgm/rugged/`, this plan.
- **validation:** `pwsh -NoProfile -File scripts/test-attention-windows.ps1 -ExecutablePath target/x86_64-pc-windows-gnu/release/tray-goblin.exe -EvidencePath .wgm/windows-attention.json && pwsh -NoProfile -File scripts/measure-performance.ps1 -Path target/x86_64-pc-windows-gnu/release/tray-goblin.exe -DurationSeconds 30 -JsonPath .wgm/windows-performance.json`
- **acceptance:** R11–R12. Real native+WSL pi dialog/error and Copilot error; quiet completion3P; attention/clear<=2P+500ms after delivered hook; burst/dedupe; Focus Assist retains menu; pi reload/crash, Explorer restart/no replay, clean quit. WSL shutdown immediately after successful read: native monitoring continues3P, record actual UNC wake behavior, source removal/restart stops reads; diagnose unavailable after10s+2P and recover<=2P+500ms after successful access. Record UTC/buildSHA/executableSHA256/PID/path/Windows+WSL versions/source kinds/load and genuine per-case observations. Measure50MB/5% budget; performance digest matches candidate. No Linux/mock substitute.
- **status:** blocked
- **notes:** Linux development host has no actual Windows/WSL runtime. Operator action after M5; standing release-readiness blocker, not proof owed by offline core/compilation tasks.

## Evidence, history and current gates
- Original planning commit8e0b193; WIP checkpointae90228. No follow-up task done.
- Full-track baseline docs audit: `.wgm/docs/audit/2026-11-03T2148Z_attention-baseline.md`, structural exit0, AMBER dispositions in index. Four fresh Copilot persona contexts plus writer; same tooling/model-family, context/lens independence only.
- Original plan ruggedRUGGED/readiness98 preceded implementation. Updated all-dialog policy explicitly approved; fresh rugged companion plan re-review is **RUGGED**,2026-11-03, report `.wgm/rugged/plan-current.md` (raw `rugged-plan-policy.log`). Exact manual/compaction/error-restoration and dispatch-inversion checks map to M1, updated tier1 holdout covers them, constitution/consistency checks PASS. Readiness98; prior Full baseline audit retained. Implementation remains FRAGILE until actual correction/reviews pass.
- M1 history: initial path approval failure was setup-only; startup heartbeat bug fixed with19→21 tests and before-fail/after-pass; earlier before_agent_start dialog handled with21→24 tests, but generic isIdle inference and upstream UI ordering failed independent review. c67522b current counterexamples and raw reports are retained externally; `.wgm/rugged/M1-current.md` is FRAGILE until actual correction/review passes. Details are preserved in ae90228's plan and run evidence M1.R1/M1.R2, not repeated in each fresh context.
- Last exact gates: Node24/24, Rust73/73, docs352/352, clean checkpoint; these do NOT clear independent correctness failures. Last intermediate holdout71.67 predates final dissent, not a current completion score. Tier2/3 not judged.
- Requirement map: R1–R3→M1; R4–R5→M2; R6–R7→M2/M3; R8→M4; R9–R10→M5; R11–R12→M6. Tier1 demo, tier2 mixed providers, tier3 recovery in `.wgm/scenarios/`; authors must not read them. Stop threshold95 after deterministic checks, each tier before next.

## Current handoff
User approved all explicit pi dialogs. Revise/commit policy and exact validation
design, run M1.R3 through fresh author and real-host tests, then fresh rugged/spec/
quality reviews. Only with all green mark M1 done and proceed M2→M5. Keep M6
blocked until actual Windows evidence. No background author/reviewer remains from
the previous checkpoint; never launch a duplicate active writer. Parent inspects
report contents, not exit0, and preserves every independent reservation.
