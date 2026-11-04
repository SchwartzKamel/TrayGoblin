# Session attention implementation plan

Full track · Ralph-full, fresh author/reviewer contexts · one task per iteration.
Root AGENTS.md and specs/CONSTITUTION.md govern; never edit existing AGENTS.md.
This follow-up governs `.wgm/specs/session-attention.md`; root plan is MVP history.

## Current state — 2026-11-04 UTC
**M1/M2/M3/M4 accepted; M5 pending; M6 field-blocked.**
Accepted commits: M1 `0d79111`, M2 `058ef74`, M3 `f4ef6de`; M4 acceptance is
recorded below and committed with the implementation. WIP `79e9220` preserves prior dissent, not its later
resolution. Historical recovery evidence remains in that commit and external run
directory; this compact plan is live state. Native tray wiring is accepted at
implementation/compilation tier, not proven on a Windows runtime.

## Approved goal and boundaries
One Windows tray alerts the operator which pi/Copilot session needs attention
across native Windows and local WSL. Pi: all explicit blocking dialogs (including
manual/compaction/settings) and final run errors. Copilot: new persisted errors
only; passive input detection unavailable. Completion/ordinary Idle/abort/recovered
tools quiet. Metadata only; optional manual label, no conversation/args/results/
tokens/credentials. Native notifications + persistent list; no daemon, server,
dashboard, remote SSH collector, agent control or automatic terminal focus.
Pi extension and all-dialog/visible-tracking-fault policies have user approval.

## Build/review contract
- Authors work only one task, do not open either scenario directory, launch
  reviewers, commit or self-certify done. Parent owns commits and acceptance.
- Done requires exact gate exit0, current ruggedRUGGED and fresh non-author
  specPASS+qualityPASS against an immutable candidate. Inspect report content;
  exit0 only means a report exists. Preserve dissent and checked source hashes.
- Tests explain transitions; do not skip/delete/weaken existing protection. The
  explicitly approved all-dialog policy superseded only idle-invisibility/busy
  inference assertions; ambiguity must assert typed tracking error, not false health.
- Bug regressions must compile and assertion-fail on the rejected source, pass
  fixed, then the COMPLETE exact gate passes. Keep raw before/after provenance.
- Use synthetic fixtures, isolated HOME/XDG and verified public compiler caches;
  never inspect actual user conversation/auth/config. Format only touched files.
- Preserve evidence tiers: policy/core, actual offline pi1.4 host, Windows
  compilation, real Windows/WSL interaction. Never convert one into another.
- No push/tag/release/upstream publication delegated. Version0.1.0 stays unpublished.
- External helpers/logs: `/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103`.
  run-one.sh/check-one.sh/review-one.sh/judge-one.sh supply fresh contexts. Do not
  edit a helper while it runs. Stage explicit owned paths, not broad baseline
  diffs/holdouts. Preserve candidate refs and ensure accepted runtime matches review.
- Current stronger-model escalation: pi/github-copilot/gpt-6.1-sol,xhigh; print,
  no-session/no-extension/no-MCP context. Prior Copilot diagnostic path-approval
  denial was setup-only. Authorized primary pi diagnostic established actual proof.

## Tasks

### M1 — Publish real pi attention status
- **objective:** Content-free publisher proven through actual installed pi loader/runner.
- **files/areas:** `integrations/pi/`, `tests/pi-{status,host}.test.mjs`, `scripts/test-pi-host.mjs`.
- **validation:** `node --test tests/pi-status.test.mjs tests/pi-host.test.mjs`
- **acceptance:** R1–R3.16KiB scalar protocol; independent lifecycle state/dialog overlay; original error ID restored; ambiguous ID-less hooks latch one ui_tracking_unavailable episode until reload. Nullable attentionReason distinguishes run_error;2s coalesced heartbeat,32 pending writes+reserved close, atomic ownership-aware shutdown; privacy traps.
- **status:** done
- **evidence:** Commit0d79111 matches reviewed runtime c41f964;33/33 tests (25 core,8 actual offline pi1.4 host). Fresh ruggedRUGGED/specPASS/qualityPASS. Before-fail/after-pass and cap/close mutants retained under M1.R4; current verdict `.wgm/rugged/M1-current.md`. Spec-metadata reservation preserved in prior plan; final commit includes corrections, code byte-identical. Tier1 intermediate84.58, consumer/demo then absent. No live Windows proof.

