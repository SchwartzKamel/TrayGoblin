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
  asserting idle dialogs stay invisible or generic busy metadata fabricates a run
  may change under the approved all-dialog/prior-state-restoration policy; replace
  them with manual-dialog attention/restoration and no-inferred-run assertions.
  Ambiguous UI-order tests must assert explicit tracking error (never resurrected
  input or false healthy restoration), under the R7 visible-degradation contract. Do not weaken
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
- **status:** done
- **acceptance record (2026-11-03):** Exact Node gate exited0,33/33 (25 core,8 actual offline pi-host), no skips. Reviewed immutable candidate c41f964725c68a9f88ef03bbdf125d52721acc01; current ruggedRUGGED, independent fresh non-author specPASS and qualityPASS (same CLI/model-family, context/lens independence). Parent checked runtime/tests identical to candidate and named branches/constants. Before regression fails3/3 on af726b0; fixed passes; queue-cap and close-slot mutants both assertion-killed. Schema includes nullable attentionReason and latched ui_tracking_unavailable. Reviewer reservation retained: corrected spec metadata was supplied worktree input, not frozen spec; final commit includes it and runtime remains byte-identical. No additional blocking reservations. Raw reports/evidence externally M1.{validation,rugged,spec,quality,judge}.log and M1.R4; current verdict `.wgm/rugged/M1-current.md`.
- **intermediate satisfaction:** Fresh independent tier1 score84.58 (steps76.67/65/97.5/83.75/100); only admitted scenario judged, tiers2/3 not opened. Missing Rust consumer/controller/demo and concurrent Working-host observation are explicit future-task gaps, not M1 defects. Judge independently reran33 tests, legacy probe(exit0) and absent --all-sessions(exit1). No Windows evidence claimed. Exactly one task advanced: M1.
- **resume / M1.R4 author handoff (2026-11-03):** Implementation and exact gate are green; leave M1 in_progress for parent acceptance. Ambiguous ID-less UI delivery now latches state=error, attentionReason=ui_tracking_unavailable and one stable episode until publisher replacement. Coherent dialogs still overlay independent lifecycle state and restore the SAME old run-error identity; final errors use run_error and non-errors null. No ctx.isIdle inference, signed debt, private host patch, load-order restriction or service. The smallest specified design uses an observed-interval boolean plus a latched fault ID; later UI callbacks cannot allocate new fault IDs or clear it.
- **M1.R4 boundedness:** At most32 retained publication snapshots INCLUDING the executing write; one extra ordered close barrier bypasses capacity. Overflow rejects with fixed write-queue-full while preserving latest in-memory state for later heartbeat recovery. Two blocked-I/O tests send80 lifecycle updates plus an overflowing prompt start: exactly32 accepted,49 overflow diagnostics, no queued heartbeats, latest rejected state recovers, and cleanup drains successfully at full capacity. Metadata/file/privacy bounds are unchanged; attentionReason is the only new allow-listed scalar.
- **M1.R4 exact results:** `node --test tests/pi-status.test.mjs tests/pi-host.test.mjs` exited0:33/33 passed,0 skipped/cancelled,22496.033217ms, Node25.2.1. Same final real-host regression files against publisher af726b0c8cb9495a9b648573e893ff54f0a2e12d in owned scenario-free scratch exited1:0/3 passed,3 failed,0 skipped,2227.559202ms. Idle/Working falsely expose their base state while NEW confirm waits; Error lacks the distinct tracking reason. Fixed regression passes all3, with stable fault through reverse delayed-start release, NEW close, another rejected dialog, actual timer-only heartbeats and error/completed/aborted lifecycle changes; publisher replacement resets healthy and coherent error restoration remains valid.
- **M1.R4 evidence:** `/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103/M1.R4/{before.command,before.log,before.exit,before.sha256,before.scratch,after.command,after.log,after.exit,candidate.sha256,results.txt,review-runtime.diff,review-tests.diff}`. The owned before-Lw4RTT copy contains the exact final host harness/tests and af726b0 publisher. First green33-test log retained as after-initial.log; final run strengthens heartbeat checks to wait from the last delivered callback, not an earlier timestamp. Queue-cap +1 mutant exits1,0/2; removing reserved-close exemption exits1,1/2 (capacity cleanup rejects write-queue-full). Mutants/logs remain external, never product edits.
- **M1.R4 author handoff (superseded remaining-gates notes):** Author inspected runtime/test diff, exact schema, scope and whitespace; changed ambiguity assertions are STRONGER typed-fault/stable-ID assertions, not skips/deletions. Existing healthy manual/error restoration, startup recovery, cleanup/ownership and sensitive-field traps remain green. Evidence is actual offline installed pi1.4 loader/ExtensionRunner/outermost UI wrapping, synthetic lifecycle and isolated HOME;8 host tests plus25 publisher tests, zero sensitive reads/network calls asserted. Not a model-backed session/compaction, Windows compile/interaction, WSL, container/service or release claim. No scenario access, reviewer dispatch, staging/commits, AGENTS edits or M2+ implementation. Parent must freeze/hash this candidate, independently judge holdouts and rerun rugged/spec/quality; prior independent FRAGILE verdict is not self-cleared by this author gate.
- **superseded R3 resume:** M1.R3 implementation is ready for parent independent reviews; do not mark done yet. Removed all ctx.isIdle inference. Dialogs overlay independently observed Idle/Working/Error; signed delivered-event balance retains unmatched ends across lifecycle hooks. A private monotonic allocator restores the SAME older error ID after dialogs; README explains Rust consumer dedupe/freshness and the unavoidable delayed-hook observation boundary. No consumers implemented.
- **superseded R3 results (2026-11-03):** Exact `node --test tests/pi-status.test.mjs tests/pi-host.test.mjs` exited0,28/28,0 skipped (Node25.2.1, isolated HOME). New actual pi1.4 loader/runner/UI-wrapper regressions exited1,0/2 on the c67522bd185e26ce18a7ddfa7a5a628b4cff0790 publisher in an owned scenario-free scratch copy: manual compaction fabricated runStartedAt; delayed earlier start observer resurrected input_required after end. Fixed gate passes both plus all retained heartbeat/startup/cleanup/ownership/privacy safeguards. Controls cover two unmatched ends/reverse start release, all five UI kinds, rejection, overlapping completion orders, Idle/error restoration,20 balanced three-dialog permutations per underlying state, and settlement while UI/debt remains.
- **superseded R3 evidence:** `/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103/M1.R3/{before.command,before.log,before.exit,after.command,after.log,after.exit,results.txt,candidate.sha256}`. First corrected-publisher gate was27/28 due to fast-end harness awaiting blocked publication; fixed synchronization waits for dispatch, then verifies published clearing after release. Initial log retained as `after-initial-harness.log` (exit1); not a waived failure. Diff/whitespace reviewed; only approved idle/busy-inference expectations replaced, no skipped/deleted tests.
- **Historical R3 scope/remaining gates (superseded):** Actual offline installed loader/runner/UI wrapper, synthetic lifecycle events and isolated filesystem; installed AgentSession compaction getters on synthetic state, NOT full model-backed compaction. Zero sensitive reads/network calls asserted. No Windows compilation/interaction, WSL, container/service, or release claim. Parent owns fresh rugged/spec/quality reviews and holdout judging; current prior FRAGILE verdict remains uncleared until those reviews. No scenarios opened, reviewers launched, commits/staging/publication, AGENTS edits, or M2+ work.

