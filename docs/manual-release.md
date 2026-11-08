# Manual release

**Audience:** Agents

This includes the independent manual, no-CI fallback and OPTIONAL manually
dispatched Releases CI, NOT authorization to publish. Version 0.1.0
is **unpublished**. The active [attention plan](../.wgm/IMPLEMENTATION_PLAN.md)
governs this candidate; M6 actual Windows/WSL and resource evidence remains
blocked. An implementation/cross-build/archive is not a delivered release.
Root MVP plans are historical. See [Release responsibilities](agent/release-responsibilities.md).

## Prerequisites

Stable Rust, MinGW-w64 and `x86_64-pc-windows-gnu`; Node with installed pi1.4.0;
PowerShell, Podman with cached Debian bookworm-slim, `zip`, `unzip`, `sha256sum`,
`file`, Git, Python 3.11+ (`tomllib`), GNU coreutils/`timeout`, plus tooling-only
PyYAML/actionlint for optional controls. Use the one bounded
[release-tool provisioning/offline-check recipe](agent/development.md#release-tool-prerequisites),
including `gh` installation; authenticated `gh` is needed only for separately
authorized live inspection/publication, not offline checks.
Optional read-only Actions test ZIPs are not releases or prerequisites; local
manual validation/build/package remains authoritative. See
[Release responsibilities](agent/release-responsibilities.md#optional-ci-never-a-release-prerequisite).
A real Windows 10/11 x86-64 + WSL
host is required for field verification, not replaced by OCI or cross-compilation.
Use the shipped [provisioning/check recipe](../integrations/pi/README.md#offline-gate-prerequisites)
for the actual npm pi1.4.0 package/override and cached image. Online preparation
requires SEPARATE operator authorization; tests/offline gates never install or
pull. Under isolated HOME select the verified public Podman root/runroot explicitly,
not credential or user-config mounts.

Every dependent Bash fence below is a fail-fast subshell. Run each whole fence;
do not paste individual later lines after failure. The offline
`scripts/test-release-docs.mjs` executes these exact fences against isolated PATH
seams, verifies failure markers/no-downstream execution and success controls, and
kills an error-swallowing canary. That proves control flow, NOT publication.

## Step 1 — Freeze the version

Change Cargo.toml and archive references together, then have the parent accept and
commit the candidate. Do not tag/push/publish during an author iteration. Official
release artifacts must come from a clean tagged commit per the
[constitution](../specs/CONSTITUTION.md). Packaging an uncommitted author candidate
records `worktreeDirty: true` and is not an official release.

## Step 2 — Prove the working tree is clean

```bash
(
  set -euo pipefail
  status="$(git status --porcelain)" || exit $?
  [[ -z "$status" ]] || { echo 'error: release tree is dirty' >&2; exit 1; }
)
```

Any output is an abort condition. Resolve changes through the parent, not an
automatic reset/stash. Root `AGENTS.md` remains protected.

## Step 3 — Run the full deterministic validation

```bash
(
  set -euo pipefail
  cargo test --workspace || exit $?
  cargo clippy --all-targets -- -D warnings || exit $?
  bash scripts/validate-toolchain.sh || exit $?
  bash scripts/demo-attention.sh --container podman || exit $?
  bash scripts/demo.sh 0.1.0 || exit $?
  bash scripts/check-docs.sh || exit $?
)
```

Run in isolated HOME/USERPROFILE/XDG/pi data with verified public tool caches;
see [Testing](agent/testing.md). Preserve logs/exits and source/executable digests.
The original MVP demo still asserts exactly six fields, privacy and both fixtures.

## Step 4 — Verify the installer and packaging scripts

```bash
(
  set -euo pipefail
  pwsh -NoProfile -File scripts/test-installer.ps1 || exit $?
  node scripts/test-release-docs.mjs || exit $?
)
```

Linux installer checks are sandbox-only. The Windows-shortcut case remains
explicitly skipped there, not passed. Field/performance collectors reject Linux.

## Step 5 — Tag the commit

Only after parent acceptance AND separate release authorization:

```bash
(
  set -euo pipefail
  git tag -a v0.1.0 -m "TrayGoblin 0.1.0 preview" || exit $?
  git rev-parse --verify 'refs/tags/v0.1.0^{commit}' || exit $?
)
```

Record that **commit** SHA. An annotated tag has a DIFFERENT tag-object ID;
`git rev-parse v0.1.0` alone is not commit identity. Do not push yet.

## Step 6 — Build and package from the tag

```bash
(
  set -euo pipefail
  git checkout --detach v0.1.0 || exit $?
  bash scripts/package-release.sh 0.1.0 || exit $?
)
```

Exact archive file allowlist (directories are also checked):

- `tray-goblin.exe`, `install.ps1`, `uninstall.ps1`, `LICENSE`, `README.txt`
- `integrations/pi/tray-goblin.js`, `integrations/pi/package.json`, `integrations/pi/README.md`
- `resources/SETUP.md`, `resources/config.example.json`, `resources/build.json`
- `resources/test-attention-windows.ps1`, `resources/measure-performance.ps1`

The script cross-builds and verifies an x86-64 PE32+ executable, packages it twice
with normalized permissions/timestamps, compares hashes, checks the exact entries,
checks packaged PE bytes against the build, and generates/verifies the ZIP SHA256.
`SOURCE_DATE_EPOCH` overrides the fixed timestamp. Reproducibility is for a given
executable/tool version, NOT independently compiled binaries across machines.
Persistent resources are copied below install root. No silent pi settings edits.
Standalone [setup source](../assets/attention/SETUP.md) is shipped as resources/SETUP.md.

## Step 7 — Record the artifact identity

```bash
(
  set -euo pipefail
  sha256sum dist/tray-goblin-0.1.0-windows-x86_64.zip || exit $?
)
```

Save the hash and peeled commit SHA. `resources/build.json` binds the actual
executable SHA256 and commit/dirty flag. Never substitute archive SHA for PE SHA.

## Step 8 — Operator verification on Windows

Use the actual archive; verify `Get-FileHash` against its `.sha256`, install as a
standard user, then follow [First run](operator/first-run.md) and the
[Manual Windows checklist](agent/testing.md#manual-windows-checklist).
Collectors are IN the archive; no source checkout is required:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pwsh -NoProfile -File "$tg\resources\measure-performance.ps1" -Path "$tg\tray-goblin.exe" -DurationSeconds 30 -JsonPath .\windows-performance.json
if ($LASTEXITCODE -ne 0) { throw 'Actual performance gate failed' }
pwsh -NoProfile -File "$tg\resources\test-attention-windows.ps1" -ExecutablePath "$tg\tray-goblin.exe" -EvidencePath .\windows-attention.json -PollIntervalMs 1000
if ($LASTEXITCODE -ne 0) { throw 'Actual attention field gate failed' }
```

`-PollIntervalMs` declares the EXPECTED running cadence, not a tray setting.
Match `pollIntervalMs` or `poll_interval_ms`; omitted/null means 1000ms. For a
nondefault config location, `-ConfigurationPath` must select the actual running
file. For example, ONLY for a tray already using this file at 2000ms, replace the
attention collector invocation above with the invocation below, retaining the
preceding performance invocation and its exit check:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pwsh -NoProfile -File "$tg\resources\test-attention-windows.ps1" -ExecutablePath "$tg\tray-goblin.exe" -EvidencePath .\windows-attention.json -PollIntervalMs 2000 -ConfigurationPath 'D:\TrayGoblinProfile\TrayGoblin\config.json'
if ($LASTEXITCODE -ne 0) { throw 'Actual attention field gate failed' }
```

Neither parameter changes tray cadence or supplies a GUI config flag. Edit the
actual tray config and restart first. P sets attention/clear (2P+500ms) and quiet
checks (3P). Stop the whole collection on any native failure.

Measure performance FIRST, then start the field collector on that SAME running
PID/path/processStartUtc. The field journey eventually restarts/quits TrayGoblin;
collecting resources after that would not establish the same initial process.
Both reports must establish NT major 10, matching full version/build and Windows
10 (build 10240–21999) or Windows 11 (build >=22000).

The attention script is genuinely INTERACTIVE, with per-case observations and
conservative stopwatch timings, not yes/no capability inference. Both reports
must identify the SAME executable digest. The Path performance gate rejects a
mismatched running process. Record Windows/WSL versions, UTC, build SHA/PID/path,
source kinds/load and genuine case outcomes. Native notifications may be
suppressed; persistent list, actual UNC wake/recovery and removal/restart/no reads
are part of the field journey. Also verify Startup/sign-in and uninstall.
**This measurement must pass before promoting a preview build to stable.**
For this attention follow-up M6 field AND resource evidence must pass before any
publication is considered. No historical preview exception waives that blocker.

## Step 9 — Publish

Only a separately authorized release owner may do this AFTER all gates/independent
reviews. Set `WINDOWS_RESULT` and `PERFORMANCE_RESULT` from the actual matching
field reports. Missing values fail before notes creation; never invent scores.

```bash
(
  set -euo pipefail
  TAG_SHA="$(git rev-parse --verify 'refs/tags/v0.1.0^{commit}')" || exit $?
  ARCHIVE_SHA="$(sha256sum dist/tray-goblin-0.1.0-windows-x86_64.zip | awk '{print $1}')" || exit $?
  : "${WINDOWS_RESULT:?set from actual Windows field report}"
  : "${PERFORMANCE_RESULT:?set from matching Windows performance report}"
  cat > release-notes.md <<EOF
# TrayGoblin 0.1.0 preview

- Tag commit: ${TAG_SHA}
- Archive SHA-256: ${ARCHIVE_SHA}
- Windows verification: ${WINDOWS_RESULT}
- Performance: ${PERFORMANCE_RESULT}
- Limitations: Windows x86-64; unsigned; no auto-update; per-user install; Copilot input unavailable; UNC may wake WSL; notifications may be suppressed.
EOF
  if grep -q '<' release-notes.md; then
    echo 'error: unresolved release placeholders' >&2; exit 1
  else
    rc=$?; [[ $rc -eq 1 ]] || exit "$rc"
  fi
)
```

After inspecting notes and confirming publication authorization:

```bash
(
  set -euo pipefail
  git push origin v0.1.0 || exit $?
  gh release create v0.1.0 \
    dist/tray-goblin-0.1.0-windows-x86_64.zip \
    dist/tray-goblin-0.1.0-windows-x86_64.zip.sha256 \
    --title "TrayGoblin 0.1.0 (preview)" --notes-file release-notes.md --prerelease || exit $?
)
```

Keep it a prerelease until genuine field/resource evidence and operator acceptance
are recorded. This run performs NO tag/push/publication. No web-UI bypass of gates.

## Step 10 — Verify the published release

```bash
(
  set -euo pipefail
  gh release view v0.1.0 || exit $?
  mkdir -p dist/verify || exit $?
  cd dist/verify || exit $?
  gh release download v0.1.0 || exit $?
  sha256sum --check tray-goblin-0.1.0-windows-x86_64.zip.sha256 || exit $?
)
```

Require `OK` and the saved archive digest. Verification failure blocks handoff;
local tests cannot be relabeled published-asset evidence.

## Abort conditions

Dirty tree; nonzero prerequisite; incorrect tag/version/PE/digest; two-pack
mismatch; elevation or unsafe deletion; failed/missing real Windows/WSL case;
resource budget exceeded/unmeasured; missing independent acceptance: **stop**.
Do not publish to find out whether the candidate works.

## Rollback

Only the authorized release owner executes destructive/public operations. Preserve
evidence. Before pushing:

```bash
(
  set -euo pipefail
  git tag -d v0.1.0 || exit $?
  rm -rf dist || exit $?
  rm -f release-notes.md || exit $?
)
```

Tag pushed, release not created:

```bash
(
  set -euo pipefail
  git push origin :refs/tags/v0.1.0 || exit $?
  git tag -d v0.1.0 || exit $?
  rm -f release-notes.md || exit $?
)
```

Published defective release (withdraw, do not replace bytes under the same name):

```bash
(
  set -euo pipefail
  gh release delete v0.1.0 --yes || exit $?
  git push origin :refs/tags/v0.1.0 || exit $?
  git tag -d v0.1.0 || exit $?
  rm -f release-notes.md || exit $?
)
```

Then use a NEW version; never move an already-published tag. Operator rollback:
remove pi opt-in/reload, uninstall and install the prior known-good archive;
TrayGoblin configuration is kept by default.

## Optional manual Releases CI

This alternative promotes an EXISTING canonical Windows test-build ZIP/checksum
unchanged to an actual GitHub Release. Public modes additionally require a manual
build from the exact clean annotated tagged canonical main source and verification
that its ZIP AND executable equal the promoted CI bytes. CI success is not that
manual build/acceptance. Build manually without a write token; a byte mismatch
blocks public admission rather than rebuilding/repacking in the write-token job. It does not rebuild/repackage measured
bytes or create tags. Local Steps 1–8/manual fallback remain independent; CI is
never a validation/release prerequisite. Existing read-only Windows test CI remains.

**Now: DRAFT preparation only.** M6 actual Windows/WSL AND performance are absent.
After REL1 local gates and fresh independent acceptance, the parent may push main,
manually create/push an annotated `v0.1.0` from clean validated source if absent
(Step 5; never retag), and identify a successful **Windows test build** run whose
head SHA EXACTLY equals `git rev-parse 'refs/tags/v0.1.0^{commit}'`. The build run
must have one nonexpired canonical artifact. Lightweight/unpeeled/wrong-source
or version-mismatched tags fail; no branch fallback. No author runs these actions.

Dispatch command template (NOT executed by offline fallback-fence tests): the
authorized owner sets `BUILD_RUN_ID` to that actual numeric run ID before use.

```text
(
  set -euo pipefail
  : "${BUILD_RUN_ID:?set actual successful exact-tag Windows build run ID}"
  gh workflow run release.yml --repo SchwartzKamel/TrayGoblin --ref main -f tag=v0.1.0 -f build_run_id="$BUILD_RUN_ID" -f mode=draft || exit $?
  bash scripts/verify-release.sh --repo SchwartzKamel/TrayGoblin --tag v0.1.0 --draft || exit $?
)
```

Run the verifier from a CLEAN local `main` whose HEAD, `refs/heads/main` and
`refs/remotes/origin/main` equal the approved dispatch commit. Origin must be the
named GitHub repository; live remote main must still match. Verify before further
main metadata changes; there is no branch/SHA fallback. Inspected run ID, main,
source SHA and workflow ID/path must match this pin, distinct from the immutable
tag/build SHA. Completed successful runs across bounded paginated results are
inspected first. Unrelated running/failing jobs are never watched and cannot
preempt success; discovery polls for completion for 120 seconds, then fails closed.

The workflow creates a DRAFT, attaches both exact files, downloads actual Release
assets and verifies their digests before success. Notes disclose untested,
unpublished/M6, unsigned per-user local install and limitations. Draft downloads
require owner/collaborator GitHub sign-in; they are not anonymous public downloads.
Draft assets persist independently of Actions' 14-day retention. The REL2 verifier
reports actual owner-visible Release URL/IDs, not an Actions artifact link. Its
read-only source run checks do not require the original artifact to remain unexpired
after Release creation. `resources/build.json`'s published=false means BUILD TIME;
never edit/repack it to claim publication.

**Current v0.1.0 is DRAFT-ONLY:** its `mode=draft` ownership cannot be promoted by
same-tag public retry or UI. Current draft field work is exploratory, NOT acceptance
for a different future version. Preserve the immutable current tag/build/assets.

**Future public prerelease/stable:** requires a separately authorized NEW version,
annotated tag and exact-source CI test build, actual same-payload manual/CI ZIP AND
EXE byte equivalence, and fresh real Windows 10+11/M6/performance/independent/operator
records bound to THAT version. Before initially dispatching that version in a
public mode, obtain its actual verified CI **test archive**, verify/install as a
standard user on actual Windows 10/11 + local WSL, explicitly configure native+WSL
pi/Copilot sources, then run both exact Step 8 collector commands against the
installed executable (performance FIRST, same initial PID/path/processStartUtc).
Include ALL 11 checklist items/facets on BOTH targets, Startup/sign-in/uninstall,
and fresh independent M6 acceptance. Public-mode evidence is required BEFORE ANY
draft mutation: do NOT prepare `mode=draft` first and promise later promotion.
Both original reports bind the new peeled build SHA, clean flag and downloaded EXE
digest. Do not rebuild after measuring; a manual/CI byte mismatch blocks admission.

Place the original collector JSON (no report editing, pass autofill or fake fixtures)
under `.wgm/release-evidence/<NEW-version>/field.json` and
`.wgm/release-evidence/<NEW-version>/performance.json` on trusted dispatch main
(replace `<NEW-version>` with the actual version, never a literal placeholder).
The v0.1.0 literal paths/schema/command fences below are version-substitution
examples, NOT current-draft public permission: collect fresh paths, tag/sourceRun,
source SHA, archive/EXE/report digests and dated records, never carry forward
current tag/build/reports. Reports
may be committed AFTER the immutable release tag; the report buildSha still binds
the tag, not dispatch HEAD. The owner reviews actual collection provenance and
privacy before placing files: no real sessions, auth/config/conversation data or
private path/user metadata may be exposed in a public repository. Use an isolated
synthetic Windows profile/load for real observations; never synthetic observations.
The workflow reads bounded JSON DATA only from that area (no traversal/symlinks),
never executes it or logs report contents.

The gate checks all 55 expected named field cases, observed PASS, exact per-case
cadence/deadlines/quiet durations, completed/passed, process/hash/time identity,
Windows/WSL/source/load/UNC information. Performance uses the CURRENT collector's
`durationSeconds` (>=30), samples/processors, `peakWorkingSetMb`/`averageCpuPercent`,
`maxWorkingSetMb`<=50/`maxIdleCpuPercent`<=5 and BOTH raw-comparison pass flags plus
passed. Those flags are computed from unrounded measurements; the collector does
not serialize raw byte/CPU counters. Rounded values alone cannot waive the flags.
Wrong tier/source/hash, partial/failing/contradictory reports fail before any
Release mutation. Offline fake reports are only shape/negative controls, never M6.

Only after genuine M6 acceptance and explicit public authorization, dispatch main
with mode `prerelease` (or `stable`) and inputs `field_evidence`,
`performance_evidence` and `acceptance_evidence` set to the exact repository paths
for that NEW version. Freeze that request's tag/source build run/mode/evidence
bindings; only matching retries of that same request keep them unchanged. Both
public modes first prepare/verify their own matching draft AFTER admission, then
perform the single publish call. A previously prepared mode=draft record is NOT a
public-mode retry: mode/notes/evidence ownership must match. Do not rewrite it to
promote, and never bypass this fail-closed boundary via UI. Public assets are anonymously downloadable only AFTER
confirmed publishing, unlike drafts. Neither public mode is run by this author task.

**Recovery:** check invocation syntax/help first (`python3 scripts/publish-release.py --help`
or `bash scripts/verify-release.sh --help`). Pre-allocation argument validation may
create NO owned directory; inspect the printed owned path only when allocated.
A local publisher/verifier child at the chosen out-root is durable: retain it,
sanitized commands/exits and ownership/notes on API/upload failure. The hosted
publisher child is under `runner.temp`, ephemeral and **NOT exported** by
`release.yml`; do not promise scratch retention or post-job retrieval.

For hosted failures preserve accessible failed Actions logs and command/job exits
(`gh run view RUN_ID --repo SchwartzKamel/TrayGoblin --log-failed`, with its CLI
exit, and read-only `gh run view RUN_ID --repo SchwartzKamel/TrayGoblin --json status,conclusion,headSha,url`).
Before retry, inspect/save read-only run/source and draft/assets state with
`gh api repos/SchwartzKamel/TrayGoblin/actions/runs/RUN_ID`,
`gh api repos/SchwartzKamel/TrayGoblin/releases/RELEASE_ID` and
`gh api repos/SchwartzKamel/TrayGoblin/releases/RELEASE_ID/assets` (actual numeric
IDs; authorized access only, sanitize private data). Compare tag/commit/title/body/
mode/ownership/digests against the SAME request before retry. The recorded first
hosted failure's narrower cause remains **UNKNOWN**; generic failure logs do not
prove a first cause. No success URL is emitted until BOTH permanent assets match. Rerun the SAME tag/run/mode/evidence only for a matching
owned draft; missing uploads resume into a NEW local child. Published releases,
unknown/different drafts/assets, tag changes or changed notes/digests/mode are
refused; no clobber/delete/rewrite. Inspect an incomplete server-side upload with
the owner rather than automatically deleting it. Never discard caller files or
use an Actions artifact success as a Release handoff.

## Complete public acceptance record

A separate REQUIRED input for BOTH public modes, not a flag or collector
replacement. Missing/failing acceptance rejects BEFORE draft creation or upload
for a public request; all evidence is re-read before the sole `PATCH draft=false`.
Draft mode does not read or require it.

### Creation and placement

Only the authorized owner creates real acceptance; this author task does not.
For future public admission every v0.1.0 literal below must be substituted with the
separately authorized NEW version and its fresh collection paths/identities, as
specified above. Obtain that version's verified CI test bytes and finish the real
records BEFORE its first public-mode dispatch, never via a draft-mode promotion.

1. Before evidence commits, freeze/tag accepted canonical main as Steps 1–5 require.
   In a clean checkout with canonical origin/main at that same commit, detach to
   the tag and repeat the exact-source command list below. Retain actual UTC,
   argv, exits and content-free result summaries from real logs. Manually package
   without a write token. CI success never substitutes for these commands.
2. Copy BOTH manually built files into
   `.wgm/release-evidence/v0.1.0/manual-bytes/`: exactly
   `tray-goblin-0.1.0-windows-x86_64.zip` and its `.zip.sha256` sibling, no extras.
   These are validation artifacts, not replacements for CI assets. Admission reads
   the actual ZIP/checksum/PE/clean manifest and requires BOTH ZIP and EXE hashes
   to equal promoted CI bytes. Compiler/tool-version mismatch blocks admission.
3. On BOTH real Windows 10 and Windows 11 x64 hosts, verify the promoted archive
   and perform ALL 11 checklist items. Record every facet below separately with
   actual UTC/outcome/commands/results, including Explorer/shell/menus, sign-out/
   sign-in and both uninstall variants. Use isolated synthetic profiles, never
   private sessions. On the primary collector host measure performance FIRST,
   then field on that same initial process. Preserve original reports as
   `.wgm/release-evidence/v0.1.0/field.json` and
   `.wgm/release-evidence/v0.1.0/performance.json`.
4. Using an ordinary JSON editor, create UTF-8
   `.wgm/release-evidence/v0.1.0/acceptance.json` with EXACTLY the schema below,
   transcribing real logs and observations, not autofilled PASS. Three fresh,
   distinct non-author/non-builder/non-observer reviewers supply spec/quality/
   rugged M6 dispositions and concrete review coverage. The observing operator
   explicitly authorizes public publishing AFTER reviews. Parent privacy-reviews
   and commits evidence on trusted main AFTER the immutable tag. This directory
   is git-ignored: parent explicitly force-adds only the reviewed evidence files;
   this author never stages/commits them.
5. Owner dispatches `acceptance_evidence` as
   `.wgm/release-evidence/v0.1.0/acceptance.json`, with the two collector paths above.
   Every public retry uses the SAME record/digests. No UI/flag bypass or generic
   `passed:true`. Never put offline positive controls in these live paths.

### Acceptance schema v1

Unique JSON keys; safe relative paths inside `.wgm/release-evidence/`, no symlinks/
traversal. JSON <=256 KiB, manual ZIP <100 MiB. Unknown/missing record keys reject.
Strings are nonempty and <=4096 characters except clean status stdout is empty.
Dates are actual ISO-8601 timezone-aware UTC. Boolean/numeric fields are typed.
Participant identities (`author`, `builder`, observers/reviewers/operator) are
consistent canonical lowercase IDs: 1–128 characters from letters/digits/`._@-`,
starting with a letter/digit. Whitespace/case aliases are rejected; parent must
verify that different IDs really identify different independent people.
The publisher logs only sanitized operations/exits and digests, not report bodies.

Top-level keys (exact):

- `schema`: `tray-goblin-acceptance-v1`.
- `identity`: exactly `repo`, `tag`, `sourceRun` (positive integer build run ID),
  `sourceSHA` (peeled 40-lowercase-hex commit), `archiveSha256`,
  `executableSha256`, `fieldSha256`, `performanceSha256` (64-lowercase-hex).
  Report digests cover ORIGINAL FILE BYTES including BOM/newlines; ZIP and EXE
  hashes are separate. All values must equal downloaded/validated identities.
- `createdUtc`, `author`: record UTC and actual candidate author's identity.
- `manual`: exactly `sourceSHA`, `tag`, `worktreeDirty` (false), `canonicalMainSHA`
  (same tag commit), `builder`, `startedUtc`, `finishedUtc`, `commands`,
  `archivePath` (the manual ZIP path above).
- `targets`: exactly two records, one `Windows10`, one `Windows11`, defined below.
- `reviews`: exactly three records, one each `spec`, `quality`, `rugged`.
- `operator`: exactly `identity` (full identical binding), `name` (one target
  observer), `acceptedUtc`, `decision` (exactly
  `AUTHORIZE-PUBLIC-PRERELEASE-AND-STABLE`), `statement` (>=20 characters of actual
  explicit acceptance/authorization, not a flag).

`manual.commands` is the ordered argv-array list below. Each entry has EXACTLY
`argv`, `exit` (integer 0), `startedUtc`, `finishedUtc`, `result` (actual content-free
summary/stdout), `resultSha256` (SHA256 of the UTF-8 result STRING, no implicit
newline). Summarize long logs within 4096 characters while retaining real counts/
results; keep full raw logs with the operator. For v0.1.0, use exactly:

```text
git checkout --detach v0.1.0
git status --porcelain
git rev-parse --verify HEAD
git rev-parse --verify refs/tags/v0.1.0^{commit}
git rev-parse --verify refs/heads/main
git rev-parse --verify refs/remotes/origin/main
git remote get-url origin
git cat-file -t refs/tags/v0.1.0
cargo test --workspace
cargo clippy --all-targets -- -D warnings
bash scripts/validate-toolchain.sh
bash scripts/demo-attention.sh --container podman
bash scripts/demo.sh 0.1.0
bash scripts/check-docs.sh
pwsh -NoProfile -File scripts/test-installer.ps1
node scripts/test-release-docs.mjs
bash scripts/package-release.sh 0.1.0
```

Split tokens into `argv` strings in this order. Status result is empty; all four
rev-parse results equal `sourceSHA`; origin is canonical HTTPS with/without `.git`
or `git@github.com:SchwartzKamel/TrayGoblin.git`; cat-file result is `tag`.
Package result includes its actual archive hash and exact emitted line
`Reproducible: two independent packing runs matched.` Compute result digests
locally as `hashlib.sha256(result.encode('utf-8')).hexdigest()` in Python; file
hashes use `sha256sum` or `Get-FileHash`. Later versions substitute their actual
tag/version in argv and filenames; no undefined placeholder strings are accepted.

Each target has EXACTLY `os` (`Windows10`/`Windows11`), `architecture` (`x64`),
`windows` (collector-shaped `caption`, `version`, `build`), `observer`, `startedUtc`,
`finishedUtc`, `checks`. Full version is `10.0.` plus build; Windows10 builds
10240–21999, Windows11 >=22000. One target's metadata/window must match BOTH
primary reports. Performance NT version/build AND PID/path/processStartUtc match
field provenance. Do not edit metadata to match; measure before field quits.

`checks` has EXACTLY the 11 keys and facet keys below. Each facet value has
EXACTLY `outcome` (`PASS`), `observedUtc`, `observation` (>=20 characters of specific
actual command/result or UI/OS observation, not merely PASS/name):

| Checklist key | Required facet keys |
|---|---|
| `install` | `standard-user`, `files-localappdata`, `startup-shortcut` |
| `icon` | `registered`, `explorer-restart-restored` |
| `legibility` | `idle-working-attention-shape-color`, `high-contrast` |
| `magic-moment` | `blocking-and-manual-dialog`, `other-session-progress`, `bounded-attention-clear`, `quiet-completion-abort-recovery`, `copilot-error-not-completion` |
| `tooltip` | `aggregate-list-agreement`, `same-directory-identity`, `submenu-model-directory-run-start-duration`, `bounded-shell-text` |
| `menu` | `attention-first-persistent-identity-metadata`, `suppressed-notifications-list`, `refresh`, `vscode`, `copilot-logs`, `settings`, `quit` |
| `configuration` | `out-of-range-content-free-dialog` |
| `startup` | `sign-out-sign-in-autostart` |
| `uninstall` | `files-shortcut-removed`, `configuration-kept`, `remove-configuration` |
| `field` | `all-55-native-wsl-cases`, `file-access-trace-no-reads` |
| `performance` | `same-exe-host-30s-50mb-5percent` |

Each review has EXACTLY `role`, `reviewer`, `reviewedUtc`, `verdict` (`ACCEPT`),
`identity` (full identical binding), `coverage`, `findings` (empty list: unresolved
findings block). Reviewers are distinct from one another AND author, manual builder
and all observers. `coverage` has exactly six keys:
`manual-source-validation-repro`, `manual-ci-byte-equivalence`, `Windows10-all11`,
`Windows11-all11`, `field-55-wsl-privacy`, `performance-supported-same-host-budgets`.
Each value is >=20 characters describing actual reviewed command results/digests,
observations, resource flags/budgets and disposition. Name/role alone is insufficient.

Chronology: ordered manual commands fit manual start/finish; target windows follow
manual finish; every observation fits its target window; BOTH primary reports fit
one matching target window. Reviews follow BOTH targets and precede operator
acceptance, which precedes creation. Reviews/creation are within seven days of
completion; creation is in the past seven days, never future. Stale/missing/failed/
partial/non-independent/wrong-binding/digest records reject. Structural admission
cannot prove a human told the truth: trusted owner privacy/provenance review of
actual raw logs and observations is mandatory. Synthetic controls prove logic ONLY.
M6 remains BLOCKED; no live acceptance record is claimed by this task.

## Related

- [Release responsibilities](agent/release-responsibilities.md)
- [Testing](agent/testing.md)
- [Installation](operator/installation.md)
