# Docs Audit Report — 2026-11-04 UTC — M4/M5 final handoff consolidation

- **Track:** Full.
- **Trigger:** Final implementation-tier handoff audit after the M5 exact gate.
- **Overall verdict:** **RED**. Verified checklist drift remains RED; additional mechanical Agent work remains open.
- **Status:** M1–M4 accepted. M5 remains `in_progress`, not accepted. M6 remains **BLOCKED**. No release-readiness or publication claim.
- **Scope:** Root/documentation indexes; `docs/operator/*.md`; `docs/agent/*.md`; Architecture, Manual release and MVP references; pi README; protected root `AGENTS.md`; constitution; active attention spec/glossary/plan; deferred findings and attention audit index.
- **Delivery:** STDOUT only under the read-only contract. Dispatcher owns capture, structural checking and filing.

## Persona findings

The following normalize the four supplied reports. Action identifiers refer to this report, not historical checkpoint identifiers.

### wgm-docs-junior — clarity & onboarding

| Doc | Observation | Severity | Recommended action |
|---|---|---|---|
| Testing, manual checklist | Legacy `Last turn:`/detail-tooltip expectations and two-second target contradict the current aggregate view and timing contract. Source-verified, not Windows-observed. | RED | A01: correct checklist expectations. |
| Troubleshooting; shipped setup | Paired collector examples omit immediate native-exit checks. | AMBER | A02: propagate canonical guards and exact-fence controls. |
| Testing; Manual release; Troubleshooting | Nondefault cadence/configuration guidance is missing from field-collection instructions. | AMBER | A03: explain matching `-PollIntervalMs` and nondefault `-ConfigurationPath`. |
| Development; Manual release; pi README | Installed pi1.4.0 and cached Debian prerequisites lack a provisioning recipe or exact setup link. | AMBER | A04: provide concise setup/check instructions separate from the offline gate. |
| Protected `AGENTS.md` | Future-T6 wording remains stale; Development mitigates routing ambiguity. | AMBER | O01: preserve protection pending scoped approval. |
| Manual release; deferred findings | Canonical historical A11/A12 corrections have current-document-matching evidence. | GREEN | Retain closure of corrected canonical defects and historical dissent. |
| Configuration and fixture examples | Existing-binary tails reproduced aggregate output, exact errors and legacy six-field states. Full Cargo fences were not executed. | GREEN | Retain examples and execution attribution. |
| Indexes; standalone resources | All 181 scoped relative targets existed; scoped guides were indexed; inspected archive resources matched source. | GREEN | Preserve grouping and persistent installed paths. |
| Active authority/status | Recorded M5 gate supports implementation evidence, not audit acceptance; collectors rejected Linux without field reports. | GREEN | Keep M5 conditional and M6 blocked. |

### wgm-docs-senior — correctness, completeness, maintainability

| Doc | Observation | Severity | Recommended action |
|---|---|---|---|
| Testing, manual checklist | Current native dispatch uses aggregate rendering; legacy tooltip requirements are incorrect. | RED | A01: update aggregate/submenu expectations and timing. |
| Troubleshooting; shipped setup | Exact offline fence controls showed first-collector failure73 still launches performance and ends successfully. | AMBER | A02: guard both copies and extend controls. |
| Manual release; audit dispositions | Historical A11 is corrected; canonical A12 controls match the current document. Original severity disagreement remains. | GREEN | Close only verified canonical corrections. |
| Configuration; Testing; Troubleshooting | Existing-binary named fixtures and explicit-config failures matched documentation; no fresh Cargo build was claimed. | GREEN | Preserve legacy/aggregate and execution distinctions. |
| Indexes; active authority | Audience routing, accepted milestones, superseded M3 history and unpublished status are identifiable. | GREEN | Preserve authority and history. |
| Installation; pi README; Manual release | Actual archive resources/PE/checksum matched inspected sources/build; identity is dirty/unpublished. | GREEN | Retain standalone instructions without claiming Windows installation. |
| Protected `AGENTS.md` | Future-T6 and literal “never read” wording conflict with completed docs and explained raw-byte scanning. | AMBER | O01: obtain scoped approval before root edits. |
| Field evidence | Genuine Windows/WSL/resource evidence remains absent; Linux rejection is not delivery proof. | GREEN | O02: retain the target-host evidence boundary. |

