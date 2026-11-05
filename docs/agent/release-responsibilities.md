# Release responsibilities

**Audience:** Agents

Who does what for a TrayGoblin release. The step-by-step commands are in
[Manual release](../manual-release.md); this page is the ownership split and the gate rules.

## Split of duties

| Stage | Owner | Output |
|---|---|---|
| Version freeze in `Cargo.toml` and docs | Agent | One commit naming the release version |
| Clean-tree proof (`git status --porcelain` empty) | Agent | An artifact that corresponds to the tag |
| Deterministic validation | Agent | `cargo test --workspace`, clippy, `bash scripts/validate-toolchain.sh`, `bash scripts/check-docs.sh` all exit zero |
| Installer verification | Agent | `pwsh -NoProfile -File scripts/test-installer.ps1` exits zero, with Windows-shortcut checks reported as skipped off Windows |
| Annotated tag | Agent | `v<version>` pointing at the validated commit |
| Reproducible package and checksum | Agent | `dist/tray-goblin-<version>-windows-x86_64.zip` plus `.sha256` |
| Checksum verification on Windows | Operator | `Get-FileHash` matches the published hash |
| Install, interaction, and startup checks | Operator | Signed-off manual checklist |
| Performance measurement | Operator | `scripts/measure-performance.ps1` passes on Windows 10/11 |
| GitHub release entry and asset upload | Agent | Prerelease with notes, archive, and checksum |
| Post-publish download verification | Agent | `sha256sum --check` reports `OK` |
| Withdrawal or rollback decision | Agent, on operator evidence | Deleted release and tag, or a new patch version |

## Gates

Version0.1.0 is unpublished. The active attention plan (`.wgm/IMPLEMENTATION_PLAN.md`),
not the historical root MVP completion record, is the sole authority for the current
candidate, exact gate results, review/audit verdicts and next acceptance action.
M1–M5 are accepted at implementation/compilation tier and remain unpublished;
M6 actual Windows/WSL and resource evidence is BLOCKED. Optional CI/pushing code
is user-approved, but the parent owns any push after offline gates and independent
review. No CI1 author is authorized to stage/commit/push/tag/publish.
A release advances only when the preceding gate is green.

1. **Behaviour gate** — the deterministic suite and the cross-build pass. Nothing is tagged before
   this.
2. **Reproducibility gate** — packaging produces the same archive hash twice; the checksum file
   verifies. A non-reproducible package is never published.
3. **Windows interaction gate** — the operator's manual checklist passes on real Windows. Preview
   publication requires this.
4. **Performance gate** — `scripts/measure-performance.ps1` passes on Windows 10 or 11 within the
   50 MB working-set and 5% CPU budgets. **A preview build must not be promoted to stable until
   this measurement passes.** For the attention follow-up, M6 field AND resource
   evidence must pass before ANY publication is considered; the historical MVP
   preview exception does not waive this blocker.

## What is deterministic and what is not

| Provable on Linux or WSL | Only provable on Windows |
|---|---|
| Parsing, monitoring, state transitions, and privacy contracts | Notification-area icon registration and appearance |
| Configuration range enforcement and error text | Tooltip rendering in the real shell |
| Windows x86-64 cross-build producing a PE32+ artifact | Menu commands launching VS Code, Explorer, and the settings file |
| Byte-reproducible packaging and checksums | Startup shortcut behaviour after sign-in |
| Installer and uninstaller logic in a sandbox profile | Startup shortcut creation via Windows Script Host |
| Documentation contract via `bash scripts/check-docs.sh` | Working-set and CPU budgets |

Never describe a manual Windows result as if the deterministic suite had proven it, and never treat
a skipped check as a passed check.

## Optional CI, never a release prerequisite

The read-only Windows test workflow offers ZIP/checksum artifacts on successful
main/manual runs, retained 14 days. Downloads require a signed-in GitHub account
and live under **Actions**, not **Releases**. It runs Rust tests and Windows-target
compilation/package only: no signing, release creation, tag writes or deployment;
no real pi-host/model, Windows/WSL interaction or performance claim. CI2 owns actual
run/download verification; offline controls are not proof of GitHub availability.
See [Development](development.md#optional-ci-test-builds) and
[Installation](../operator/installation.md#1-obtain-the-archive).

Local validation, the full manual attention demo and clean-tagged release remain
authoritative and independent. Adding a hosted pipeline as a *release prerequisite*
would violate [`specs/CONSTITUTION.md`](../../specs/CONSTITUTION.md). M6 remains required.

## Release notes must state

- The tag and the commit SHA it points at
- The archive SHA-256
- Windows+WSL versions, real per-case observations/timings and working-set/CPU
  numbers for the SAME executable SHA256/build identity/PID/path; no inferred results
- Known limitations: Windows x86-64 only, per-user install, unsigned, no auto-update,
  Copilot input unavailable, UNC may wake WSL, notifications may be suppressed

## Withdrawal

If a defect is found after publication, withdraw rather than replace: delete the release and its
tag, then release a new patch version. Never re-tag a published version and never replace an
uploaded asset with different bytes under the same version, because a previously downloaded archive
would then fail its published checksum. The exact commands are in
[Manual release](../manual-release.md).

## Related

- [Manual release](../manual-release.md)
- [Testing](testing.md)
- [Installation](../operator/installation.md)
