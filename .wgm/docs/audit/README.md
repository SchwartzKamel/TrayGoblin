# Session attention documentation audits

| Date | Verdict | Coverage | Report |
|---|---|---|---|
| 2026-11-04 | RED checkpoint; coupled Agent findings corrected, pre-existing release findings deferred; not qualitative GREEN | Accepted M2 and blocked M3 WIP; four fresh personas plus writer, same-family context/lens independence | [Blocked checkpoint](2026-11-04T0114Z_attention-wip-checkpoint.md) |
| 2026-11-03 | AMBER findings addressed before build; see dispositions below | Full-track plan baseline; four fresh Copilot persona contexts plus a writer, same tooling/model family | [Baseline](2026-11-03T2148Z_attention-baseline.md) |

## Checkpoint dispositions
- Structural report check: installed `scripts/audit.sh --check-report
  .wgm/docs/audit/2026-11-04T0114Z_attention-wip-checkpoint.md` exited0. This does
  not convert RED to GREEN. The full report preserves original persona findings,
  disagreements, rejected interpretation and execution limits unchanged.
- A01-A10 Agent actions: a fresh documentation-only author corrected the seven
  named status/config/privacy/development/testing/architecture documents. Direct
  synthetic probe examples executed successfully, with exact aggregate/error
  outputs and six legacy fields. Isolated docs gate exited0:381 checks,0 skipped.
  Raw evidence is external under `checkpoint-docs/`; no product code changed.
- A13 Agent action: parent marked obsolete status/gates/author handoffs
  superseded and made accepted M1/M2, blocked M3, pending M4/M5 and blocked M6 the
  authoritative checkpoint. Reviewer dissent is retained, not averaged.
- A11/A12 remain genuine **pre-existing / deferred Agent actions**, documented in
  [deferred work](../../deferred-work.md), not silently dismissed or reclassified.
  Manual-release documentation is byte-identical to pre-run0d79111.
- O01 remains an Operator action: root AGENTS.md is protected and unchanged.
  Editable Development guidance now explains the historical routing.
- O02 remains an Operator/evidence action: actual Windows/WSL interaction and
  resource evidence cannot be manufactured here. M3 acceptance is additionally
  blocked before M4/M5; this checkpoint is not a final feature audit or release.

## Baseline dispositions
- F1 Agent action: README and docs index now link the planned, unshipped follow-up
  plan/spec/glossary and keep M6's real Windows/WSL boundary explicit.
- F2 Operator-action recommendation reclassified as an explicit conventional
  recovery default under the user's approval to proceed simply: errors persist
  through Idle/notification acknowledgement, clear on new provider work; dialog end
  clears input only; shutdown/replacement drops old-instance pending alerts.
  Recorded tests are owed in M1–M3. No new privacy/product capability is assumed.
- F3 Agent action: removed the overpromised atomic no-wake requirement and its
  liveness subprocess/cache. Configured UNC reads may wake/keep WSL running;
  documents must say so. M6 records actual host behavior, source removal disabling
  reads, and native monitoring during shutdown. This is a declared platform
  trade-off, not a claim that a cached positive query prevents shutdown races.
- F4 Agent action: Source now includes implicit defaults and explicit sources.
- Structural contract: `audit.sh --check-report` is run before indexing; dispatcher
  exit0 already validated the produced report. This does not turn AMBER into an
  independent qualitative GREEN. Corrected plan contracts receive rugged re-review.
