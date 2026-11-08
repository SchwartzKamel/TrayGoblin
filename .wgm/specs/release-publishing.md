# Spec: Actual optional GitHub Releases publishing

> Conforms to `../../specs/CONSTITUTION.md`. Optional manual dispatch complements
> the fully functional manual fallback; it is NOT a release prerequisite. Existing
> M6 actual Windows/WSL AND resource evidence is required before public publishing.
> Draft preparation/asset upload is not public release or Windows validation.

## Goal / approval
User explicitly requested actual Releases CI after binary-only Actions artifacts
proved insufficient. Add real GitHub Release creation/ZIP+checksum asset upload,
not another artifact-only workflow. Parent may push the workflow, manually create
an annotated release tag from clean validated source, and execute DRAFT preparation.
No waiver of M6, stable promotion, tag replacement or published-asset replacement.

## Smallest end-to-end / magic moment
Manually dispatch Releases for an existing validated tag and successful test-build
run. The workflow creates a draft GitHub Release with permanent ZIP/SHA assets;
owner can open Releases and download exact bytes. Later verified Windows evidence
allows explicitly requested prerelease/stable publishing. A green Actions upload
without an actual Release record/assets does not satisfy this spec.

## Workflow boundary
- `.github/workflows/release.yml`, workflow_dispatch ONLY; never auto-publish on
  main pushes, PR/fork events or schedules. Inputs: existing tag, successful source
  build run ID, mode draft/prerelease/stable (default draft), and optional repository
  field/performance report paths for publish modes. All inputs passed as data/env,
  never injected into shell code. Preserve existing read-only test-build workflow.
- Pin official checkout action SHA; credentials not persisted; fetch enough Git
  history/tags for peeled commit verification. Single bounded job/concurrency per
  release tag. Least required contents:write/actions:read solely for trusted
  release operation, no environment/PR/build secrets or Cargo execution with write
  token. GH_TOKEN uses scoped job token; never echo it/report JSON/private metadata.
- Parent creates an annotated tag manually after clean behaviour/repro checks;
  workflow REQUIRES existing tag, peels ^{commit}, validates semver/Cargo version
  at tag and source build head SHA exactly. No script-created tag or retagging.
- Reuse the successful CI artifact without rebuilding: download exact source run,
  verify workflow success/nonexpired singleton artifact, ZIP/SHA/GUI PE/buildCommit
  clean/published-at-build=false/resources. Release assets remain exact downloaded
  bytes, preserving M6's executable digest. No checksum rewriting/repacking.
- All modes first create a DRAFT release and upload ZIP+SHA, verify downloaded
  Release asset digests, then optional publish. Draft notes explicitly say Windows
  field/performance unverified/not released. Build manifest published=false records
  build-time state, not a claim about later release lifecycle.
- Existing published release is refused before mutation. Existing draft retry may
  resume only if tag/commit/mode/notes/assets match; never --clobber unknown or
  different assets, delete release/tag, or replace a published file. On partial
  failure keep draft/ownership/evidence and report fixed actionable failure;
  success only after both assets match. No false success URL.

## Publish gates (not fulfilled here)
Draft mode requires no fabricated M6 evidence. Public prerelease/stable BOTH
require actual current collector reports from trusted repo paths, checked before
ANY publish call. Reject absent/incomplete/failing/incorrect-tier/dirty/wrong-commit/
wrong-exe-digest evidence. Reports must bind exact downloaded executable SHA and
peeled tag commit, actual Windows/WSL observations and Windows process metrics,
all expected cases/timings/pass flags, at least30s sampling,<=50MB and<=5% normalized
CPU with raw pass flags. No synthetic/mocked/local CI proxy can satisfy M6.
Read reports as bounded data, not executable code; never log path/user/process
report contents. Paths stay inside explicit evidence area, no symlink/traversal.
Evidence may be committed on the dispatch source branch AFTER immutable release
tag; its buildSha must still match the tag/asset source. Document exact collection
and placement; fake fixture evidence is used ONLY in offline gate tests, never live.
M6 remains blocked, so live validation creates draft only, not public publishing.

## Exact acceptance/backpressure
| ID | Claim | Signal and origin |
|---|---|---|
| P1 | Correct dispatch/permissions/pins/data-only input flow; draft-first/no auto releases. | `actionlint .github/workflows/release.yml && python3 scripts/test-release-workflow.py` local schema/security controls. |
| P2 | Existing canonical CI artifact promoted with validated peeled tag/commit/checksum/PE, no new build. | `python3 scripts/publish-release.py --self-test` offline gh stubs + actual byte fixtures; keep live boundary separate. |
| P3 | Public publishing fails closed on missing/wrong/fake/partial evidence and protects published/mismatched drafts. | Same self-test: explicit negative cases for every gate, interrupted second upload, unknown existing asset, published retry; fixed deadlines and recovered matching draft, no success when partial. |
| P4 | Docs state draft vs public/owner authentication, manual tag/fallback and M6; current build CI retained. | `bash scripts/check-docs.sh && python3 scripts/test-ci-workflow.py` plus independent Standard docs audit. |
| P5 | Actual Releases workflow runs and actual draft ZIP/SHA download matches successful source run/tag. | `bash scripts/verify-release.sh --repo SchwartzKamel/TrayGoblin --tag v0.1.0 --draft` after real manual tag/dispatch; actual gh run/watch/API/download, not mocks. |

## Failure/recovery contract
Production discovery120s, watch1500s, download/upload120s kill-after5; CLI API errors
must propagate. Offline self-test2s test-only interruption returns failure<=7s,
keeps owned partial directory/draft asset evidence, never mutates published state,
then resumes only matching owned draft into new local child; caller sentinel stays
unchanged. Missing-second-asset/corrupt ZIP/SHA/wrong-run/wrong-tag/unpeeled tag/
expired artifact/source mismatch/token echo/traversal/evidence mismatch tests fail.
Draft or permanent success URL is emitted only after BOTH downloaded assets verify.
No arbitrary tag as shell, body templates passed by file. Secret environment values
are not inherited by any fake fixture/log outputs or serialized evidence.

## Operator/environment/ruggedness
Actual user wants GitHub Releases downloads on Windows/WSL; coding/validation on
Linux. Dominant operational risk: claiming publishing from build CI, substituting
rebuilt bytes for measured executable, irreversible published mutation or bypassing
manual field gates. Simplification: reuse exact CI artifact, one manual workflow,
draft-first, real evidence gates, no auto stable/tagging/new services. P1–P5 name
exact local/live/failure/recovery checks; missing Windows proof blocks publish, not
draft preparation. Record Plan/review current verdicts under `.wgm/rugged/` and plan.

## Holdout / scope
`.wgm/scenarios/release-draft.yaml` tier1, validator only. No runtime feature code,
root AGENTS edits, signing/update/install privilege changes or fabricated Windows
results. No published release created until actual gates pass.
