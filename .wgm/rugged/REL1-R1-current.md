## Fresh independent RUGGED review — RUGGED

**Candidate:** `67b997b2023d5ca9b259785184db17bef0d063c6`  
**Tree:** `5d9c0891d4849d0d75a538fc96bb04f4db657b44`  
**Reviewed:** 2026-11-08 UTC. I authored neither candidate.

**Evidence root (`E`):** `/home/lafiamafia/release-coordinator-r1/REVIEW-RUGGED-evidence`

### 1. Triage and actual context

Full review of the specified release workflow, publisher/verifiers/tests, active release spec/docs and REL1/R1/REL2 plan entries. Protected AGENTS/constitution and unchanged CI, byte verifier and collectors were examined for connected contracts.

Reviewed frozen `git show` exports—not the index or checkout. Checkout HEAD was `792de03511eb6d58d172f1d70e88306be58469ec`. All 12 scoped product hashes matched parent evidence before and after testing; nine protected files had zero candidate-to-baseline diff.

Actual operators are the Linux release maintainer and Windows 10/11 x64 + WSL observers, using authenticated GitHub and potentially interrupted uploads. Staffing/training guarantees are unstated; none assumed.

### 2. Bottleneck decomposition

- **Intrinsic:** JSON admission cannot establish human truth, reviewer independence or actual Windows behavior by itself.
- **User capacity:** Complete manual-source results, both-target observations and independent reviews require substantial operator work.
- **Operational stress:** Lost upload acknowledgments, partial drafts, stale evidence, misleading history and interrupted subprocesses threaten reliable handoff.

Dominant risk remains substituting plausible paperwork or CI success for actual acceptance. The candidate now makes prerequisite omissions fail visibly rather than assuming a careful owner supplied them.

### 3. Simplification and trade-offs

The retained design is justified: one manual workflow, exact existing bytes, draft-first operation, one bounded acceptance schema and non-destructive matching retries.

Accepted costs: stricter evidence preparation, manual/CI byte mismatch rejection, and refusal to silently promote a mode=draft ownership record through public-mode retry. These favor safety over convenience. Failed local children/server drafts remain owned and inspectable; unknown or published assets are not automatically repaired or replaced.

### 4. Deterministic field/stress evidence

Tests used isolated synthetic HOME/USERPROFILE/XDG, offline public compiler-cache settings, prescribed PYTHONPATH and bytecode suppression. GitHub transports were explicitly offline fixtures.

| Executed command | Exit | Observation |
|---|---:|---|
| `env -i PATH="$PATH" bash "$E/run-controls.sh"` | 0 | actionlint; release policy 31+6; publisher 259 command/7 path controls/2 mutants; unchanged CI policy 25+6 |
| `python3 -B "$E/regressions.py" original` | 1 | 36 unchanged safety controls; 32 failed expectations on original production blobs |
| `python3 -B "$E/regressions.py" fixed` | 0 | Same 36 controls; zero failed expectations on frozen candidate |
| `python3 -B "$E/adversarial.py"` | 0 | 155 production-entrypoint controls; zero failures |
| `python3 -B "$E/recovery.py"` | 0 | Four lost-acknowledgment/TERM-resistant-child recovery controls |
| `python3 -B "$E/docs-schema.py"` | 0 | Five command/schema/wiring/order checks |

Exact child argv, exits, logs and summaries are retained beneath `E`; `review-final.sha256` binds the review evidence.

**Verified claims:**
- Original absent-acceptance and Windows 7 controls published against the fixture transport: four mutations, including one publish PATCH. Candidate rejected them before any Release mutation, in both public modes.
- Both public modes require all 11 checklist keys and all 31 observation facets on both targets. Independent controls removed **every facet**, not merely checklist headings.
- Admission binds tag/run/source/ZIP/EXE/report digests; ordered 17-command clean-tagged canonical manual validation; actual manual ZIP/checksum byte equivalence; dated review coverage, independence constraints and explicit operator authorization.
- NT-major-10/build consistency and same initial PID/path/process-start provenance reject unsupported or mismatched reports. Rounded boundary metrics do not override false raw-comparison flags.
- Evidence changes before publishing prevent the PATCH. Draft preparation succeeds with deliberately unreadable evidence paths, confirming evidence-free separation.
- Historical completed-success run 99 from an earlier main SHA was inspected and skipped; matching current-source run 29 on a later page succeeded. No historical-job/log inspection or unrelated watch occurred.
- Wrong approved remote SHA, job name/count/conclusion, source ID/SHA and permanent bytes did not produce success.
- Interrupted upload failed in **3.153s**. TERM-resistant child failure completed in **6.112s**; first asset, partial evidence and caller sentinel survived. Lost-acknowledgment retry recognized committed bytes and uploaded only the missing asset into a new owned child.
- Installed `gh` 2.88.1 help matched all eight saved parent help files, including typed PATCH fields, pagination/slurp, verify-tag and non-clobber semantics.
- Linux collectors rejected actual invocation with exit 1; no field report was produced.

Parent raw exact-gate command/log/exit and hashes were inspected: exit **0**, docs **452/0 skipped**, Rust **141**, x64 GUI PE. Raw original/fixed regression exits were **1/0**. Evidence-manifest verification exited **0**. This inspection is not relabeled independent full-gate execution.

### 5. Findings, certainty and limits

**Product findings: 0.** Full production admission, mutation/recheck, source-discovery, permanent-byte verification, privacy and recovery paths were examined; deterministic adversarial calls supported the scoped claims rather than merely producing expected green output.

**Certainty:** rung **4** for tested implementation safeguards. No rung-5 GitHub or Windows claim. Human provenance still requires the documented trusted-owner review.

Historical dissent was preserved and its four blocker families independently replayed. **M6 remains BLOCKED; REL2 live acceptance remains unverified.** This verdict does not authorize public publishing.

Process deviation: an initial shallow `.wgm/` tree listing exposed a holdout-directory entry. No holdout contents were opened, exported or used as evidence.

**Highest-leverage next action:** After fresh spec/quality acceptance, execute the actual REL2 read-only draft-verification gate against the authorized parent-prepared draft and retain real run/API/download evidence.
