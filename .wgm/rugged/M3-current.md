# Parent disposition: M3 remains BLOCKED

The candidate below has one current rugged verdict, RUGGED, but the independent
spec reviewer requested changes for a source-recovery episode-ID collision.
Quality PASS does not override that dissent. Do not follow the review's proposed
M4 next action until M3 is independently accepted. The original reviewer stdout
is retained externally in `M3.rugged.log` and `M3.R3/parent-rugged.log`.

The M3 diff separates delivery history from visible rows, preserving dedupe through saturation and incomplete scans. I’m checking the gate’s candidate binding and rerunning only in-memory policy/concurrency probes; Windows adapter and field evidence remain outside M3.


## M3 review — `e193b6c49d66500083977d98eb0e6d71b1bb56e9`

**No blocker found within M3’s controller/source-isolation acceptance.** Reviewed the immutable implementation diff against `.wgm/IMPLEMENTATION_PLAN.md:86–90` and `.wgm/specs/session-attention.md`, plus the exact supplied implementation and validation logs.

### Actual environment and decomposition

Actual operator: one person multiboxing pi/Copilot sessions across native Windows and local WSL, wanting low-noise alerts without watching every terminal. Training/support capacity is unspecified.

Review environment: **Linux x86-64, kernel `6.8.0-147-generic`**. Candidate source/tests/manifests match the working tree; review used immutable candidate content.

- **Intrinsic constraint:** Uninterruptible filesystem reads cannot be cancelled safely in-process. Fixed workers isolate them; process exit remains the cancellation boundary. Copilot input detection remains explicitly unavailable.
- **User capacity:** Explicit source paths and restart-to-reconfigure avoid discovery, replacement-worker management and additional services.
- **Operational stress — dominant risk:** Simultaneous attention, blocked sources, stale publishers and incomplete scans could otherwise cause freezes, false input visibility or notification replay.

### Simplification and accepted trade-offs

Keep one worker/source, one queued poll and one latest-result slot—not replacement threads or accumulated work (`src/attention_worker.rs:79–182`). Stop/drop never joins blocked I/O; the blocked thread retains its ownership record and handle until exit (`62–68,235–251`).

Keep authoritative membership separate from the 64-row display projection. Incomplete scans preserve delivery history rather than treating omission as deletion (`src/attention.rs:290–345`). History is capped at **4096 identities/source, four reason slots/identity**. At capacity, new attention remains visible with `delivery_history_overflow`, but notifications are withheld. This explicitly trades notification coverage under prolonged incomplete-scan churn for bounded retention and no replay.

No new dependency, daemon, database or deployment process is introduced. Packaging/setup remains M5.

### Exact deterministic evidence and certainty

Evidence directory: `/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103/`.

`M3.candidate` identifies the requested commit; `M3.validation.command` records:
```sh
cargo test --test attention_monitor --test attention_controller && cargo test --workspace
```
`M3.validation.exit` is **0**. Validation log timestamp: **2026-11-04 01:04:39 UTC**.

| Claim | Observed evidence | Certainty |
|---|---|---|
| Exact acceptance gate | `M3.validation.log`: 41 targeted and 122 workspace tests passed; zero failures/ignored | Rung 4 |
| Concurrency/isolation | Ten stall/recovery cycles; 100 UI request/drains under 250 ms; independent source progresses under 1 s; exactly two actual worker thread IDs | Rung 4; `tests/attention_controller.rs:362–447` |
| Cleanup/ownership | Stop and blocked-owner drop under 250 ms; one queued command; retained handle; late publication rejected; eventual exit after release | Rung 4; worker tests `:328–379` |
| Boundedness | Latest slot survives 100 generations; contended drain does not wait; 80 incomplete generations cap history and recover visibly | Rung 4; worker `:385–417`, attention `:502–542` |
| Delivery policy | Coalescing, same-directory distinct identities, success-only acknowledgement, three attempts ≥5 s apart, Refresh spacing, obsolete callbacks and restored-error dedupe | Rung 4; controller tests `:99–326` |
| Saturation recovery | Acknowledgement/exhaustion survive malformed/stale row omission and actual 4096-entry scan cutoff; complete removal still prunes | Rung 4; controller tests `:524–751` |
| Freshness/recovery | Cached publisher expiry during blocked reads; post-read observation clock; timestamped Copilot error not consumed incorrectly; 32 pi + 32 Copilot sessions exercise actual workers/controller | Rung 4; controller tests `:894–1298` |
| Privacy | New transport/history carries typed identity/status only; fixed diagnostics; existing sensitive-field exclusion passes; legacy probe regression passes | Rung 4 within existing privacy contract |

I additionally ran **nine write-free tests from the existing gate binaries**, all passing: privacy exclusion, both history-bound tests, all three worker tests, delayed-source isolation, retry exhaustion/Refresh and mixed-retry pruning. Filesystem-writing tests were inspected through the recorded gate, not rerun.

`M3.implement.log` reports repeat/counterfactual runs; those author summaries are not counted as independently verified executions here.

### Evidence boundaries

M3 reaches **rung 4 for its claimed core validation signal**, not rung 5 target-host operation. Synthetic files do not establish real pi-host execution. Neither source inspection nor Linux concurrency tests establish Windows compilation, notification delivery, UNC wake/shutdown behavior or Windows resource usage.

M4 owns native adapter wiring/compilation; M5 owns packaging/demo. **M6 remains a release blocker** for actual Windows+WSL field and performance evidence.

No scenario directories or user runtime data were inspected. No project, documentation, configuration or Git changes were made.

**Verdict: RUGGED**

**Highest-leverage next action:** Complete M4’s native adapter wiring and exact Windows cross-build gate, preserving the tested nonblocking worker boundary and explicit submission acknowledgement.
