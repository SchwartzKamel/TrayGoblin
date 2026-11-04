# Session attention implementation plan

Full track · Ralph-full, fresh author/reviewer contexts · one task per iteration.
Root AGENTS.md and specs/CONSTITUTION.md govern; never edit existing AGENTS.md.
This follow-up governs `.wgm/specs/session-attention.md`; root plan is MVP history.

## Current state — 2026-11-04 UTC
**M1/M2 accepted; M3 unaccepted WIP; M4/M5 pending; M6 field-blocked.**
Accepted commits: M1 `0d79111`, M2 `058ef74`. Buildable WIP `79e9220` preserves M3
and its dissent, not acceptance. Full chronological recovery evidence remains in
that commit's plan and the external run directory; it is superseded as live state
by this compact plan. No runtime Windows notification feature is claimed yet.

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
- **status:** pending
- **current remediation M3.R4 — regenerated after bounded stall:** A fresh stronger read-only diagnostic has PROVEN the remaining e193b6c49d66500083977d98eb0e6d71b1bb56e9 defect through compiled real library/owned workers/controller: acknowledged error1, same lock/instance, unavailable root clears parser cache, log truncates, observed empty recovery baseline, NEW error visibly notify=true reuses1 and is suppressed, even after Refresh.0/1 assertion-failed,20/20 repeats; three controls pass.23 selected source/manifest/test/asset files matched candidate before/after; no repository writes. Evidence `M3.R4-diagnostic/{REPORT.md,FIX-DESIGN.md,results.txt}` and exact logs/commands. This supersedes the earlier diagnostician's permission-blocked attempt, not its retained history.
- **authorized narrow repair:** Allocate checked, non-recycling process-lifetime Copilot error IDs outside volatile parser/session/monitor state. Preserve bounded known parser/provenance through temporary source loss while returning explicitly unavailable snapshots; unchanged known errors keep identity, pending/exhausted budgets and quiet acknowledgement. Retain true removal/lock replacement cleanup, quiet historical reset and new-event eligibility. Never fix collision by forgetting controller acknowledgement, always rearming Refresh, inventing lock IDs, using clock/filelength/truncated hashes, or adding a DB. Theoretical allocator exhaustion must degrade visibly, never wrap. Pi episode allocation/restoration unchanged.
- **required signal:** Add actual monitor→workers→controller regression `copilot_new_error_after_source_cache_loss`; compile/assertion-fail on e193b6c, pass fixed. Keep unchanged-acknowledged, historical replacement and normal truncation controls; add pending/exhausted/new-budget, obsolete-ack, reconstruction/multiple-monitor controls. Process-wide IDs need not start1 per test/session; replace only accidental literal-start assertions with stronger positive/distinct/stable provenance assertions, never weaken identity/delivery protection. Exact M3 full gate remains mandatory plus repeats; no raw provider body/new sensitive fields.
- **prior evidence / dissent:** WIP79e9220 exact41 targeted/122 workspace and ruggedRUGGED/qualityPASS, BUT specCHANGES-REQUESTED; not accepted. Earlier R1 fixed expiry, R2 time/placeholder identity, R3 capacity-membership loss with assertion-fail counterexamples and repeats. All code/history preserved under M3.R0–R3 and commit79e9220. Replan is based on actual new compiled proof and stronger model, not a fourth blind patch loop. Parent must rerun all candidate-bound reviews after correction.

### M4 — Native Windows notifications and usable session identity
- **objective:** Connect tested workers/controller to existing Win32 tray/balloons/menu.
- **files/areas:** `src/{tray,app,actions,icon,attention,attention_worker,monitor}.rs`, `tests/{attention_controller,attention_monitor}.rs`, `integrations/pi/{tray-goblin.js,README.md}`, `tests/pi-{status,host}.test.mjs`, `scripts/test-pi-host.mjs`, `docs/operator/{status-reference,first-run}.md`.
- **validation:** `node --test tests/pi-status.test.mjs tests/pi-host.test.mjs && cargo test --workspace && cargo build --release --target x86_64-pc-windows-gnu --bin tray-goblin`
- **acceptance:** R8 implementation tier. UI drains workers without filesystem I/O; bounded content-free NIF_INFO, attention-first persistent list/counts/tooltip; explicit capabilities/diagnostics and success-only submission. Explorer re-add doesn't reset acknowledgements; preserve actions. Optional bounded TRAY_GOBLIN_LABEL through publisher/parser/display gives human per-terminal identity, no prompt-derived naming. Test unlabeled compatibility/labels and document Windows/WSL launch examples. No M4 while M3 unaccepted; compilation/policy is NOT target delivery proof.
- **status:** pending

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
- M1/M2 current implementation ruggedRUGGED; M3 current implementation remains blocked by actual counterexample regardless of prior narrower RUGGED. M3.R4 regenerated producer-lifetime repair/check design is **RUGGED**,2026-11-04, fresh companion plan review `.wgm/rugged/plan-current.md` / external `rugged-plan-M3-provenance.log`. Current e193 implementation verdict is corrected to FRAGILE by actual compiled proof; no acceptance implied. Preflight readiness98 and clean committed metadata baseline precede the authorized stronger writer.
- Last committed gates: M1 Node33, M2 Rust93, WIP M3 Rust122/targeted41, docs381. All passed, but do not override the remaining independent correctness blocker. Task progress is2 accepted, not zero and not complete.
- All previous coordinator/author/reviewer processes exited. Fresh read-only pi diagnostic exited0 without repository writes. Next authorize one stronger fresh pi M3.R4 writer after revised plan gate; no duplicate writers. Author leaves in_progress; parent freezes/reviews/judges/commits, then M4→M5. Keep M6 blocked and no release claim.