### M2 — Concurrent content-free sources and aggregate probe
- **objective:** Independent pi/Copilot state, bounded parsing/config and legacy compatibility.
- **files/areas:** `src/{config,monitor,events,attention,lib}.rs`, `src/bin/tray-goblin-probe.rs`, `tests/attention_monitor.rs`, `tests/fixtures/attention/`, `docs/operator/configuration.md`.
- **validation:** `cargo test --workspace`
- **acceptance:** R4–R5,R7 parsing,R9.16 sources,64 live rows/source,4096 scan/membership bound,16KiB+1 pi reads,10s heartbeat expiry; UUID regular files only, no symlink escape. Stale files do not consume live capacity. Native defaults, legacy sessionRoot override, explicit sources replace defaults/empty disables. New --all-sessions; old six-field probe unchanged. New persisted Copilot error only; no historical/tool-only alert. UNC can wake WSL; explicit paths, no distro launch/discovery. Restored pi error ID may decrease; freshness is not episode direction.
- **status:** done
- **evidence:** Commit058ef74;93 tests, same actual publisher→parser/probe boundary, read-cap/directory-pinning mutants. RuggedRUGGED + fresh spec/qualityPASS; Source saturation regression fixed M2.R1. Direct publisher/core integration is not a new pi-host or Windows field result. No blocking reservations.