### wgm-docs-principal — architecture, strategic fit, consistency

| Doc | Observation | Severity | Recommended action |
|---|---|---|---|
| Configuration, build→probe fence | Injected build failure73 did not prevent successful output from the existing executable; fence exited0. | AMBER | A05: enforce prerequisite failure and add exact-fence controls. |
| Troubleshooting; shipped setup | Paired collector fences continue after first failure, unlike Manual release. | AMBER | A02: propagate native-exit guards and regression coverage. |
| Manual release — historical A11 | Both commit-identity commands now peel annotated tags; retained reports demonstrate distinct tag-object/commit identities. | GREEN | Close mechanical defect, preserve severity dissent. |
| Manual release — historical A12 | Current canonical controls cover 12 Bash and one PowerShell fences, not every duplicate example. | GREEN | Preserve canonical correction without blanket closure. |
| Indexes; architecture; guides; pi README | Audience/authority/module boundaries are coherent; archive resources and existing-binary fixtures support bounded implementation claims. | GREEN | Retain structure and standalone links. |
| Protected `AGENTS.md` | Historical routing and literal never-read wording retain a protected-file reservation. | AMBER | O01: require explicit file-and-scope authorization. |
| Active plan and M5 evidence | Existing gate evidence supports implementation-tier results; M3 blockers are superseded and M6 is not proved. | GREEN | Keep acceptance conditional and evidence tiers separate. |

### wgm-docs-pm — status, risk, traceability

| Doc | Observation | Severity | Recommended action |
|---|---|---|---|
| Testing, manual checklist | Legacy tooltip/timing acceptance criteria conflict with current aggregate implementation. | RED | A01: correct before Windows acceptance. |
| Troubleshooting; shipped setup | A12 guards are incompletely propagated; offline seams showed downstream collection after failure. | AMBER | A02: correct duplicates and controls. |
| Protected `AGENTS.md` | T6-future and never-read wording remains inconsistent; editable routing mitigates it. | AMBER | O01: preserve protection pending scoped approval. |
| M6; release guidance | Actual Windows/WSL, recovery, installation/Startup and 50MB/5% evidence remains missing. | AMBER | O02: obtain genuine matching-executable observations. |
| Manual release — historical A11 | Current peeled identity and matching R2 control evidence resolve the mechanical defect. | GREEN | Preserve historical severity disagreement. |
| Manual release — historical A12 | Current canonical exact-fence failure/success/canary evidence is present. | GREEN | Separate that correction from remaining duplicate defects. |
| Indexes; status summaries | Routing/history are traceable; some summaries still imply reviews are pending although current reviews are green. | GREEN | A06: synchronize metadata without prematurely accepting M5. |
| Configuration; Installation; resources | Existing binaries/archive support documented implementation claims, not full fresh build/install journeys. | GREEN | Retain bounded claims and execution attribution. |
| M5 evidence | Current log records Rust141, installer56/0/1 skip, docs404/0 skipped, 14 continuous-host phases and separate OCI evidence. | GREEN | Preserve current-versus-superseded evidence and skip attribution. |

## Consolidated report

### README index check

The existing **Operator guides / Agent guides / Reference / Authoritative sources** organization is retained.

Writer verification read the scoped Markdown links with Python3, resolved relative targets without traversing holdout directories, and checked existence: **181 targets, none missing**. Every scoped Operator/Agent page has an incoming index link. Reference and active-authority documents are linked; standalone setup is linked from the docs index and release guide, and pi documentation is linked through setup/testing.

The archive’s two setup links resolve internally. `resources/SETUP.md` is an **archive/installed path**; its checkout source is `assets/attention/SETUP.md`, not root `resources/SETUP.md`. No missing-file finding arises from that distinction.

### Operator guides

Surviving mechanical work concerns duplicate collector guards, field-collection cadence guidance, and the Configuration build→probe recipe. Explicit `--config` loading, aggregate output, legacy compatibility, persistent pi opt-in/removal and standalone resource links remain supported.

### Agent guides

The Windows checklist contains the verified RED defect. Development prerequisites need executable provisioning guidance. Current review metadata needs a narrow synchronization, not a broad rewrite or acceptance claim.

### Reference and authoritative sources

