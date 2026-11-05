# Spec: Optional Windows CI test artifacts

> Conforms to `../../specs/CONSTITUTION.md`. CI is an OPTIONAL convenience, not
> a validation/release prerequisite. Manual clean-tagged release and actual M6
> Windows/WSL/performance evidence remain required. No official release is published.

## Goal / JTBD
The operator wants pushed code and a Windows ZIP downloadable from GitHub, rather
than a build available only on this Linux checkout.

## Confirmed scope and defaults
- User explicitly approved pushing existing commits and adding an optional GitHub
  Actions build uploading downloadable Windows test ZIPs.
- Existing14 commits were pushed; remote main matched dbe78f7 on intake.
- Use Ubuntu24.04 runner and existing Rust/MinGW scripts to cross-build Windows
  x86-64 GNU; this is Windows-target compilation, NOT an interactive Windows run.
- Trigger on push to main and workflow_dispatch. No pull_request_target, release,
  scheduled or fork-triggered workflow.20-minute job timeout and branch concurrency.
- Root GITHUB_TOKEN permissions exactly contents:read; no job permission escalation,
  secrets passed to scripts, release creation, tag writes, automatic promotion or
  deployment. Checkout credentials are not persisted.
- Pin checkout/upload actions to reviewed immutable commit SHAs. Rust stable and
  x86_64-pc-windows-gnu target; install public apt MinGW/zip/file/python3-yaml tools.
- Run actual repository core/cross-build signal and canonical packager. Keep manual
  full pi-host/OCI/Windows gates separate. Packaging includes persistent pi/setup
  resources and build.json, SHA256; it must say published=false and clean source SHA.
- Upload only dist/*.zip and dist/*.zip.sha256, artifact named
  tray-goblin-windows-x86_64, retention14days, error on missing files. Upload only on
  successful build. No caches or extra third-party actions required.
- GitHub Actions artifact download normally requires a signed-in GitHub account.
  This is not an anonymous GitHub Release download; docs must distinguish them.

## Magic moment / demo
- Push the workflow and current code; open the successful Actions run; download the
  artifact, verify its inner ZIP/checksum and start the standard-user Windows setup.
- Smallest end-to-end slice: actual GitHub job runs canonical build/package,
  uploads nonempty artifact, gh downloads it, checksum/PE/build SHA are verified.
- Merely functional: a green YAML linter without a downloadable asset is insufficient.

## Acceptance → exact backpressure
| ID | Requirement | Exact signal / origin |
|---|---|---|
| C1 | Workflow syntax, triggers, pinned actions, minimal permissions and success-only restricted artifact paths match scope. | `actionlint .github/workflows/windows-test-build.yml && python3 scripts/test-ci-workflow.py` locally; policy tests reject counterfactual unsafe workflows in memory. |
| C2 | Existing Windows-target build/package remains authoritative and unchanged. | `bash scripts/validate-toolchain.sh && bash scripts/package-release.sh 0.1.0` locally, synthetic core tests and actual PE/ZIP checks. |
| C3 | Docs explicitly state optional CI/test ZIPs, signed-in download, manual release and unverified M6 boundaries; current M1–M5 status accurate. | `bash scripts/check-docs.sh`, source cross-check, final independent four-persona+writer docs audit. |
| C4 | All approved commits are on remote main; actual GitHub workflow succeeds for the exact tested commit and has one nonexpired downloadable artifact. | `bash scripts/verify-ci-build.sh --repo SchwartzKamel/TrayGoblin --commit "$(git rev-parse HEAD)" --out-root .wgm/ci-downloads` after push; real GitHub API/run watch/download, not a mock. |
| C5 | Downloaded checksum/ZIP integrity, x86-64 Windows GUI PE and manifest SHA match the requested commit, clean=true and published=false. | Same live verify command; decode actual downloaded bytes with stdlib, no execution of the Windows binary on Linux. |

## Validation design and failure/recovery
- CI policy checker uses PyYAML BaseLoader (no unsafe constructors/on-as-bool
  ambiguity), validates full job structure and explicit paths. Pin/prohibited
  trigger/permissions/credentials/release/empty upload counterexamples must fail.
- Live verifier bounds discovery/wait, checks inspection exit codes, selects exact
  head SHA and workflow/event, fails on cancelled/failed runs or absent/ambiguous/
  expired artifacts. Downloads into a NEW owned child directory, never wipes a
  caller directory. Discovery deadline120s; run-watch1500s; download120s, with
  kill-after5s. No success/download URL printed before verification completes.
  Reports actual run/artifact browser URL and exact digest only on success.
- `bash scripts/verify-ci-build.sh --self-test` is an exact deterministic failure
  signal before live CI2: controlled gh stub cases include missing/wrong-SHA run,
  failed/cancelled watch, expired/ambiguous artifact and interrupted partial download.
  The partial-download case uses a2s test-only deadline (production remains120s),
  exits nonzero within7s, prints no success URL, preserves caller sentinel bytes
  and task-owned partial directory/evidence. Successful retry uses a DIFFERENT
  new owned child and passes checksum/PE/manifest; deleting caller root, ignoring
  inspection errors or returning a success-shaped incomplete download must fail.
  These are offline controls, never evidence of actual GitHub availability.
- If a real run fails, retain failure logs, fix only the actual defect, rerun exact
  local signals and real workflow. Offline checks are not success at GitHub boundary.
- Final push may trigger another run due metadata commit; verify latest remote HEAD
  too, and supply its actual downloadable artifact link rather than a guessed URL.

## Operator/environment and ruggedness
Operator is the requesting Windows/WSL multibox user; code is edited on Linux.
Expected stress: missing MinGW/target, failed public dependency install, skipped
upload, expired artifact, credential overreach and misleading release status.
Dominant risk is operational: a green local build is not proof of remote artifact
availability. Simplification: canonical scripts, two pinned official actions,
read-only job token, no automatic releases/caches/services. Exact C1–C5 probes
settle policy/build/live-download claims; actual Windows operation remains M6.
Plan readiness and per-diff rugged verdicts are recorded in plan and `.wgm/rugged/`.

## Non-goals
No stable/prerelease GitHub Release, signing, installer privilege change, updates,
GUI automation/performance in CI, branch protection requiring CI, pi credential
configuration, real user-session reading, or editing root AGENTS.md.

## Holdout
`.wgm/scenarios/ci-download.yaml` tier1; authors must not open holdout files.
