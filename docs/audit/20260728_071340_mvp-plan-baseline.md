# WGM Docs Audit — MVP Plan Baseline

- **Scope:** Full-track, Plan-exit baseline audit of `README.md`, `docs/MVP.md`, `AGENTS.md`, `IMPLEMENTATION_PLAN.md`, `specs/` (CONSTITUTION, CONTEXT, ADR 0001, tray-status spec + checklist), and `scenarios/`.
- **Date (UTC):** 2026-07-28T07:13:40Z
- **Track:** Full
- **Gate:** Plan-exit
- **Worst severity:** RED (README.md is title-only; `docs/MVP.md` conflicts with the accepted native-Rust ADR)
- **Unanimous:** **No** — see [Dissent](#dissent).
- **Grouping rationale:** No docs map exists yet, so findings are grouped using the repository's minimal actual layout: root README/AGENTS/plan, and `docs/` + `specs/` (spec, checklist, ADR, constitution, context).

## Junior reviewer
- `README.md` is title-only (`# TrayGoblin`, no other content): **RED**. Needs overview, current status, navigation, and quick start.
- `docs/MVP.md` still describes an Electron/Tauri + Node.js stack, but ADR 0001 and the constitution accept a native Rust process: **AMBER**. Update or prominently cross-reference the final decision.
- `AGENTS.md` references commands, scripts, and fixtures (`scripts/validate-toolchain.sh`, `scripts/check-docs.sh`, `scripts/demo.sh`, `tests/fixtures/live-session`, etc.) that do not exist yet: **AMBER**. Mark them as forward contracts fulfilled by plan tasks (T1–T6), not present-tense fact.
- `IMPLEMENTATION_PLAN.md` lacks an explicit "current status: start with T1" note for a new reader: **AMBER**.

## Senior reviewer
- `README.md` title-only: **RED** (same finding as Junior; see Dissent for severity disagreement).
- `docs/MVP.md` stack contradiction with ADR 0001: **AMBER** (same finding as Junior; see Dissent for stack-alignment disagreement with PM).
- `AGENTS.md` and `IMPLEMENTATION_PLAN.md` command/file paths are stated as forward contracts with no caveat that they don't exist yet: **AMBER**.
- The privacy operational rule ("TrayGoblin must never read or model event content, arguments, results, tokens, or credentials") lacks an explicit deterministic test contract tying it to a specific fixture/assertion: **AMBER**.
- No manual release process document exists yet (`docs/manual-release.md`): **AMBER**, correctly planned by T6.
- The satisfaction score meaning (what "95" measures, how it's computed) is not defined anywhere in `IMPLEMENTATION_PLAN.md`: **AMBER**.
- The performance criterion in `specs/tray-status.md` ("shall target under 50 MB ... and under 5% CPU") uses "target" language rather than an unambiguous hard shall/pass-fail gate: **AMBER**.
- Unknown/future Copilot CLI output-format resilience should be explicitly backpressured (test or acceptance criterion), though fixtures/scenarios are otherwise strong.

## Principal reviewer
- `README.md` title-only: **RED** (concurs with Senior; see Dissent).
- `docs/MVP.md` conflicts with the accepted native-Rust ADR 0001: **RED** (elevates severity above Junior/Senior's AMBER; see Dissent).
- `docs/MVP.md` "Future Enhancements" section should point to the spec's "Out of scope (this pass)" section (`specs/tray-status.md:51`) instead of drifting independently: **AMBER**.
- `AGENTS.md` should link to the forthcoming `docs/architecture.md` (planned by T6) rather than leaving architecture undocumented with no pointer: **AMBER**.
- T6 must ensure `docs/architecture.md`, `docs/manual-release.md`, and `scripts/check-docs.sh` are all created together, since T6's acceptance criteria depend on all three existing: **AMBER**.
- The native-Rust tradeoff itself is well recorded (ADR 0001's Context/Decision/Alternatives/Consequences are complete), and the constitution, spec, plan, and scenarios are otherwise mutually consistent.

## PM reviewer
- `README.md` title-only: **AMBER** (lower severity than the other three reviewers; see Dissent).
- The T6 validation script `scripts/check-docs.sh` does not exist yet: **AMBER**; ensure it is created by T6 or earlier so the plan's own validation command is satisfiable.
- PM considers `docs/MVP.md` generally aligned with project intent (goals, feature list, and scope are still directionally correct) despite the stack detail being stale, unlike the other reviewers who treat the stack contradiction as a correctness defect: **this dissent is preserved explicitly below.**
- All `IMPLEMENTATION_PLAN.md` tasks (T1–T6, TZ) are accurately marked `pending`, and privacy, performance, and release risks and their traceability to spec/scenarios are otherwise clear.

## Consolidated Agent action
All items below are concrete documentation/spec/plan fixes with no human judgment call required, so they are classified as Agent action.

| # | Action | Source finding(s) | Target file(s) | Severity |
|---|---|---|---|---|
| 1 | Expand `README.md` beyond the bare title: add an overview, current build/plan status, navigation to `docs/MVP.md`, `IMPLEMENTATION_PLAN.md`, and `specs/`, and a quick-start pointer. | Junior, Senior, Principal, PM (README title-only) | `README.md` | RED |
| 2 | Update `docs/MVP.md`'s Architecture/Components section to reflect the accepted native Rust process (ADR 0001), or add a prominent cross-reference banner pointing to `specs/adr/0001-native-rust-tray.md` as the authoritative decision. | Junior, Senior, Principal (stack contradiction); PM dissents on severity/necessity | `docs/MVP.md` | AMBER/RED (disputed — see Dissent) |
| 3 | Add a caveat in `AGENTS.md` and `IMPLEMENTATION_PLAN.md` clarifying that referenced scripts, commands, and fixtures are forward contracts to be created by the corresponding plan tasks (T1–T6), not currently present. | Junior, Senior (forward-contract paths) | `AGENTS.md`, `IMPLEMENTATION_PLAN.md` | AMBER |
| 4 | Add an explicit "current status" line to `IMPLEMENTATION_PLAN.md` stating the plan starts at T1 and nothing is done yet. | Junior | `IMPLEMENTATION_PLAN.md` | AMBER |
| 5 | Tie the privacy operational rule in `AGENTS.md`/spec to a named deterministic test/fixture contract (e.g., reference the T2 fixture tests that assert no content fields are modeled). | Senior | `AGENTS.md`, `specs/tray-status.md` | AMBER |
| 6 | Define what the satisfaction score (threshold 95) measures and how it is computed in `IMPLEMENTATION_PLAN.md`. | Senior | `IMPLEMENTATION_PLAN.md` | AMBER |
| 7 | Reword the performance criterion in `specs/tray-status.md` from "shall target" to an unambiguous pass/fail gate (e.g., "shall not exceed"), consistent with `scripts/measure-performance.ps1` as the enforcing check. | Senior | `specs/tray-status.md` | AMBER |
| 8 | Add an explicit acceptance/test note backpressuring unknown/future Copilot CLI output-format changes (beyond "non-fatal"), e.g., a named degraded-format fixture case. | Senior | `specs/tray-status.md`, `IMPLEMENTATION_PLAN.md` (T2/T3) | AMBER |
| 9 | Point `docs/MVP.md`'s "Future Enhancements" section to `specs/tray-status.md`'s "Out of scope (this pass)" section instead of maintaining a separate, drifting list. | Principal | `docs/MVP.md` | AMBER |
| 10 | Add a link from `AGENTS.md` to the forthcoming `docs/architecture.md` (to be created by T6), noting it does not exist yet. | Principal | `AGENTS.md` | AMBER |
| 11 | Confirm T6's task description/acceptance criteria in `IMPLEMENTATION_PLAN.md` explicitly enumerate `docs/architecture.md`, `docs/manual-release.md`, and `scripts/check-docs.sh` as required T6 deliverables so none is silently dropped. | Principal, PM (check-docs.sh) | `IMPLEMENTATION_PLAN.md` | AMBER |

## Consolidated Operator action
None identified. All findings above resolve to concrete, executable documentation/spec/plan edits with no dependency on human judgment, external approval, or irreversible decisions. No item was invented to fill this section.

## Dissent
The audit is **not unanimous**. Recorded disagreements:

1. **README severity:** Junior, Senior, and Principal rate the title-only `README.md` **RED**; PM rates it **AMBER**. Consolidated severity is recorded as RED (worst-case), with PM's lower rating preserved as dissent.
2. **`docs/MVP.md` stack contradiction — severity and existence of a defect:** Junior and Senior rate the Electron/Tauri + Node vs. native-Rust-ADR contradiction **AMBER**; Principal escalates it to **RED** given it directly conflicts with an Accepted ADR. PM dissents further, judging `docs/MVP.md` "generally aligned" with project intent and not treating the stale stack detail as a correctness defect requiring urgent action. All three positions (AMBER, RED, "aligned") are preserved; the consolidated table lists this item as disputed rather than resolving it to a single severity.

No dissent was recorded on the remaining AMBER findings (forward-contract caveats, privacy test contract, missing manual-release doc, undefined satisfaction score, performance gate wording, format-resilience backpressure, future-enhancements pointer, architecture doc link, T6 deliverable completeness, and check-docs.sh existence) — all four reviewers who addressed each of those items agreed on severity and framing.

## Resolution

All Plan-exit Agent actions were applied after this baseline review:

- `README.md` now provides status, navigation, and a repository map.
- `docs/MVP.md` now reflects ADR 0001's native Rust architecture and authoritative scope.
- `AGENTS.md` and `IMPLEMENTATION_PLAN.md` identify command paths as forward contracts.
- The plan defines satisfaction scoring and explicitly requires all T6 documentation deliverables.
- The spec now has hard performance thresholds plus named privacy and future-format test contracts.

**Gate result after resolution:** PASS. The original findings and dissent remain above as the immutable audit record.
