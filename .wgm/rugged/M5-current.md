I’m reviewing only the pinned metadata repair, checking that the prior runtime evidence still applies and that M6’s Windows release boundary is unchanged.


## M5 bounded companion review

**Scope:** Fresh non-author review of `b0b244f2df20b0d959d2a947511141ef96e4fb97` → `4e5e3c1f3106e4f958fc9af15afd9cdc707dd794`, against M5 acceptance, the attention specification and re-audit A06/A07. Prior R3 reports were inspected, not reassigned to the new candidate. No project changes, scenario inspection or user-session access.

**Actual environment:** Linux x86-64, kernel `6.8.0-147-generic`; Node `25.2.1`, Rust `1.95.0`, PowerShell `7.6.9`. Intended operator multiboxes pi/Copilot across native Windows and local WSL. Actual production concurrency, training and support capacity remain unspecified. No live Windows host was available.

### Decomposition and simplification

- **Intrinsic constraint:** Passive Copilot cannot detect input prompts; adapter submission cannot establish Windows notification display.
- **User capacity:** Explicit pi opt-in, manual labels and configured Windows/WSL paths remain necessary. A06 now removes competing review-status authorities.
- **Operational stress — dominant risk:** Concurrent attention, stalled UNC reads, stale publishers and interrupted installation require independent progress, bounded retention and recoverable ownership. This repair changes none of those mechanisms.

**Accepted trade-off:** Seven live summaries now delegate current candidate/gates/verdicts to the active plan, trading duplicated convenience for one maintained authority. Local files and one native tray remain simpler than adding services or discovery. Explicit configuration/restart remains an operator responsibility.

### Findings and deterministic evidence

Evidence root: `/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103/`.

| Check | Independently verified result |
|---|---|
| Explicit diff allowlist | Exactly **nine paths**: seven live docs/indexes, active plan and newly filed historical re-audit. All nine match pinned candidate bytes. |
| No wider implementation change | **41 runtime/support tree entries**, including source, installer, collectors, tests, controls, packaging, integration, assets and manifests, retain identical Git blobs/modes. Executable documentation fences are unchanged. |
| A06 | All seven summaries use active-plan authority rather than requesting already-completed R3 reviews. Historical versus current disposition remains explicit; no perpetual pending-review statement is required. |
| A07 | Checklist now specifies **Run start (UNIX ms)** and **Last run (ms)**, matching `src/tray.rs:469–476`; historical detail-tooltip distinction remains. |
| Immutable history | Three existing historical audit blobs are unchanged. Newly filed AMBER report equals its original writer capture. Preserved R3 candidate and review/gate exit records remain intact. |
| Current exact gate | `M5.validation.command/.exit/.log` inspected: **exit 0**, **141 Rust tests**, installer **56 passed / 0 failed / one Windows-only skip**, documentation **408 checks / zero skipped**. `M5.implement.log` inspected as author handoff, not independent certification. |
| Current real pi-host journey | `M5.parent-gate-R4/journey.json`, **2026-11-04 05:02:33–05:02:40 UTC**: two concurrent installed pi1.4 publishers, **14 phases / five adapter requests**. All **14 recorded digests** independently match. |
| Prior evidence continuity | All 14 digests also match for preserved R3 and status-fix journeys. Both peers report **zero sensitive-field, network and busy-metadata reads**. Connected assertions exercise identity, heartbeat, coalescing, restoration and quiet completion. |
| Exact documentation controls | Both current-parent reports and both status-fix reports retain **12 Bash + two PowerShell + four audit fences**, **44 injected failures**, successful controls and killed canaries; relevant document/fence/archive bindings match. |
| Artifact continuity | All **13 status-fix artifact/evidence hashes** match. Nine packaged source resources and packaged executable match pinned/runtime bytes. |

Exact inspected gate:
```bash
bash scripts/demo-attention.sh --container podman && bash scripts/demo.sh 0.1.0 && bash scripts/check-docs.sh
```

**Unchanged SHA-256:**
```text
ZIP 9485a19ea81fa03763a11dcf8802132389db9b215d72a31d80ccd0a3e3f5e1a5
PE  f8acac734ebf670cec55017a952664ff0dcb12a362659ea2a17c1503a56f3602
```

### Certainty and reservations

**Rung 4:** Fresh read-only integrity assertions bind this metadata repair to deterministic current gates and unchanged prior implementation evidence. Inherited synthetic 64-session mixed-load, bounded workers/mailboxes, retry/recovery and blocked-source ownership coverage remain applicable—not measured Windows capacity.

Connected source inspection confirms interrupted installation retains its repair marker (`install.ps1:262–312`); normal journey shutdown awaits publishers and asserts empty status storage, while failure finalization retains child references for targeted termination (`scripts/demo-attention-host.mjs:231–246`). Forced termination is not graceful-cleanup proof.

Source/core tests, offline real pi-host execution, restricted OCI CLI execution and Windows PE compilation remain distinct tiers. **No rung-5 Windows/WSL delivery, suppression, Explorer/UNC recovery, Startup, native cleanup or resource evidence is claimed.** The M6 block is byte-unchanged and remains a release blocker, including matching-executable **50 MB / 5%** measurements.

The package honestly records pre-freeze `aa617e98…`, dirty and unpublished; identity is established through verified bytes, not a candidate-stamped release claim. This scoped review establishes no blocking repair defect and does not itself accept M5 or replace correctness reviews/final audit.

**Verdict: RUGGED**

**Highest-leverage next action:** Complete parent-owned acceptance review for this pinned candidate, including correctness roles and the final five-role audit, recording disposition in the active plan.