Canonical Manual release A11/A12 corrections are verified separately from surviving duplicate-fence defects. Active attention authority and historical MVP/M3 routing are retained. Protected root wording and actual M6 observations remain Operator matters.

### Verification record

| ID | Check performed | Evidence and boundary |
|---|---|---|
| V01 | Read `docs/agent/testing.md:136–139`, `src/app.rs:299–301,605–670`, `src/tray.rs:460–492`, and active spec timing contract. | Dispatch selects aggregate counts/attention identity; model/directory/timing are submenu details. Active transition target is 2P+500ms. No Windows rendering was observed. |
| V02 | Python3 extracted unchanged paired PowerShell fences from Troubleshooting, Manual release and the actual ZIP; invoked `pwsh -NoProfile -NonInteractive -Command` with in-memory `pwsh` seams, isolated HOME and failure73/success controls. | Both duplicate fences made two calls after the injected first failure, caught no exception and ended with native status0. Manual release made one call, threw and retained73. Success controls made two calls. Offline control flow only. |
| V03 | Read collector parameter/preflight implementation and the three published field-instruction pages. | Collector defaults expected cadence to1000, accepts `-ConfigurationPath`, and rejects effective-config mismatch. Pages omit matching-parameter guidance. This is documentation incompleteness, not a reopened cadence-parser defect. |
| V04 | Read Development prerequisites, Manual release prerequisites, pi offline guidance, `demo-attention.sh:8–19` and host package resolution. | Pi package override/placement is partially documented; initial provisioning is not. Cached `docker.io/library/debian:bookworm-slim` is required and missing cache fails without pulling. Finalizer report records that actual cache-visibility failure and retry. |
| V05 | Extracted unchanged Configuration build→probe fence; ran `bash --noprofile --norc -c` with only `cargo()` replaced by an exit73 marker seam. Used the actual named fixture and existing debug binary, isolated HOME and external temporary config. | `INJECTED_CARGO_EXIT73` appeared, but aggregate JSON followed and the fence exited0. No build occurred. This verifies stale-binary masking, not current-source validation. |
| V06 | Executed existing debug probe tails against both named fixtures and explicit temporary configuration. | Legacy states were `working`/`attention_needed`, each exactly six fields. Aggregate returned the documented source/session/capabilities. Interval499 and conflicting roots each exited1 with empty stdout and exact published errors; missing explicit config also failed. Not verbatim Cargo/build execution. |
| V07 | Read protected root wording, Development routing, Privacy scanning distinction, constitution and active M6 status. | Routing/privacy clarification exists outside the protected file. Actual Windows/WSL/resource observations remain explicitly unavailable. Personas report Linux collector rejection; writer did not rerun collectors or claim field results. |
| V08 | Hashed current Manual release and read both `M5.parent-gate-R2/release-doc-controls-*/report.json`; compared all 12 recorded Bash fences to current text. | Both document hashes match; each records34 Bash+2 PowerShell injected failures, success controls and killed canaries. Annotated tag-object differs from commit, with peeled identity equal to commit. Existing evidence inspected, not regenerated. |
| V09 | Read `M5.finalize/REPORT.md`, `M5.cadence-fix/REPORT.md`, `M5.validation.command/.log/.exit`, current review logs and retained journey/OCI JSON under the external run root. | Exact chain recorded exit0; Rust141, installer56/0/1 Windows-shortcut skip, docs404/0 skipped; 14 real offline-host phases and separate cached-image CLI-only OCI evidence. Current candidate reviews are RUGGED/specPASS/qualityPASS, with final audit/acceptance outstanding. Earlier handoffs remain historical. |
| V10 | Python3 `zipfile`/SHA256 inspection of the existing ZIP, checksum, selected resources, internal links and packaged PE. | ZIP digest matches current plan; setup/config/pi README/collectors match source, PE matches build. Manifest explicitly records dirty/unpublished. Inspection is not installation or delivery proof. |

External evidence root: `/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103`.

Current ZIP SHA256: `04e2df694234e19c84050a349dec2be342391b0f4791b1ef1c703c0dd6c63fb6`.

Current Manual release SHA256: `e413d27f0623bc1bf9fa0f178eade7a3f938b63fb677e503a94dccd468c09e87`.

### Agent actions

Each row is one deduplicated underlying issue, classified by the action required.

