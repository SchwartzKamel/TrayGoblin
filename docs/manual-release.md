# Manual release

**Audience:** Agents

This is the complete, reproducible procedure for publishing a TrayGoblin preview **without any CI
service**. Every step is a command you run locally, in order, from a clean tagged commit through a
checksummed archive and a manually published GitHub release.

No GitHub Actions workflow, and no other automated pipeline, builds, validates, signs, or publishes
this project. That is a non-negotiable in [`specs/CONSTITUTION.md`](../specs/CONSTITUTION.md).

Two roles appear below:

| Role | Owns |
|---|---|
| **Agent** | Everything reproducible on a Linux or WSL host: validation, cross-build, packaging, checksum, tagging, and the GitHub release entry |
| **Operator** | Everything that requires real Windows: installer run, tray interaction, and the performance measurement that gates stable promotion |

If you only need the responsibility split, read
[Release responsibilities](agent/release-responsibilities.md). This page is the procedure.

## Prerequisites

- Stable Rust with the `x86_64-pc-windows-gnu` target installed
- The MinGW-w64 x86-64 GCC toolchain on `PATH`
- `zip`, `unzip`, `sha256sum`, and `file`
- `git`, and `gh` authenticated against this repository for the publish step
- A Windows 10 or 11 x86-64 machine available for the operator checks

Confirm the toolchain before anything else:

```bash
bash scripts/validate-toolchain.sh
```

## Step 1 — Freeze the version

The packaging script refuses to build an archive whose version does not match `Cargo.toml`, so the
version is decided first.

1. Set `version` in `Cargo.toml` to the release version, for example `0.1.0`.
2. Update any documentation that names the archive file.
3. Commit the change on the branch you intend to release.

## Step 2 — Prove the working tree is clean

A release must be built from a clean tree so the artifact corresponds exactly to the tag.

```bash
git status --porcelain
```

Any output at all — modified, staged, or untracked files — is an **abort** condition. Commit,
stash, or remove the files and start again.

## Step 3 — Run the full deterministic validation

```bash
cargo test --workspace
cargo clippy --all-targets -- -D warnings
bash scripts/validate-toolchain.sh
bash scripts/check-docs.sh
```

All four must exit zero. See [Testing](agent/testing.md) for what each command covers and for the
fixture journeys.

## Step 4 — Verify the installer and packaging scripts

```bash
pwsh -NoProfile -File scripts/test-installer.ps1
```

This runs `install.ps1` and `uninstall.ps1` against a sandbox profile and statically checks all
three PowerShell scripts for elevation, machine-wide paths, and configuration-preservation
regressions. On a non-Windows host the Windows-shortcut checks are reported as **skipped**, not
passed; they are covered by the operator checks in step 8.

## Step 5 — Tag the commit

```bash
git tag -a v0.1.0 -m "TrayGoblin 0.1.0 preview"
git rev-parse v0.1.0
```

Record the commit SHA the tag points at. Do not push the tag yet: the tag is only worth publishing
once the artifact built from it has passed every check.

## Step 6 — Build and package from the tag

```bash
git checkout v0.1.0
bash scripts/package-release.sh 0.1.0
```

The script:

1. Confirms the requested version matches `Cargo.toml`.
2. Cross-builds `tray-goblin.exe` for `x86_64-pc-windows-gnu` and verifies it is a PE32+ x86-64
   executable.
3. Stages `tray-goblin.exe`, `install.ps1`, `uninstall.ps1`, `LICENSE`, and a generated
   `README.txt`.
4. Normalizes file permissions and timestamps, then packs the archive **twice** and compares the
   two SHA-256 hashes. Differing hashes are a hard failure: a non-reproducible package must not be
   published.
5. Writes and verifies `dist/tray-goblin-0.1.0-windows-x86_64.zip.sha256`.
6. Tests the archive's integrity and asserts every expected entry is present.

Set `SOURCE_DATE_EPOCH` to override the fixed archive timestamp only if you must. For a given
executable and packaging-tool version, leaving it unset makes repeated packaging runs
byte-identical. This procedure does not claim that independently compiled executables are
byte-identical across machines.

## Step 7 — Record the artifact identity

```bash
sha256sum dist/tray-goblin-0.1.0-windows-x86_64.zip
```

Save this hash next to the tag SHA from step 5. Every later verification compares against these two
values.

You may re-run step 6 on the same executable to confirm packaging determinism. If you compare
machines, compare the executable hash first: only expect the ZIP hashes to match when the
executables and packaging-tool versions already match. The packaging script's own two-pack mismatch
is an **abort** condition.

## Step 8 — Operator verification on Windows

Copy the archive and its `.sha256` file to a Windows 10 or 11 x86-64 machine. The performance script
is not part of the archive, so the operator also needs a checkout of this repository at the same tag
for step 5. The operator then:

1. Verifies the checksum in PowerShell:

   ```powershell
   Get-FileHash .\tray-goblin-0.1.0-windows-x86_64.zip -Algorithm SHA256
   ```

   It must equal the hash from step 7.
2. Extracts the archive and installs without elevation:

   ```powershell
   powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
   ```

3. Confirms the icon appears, then walks the interaction checklist: Copilot turn start shows
   **Working** within two seconds, turn end shows **Idle** with a `Last turn:` line, and each of
   **Refresh now**, **Open in VS Code**, **View Copilot logs**, **Open settings**, and **Quit**
   behaves as documented in [First run](operator/first-run.md).
