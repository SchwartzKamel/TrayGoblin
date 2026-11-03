I’ll check the reported findings against the scoped documents, preserve disagreements, and print the consolidated report to stdout to honor the read-only constraint.


# Docs Audit Report — 2026-11-03 — session-attention-plan-baseline

- **Scope:** `.wgm/IMPLEMENTATION_PLAN.md`, `.wgm/specs/session-attention.md`, `.wgm/specs/CONTEXT.md`, `specs/CONSTITUTION.md`, `AGENTS.md`, `README.md`, `docs/README.md`.
- **Track:** Full.
- **Trigger:** Plan-exit baseline.
- **Overall verdict:** **AMBER**.
- **Delivery:** Stdout for dispatcher capture; no files written.
- **Evidence boundary:** Source inspection and local-link existence checks only. Supplied baseline results—73 cargo tests and 346 documentation checks passed—were not independently reproduced. Published examples, builds, and Windows procedures were not executed. No holdout files were opened.

## Persona findings

### wgm-docs-junior — clarity & onboarding

| Doc | Observation | Severity | Recommended action |
|---|---|---|---|
| `README.md:20–21,103`; `docs/README.md:72–79` | Entry points identify the historical MVP plan as current authority but omit the follow-up plan, spec, and glossary. | AMBER | Index the follow-up as **planned, not shipped**, distinguishing MVP history and retaining M6’s Windows field boundary. |

### wgm-docs-senior — correctness, completeness, maintainability

| Doc | Observation | Severity | Recommended action |
|---|---|---|---|
| `.wgm/specs/session-attention.md:76–79,103–115`; plan M2/M3 | Error creation, acknowledgement, and retries are defined, but provider-specific error-clearing transitions are not. | AMBER | Define clearing and episode renewal across activity, success, cancellation, and session replacement; map to M1–M3 assertions. |
| `.wgm/specs/session-attention.md:61–66`; plan M3/M6 | Five-second WSL liveness caching leaves unspecified access behavior after shutdown during a cached-positive interval. | AMBER | Specify positive-cache invalidation/access behavior and test shutdown immediately after a positive query in deterministic and M6 checks. |
| Scoped command examples | Execution was prohibited; supplied baseline results were not independently reproduced. | GREEN | Preserve this evidence limitation; do not claim command-executed validation. |

### wgm-docs-principal — architecture, strategic fit, consistency

| Doc | Observation | Severity | Recommended action |
|---|---|---|---|
| `README.md:20,103`; `docs/README.md:72–79` | Public indexes omit the attention delta and describe the root MVP plan as current task authority. | AMBER | Index the follow-up plan/spec/glossary as **planned, not shipped**; preserve accurate MVP descriptions. |
| `.wgm/specs/CONTEXT.md:8` | “Source” requires explicit configuration, excluding the implicit defaults supported by the spec. | GREEN | Include default-supplied and explicitly configured sources in the definition. |

Published examples were not executed. The principal explicitly treats M6 as existing Windows field work, not an additional documentation finding.

### wgm-docs-pm — status, risk, traceability

| Doc | Observation | Severity | Recommended action |
|---|---|---|---|
| `README.md:16–28,103`; `docs/README.md:78` | Entry points omit pending follow-up tasks and planning gates, including the already-blocked M6 field validation. MVP completion is not contradicted. | AMBER | Link the planned follow-up and distinguish completed MVP work from pending attention work; retain M6 as an existing release-readiness boundary. |

## Consolidated report

### Documentation / Agent guides / Repository map

**F1 — AMBER: Follow-up planning is undiscoverable through the public indexes.** Junior, principal, and PM identify the same underlying issue. `README.md:20–21,103` and `docs/README.md:78` describe root-plan authority without exposing the follow-up, while `.wgm/IMPLEMENTATION_PLAN.md:4–5` explicitly identifies the root plan as historical. M1–M5 remain pending; M6 is blocked. This does not invalidate the completed MVP descriptions.

### Authoritative sources outside `docs/`

**F2 — AMBER: Error clearing remains underspecified.** Senior only. Full source inspection confirms that the spec defines error creation, clearing deadlines, and dropping obsolete retries, but not the provider-specific transitions that make an error obsolete. Generic recovery language in M1–M3 does not resolve that policy.