### M3 — Controller, source isolation and stable provenance
- **objective:** Low-noise bounded delivery and nonblocking source workers, with identity/time provenance surviving recovery.
- **files/areas:** `src/{attention,attention_worker,monitor,lib}.rs`, `tests/{attention_controller,attention_monitor}.rs`, `docs/architecture.md`, `docs/operator/configuration.md` for episode semantics.
- **validation:** `cargo test --test attention_monitor --test attention_controller && cargo test --workspace`
- **acceptance:** R6–R7. Distinct identities; no replay of acknowledged/same restored error; preserve exhausted budgets. One coalesced request/tick;3 attempts/episode >=5s apart; success-only acknowledgement; explicit Refresh rearm of undelivered only.16 fixed owned workers, bounded command/latest slots, no UI I/O/join/replacement threads. Controlled drain/stop<250ms, other-source progress<1s,10 stall cycles/no growth, late/stale results rejected. Publisher-derived expiry survives blocked reads; parse clock sampled after actual read. Typed membership/completeness, not display projection, governs removals;4096 retained identities/source × at most4 reason slots, visible overflow instead of silent eviction.64 mixed real-parser/worker/controller stress; fresh/clear<=2P+500ms, stale<=10s+2P. M1 byte-identical.
- **status:** done
- **acceptance record (2026-11-04 UTC):** Exact isolated-HOME gate exited0:52 targeted/136 workspace,0 failed/ignored. Runtime/test/docs match immutable e1221e2b77d8ea9bc453151505c7b0288f09d770; fresh ruggedRUGGED, non-author specPASS and qualityPASS, no blocking reservations. Parent verified source-equivalence/symbols and authoritative counts. Actual rejected/fixed before-after evidence: R4 new-error collision20/20 fails/20 passes; R4.1 pending/exhausted/ack partial-scan regressions20/20 failures then20 passes; full34-controller suite10/10 passes. Global checked IDs, bounded membership-governed parser retention/admission and explicit diagnostics preserve old delivery and fresh independent budgets without changing controller/worker policy. M1 bytes unchanged. Review identities: fresh CopilotCLI contexts (model family unqueried), stronger author pi/gpt-6.1-sol xhigh; context/lens independence, not claimed cross-family advantage. Raw M3.{validation,rugged,spec,quality,judge}.log, M3.R4/M3.R4.1 evidence; current verdict `.wgm/rugged/M3-current.md`.
- **intermediate satisfaction:** Fresh independent tier1 score93.37, no tier2/3 opened; dominant missing-continuous-demo (two real-host publishers feeding consumer in one journey). Existing core64-session behavior is proven; M5 owes continuous shipped-host→Rust demo. Incomplete integration evidence, not a remaining M3 defect. Exactly one task advanced: M3.
- **Historical R4 implementer handoff (superseded):** Narrow repair implemented in `src/monitor.rs`: checked process-lifetime atomic Copilot IDs plus retention of the existing bounded parser cache through temporary source loss. `src/attention.rs` adds only the content-free exhaustion diagnostic; controller/worker policy and `src/events.rs` are unchanged. Added seven real file→monitor→worker→controller recovery controls, one multiple-monitor/source test, and allocator/cache-pruning unit tests. Existing regressions remain; only accidental literal Copilot counter-start assertions now check positive/distinct/stable provenance. M1 publisher/Node paths are byte-identical.
- **Historical R4 exact evidence / then-pending gates (superseded):** External `M3.R4/` contains candidate source/compiled-artifact digests, commands/logs/exits and retained test binaries. Final regression compiles against exact `e193b6c49d66500083977d98eb0e6d71b1bb56e9` and assertion-fails20/20 (exit101,0/1 passed); fixed passes20/20. Full31-test controller suite passes10/10. Use SEPARATE `rejected-target`/`fixed-target`: one shared-target recheck incorrectly reused the fixed rlib; that invalid setup result is preserved/labeled, excluded from proof. Initial and final post-repeat complete exact gates exit0:49 targeted (31+18),132 workspace (67+31+18+16), no ignored tests. Final exact gate is recorded in `exact-gate-final.{command,log,exit}`; parent must inspect it and `REPORT.md`, then freeze a candidate and run fresh rugged/spec/quality/holdout roles. M3 is NOT accepted/done; M4/M5 untouched, M6 still field-blocked. No staging/commit/dispatch or target-host proof.
- **Resolved R4.1 correction / prior dissent:** R4 exact49 targeted/132 workspace and fresh ruggedRUGGED/specPASS bind `edec9fabb3dc6e6393b34fe9b4f83c29b7b14bd9`, BUT latest external qualityCHANGES-REQUESTED traced incomplete session-lock scans discarding known parser provenance. This author first added three synthetic file→real monitor→same owned worker→controller controls: first submission actually fails, all three submissions exhaust, and acknowledged unchanged error stays quiet. Real lock/log bytes and file identity stay unchanged. Exact edec9fa compiles and assertion-fails all three before production edits;20/20 rejected repeats reproduce lost retries/Refresh and historical reclassification. This confirms the formerly source-traced finding, not a new scope/design.
- **M3.R4.1 implementation / bounded recovery:** `src/monitor.rs` now prunes parsers by typed `SourceMembership` only after complete absence, before admitting candidates. Incomplete scans retain known provenance; positive lock replacement still rebuilds quietly. At64 retained parsers/source, refused new candidates get `Unavailable` plus existing `session_overflow`; no eviction, unbounded growth or silent health. A new real-files unit control covers ten churn generations, blocked admission, unchanged recovery/replacement at capacity, and complete recovery reclaiming absent entries/admitting new ones. Checked process-lifetime IDs, root-loss retention, `src/events.rs`, controller/worker policy and M1 publisher/Node paths are unchanged. Only monitor, controller tests, architecture and these notes are authored.
- **Historical R4.1 author evidence / then-pending gates:** External `/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103/M3.R4.1/` holds exact commands/logs/exits, candidate/source hashes, isolated rejected-source copy and SEPARATE rejected-target/fixed-target artifacts. New regressions pass20/20 fixed repeats; full34-test controller suite passes10/10; cache-cap/reclaim control passes. Post-repeat COMPLETE exact isolated-HOME gate exits0 (`exact-gate-final.{command,log,exit}`):52 targeted (34+18),136 workspace (68+34+18+16),0 ignored. Source and separately compiled/retained rlib/test-binary digests plus accepted-M1 byte checks are in `final-provenance.json`; exact resume is `REPORT.md`. M3 remains in_progress; parent freezes candidate and runs fresh independent rugged/spec/quality roles plus holdout judging. M1/M2 stay accepted, M4/M5 pending, M6 real Windows field-blocked. No staging, commits, reviewer dispatch, user runtime/config access, publication or Windows/WSL/OCI/service proof.
- **proven R4 baseline defect / design — historical trigger:** A fresh stronger read-only diagnostic has PROVEN the remaining e193b6c49d66500083977d98eb0e6d71b1bb56e9 defect through compiled real library/owned workers/controller: acknowledged error1, same lock/instance, unavailable root clears parser cache, log truncates, observed empty recovery baseline, NEW error visibly notify=true reuses1 and is suppressed, even after Refresh.0/1 assertion-failed,20/20 repeats; three controls pass.23 selected source/manifest/test/asset files matched candidate before/after; no repository writes. Evidence `M3.R4-diagnostic/{REPORT.md,FIX-DESIGN.md,results.txt}` and exact logs/commands. This supersedes the earlier diagnostician's permission-blocked attempt, not its retained history.
- **authorized narrow repair:** Allocate checked, non-recycling process-lifetime Copilot error IDs outside volatile parser/session/monitor state. Preserve bounded known parser/provenance through temporary source loss while returning explicitly unavailable snapshots; unchanged known errors keep identity, pending/exhausted budgets and quiet acknowledgement. Retain true removal/lock replacement cleanup, quiet historical reset and new-event eligibility. Never fix collision by forgetting controller acknowledgement, always rearming Refresh, inventing lock IDs, using clock/filelength/truncated hashes, or adding a DB. Theoretical allocator exhaustion must degrade visibly, never wrap. Pi episode allocation/restoration unchanged.
- **required signal:** Add actual monitor→workers→controller regression `copilot_new_error_after_source_cache_loss`; compile/assertion-fail on e193b6c, pass fixed. Keep unchanged-acknowledged, historical replacement and normal truncation controls; add pending/exhausted/new-budget, obsolete-ack, reconstruction/multiple-monitor controls. Process-wide IDs need not start1 per test/session; replace only accidental literal-start assertions with stronger positive/distinct/stable provenance assertions, never weaken identity/delivery protection. Exact M3 full gate remains mandatory plus repeats; no raw provider body/new sensitive fields.
- **prior evidence / dissent:** WIP79e9220 exact41 targeted/122 workspace and ruggedRUGGED/qualityPASS, BUT specCHANGES-REQUESTED; not accepted. Earlier R1 fixed expiry, R2 time/placeholder identity, R3 capacity-membership loss with assertion-fail counterexamples and repeats. All code/history preserved under M3.R0–R3 and commit79e9220. Replan is based on actual new compiled proof and stronger model, not a fourth blind patch loop. Parent must rerun all candidate-bound reviews after correction.

