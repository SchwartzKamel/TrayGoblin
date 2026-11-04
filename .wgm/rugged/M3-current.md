## M3 implementation review — 2026-11-04 UTC

**Scope:** Immutable candidate `e1221e2b77d8ea9bc453151505c7b0288f09d770`, against M3 acceptance and `.wgm/specs/session-attention.md`. Prior provenance-loss findings were revisited. No project changes, scenario inspection or user-session access.

**Actual environment:** Review and deterministic execution on Linux. Intended operator multiboxes pi/Copilot across native Windows and local WSL; ordinary operation must not require watching terminals or diagnosing lost acknowledgements. Operator training/support capacity is unspecified.

### Decomposition
- **Intrinsic constraint:** Controller delivery history outlives parser caches. Checked process-wide Copilot IDs prevent reuse; typed membership now governs parser removal rather than selected display rows (`src/monitor.rs:487–527,1047–1057`).
- **User capacity:** Explicit source configuration and restart remain necessary. No daemon, database, discovery service or additional deployment dependency was introduced.
- **Operational stress — dominant risk:** Overlapping outages, incomplete scans, dialogs and failed submissions can otherwise suppress new errors or replay old ones. Candidate preserves acknowledged/exhausted provenance while hiding unavailable attention.

### Simplification and trade-offs
Retain bounded provenance through uncertainty; reclaim it only on confirmed absence or replacement. At parser/history capacity, emit explicit overflow rather than evict remembered identities. This deliberately sacrifices new notification admission during prolonged uncertainty instead of silently replaying alerts.

Fixed workers and single-slot mailboxes avoid replacement-thread accumulation. Stop never joins blocked filesystem reads; worker ownership survives UI-owner drop until exit. The accepted trade-off is that permanently blocked OS reads require process termination (`src/attention_worker.rs:62–68,123–163,185–250`).

### Deterministic evidence and certainty
Evidence directory: `/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103/`.

| Claim | Exact evidence | Certainty |
|---|---|---|
| Candidate-bound validation | All **46** recorded fixed-source hashes match the immutable candidate; retained controller/library-test executable hashes match provenance. | Rung 4 |
| Exact M3 gate | `M3.validation.log`: `cargo test --test attention_monitor --test attention_controller && cargo test --workspace`; **52 targeted, 136 workspace passed**, none ignored. Separate retained final-gate exit is **0**, with matching counts. | Rung 4 |
| Recovery regression actually discriminates | Raw rejected log: three compiled partial-scan regressions assertion-fail. Rejected repeats **20/20 exit101**; fixed repeats **20/20 exit0**; complete controller repeats **10/10 exit0**. `M3.implement.log` agrees. | Rung 4 |
| Isolation and cleanup | Ten controlled stalls; unchanged two-worker/thread ownership, other-source progress **<1s**, drain/stop **<250ms**, late publication rejected (`tests/attention_controller.rs:914–999`). Retained-binary ownership/mailbox/generation probes rerun successfully. | Rung 4 |
| Mixed load and timing | **32 pi + 32 Copilot** synthetic sessions through real parsers, workers and controller; coalescing, restoration, clearing and exact stale boundary asserted with injected clocks (`tests/attention_controller.rs:1696–1850`). Blocked-read publisher expiry tested separately. | Rung 4 |
| Boundedness | 64-parser retention/admission/reclamation control; 4096 retained identities/source, at most four reason slots; visible overflow and complete-scan recovery. History/allocator probes rerun successfully. | Rung 4 |
| Privacy | Explicit scalar/metadata models; fixed diagnostics; unchanged `src/events.rs` and accepted M1 integration bytes. Sensitive-field contract rerun successfully. Raw log bytes are temporarily buffered—not modeled, logged or displayed as conversation content. | Rung 4 |

Seven file-free retained-binary probes passed during this review; file-writing suites were inspected, not rerun.

**Tier boundary:** These results establish M3 source/core behavior, not actual pi-hook execution, Windows compilation or live Windows/WSL delivery. None substitutes for another. M4 native integration and M5 packaging are future tasks, not M3 defects. **M6 remains a release blocker**, including real UNC wake/recovery, notification suppression, Explorer recovery and resource measurements.

**Verdict: RUGGED**

**Highest-leverage next action:** Obtain fresh independent spec/quality acceptance against this exact immutable candidate before advancing to M4.
