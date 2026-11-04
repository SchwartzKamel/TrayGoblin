# Deferred pre-existing documentation findings

These are **Agent actions**, not waived findings or Operator decisions. They
predate this attention run: `git diff --exit-code 0d79111 -- docs/manual-release.md`
exited0 at the blocked checkpoint. Do not mix unrelated historical release
changes into M3's product recovery. No release operation is authorized.

| Origin | Severity | Finding and required follow-up |
|---|---|---|
| 2026-11-04 checkpoint docs audit A11 | RED (Senior), AMBER (Principal) | Annotated-tag object is described as commit identity in `docs/manual-release.md`. In a dedicated documentation task, peel and verify `refs/tags/v0.1.0^{commit}` wherever a commit identity is required; prove the distinction with synthetic local tags, not a publication. |
| 2026-11-04 checkpoint docs audit A12 | AMBER | Dependent manual-release examples do not enforce fail-fast behavior. Correct exact fenced commands in a dedicated task and exercise prerequisite-failure markers, no-downstream-launch assertions, success control and an error-swallowing canary in an isolated offline harness. Never publish as a test. |

Full observations, original severity disagreements and verification limits:
[checkpoint report](docs/audit/2026-11-04T0114Z_attention-wip-checkpoint.md).