| ID / kind | Severity | Raised by | Doc(s) / evidence | Required action |
|---|---|---|---|---|
| A01 — **Agent action** | RED | Junior, Senior, PM | `docs/agent/testing.md:136–139`; V01 | Replace legacy tooltip/magic-moment criteria with aggregate tooltip, session-submenu details and configured 2P+500ms contract. Keep historical MVP expectations explicitly historical. |
| A02 — **Agent action** | AMBER | All four | Troubleshooting; `assets/attention/SETUP.md` → shipped `resources/SETUP.md`; V02 | Check each collector’s native exit immediately in both copies. Extend exact published/package-fence failure-marker, no-downstream, success and error-swallowing-canary controls. Preserve offline-versus-Windows attribution. |
| A03 — **Agent action** | AMBER | Junior | Testing; Manual release; Troubleshooting; V03 | Explain `-PollIntervalMs` must match effective running configuration and show `-ConfigurationPath` for nondefault config selection. Neither parameter changes the tray’s cadence. |
| A04 — **Agent action** | AMBER | Junior | Development; Manual release; pi README; V04 | Supply one concise prerequisite provisioning/check recipe or exact linked page for pi1.4.0 placement/override and cached Debian image preparation/verification. Separate separately authorized online preparation from the network-free gate. |
| A05 — **Agent action** | AMBER | Principal | Configuration build→probe fence; V05 | Chain/check dependent steps so a failed build prevents probe execution. Add exact-fence failure-marker/no-downstream, success and swallowed-error-canary controls. Preserve the actual named fixture and explicit-config example. |
| A06 — **Agent action** | GREEN | PM | README/index/status summaries; V09 | Synchronize completed review metadata during parent disposition. Keep M5 unaccepted while genuine audit work remains; do not rewrite historical reports or imply release readiness. |

### Operator actions

| ID / kind | Severity | Raised by | Doc(s) / evidence | Required action / human boundary |
|---|---|---|---|---|
| O01 — **Operator action** | AMBER | All four | Protected `AGENTS.md`; Development/Privacy; V07 | Obtain explicit authorization naming root `AGENTS.md` and wording scope before any correction. Retain existing editable mitigation meanwhile; no permission is inferred. |
| O02 — **Operator action** | PM AMBER; others GREEN boundary records | All four | Active M6; Testing; release guidance; V07/V09 | Supply genuine matching-executable Windows/WSL interaction, suppression/recovery, installation/Startup and resource observations. Keep M6 blocked until actual evidence exists. Packaging, real offline pi host, OCI CLI, compilation, mocks and syntax cannot substitute. |

### Rejected findings

These are historical defects or interpretations now disproved/already mitigated, not allegations that current personas re-raised them.

| Finding | Raised by | Exact check or source verification | Evidence / disposition |
|---|---|---|---|
| Historical A11: annotated-tag object still recorded as commit identity | Historical Senior RED / Principal AMBER; correction recognized by all four current reports | Read both current `git rev-parse --verify 'refs/tags/v0.1.0^{commit}'` locations; `hashlib.sha256(Path('docs/manual-release.md').read_bytes())`; compare R2 `documentSha256` and `annotatedTag` objects (V08). | Both retained reports match current text and show tag-object≠commit, peeled=commit. Current mechanical defect closed; historical severity dissent remains. No tag operation rerun. |
| Historical A12: canonical Manual release fences still lack enforced fail-fast controls | Historical Junior/Senior AMBER; correction recognized by all four | Compare current extracted fences with R2 reports:12 exact Bash fences,34 Bash+2 PowerShell failures, success controls/canaries; directly replay canonical PowerShell failure/success seams (V02/V08). | Canonical correction verified. Reject continued canonical-defect status, **not** A02 duplicate defects or A05’s separate recipe defect. No blanket fleet-wide closure. |
| Historical checkpoint aggregate-config journey lacks explicit loading/output/errors | Historical Junior/Senior/PM; current reports retain corrected examples | Read Configuration loading rule and example; run existing debug executable with named fixture, temporary explicit config, interval499 and conflicting roots (V06). | Loading distinction/output/exact failures now present and reproduced. A05 remains independently open because build failure can still be masked. |
| Historical editable routing/privacy omissions remain unchanged | Historical checkpoint findings; current reports recognize mitigation | Read Development active routing and Privacy raw-byte/allowlist/GUI-versus-probe distinctions; compare protected root separately (V07). | Editable mitigations exist. Do not retain those old omissions as current defects; O01 remains open for protected wording. |
| Superseded M3 rejection is a current implementation blocker | Historical checkpoint, explicitly superseded by current personas | Read active plan M3 accepted evidence/history, Architecture M3 recovery checkpoint and current audit-index dispositions. | Accepted recovery/fix supersedes old stop instructions; dissent/history remains intact. No fresh M3 runtime reproduction claimed. |
| Historical project-wide notification non-goal contradiction | Historical Principal interpretation, already rejected at checkpoint | Read checkpoint R01 and current MVP/Architecture historical scope against active additive attention spec. | Historical “this pass” scope does not prohibit the approved follow-up. Preserve rejection and narrower historical clarity concern; do not invent a live policy conflict. |

