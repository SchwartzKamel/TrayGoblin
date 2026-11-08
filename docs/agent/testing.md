# Testing

**Audience:** Agents

TrayGoblin has two clearly separated validation layers. Confusing them is the main way a broken
build reaches an operator.

| Layer | Runs on | Determinism | Gates |
|---|---|---|---|
| **Deterministic cross-validation** | Linux, WSL, or any host with the toolchain | Fully repeatable, no Windows, no network | Every change |
| **Manual Windows checks** | A real Windows 10/11 x86-64 machine | Manually observed, machine-dependent | Every release, and any change to `src/tray.rs`, the installer, or the icons |

A green deterministic run does **not** prove the tray works. It proves the parsing, monitoring,
decision, packaging, and cross-build contracts hold.

## Deterministic commands

| Command | Covers |
|---|---|
| `cargo test --lib` | Parser, session metadata, configuration, status, icon, action, and app-decision unit tests, including the privacy contract |
| `cargo test --test monitor` | Monitor integration over fixture session folders: session selection, offset caching, state transitions, degraded input |
| `cargo test --workspace` | All Rust unit/integration tests, including accepted aggregate monitoring/controller/workers; see evidence tiers below |
| `cargo clippy --all-targets -- -D warnings` | Lint contract on the host target |
| `bash scripts/validate-toolchain.sh` | Workspace tests plus the Windows cross-build, asserting a PE32+ x86-64 artifact |
| `bash scripts/package-release.sh 0.1.0` | Reproducible packaging, checksum generation, archive integrity, and expected entries |
| `bash scripts/check-docs.sh` | The documentation contract: required files, audience labels, links, required commands, stale status claims, shell syntax |
| `pwsh -NoProfile -File scripts/test-installer.ps1` | Installer and uninstaller behaviour in a sandbox profile, plus static safety checks on all PowerShell scripts |
| `bash scripts/demo.sh 0.1.0` | The historical MVP-only end-to-end gate: workspace tests, the Windows cross-build, both fixture journeys in tier order, reproducible packaging, and archive plus checksum verification, leaving the current unpublished package in `dist/`; its original assertions remain MVP-only, not an attention journey |

`scripts/test-installer.ps1` reports Windows-shortcut behaviour as **skipped** on a non-Windows
host. Skipped is not passed; those cases are covered by the manual checklist below.

## Attention checkpoint gates

M1–M5 are accepted at implementation tier and unpublished; recorded counts are
acceptance history, not a new field result. M6 real Windows/WSL/resource
evidence remains BLOCKED. The [active plan](../../.wgm/IMPLEMENTATION_PLAN.md) is
the sole authority for the current candidate, exact gate results, review/audit
verdicts and next acceptance action. This inventory does not imply release readiness:

| Milestone | Exact gate | Recorded disposition |
|---|---|---|
| M1 publisher and actual offline pi host | `node --test tests/pi-status.test.mjs tests/pi-host.test.mjs` | Accepted `0d79111`, 33/33; includes the real offline host, not Windows/WSL interaction |
| M2 aggregate monitor/config/probe | `cargo test --workspace` | Accepted `058ef74`, 93 tests; bounded pi reads/freshness, configured sources and independent Copilot sessions |
| M3 controller/workers | `cargo test --test attention_monitor --test attention_controller && cargo test --workspace` | Accepted `f4ef6de`, 52 targeted/136 workspace; rejected-candidate dissent and compiled regressions retained in plan/history |
| M4 native wiring/manual labels | Node host gate + workspace + Windows release cross-build | Accepted `d94c85c`, 36 Node/141 Rust, PE/import validation; NOT actual shell delivery |
| M5 continuous offline demo/package | `bash scripts/demo-attention.sh --container podman && bash scripts/demo.sh 0.1.0 && bash scripts/check-docs.sh` | Accepted at implementation tier; 14 actual-host phases/5 adapter requests, separate restricted OCI, 141 Rust, 56 installer/one explicit Windows shortcut skip; current candidate/gates/verdicts are in the active plan above |

The attention demo installs the actual ZIP into an external synthetic root,
removes extraction, then loads that persisted extension in TWO concurrent actual
installed pi1.4.0 processes sharing ONE synthetic cwd and distinct session/instance
IDs/manual labels. It reuses `scripts/test-pi-host.mjs` loader/runner/privacy traps,
real blocking UI wrapper and final settlement boundary. Both stay alive through
all observations; their actual atomic files traverse shipped Rust monitor/workers/
controller/AttentionRuntime. It asserts nonempty requests/submissions, manual
Idle input, other-session progress, quiet recovery/completion/abort, dedupe and
error→dialog→same error restoration. No model calls/user config/network.

