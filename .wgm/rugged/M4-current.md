## M4 implementation review — 2026-11-04 UTC

**Scope:** Only commit `5894cd119b80fc47bace1f36002574cc38248828` against its parent, M4 acceptance (`.wgm/IMPLEMENTATION_PLAN.md:84–95`) and `.wgm/specs/session-attention.md`. No project changes, scenario inspection or user-session access.

### Actual environment
Review ran on Linux x86-64, kernel 6.8.0-147, Node 25.2.1, Rust 1.95.0. The actual operator multiboxes pi/Copilot across native Windows and local WSL and cannot continuously watch terminals. Training/support capacity is unspecified. No live Windows host was available.

### Decomposition
- **Intrinsic constraint:** Passive Copilot cannot detect input dialogs; the UI explicitly reports that limitation. Successful Win32 submission cannot establish notification display.
- **User capacity:** Explicit pi opt-in, per-terminal labels and configured WSL roots replace discovery or automatic path conversion. Native/WSL examples and fixed recovery diagnostics make those obligations legible.
- **Operational stress — dominant risk:** Concurrent attention, stalled UNC reads, failed submissions and Explorer recovery must preserve independent monitoring and delivery history. The production integration retains the accepted fixed-worker/controller architecture rather than introducing another scheduler or service.

### Simplification and trade-offs
Keep local status files, one native tray and existing Win32 balloons; no daemon, database, network collector or resident Node sidecar. Manual configuration and unmapped-POSIX action refusal trade convenience for predictable deployment.

Keep workers/controller independent of icon registration (`src/tray.rs:304–405`). Stop requests never join blocked readers; ownership survives until worker exit/process termination (`src/attention_worker.rs:62–68,235–251`). This accepts process exit as cancellation for uninterruptible filesystem reads. Publisher deletion failure remains explicit and leaves an ownership-marked stale file rather than deleting another instance.

### Deterministic evidence and certainty
Evidence root: `/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103/`.

| Claim | Exact evidence inspected | Certainty |
|---|---|---|
| Candidate binding | **22/22** gate source hashes match immutable candidate bytes; **7/7** retained executable hashes match provenance. | Rung 4 |
| Mandatory M4 gate | `M4.validation.log`: **36 Node + 141 Rust passed**, none skipped/ignored; Windows release cross-build completed. Separately, `M4.author/exact-gate-final2.{command,log,exit}` records the exact command below, exit **0**, matching totals. | Rung 4 |
| Real production dispatch policy | `tests/attention_controller.rs:578–1017` exercises shipped publisher → real workers/parser/controller → production submit callback. Tests enforce completion-based **4,999/5,000 ms** retry boundaries, failure retention, success-only acknowledgement, exhaustion and Refresh. Three compiled mutants assertion-fail for omitted dispatch, premature completion and false acknowledgement. | Rung 4 |
| Concurrency, recovery and cleanup | Production-driver blocked-source test asserts drain/Refresh/stop/drop **<250 ms** and independent parser progress **<1 s**. Four driver controls pass **20/20** repetitions; full 38-test controller suite passes **5/5**. Three file-free worker ownership/mailbox/staleness controls reran successfully. | Rung 4 |
| Multibox load and bounds | Gate includes **32 pi + 32 Copilot** real-parser/worker/controller synthetic load, coalescing, clearing and recovery. Existing limits remain 16 sources, 64 live sessions/source, bounded mailboxes/history and visible overflow—not measured Windows capacity. | Rung 4 |
| Explicit privacy and usable identity | Optional manual label traverses actual offline **pi 1.4.0** host, typed parser and display; absent/blank compatibility, 64-byte bounds and poisoned private-field getters are tested. File-free retained UI/action and sensitive-field controls reran successfully. | Rung 4 |
| Native wiring and compilation | `src/tray.rs:813–822` constructs bounded, terminated UTF-16 `NIF_INFO` and returns actual shell acceptance. PE evidence verifies x86-64 Windows GUI plus required imports. Explorer re-add leaves engine/history intact. | Rung 3 wiring trace; rung 4 compilation |

Exact mandatory command:
```sh
node --test tests/pi-status.test.mjs tests/pi-host.test.mjs && cargo test --workspace && cargo build --release --target x86_64-pc-windows-gnu --bin tray-goblin
```

`M4.implement.log` agrees with the inspected implementation and gate results; its additional claims were checked against retained evidence, not accepted as proof alone. Five narrow retained-binary controls passed during this review without project-file writes.

**Tier boundary:** Source/core policy, actual offline pi-host execution, Windows compilation and live Windows/WSL operation remain distinct. No rung-5 delivery, suppression, Explorer/UNC recovery, native cleanup or resource-performance evidence is claimed. M5 packaging/continuous-demo obligations are outside M4. **M6 remains a release blocker**, including genuine Windows+WSL observations and matching-digest 50 MB/5% measurements.

**Verdict: RUGGED**

**Highest-leverage next action:** Complete independent spec/quality acceptance against this exact immutable M4 candidate before advancing the plan.