**F3 — AMBER: Cached-positive WSL shutdown behavior remains unspecified.** Senior only. The spec requires checking liveness before filesystem access, caches that result for five seconds, and prohibits UNC reads for stopped/unknown distros. Neither it nor M3 explains how access is prevented after shutdown during a cached-positive interval. Existing M6 stopped-distro observations do not explicitly cover that timing case. This is a prospective specification gap, not evidence that nonexistent implementation wakes WSL.

**F4 — GREEN: Source vocabulary excludes supported defaults.** Principal only. The glossary’s explicit-only definition conflicts with the spec’s implicit native Copilot and pi roots.

### README index check

A read-only Python check extracted Markdown links from both indexes, removed fragments, and tested local target existence:

| Check | Result |
|---|---|
| `README.md` local linked targets | 22 occurrences; none missing |
| `docs/README.md` local linked targets | 31 occurrences; none missing |
| Follow-up plan/spec/glossary | All three exist; neither public index links them |

The follow-up artifacts reference one another, so they are not claimed to be unreferenced everywhere. Their absence from both public entry points is consolidated into F1, not counted again. This check covered file existence, not anchor correctness or command executability.

### Agent actions

| # | Finding | Raised by | Doc(s) | Action |
|---|---|---|---|---|
| F1 | AMBER — Missing follow-up navigation and authority distinction | Junior, principal, PM | `README.md`, `docs/README.md` | Add **planned, not shipped** links to the follow-up plan/spec/glossary; distinguish historical MVP authority and retain M6’s explicit boundary. |
| F3 | AMBER — Cached-positive shutdown access contract missing | Senior | Attention spec; plan M3/M6 | Specify guarded access/cache behavior consistent with no-wake requirements; add the immediate-after-positive-query shutdown case to deterministic and field checks. |
| F4 | GREEN — Explicit-only Source definition | Principal | `.wgm/specs/CONTEXT.md:8` | Define Source to include implicit defaults and explicit configuration. |

### Operator actions

| # | Finding | Raised by | Doc(s) | Why it needs a human |
|---|---|---|---|---|
| F2 | AMBER — Provider-specific error-clearing policy missing | Senior | Attention spec; plan M1–M3 | Deciding which subsequent activity, success, cancellation, or replacement clears each provider’s outstanding error changes observable alert behavior. Confirm that policy, then record transitions and corresponding assertions. |

M6 remains **existing Operator action**, explicitly blocked until Windows/WSL access is available after M5. It is not a new audit finding or a requirement to obtain field evidence before implementing the plan.

### Rejected findings

**None.** All four deduplicated documentation findings survived source verification. The senior’s GREEN execution limitation is evidence metadata, not a remediation finding.

| Finding checked | Verification performed | Evidence / disposition |
|---|---|---|
| F1 | Read `README.md`, `docs/README.md`, and the follow-up plan in full; run the local-link existence/index-membership check described above. | Authority wording and all three missing public-index entries confirmed. Retained. |
| F2 | Read the entire attention spec and plan, specifically spec lines 73–79, 103–116, R1/R2/R5, and M1–M3. | Clearing deadlines and obsolete-retry rules exist, but provider-specific clearing triggers do not. Retained. |
| F3 | Read spec lines 61–66, 123–125 and plan M2/M3/M6. | Liveness guard and field checks already exist; cached-positive shutdown timing remains unspecified. Retained narrowly. |
| F4 | Compare glossary line 8 against spec lines 57–60. | Explicit-only vocabulary conflicts with implicit defaults. Retained at original GREEN severity. |

### Dissent

No explicit severity or recommended-action disagreement appears in the supplied reports. Junior, principal, and PM converge on F1 at AMBER; senior does not address it. F2/F3 are senior-only, and F4 is principal-only. Silence is not treated as four-persona unanimity.

Classifying F2 as Operator action reflects the behavioral decision required, not a change to the senior’s observation or severity.

## Verdict history

This is the supplied Full-track Plan-exit baseline consolidation. No previous report was supplied for comparison; improvement or regression is not established.

The report is emitted for dispatcher capture and structural validation. The writer did not run `audit.sh --check-report` against a persisted report file and does not claim structural-check success. Structural completion would not change the qualitative **AMBER** verdict.

