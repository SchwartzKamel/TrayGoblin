# Session attention documentation audits

| Date | Verdict | Coverage | Report |
|---|---|---|---|
| 2026-11-03 | AMBER findings addressed before build; see dispositions below | Full-track plan baseline; four fresh Copilot persona contexts plus a writer, same tooling/model family | [Baseline](2026-11-03T2148Z_attention-baseline.md) |

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