### M4 — Native Windows notifications and usable session identity
- **objective:** Connect tested workers/controller to existing Win32 tray/balloons/menu.
- **files/areas:** `Cargo.toml` (existing windows-sys feature flags only), `src/{tray,app,actions,icon,attention,attention_worker,monitor}.rs`, `tests/{attention_controller,attention_monitor}.rs`, `integrations/pi/{tray-goblin.js,README.md}`, `tests/pi-{status,host}.test.mjs`, `scripts/test-pi-host.mjs`, `docs/operator/{status-reference,first-run}.md`.
- **validation:** `node --test tests/pi-status.test.mjs tests/pi-host.test.mjs && cargo test --workspace && cargo build --release --target x86_64-pc-windows-gnu --bin tray-goblin`
- **acceptance:** R8 implementation tier. UI drains workers without filesystem I/O; bounded content-free NIF_INFO, attention-first persistent list/counts/tooltip; explicit capabilities/diagnostics and success-only submission. Explorer re-add doesn't reset acknowledgements; preserve actions. Optional bounded TRAY_GOBLIN_LABEL through publisher/parser/display gives human per-terminal identity, no prompt-derived naming. Test unlabeled compatibility/labels and document Windows/WSL launch examples. No M4 while M3 unaccepted; compilation/policy is NOT target delivery proof.
- **status:** done
- **acceptance record (2026-11-04 UTC):** Exact isolated-HOME gate exit0:36 Node (26 core/10 actual offline pi1.4 host),141 Rust (68 library+38 controller+19 aggregate+16 legacy),0 skipped/ignored; Windows release cross-build and PE32+ x86-64 GUI/import checks PASS. Immutable candidate `5894cd119b80fc47bace1f36002574cc38248828`: fresh ruggedRUGGED, non-author specPASS+qualityPASS. Parent verified actual wiring, labels, runner counts and candidate/worktree source equality; reviewers verified22/22 source and7/7 artifact hashes. Fresh CopilotCLI review contexts/lenses (model family unqueried), stronger pi/gpt-6.1-sol xhigh author; no cross-family/tooling claim.
- **implementation:** `AttentionRuntime::cycle` owns fixed workers/controller across Explorer; UI only requests/drains/stops, never source I/O/join. Epoch clocks remain separate from uptime, including actual adapter-completion timestamp. Native NIF_INFO, UTF16-safe bounded text, attention-first persistent menus/counts/capabilities/diagnostics; optional64-byte explicit manual labels through actual pi host→parser→display. Unmapped POSIX actions fail safely; Copilot logs never select pi; native singleton retained.
- **evidence/reservations:** External `M4.{validation,rugged,spec,quality}.log`, `M4.author/REPORT.md` and `final-provenance.json`; current verdict `.wgm/rugged/M4-current.md`. Four driver controls20/20 and full38 suite5/5; three separately compiled dispatch/clock/ack mutants assertion-fail, privacy mutation hits both strengthened getter controls. Early setup/new-test failures and wonder/reflect remain preserved externally. Spec reservations:none within M4 tier. Quality reservation: actual Windows/WSL delivery/suppression/recovery/resources remain unverified M6, not waived. Exact executable SHA256 `f8acac734ebf670cec55017a952664ff0dcb12a362659ea2a17c1503a56f3602`. No Windows runtime or release-readiness claim. Fresh validator follows acceptance; M5 still owes continuous two-real-host→Rust demo.