### M2 — Monitor concurrent Windows/WSL sources safely
- **objective:** Add source config, typed pi parsing and independent Copilot state while preserving legacy probe behavior.
- **files/areas:** `src/config.rs`, `src/monitor.rs`, `src/events.rs`, `src/attention.rs`, `src/lib.rs`, `src/bin/tray-goblin-probe.rs`, `tests/attention_monitor.rs`, `tests/fixtures/attention/`, `docs/operator/configuration.md`.
- **validation:** `cargo test --workspace`
- **acceptance:** R4–R5,R7 parsing,R9 compatibility. Bound16 sources,64 live sessions/source,16KiB pi file,10s stale heartbeat; independent identities/offsets. Validate malformed/partial/stale/future input safely. No historical Copilot error replay or failed-tool-only alerts. Default six-field probe unchanged; --all-sessions emits configured sources, capabilities and diagnostics. Missing config uses implicit native Copilot+pi; sessionRoot overrides Copilot only; explicit sources replaces both defaults, empty disables; duplicate IDs/unknown kinds fail. Missing implicit pi root is normal. Explicit UNC WSL reads may wake/keep a distro running; wsl_distro is attribution, not discovery/launch. Consume actual M1 schema including distinct UI-tracking-unavailable reason; restored old error IDs are not stale generations.
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
- Superseded independent M1 implementation verdict FRAGILE on af726b0: old-debt/new-dialog false healthy state reproduced through real pi1.4 wrapper. M1.R4 author remediation now passes33/33 and its final host regressions fail3/3 on af726b0; this is not independent acceptance. Its revised fault/queue-bound plan is **RUGGED** per fresh companion plan review2026-11-03, report `.wgm/rugged/plan-current.md` / external `rugged-plan-visible-fault.log`. Parent still owns the candidate's implementation verdict. Signed counts alone are explicitly rejected.
- M1 history: initial path approval failure was setup-only; startup heartbeat bug fixed with19→21 tests and before-fail/after-pass; earlier before_agent_start dialog handled with21→24 tests, but generic isIdle inference and upstream UI ordering failed independent review. c67522b current counterexamples and raw reports are retained externally; `.wgm/rugged/M1-current.md` is FRAGILE until actual correction/review passes. Details are preserved in ae90228's plan and run evidence M1.R1/M1.R2, not repeated in each fresh context.
- Last exact gates: Node24/24, Rust73/73, docs352/352, clean checkpoint; these do NOT clear independent correctness failures. Last intermediate holdout71.67 predates final dissent, not a current completion score. Tier2/3 not judged.
- Requirement map: R1–R3→M1; R4–R5→M2; R6–R7→M2/M3; R8→M4; R9–R10→M5; R11–R12→M6. Tier1 demo, tier2 mixed providers, tier3 recovery in `.wgm/scenarios/`; authors must not read them. Stop threshold95 after deterministic checks, each tier before next.

