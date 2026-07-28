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
| `cargo test --workspace` | Both of the above in one run |
| `cargo clippy --all-targets -- -D warnings` | Lint contract on the host target |
| `bash scripts/validate-toolchain.sh` | Workspace tests plus the Windows cross-build, asserting a PE32+ x86-64 artifact |
| `bash scripts/package-release.sh 0.1.0` | Reproducible packaging, checksum generation, archive integrity, and expected entries |
| `bash scripts/check-docs.sh` | The documentation contract: required files, audience labels, links, required commands, stale status claims, shell syntax |
| `pwsh -NoProfile -File scripts/test-installer.ps1` | Installer and uninstaller behaviour in a sandbox profile, plus static safety checks on all PowerShell scripts |
| `bash scripts/demo.sh 0.1.0` | The end-to-end gate: workspace tests, the Windows cross-build, both fixture journeys in tier order, reproducible packaging, and archive plus checksum verification, leaving the release candidate in `dist/` |

`scripts/test-installer.ps1` reports Windows-shortcut behaviour as **skipped** on a non-Windows
host. Skipped is not passed; those cases are covered by the manual checklist below.

## Fixture journeys

The fixtures are content-free by construction — no fixture contains prompt or response text.

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
4. **Magic moment** — send a Copilot prompt: **Working** within two seconds; on completion,
   **Idle** with a `Last turn:` line.
5. **Tooltip** — model, repository or directory, and duration lines render correctly and are not
   silently cut by the shell.
6. **Menu** — **Refresh now**, **Open in VS Code**, **View Copilot logs**, **Open settings**, and
   **Quit** all behave as documented in [First run](../operator/first-run.md).
7. **Configuration errors** — an out-of-range `pollIntervalMs` produces the content-free startup
   dialog instead of a silent clamp.
8. **Startup** — sign out and back in; TrayGoblin starts automatically.
9. **Uninstall** — `uninstall.ps1` removes files and the shortcut and keeps configuration;
   `-RemoveConfiguration` removes `config.json`.
10. **Performance** — see below.

## Performance measurement

```powershell
pwsh -NoProfile -File scripts/measure-performance.ps1 -DurationSeconds 30 -JsonPath perf.json
```

The script samples the running process for the window, fails when peak working set exceeds 50 MB or
average CPU exceeds 5% of total capacity, and writes a content-free JSON report. Run it on an
otherwise idle machine; a single ambiguous run with several `tray-goblin` processes is rejected
rather than averaged.

**This measurement must pass before promoting a preview build to stable.** Results from a
non-Windows host are indicative only and never satisfy the gate.

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
