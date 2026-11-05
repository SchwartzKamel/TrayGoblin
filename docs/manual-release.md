# Manual release

**Audience:** Agents

This is the manual, no-CI procedure, NOT authorization to publish. Version 0.1.0
is **unpublished**. The active [attention plan](../.wgm/IMPLEMENTATION_PLAN.md)
governs this candidate; M6 actual Windows/WSL and resource evidence remains
blocked. An implementation/cross-build/archive is not a delivered release.
Root MVP plans are historical. See [Release responsibilities](agent/release-responsibilities.md).

## Prerequisites

Stable Rust, MinGW-w64 and `x86_64-pc-windows-gnu`; Node with installed pi1.4.0;
PowerShell, Podman with cached Debian bookworm-slim, `zip`, `unzip`, `sha256sum`,
`file`, Git and (only when separately authorized to publish) authenticated `gh`.
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
pwsh -NoProfile -File "$tg\resources\test-attention-windows.ps1" -ExecutablePath "$tg\tray-goblin.exe" -EvidencePath .\windows-attention.json -PollIntervalMs 1000
if ($LASTEXITCODE -ne 0) { throw 'Actual attention field gate failed' }
pwsh -NoProfile -File "$tg\resources\measure-performance.ps1" -Path "$tg\tray-goblin.exe" -DurationSeconds 30 -JsonPath .\windows-performance.json
if ($LASTEXITCODE -ne 0) { throw 'Actual performance gate failed' }
```

`-PollIntervalMs` declares the EXPECTED running cadence, not a tray setting.
Match `pollIntervalMs` or `poll_interval_ms`; omitted/null means 1000ms. For a
nondefault config location, `-ConfigurationPath` must select the actual running
file. For example, ONLY for a tray already using this file at 2000ms, replace the
first collector line above with:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pwsh -NoProfile -File "$tg\resources\test-attention-windows.ps1" -ExecutablePath "$tg\tray-goblin.exe" -EvidencePath .\windows-attention.json -PollIntervalMs 2000 -ConfigurationPath 'D:\TrayGoblinProfile\TrayGoblin\config.json'
if ($LASTEXITCODE -ne 0) { throw 'Actual attention field gate failed' }
```

Neither parameter changes tray cadence or supplies a GUI config flag. Edit the
actual tray config and restart first. P sets attention/clear (2P+500ms) and quiet
checks (3P). Stop the whole collection on any native failure.

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

## Related

- [Release responsibilities](agent/release-responsibilities.md)
- [Testing](agent/testing.md)
- [Installation](operator/installation.md)