## Current handoff
M1 is DONE with33 green tests, current ruggedRUGGED and two fresh independent
PASS verdicts against c41f964. Do not reopen old inference/balance approaches;
healthy dialogs restore prior state, ambiguous ordering produces one latched
tracking-error episode until reload. Proceed M2→M3→M4→M5, one accepted task/commit
per iteration with actual validation, independent reviews and judging. M6 stays
blocked until actual Windows evidence. Runtime source/tests must remain identical
to the reviewed M1 candidate when this acceptance metadata is committed. A fresh
coordinator may delegate through the recorded external helpers, but must inspect
verdict content and verify each claimed artifact, never trust process exit0.

## Superseded M1.R4 handoff
M1.R4 implementation and exact Node gate are green33/33; all three real-wrapper
old-debt/NEW-open-dialog regressions fail on af726b0 and pass fixed. Typed stable
fault, healthy restoration and32-write/reserved-close bound are implemented.
Parent: freeze the current candidate (external M1.R4/candidate.sha256 binds product
and test files), inspect exact before/after evidence, independently judge holdouts
and run fresh rugged/spec/quality. Do not restart implementation or proceed M2
unless review finds a concrete remaining defect. Keep M1 in_progress until all
parent gates pass; do not treat author diff review or command exit0 as acceptance.
M6 remains blocked on this Linux host. No author reviewer/daemon/process remains.

## Superseded R3 handoff
M1.R3 author implementation and exact Node gate are green (28/28); uncommitted
changes and before/after evidence are ready for parent fresh rugged/spec/quality
reviews and holdout judging. Leave M1 in_progress until those gates pass.
Only with all green mark M1 done and proceed M2→M5. Keep M6
blocked until actual Windows evidence. No background author/reviewer remains from
the previous checkpoint; never launch a duplicate active writer. Parent inspects
report contents, not exit0, and preserves every independent reservation.