4. Signs out and back in to confirm the Startup shortcut launches TrayGoblin.
5. Runs the performance gate:

   ```powershell
   pwsh -NoProfile -File scripts/measure-performance.ps1 -DurationSeconds 30 -JsonPath perf.json
   ```

   The script fails when the peak working set exceeds 50 MB or average CPU exceeds 5% of total
   capacity. **This measurement must pass before promoting a preview build to stable.** A preview
   may be published while the measurement is still outstanding, provided the release notes say so
   explicitly; a stable promotion may not.
6. Confirms uninstall is clean and preserves configuration:

   ```powershell
   powershell -NoProfile -ExecutionPolicy Bypass -File .\uninstall.ps1
   ```

Record the Windows build number, the pass/fail result of each checklist item, and the performance
numbers. These belong in the release notes.

## Step 9 — Publish

Before publishing, create the notes file used by the command. Fill the Windows values from the
operator record in step 8; for a preview whose performance gate is explicitly outstanding, say so
instead of inventing a result.

```bash
TAG_SHA="$(git rev-parse v0.1.0)"
ARCHIVE_SHA="$(sha256sum dist/tray-goblin-0.1.0-windows-x86_64.zip | awk '{print $1}')"
WINDOWS_RESULT="<Windows version and interaction-check result>"
PERFORMANCE_RESULT="<performance numbers, or explicitly outstanding for preview>"

cat > release-notes.md <<EOF
# TrayGoblin 0.1.0 preview

- Tag commit: ${TAG_SHA}
- Archive SHA-256: ${ARCHIVE_SHA}
- Windows verification: ${WINDOWS_RESULT}
- Performance: ${PERFORMANCE_RESULT}
- Limitations: Windows x86-64 only; unsigned; no auto-update; per-user install.
EOF
```

Replace both angle-bracket placeholders before continuing:

```bash
if grep -q '<' release-notes.md; then
  echo "error: release-notes.md still contains placeholders" >&2
  exit 1
fi
```

For a preview, steps 2–7 plus the Windows checksum, install, and interaction checks in step 8 must
pass; performance may be explicitly outstanding. For stable promotion, all of step 8, including
performance, must pass.

```bash
git push origin v0.1.0
gh release create v0.1.0 \
  dist/tray-goblin-0.1.0-windows-x86_64.zip \
  dist/tray-goblin-0.1.0-windows-x86_64.zip.sha256 \
  --title "TrayGoblin 0.1.0 (preview)" \
  --notes-file release-notes.md \
  --prerelease
```

Publish as a prerelease until the Windows performance measurement in step 8 has passed and the
interaction checklist is complete. Promote to a normal release only afterwards, by editing the
existing release rather than rebuilding the artifact.

If `gh` is unavailable, the same result is achieved through the web UI: create a release from the
existing tag, upload both files, paste the notes, and mark it as a prerelease.

Release notes must include:

- the tag and the commit SHA it points at
- the archive SHA-256 from step 7
- the Windows version used for verification and the performance numbers, or an explicit statement
  that the performance gate is still outstanding for this preview
- known limitations: Windows x86-64 only, no code signing, no auto-update, and installation is
  per-user

## Step 10 — Verify the published release

```bash
gh release view v0.1.0
mkdir -p dist/verify && cd dist/verify
gh release download v0.1.0
sha256sum --check tray-goblin-0.1.0-windows-x86_64.zip.sha256
```

The check must report `OK` and the hash must match step 7. Delete `dist/verify` afterwards.

## Abort conditions

Stop and do not publish when any of these is true:

| Condition | Why it blocks the release |
|---|---|
| `git status --porcelain` printed anything at step 2 | The artifact would not correspond to the tag |
| Any command in step 3 or step 4 exited non-zero | The behaviour contract or documentation contract is unproven |
| The cross-build did not produce a PE32+ x86-64 executable | The archive would not run on the target platform |
| The two packing runs produced different hashes | Packaging is not reproducible, so the checksum proves nothing |
| The Windows checksum in step 8 differs from step 7 | The archive was altered in transit |
| The installer required elevation or wrote outside the user profile | A non-negotiable was violated |
| An interaction checklist item failed | The user-visible contract is broken |
| `measure-performance.ps1` failed or is outstanding | The build may be published only as a preview, not promoted to stable |
| The version in the tag, `Cargo.toml`, and the archive name disagree | The artifact identity is ambiguous |

## Rollback

Choose the earliest applicable case.

**Not yet pushed.** Nothing is public. Delete the local tag and artifacts, fix the problem, and
restart from step 1:

```bash
git tag -d v0.1.0
rm -rf dist
rm -f release-notes.md
```

**Tag pushed, release not created.** Delete the remote tag, then the local tag:

```bash
git push origin :refs/tags/v0.1.0
git tag -d v0.1.0
rm -f release-notes.md
```

**Release published, defect found.** Prefer withdrawing over editing in place, so no one keeps a
binary whose checksum no longer matches published notes:

```bash
gh release delete v0.1.0 --yes
git push origin :refs/tags/v0.1.0
git tag -d v0.1.0
rm -f release-notes.md
```

Then bump to the next patch version and restart from step 1. Never re-tag a different commit with
an already-published version, and never replace an uploaded asset with different bytes under the
same version — a downloaded archive would then fail its published checksum.

**Operator rollback.** An operator who installed a withdrawn build runs `uninstall.ps1`, then
installs the previous known-good archive. Configuration is preserved across both operations, so no
settings are lost.

## Related

- [Release responsibilities](agent/release-responsibilities.md)
- [Testing](agent/testing.md)
- [Development](agent/development.md)
- [Installation](operator/installation.md)