### M5 — Package, document and run offline attention demo
- **objective:** Installable artifact and actual shipped publisher→Rust end-to-end demo with complete setup.
- **files/areas:** `scripts/{demo-attention.sh,test-attention-windows.ps1,package-release.sh,check-docs.sh,measure-performance.ps1,test-installer.ps1}`, `install.ps1`, `uninstall.ps1`, `integrations/pi/`, `assets/` for packaged operator resources, `README.md`, `docs/`, `.wgm/docs/audit/`.
- **validation:** `bash scripts/demo-attention.sh --container podman && bash scripts/demo.sh 0.1.0 && bash scripts/check-docs.sh`
- **acceptance:** R9–R10. Exact package allow-list includes persistent pi integration/resources; never silently edits pi settings. Real shipped extension→Rust demo verifies identity/privacy/quiet completion/dedupe; Windows cross-build/package and preserved MVP demo/checksum. OCI CLI-only: nonroot,network-none,read-only executable+fixtures, existing Debian bookworm-slim, assert real JSON. Clear Windows/WSL config, labels, install/remove, limitations, UNC wake, unavailable/clock/tracking faults and notification suppression. Windows field script rejects Linux and collects real observations; performance Path binds PID/path/hash and rejects mismatch. Resolve checkpoint audit Agent findings (including annotated tag must peel ^{commit}, no published-status falsehood, correct public CLI examples) and run final independent five-role docs audit/structural check. Preserve dissent; no structural COMPLETE relabeled GREEN. Add any tightly coupled helper filename to explicit area/allowlist before authoring/freezing.
- **status:** pending

### M6 — Real Windows/WSL field and resource evidence
- **objective:** Verify actual delivery/recovery and50MB/5% budget before release readiness.
- **files/areas:** `scripts/test-attention-windows.ps1`, `scripts/measure-performance.ps1`, `.wgm/rugged/`, this plan.
- **validation:** `pwsh -NoProfile -File scripts/test-attention-windows.ps1 -ExecutablePath target/x86_64-pc-windows-gnu/release/tray-goblin.exe -EvidencePath .wgm/windows-attention.json && pwsh -NoProfile -File scripts/measure-performance.ps1 -Path target/x86_64-pc-windows-gnu/release/tray-goblin.exe -DurationSeconds 30 -JsonPath .wgm/windows-performance.json`
- **acceptance:** R11–R12. Real native+WSL pi dialogs/errors/Copilot errors; quiet completion3P; attention/clear deadlines; dedupe,Focus Assist/menu,pi reload/crash,Explorer no replay,WSL shutdown/native isolation/actual UNC wake and recovery,source removal,clean quit. Record UTC/buildSHA/executableSHA256/PID/path/Windows+WSL versions/source kinds/load and genuine observations. Performance report matches candidate digest. No Linux/mock substitute.
- **status:** blocked
- **reason:** No actual Windows/WSL host available here. Operator action after M5; no promotion/publication until genuine evidence.

## Artifacts and gates
- Goal/glossary/spec/constitution consistent; R1–R3→M1,R4–R5→M2,R6–R7→M3,R8→M4,R9–R10→M5,R11–R12→M6. Validator-only tier1→2→3 scenarios in `.wgm/scenarios/`, convergence95. Intermediate scores are not completion or root MVP regressions.
- Full baseline audit `.wgm/docs/audit/2026-11-03T2148Z_attention-baseline.md` completed; checkpoint `.wgm/docs/audit/2026-11-04T0114Z_attention-wip-checkpoint.md` is RED, with coupled corrections/deferred Agent actions recorded in its index and `.wgm/deferred-work.md`. M5 owes final audit and remaining release-doc corrections; never waive RED as structural pass.
- Current M1/M2/M3/M4 implementation verdicts are RUGGED with both correctness passes; prior e193 FRAGILE/static-proof history is superseded by compiled regressions and accepted e1221e2. M3 regenerated plan had RUGGED/readiness98 before stronger author work. No implementation gate was waived.
- Latest accepted gates: M1 Node33, M2 Rust93, M3 Rust136/targeted52, M4 Node36/Rust141/Windows cross-build; last docs381. Progress4 accepted, not complete. Last tier1=93.37; M4 validator next. M5 owns continuous-demo evidence, tiers2/3 stay gated until95.
- M4 author/check/review children exited; parent accepts only the pinned runtime. Next run fresh M4 validator, record scores, then authorize one fresh M5 author with explicit helper areas staged. Do not edit an active helper. M6 stays blocked; no actual Windows/release claim.