OCI is SEPARATE: a freshly built host release probe runs in existing cached
Debian bookworm-slim through Podman with nonroot UID, network none, read-only
rootfs and only read-only executable/synthetic fixture mounts. Nonempty JSON,
identities/states/capabilities/allowlisted fields are asserted. No image pull,
host SDK/config mount or Windows/pi-host claim. If Podman/image is unavailable,
the gate fails explicitly rather than claiming OCI ran.

Set external `TRAY_GOBLIN_EVIDENCE_DIR` to retain journey.json, oci.json, exact
commands, observed adapter requests, UTC/timings and executable/source digests.
When isolating HOME, an already verified engine cache may be selected with
`TRAY_GOBLIN_PODMAN_ROOT` and matching `TRAY_GOBLIN_PODMAN_RUNROOT` (get actual
paths from `podman info` BEFORE isolation). No user config is mounted. `CARGO_HOME`
and `RUSTUP_HOME` may reference verified public compiler caches; clear ambient
`CARGO_TARGET_DIR` because this gate checks its own target paths.

`node scripts/test-release-docs.mjs` runs exact published release fences through
isolated failure seams with success controls, no-downstream assertions and a
swallowed-error canary, plus real isolated annotated tags. That is offline
control-flow evidence, NOT permission to tag/publish. Parent runs independent
holdout judging/audit; authors never open scenarios or invent satisfaction scores.
See the [pi integration guide](../../integrations/pi/README.md).