### Dissent

No unanimous-clean conclusion is recorded. Silence by a persona on a uniquely raised issue is not disagreement or endorsement.

| Finding | Views retained | Disposition |
|---|---|---|
| Current M6 evidence gap | PM AMBER emphasizes missing operational evidence; Junior/Senior/Principal GREEN recognize accurate disclosure of that same limitation. | One **Operator action**, O02. Preserve risk-versus-documentation severity distinction; no evidence disagreement is invented. |
| Historical A11 | Senior RED; Principal AMBER. | Mechanical correction verified, both historical severities retained. |
| Historical A13 contradictory plan status | PM RED; Junior/Senior/Principal AMBER. | Historical correction/supersession retained; no old M3 blocker revived. A06 is only current review-metadata synchronization. |
| Historical A04 privacy inventory | Junior GREEN; Senior/Principal AMBER; PM did not raise it. | Corrected inventory remains recognized without rewriting historical urgency. |
| Historical O01 protected root wording | PM GREEN; Junior/Senior/Principal AMBER. Current four reports all record AMBER. | Preserve historical difference and current convergence; approval remains an **Operator action**. |
| Historical A01 public status | PM AMBER; Principal GREEN qualification; other personas did not raise that specific item. | Preserve historical difference; current active-authority/status qualifications are supported. |
| Historical A10 non-goal interpretation | Principal AMBER broad interpretation was rejected by source scope; narrower clarity concern survived. | Retain rejected interpretation and narrowed history, not invented consensus. |
| Historical E09/O02 executability | Junior AMBER reported unavailable checks; other personas’ GREEN records had differing executed boundaries; checkpoint writer’s batch was denied. | Preserve role-specific history. Current successful binary/seam checks do not retroactively grant execution to prior roles. |
| Historical M3 review disagreement | Rejected candidate had spec CHANGES-REQUESTED despite other green signals; later compiled proof/fix received independent acceptance. | Preserve rejection and later acceptance as sequential history, not competing current verdicts. |

## Verdict history

Previous entry: `.wgm/docs/audit/2026-11-04T0114Z_attention-wip-checkpoint.md`, immutable historical **RED**.

This consolidation remains **RED**, with changed live causes: accepted M3 recovery and canonical A11/A12 corrections are recognized, while the verified current checklist defect and additional Agent actions remain open. Protected-root authorization and M6 evidence are separate Operator limitations, not grounds to waive mechanical work.

## Execution and filing limits

No project source/docs/git/config mutations, builds, formatters, publication/network/model-service calls, other agents or holdout-directory reads were performed. Temporary probe/control files were external and cleaned up.

The initial `python` command was unavailable (exit127); verification used `python3`. An initial PowerShell stdin harness produced no result marker and was discarded as inconclusive; complete `-Command` scriptblocks produced the V02 observations. Neither preliminary attempt is counted as positive evidence.

Dispatcher must capture this STDOUT and run:

```bash
bash /home/lafiamafia/.agents/skills/wgm/scripts/audit.sh --check-report /home/lafiamafia/sandbox/TrayGoblin/.wgm/audit-run-48wjkx/wgm-docs-writer.md
```

Writer does not claim that pending structural check exited0. Filing/indexing belongs to the dispatcher after successful checking. Structural COMPLETE/exit0 cannot convert this qualitative **RED** report to GREEN.

