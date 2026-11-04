I’m reviewing only M2 at the frozen commit, separating core evidence from real pi-host, Windows-build, and live Windows/WSL evidence.


The frozen candidate fixes the stale-file saturation failure: live sessions are admitted before unavailable placeholders, and the recorded regression fails on the rejected candidate and passes here. I’m verifying the evidence hashes and rerunning only pure parser/read-cap tests that cannot write project files.


## M2 implementation review — `b324533d5de5ea68380fc0dcd1a0d1d165577558`

**M2’s previous stale-publication saturation finding is resolved. No remaining blocker was found within its claimed core scope.** Reviewed the immutable implementation diff against `.wgm/IMPLEMENTATION_PLAN.md` and `.wgm/specs/session-attention.md`, including the supplied validation/implementation logs and prior M2 review.

### Actual environment and decomposition

Actual operator: one person multiboxing pi/Copilot sessions across native Windows and local WSL, without continuously watching terminals. Training/support capacity is unspecified. Review execution: **Linux x86-64, kernel `6.8.0-147-generic`**, synthetic inputs only.

- **Intrinsic design:** Independent source/session state and Copilot offsets; explicit provider capabilities; typed pi schema and fixed diagnostics. Passive Copilot input detection remains explicitly unavailable.
- **User capacity:** Explicit paths and opt-in pi installation, not discovery or a service. Defaults, replacement/empty-source semantics, legacy aliases and UNC wake behavior are documented.
- **Operational stress — dominant risk:** Crash residue, malformed publications and simultaneous attention. Freshness now precedes pi admission (`src/monitor.rs:400–415`); unavailable placeholders occupy only spare capacity (`484–499`), so residue cannot displace live attention within the bounded scan.

### Simplification and accepted trade-offs

Keep local files, explicit roots, bounded snapshots and aggregate residue diagnostics—no cleanup daemon, database or consumer deletion of publisher files. At capacity, per-file unavailable detail yields to live attention and a bounded summary.

The repair increases worst-case pi reading to **4096 × 16,385 bytes per source/poll**; memory remains bounded. Overflow is visible, but coverage beyond 4096 entries is not promised. This is a bounded-work contract, **not** a Windows latency/resource measurement.

Read handles are scope-owned; disappearance clears retained source state, removed sessions are pruned, and failures expose unavailable states rather than fabricated healthy/input states. Worker cancellation/ownership is M3, not an unfulfilled M2 claim.

### Deterministic evidence and certainty

Evidence directory: `/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103/`.

| Claim | Exact observed evidence | Certainty |
|---|---|---|
| M2 backpressure | `M2.validation.log`: `cargo test --workspace`, 60 library + 17 aggregate + 16 legacy tests passed; zero failures/ignored | Rung 4: recorded execution |
| Evidence correspondence | All nine `M2.R1/gate-before.sha256` entries match immutable candidate blobs; implementation worktree matches candidate | Digest/source verification |
| Saturation recovery | Identical regression fails on rejected `33ad6ece` with “stale residue must not displace live attention,” exit 101; current gate passes `pi_stale_saturation_does_not_hide_fresh_attention` | Rung 4 |
| Bounds under residue | Tests cover 64 live + 64 stale, exactly 4096 examined from 4097 files, and three alternating generations retaining at most 64 timestamps | Rung 4 |
| Identity/concurrency | Mixed 64-session load, 64/source capacity, visible overflow, same-directory/source separation and independent offsets | Rung 4; synthetic filesystem load |
| Parsing/recovery | Exact 16-KiB/10-second boundaries; malformed/partial/future/regressed status; older restored error IDs; replacement/removal; partial Copilot appends and oversized history | Rung 4 |
| Privacy/compatibility | Sensitive-field exclusion, historical-error notification suppression, no failed-tool-only alerts, original six-field probe and explicit aggregate output | Rung 4 |

I additionally reran three **write-free existing gate-binary tests**, all passing: `events::tests::does_not_model_sensitive_fields`, `monitor::attention_io_tests::pi_reader_consumes_only_one_byte_beyond_the_limit`, and `pi_parser_matches_m1_schema_and_rejects_inconsistent_scalars`. The full filesystem-writing gate was inspected, not rerun.

### Evidence boundaries

M2 reaches **rung 4 for its core acceptance**. Schema compatibility is not new real-pi-host execution. Windows directory/reparse branches have source evidence here, not execution evidence. Synchronous polling is expressly core-only: M3 owns blocked-source isolation, M4 Windows compilation/adapter wiring, and M5 packaging/end-to-end deployment checks.

**M6 remains a release blocker** for actual Windows/WSL interaction, UNC shutdown/wake behavior and resource measurements; no rung-5 target-host result is claimed.

No scenario directories or user runtime data were inspected. No project files, configuration or Git state were changed.

**Verdict: RUGGED**

**Highest-leverage next action:** Implement M3’s fixed-owned source workers and prove with controlled blocked-reader tests that stalled WSL I/O cannot stop native monitoring or block UI drain/stop.
