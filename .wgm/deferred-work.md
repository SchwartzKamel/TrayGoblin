# Pre-existing documentation findings — M5 dispositions

These remain **Agent actions**, not waived findings or Operator decisions. They
predate this attention run; the checkpoint's byte-equivalence observation is
historical. M5 now owns the coupled release-documentation corrections. No release
operation is authorized. Author implementation/controls are not independent
acceptance or a qualitative GREEN audit. The [active plan](IMPLEMENTATION_PLAN.md)
is the sole authority for the current candidate, exact gate results, review/audit
verdicts and next acceptance action. M5 remains in_progress/unaccepted pending
actual disposition; M6 is BLOCKED; the implementation is unpublished, not release-ready.

| Origin | Severity | Finding and required follow-up |
|---|---|---|
| 2026-11-04 checkpoint docs audit A11 | RED (Senior), AMBER (Principal) | M5 author corrected tag/notes identity to `git rev-parse --verify 'refs/tags/v0.1.0^{commit}'`. `node scripts/test-release-docs.mjs` proves real isolated annotated tag-object≠commit; the exact published command now resolves the commit. Rejected historical doc assertion-fails; corrected doc passes. Current acceptance state belongs only in the active plan. |
| 2026-11-04 checkpoint docs audit A12 | AMBER | M5 author made dependent release fences explicit fail-fast subshells. Pre-docs evidence ran 12 EXACT Bash + 1 PowerShell fences, 36 independent injected native failures with markers/no-downstream assertions, success controls, dirty/placeholder negatives and per-fence error-swallowing canaries. Historical fence assertion-fails; corrected fences pass. These are isolated PATH seams, NOT publication/network/Windows results. |

## Final audit A01–A06 — recorded author changes

The immutable [final RED report](docs/audit/2026-11-04T0405Z_attention-final.md)
verified canonical A11/A12 corrections, not every duplicate. M5 remains unaccepted.

| Finding | Current author disposition |
|---|---|
| A01 RED | Aggregate Windows checklist/counts/attention identity, persistent submenu metadata and configured2P+500ms; MVP six-field/detail-tooltip expectations explicitly historical. |
| A02/A05 AMBER | Both duplicate collectors immediately guard each native exit; build→probe fails fast. Exact101442e source AND actual oldZIP fences execute and assertion-fail on marker-proven downstream work/exit0; fixed source/actual NEW package pass with success/no-downstream/canary controls. Offline control flow, not Windows/build evidence. |
| A03/A04 AMBER | Cadence aliases/default/null1000, matching PollIntervalMs/actual ConfigurationPath; one shipped pi README provisioning/check recipe with separately authorized online preparation, never tests/pulls/network in the offline gate. |
| A06 GREEN action | Live summaries point to the active plan for current candidate/gates/verdicts instead of duplicating transient review state. Historical audit findings and dissent remain unchanged. |

External `RUN/M5.docs-fix/REPORT.md`: complete isolated gate0, Rust141,
installer56+1 existing Windows skip, docs408/0 skipped; 14 real-host phases/5
requests, separate OCI JSON/cleanup. Each complete release-doc run:12 canonical
Bash+2 PS+4 audit fences,44 injected failures plus per-fence swallowed-error
canaries. Raw logs/exits/hashes and separate rejected/fixed sources retained.
Current candidate/gates/verdicts and next acceptance action are recorded only in
the active plan, not inferred from this retained evidence. No self-certified GREEN
or acceptance. O01 protected AGENTS authorization and O02 real Windows/WSL/resource
evidence remain Operator gaps.

Full observations, original severity disagreements and verification limits:
[checkpoint report](docs/audit/2026-11-04T0114Z_attention-wip-checkpoint.md).