Optional CI test ZIPs cover synthetic Rust tests and Windows-target compilation/
packaging only. They do not run this full manual attention demo or prove real
pi-host/model, interactive Windows/WSL or performance behavior. Offline verifier
stubs do not prove a downloadable GitHub asset; CI2 owns that live boundary.
See [Development](development.md#optional-ci-test-builds).

## Optional release controls

REL1's exact local gate (actionlint/PyYAML are tooling only): first use the bounded
[release-tool provisioning/offline-check recipe](development.md#release-tool-prerequisites)
for Python 3.11+, GNU coreutils/timeout, gh and those tools; provisioning needs
separate operator authorization, never installs/downloads in this gate.

```bash
actionlint .github/workflows/release.yml && python3 scripts/test-release-workflow.py && python3 scripts/publish-release.py --self-test && python3 scripts/test-ci-workflow.py && bash scripts/check-docs.sh && bash scripts/validate-toolchain.sh
```

Use synthetic fixtures/isolated HOME and verified public compiler caches. The
self-test runs controlled gh/Git stubs with real ZIP/PE fixture bytes, mandatory
case removal, contradictory field/performance reports, protected published/draft
mutations and interrupted second-upload/retry controls. Fake positive JSON checks
schema only: it is NEVER live publication evidence. A 2-second offline deadline
must fail within 7 seconds, retain owned partial/draft evidence and caller sentinel;
retry uses a NEW child without clobbering unknown files. Publisher deadlines remain
120 discovery, 1500 watch, 120 download/upload, kill-after 5 seconds. The read-only
Release verifier polls completed successful candidates across all bounded pages
for 120 seconds; it never watches an unrelated in-flight dispatch.

REL2's separate real, read-only owner download boundary (run only after authorized
parent push/manual annotated tag/source build/DRAFT dispatch):

```bash
bash scripts/verify-release.sh --repo SchwartzKamel/TrayGoblin --tag v0.1.0 --draft
```

This discovers an actual successful release.yml run, actual draft Release ID/URL
and both permanent assets, verifies tag/source/manifest/17 entries/GUI PE/digests,
and retains sanitized evidence in a new child of `.wgm/release-downloads`.
It is not an Actions artifact link and does not dispatch/create/publish/tag.
On failure check syntax/help first: pre-allocation argument errors may create NO
owned directory. Inspect a printed owned path only if allocation occurred. Local
verifier children are durable at the chosen out-root; hosted publisher runner.temp
children are ephemeral and NOT exported by the workflow. Preserve accessible
Actions logs/exits and read-only run/draft/asset state before a matching retry;
see [Recovery](../manual-release.md#optional-manual-releases-ci).
M6 remains blocked: actual Windows/WSL field AND same-EXE performance collection
below are required for BOTH public modes. A separate
[complete acceptance record](../manual-release.md#complete-public-acceptance-record)
is required before any public-mode Release mutation and rechecked before publish:
actual manual clean-tagged canonical validation/build and CI byte equivalence,
all 11 dated checklist items on BOTH Windows 10/11 x64 targets, distinct fresh
independent M6 review coverage and explicit operator acceptance. Negative controls
remove each checklist item on each target, fail installer/Startup/uninstall,
validation/reviews/bindings/digests, change evidence before publish and manual ZIP
bytes, and falsify unsupported/mismatched OS/host provenance in both modes.
Separate verifier controls reject copied logs from non-main/wrong SHA/ID/workflow
and prove unrelated-running-first and paginated matching success without watch.
See [collection and placement](../manual-release.md#optional-manual-releases-ci).

## Fixture journeys (historical MVP contract)

These six-field probe expectations remain historical MVP compatibility checks,
not current aggregate GUI expectations. The fixtures are content-free by
construction — no fixture contains prompt or response text.

```bash
cargo run --bin tray-goblin-probe -- --session-root tests/fixtures/live-session
```

Expected: `"state": "working"` with a model, repository, and active directory. `session-old` has no
lock marker and must be ignored; `selects_the_newest_active_session` separately proves that the
most recently modified lock wins when several sessions are active.

```bash
cargo run --bin tray-goblin-probe -- --session-root tests/fixtures/degraded-session
```

Expected: `"state": "attention_needed"` with `"attention_reason": "tool_failed"` and a recorded
`last_turn_duration_ms`, despite the fixture containing unknown event types and unknown fields.
Malformed-line tolerance is covered by the parser fixture test. Together they are the future-format
compatibility proof.

`scripts/demo.sh`, owned by task TZ in [`IMPLEMENTATION_PLAN.md`](../../IMPLEMENTATION_PLAN.md),
runs both journeys in that tier order and asserts every field above. It also asserts that the probe
emits exactly its allow-listed fields and that no snapshot echoes the `SENSITIVE_SENTINEL` value
planted in the degraded fixture, so removing that sentinel fails the demo rather than weakening it
silently. Run the probe commands above directly when you want to inspect a single journey.

## What each contract test protects

| Test | Contract |
|---|---|
| `events::tests::does_not_model_sensitive_fields` | No sensitive field is ever modeled |
| `events::tests::unknown_future_event_is_non_fatal` | A Copilot format change degrades safely |
| `turn_start_sets_generating` | Turn start reaches **Working** |
| `turn_end_sets_idle_and_duration` | Turn end reaches **Idle** with a duration |
| `failed_tool_sets_attention_needed` | A failed tool execution reaches **Attention needed** |
| `attention_needed_persists_until_next_turn_start` | Failures are not missed while away from the machine |
| `missing_events_file_sets_state_unavailable` | Unreadable state is a safe status, not a crash |
| `app::tests::manual_refresh_requests_poll` | **Refresh now** polls immediately |
| `actions::tests` | Menu targets resolve, and failures stay content-free |

The authoritative criterion-to-command mapping lives in
[`specs/tray-status.md`](../../specs/tray-status.md).

## Manual Windows checklist

Run on Windows 10 and Windows 11, x86-64, from the packaged archive rather than a developer build.
This checklist is the acceptance evidence for step 8 of
[`manual-release.md`](../manual-release.md#step-8-operator-verification-on-windows). Give it to the
Operator performing Windows verification, then collect their dated results, Windows build number,
and performance report before publication.

1. **Install without elevation** — `install.ps1` completes from a standard account, files land
   below `%LOCALAPPDATA%\Programs\TrayGoblin`, and a Startup shortcut is created.
2. **Icon registration** — the icon appears; restart Explorer and confirm it comes back by itself.
3. **State legibility** — Idle, Working, and Attention needed are distinguishable by shape as well
   as color, including in a high-contrast theme.
4. **Magic moment** — open a blocking pi dialog (including an idle/manual dialog)
   while another session works. Within configured 2P+500ms of delivered hooks,
   attention identifies that session; answering clears the overlay within the
   same bound. Completion/abort/recovered tool failures stay quiet. A new persisted
   Copilot session error alerts; ordinary Copilot completion does not.
5. **Tooltip** — aggregate state/counts and attention identity agree with the
   persistent list, including two same-directory sessions. Model, directory and
   **Run start (UNIX ms)** timestamp and last-run duration (**Last run (ms)**)
   belong in each persistent session submenu, not legacy detail-tooltip or
   `Last turn:` expectations. Check bounded text in the shell.
6. **Menu** — attention-first persistent session submenus retain provider/source,
   session/instance/manual-label identity and metadata even when Windows suppresses
   notifications. **Refresh now**, **Open in VS Code**, **View Copilot logs**,
   **Open settings**, and **Quit** follow [First run](../operator/first-run.md).
7. **Configuration errors** — an out-of-range `pollIntervalMs` produces the content-free startup
   dialog instead of a silent clamp.
8. **Startup** — sign out and back in; TrayGoblin starts automatically.
9. **Uninstall** — `uninstall.ps1` removes files and the shortcut and keeps configuration;
   `-RemoveConfiguration` removes `config.json`.
10. **Attention field journey** — run the genuinely INTERACTIVE collector below
    against the packaged executable. It records all native+WSL dialog kinds/final
    errors, Copilot persisted errors, quiet completion/abort/recovery, identity,
    deadlines (2P+500ms), suppression/list, reload/crash staleness (10s+2P), Explorer
    no replay, WSL shutdown/native isolation/actual UNC wake and recovery, removal/
    restart/no reads and Quit. Prepare actual Windows file-access tracing for the
    no-reads case. Conservative stopwatch timings start BEFORE each trigger and
    stop AFTER observation; operator time is included, never subtracted/invented.
11. **Performance** — see below, same PE digest as the field report.

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pwsh -NoProfile -File "$tg\resources\test-attention-windows.ps1" -ExecutablePath "$tg\tray-goblin.exe" -EvidencePath .\windows-attention.json -PollIntervalMs 1000
if ($LASTEXITCODE -ne 0) { throw 'Actual attention field gate failed' }
```

`-PollIntervalMs` declares the EXPECTED cadence, not a setting: match the actual
running tray's `pollIntervalMs` or `poll_interval_ms` (omitted/null means 1000ms).
The collector rejects mismatch; neither it nor `-ConfigurationPath` changes tray
cadence. `-ConfigurationPath` selects the actual running config when it is not at
`%APPDATA%\TrayGoblin\config.json`; it is not a GUI config flag. For example,
ONLY if the running tray already uses this file with a 2000ms cadence:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pwsh -NoProfile -File "$tg\resources\test-attention-windows.ps1" -ExecutablePath "$tg\tray-goblin.exe" -EvidencePath .\windows-attention.json -PollIntervalMs 2000 -ConfigurationPath 'D:\TrayGoblinProfile\TrayGoblin\config.json'
if ($LASTEXITCODE -ne 0) { throw 'Actual attention field gate failed' }
```

Change the tray's actual config and restart it before collecting a new cadence.
Use P=that cadence for attention/clear (2P+500ms) and quiet checks (3P). Historical
MVP GUI acceptance used a two-second transition and `Last turn:` detail tooltip;
those are history, not current aggregate acceptance criteria.

Both collectors ship in the ZIP and use resources/build.json for build SHA/dirty
flag/executable SHA256. A checkout invocation without that manifest must supply
actual `-BuildSha`; it is not guessed from an unrelated checkout. Field results
record PID/path, UTC, Windows+WSL versions, source kinds/load and genuine per-case
outcomes/timings. The script does not autofill pass from capability answers.

## Performance measurement

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pwsh -NoProfile -File "$tg\resources\measure-performance.ps1" -Path "$tg\tray-goblin.exe" -DurationSeconds 30 -JsonPath perf.json
if ($LASTEXITCODE -ne 0) { throw 'Actual performance gate failed' }
```

For public acceptance, run this performance collector BEFORE the field collector
on the SAME initial PID/path/processStartUtc: the field journey restarts/quits the
process. Match full NT major-10 version/build to field Windows metadata; Windows
7 or different build/process provenance is rejected. Preserve the current rounded
presentation metrics AND unrounded-comparison pass flags (the collector does not
emit raw byte/CPU counters). Record this measurement plus the all-11 checklist on
both supported Windows targets in the separate acceptance record above.

The script samples the running process for the window, fails when peak working set exceeds 50 MB or
average CPU exceeds 5% of total capacity, and writes a content-free JSON report. Run it on an
otherwise idle machine; a single ambiguous run with several `tray-goblin` processes is rejected
rather than averaged.

**This measurement must pass before promoting a preview build to stable.** Results from a
non-Windows host are REJECTED, with no field report. `-Path` binds the exact
measured PID/canonical path/executable SHA256; mismatched existing processes are
rejected, not silently measured instead. Match this digest to the field report.
M6 remains blocked until genuine Windows evidence is supplied.

## Adding tests

- Every state transition and parser boundary needs deterministic fixture coverage.
- Fixtures must stay content-free: use synthetic model names, repository names, and directories.
- Each test comment should say which transition or contract it protects, matching the style already
  used in `src/`.

## Related

- [Development](development.md)
- [Release responsibilities](release-responsibilities.md)
- [Manual release](../manual-release.md)
- [Troubleshooting](../operator/troubleshooting.md) for the operator-facing version of the probe
  workflow
