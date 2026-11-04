# Troubleshooting

**Audience:** Operators

Use synthetic activity and content-free metadata only. Never share conversations,
events.jsonl contents, prompts/results/arguments/tokens or credentials.

## No icon or startup failure

Confirm `%LOCALAPPDATA%\Programs\TrayGoblin\tray-goblin.exe` exists. Check Windows
notification-area overflow/Taskbar settings. Start the installed executable once;
a second instance in the same interactive session exits quietly. Explorer restart
re-adds the icon without resetting the controller or replaying acknowledged alerts.

Startup errors are fixed safe messages: invalid config (JSON, sources or interval),
unreadable config, unresolved native home/settings root, or worker initialization
failure. Fix the setting/access and restart. Missing config uses defaults;
`pollIntervalMs` range is 500–10000. **Open settings** creates defaults only when
missing; **Refresh now** never reloads configuration.

## Pi never appears

Explicitly opt in using the INSTALLED package, then reload/restart pi:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pi install "$tg\integrations\pi"
```

The installer never edits pi settings. Do not register an extracted folder you
later delete. See [Installation](installation.md), [First run](first-run.md) and
installed `resources/SETUP.md` for durable WSL copy/opt-in.

Pi writes in the HOME of the process running pi, default `~/.traygoblin/pi`.
`TRAY_GOBLIN_STATUS_DIR` changes that producer root; the Windows consumer must
point to the SAME folder. Assign manual `TRAY_GOBLIN_LABEL` values in separate
terminals; same-directory sessions still have distinct session/instance IDs.

## Unavailable source, stale clock or tracking fault

| Observation | Recovery |
|---|---|
| `source_unavailable` / `session_unavailable` | Correct path/access, recover source; independent native sources should keep progressing |
| Stale pi status | Heartbeat is every 2s; older than 10s is Unavailable, never inferred waiting. Recover/reload actual publisher |
| Future/regressed timestamp | Fix producer/consumer clocks, reload publisher; episode IDs are NOT freshness counters |
| Invalid/oversized status | Use shipped pi1.4-compatible publisher, regular UUID files, max16KiB; ignore .tmp, never hand-edit publication |
| `ui_tracking_unavailable` | Ambiguous ID-less UI hook ordering; one latched integration error until pi reload. Investigate delayed hooks in other extensions if recurring; not model failure/input inference |
| Session/delivery overflow | Reduce configured load/churn and recover complete source access. Visible diagnostics are not a guarantee of notifications beyond bounded capacity |

Only configured roots are read. UNC access **may wake or keep WSL running**;
there is no distro discovery or atomic no-wake promise. To stop WSL reads, remove
that source, **Quit** and restart TrayGoblin. A blocked source cannot freeze the
UI/other sources; OS process exit is the cancellation boundary for blocked reads.

## Attention is missing, repeated or will not clear

- Pi alerts for ALL explicit blocking dialogs, including idle/manual/settings/
  compaction, and FINAL settled run errors. Resolve/reject restores prior state,
  including the SAME error ID without replay. New work clears a run error.
- Copilot alerts only for NEW persisted `session.error`. Historical errors stay
  visible without replay; tool failures alone are quiet. New `assistant.turn_start`
  clears a session error. Passive input detection is **unavailable**, not a bug
  that can be fixed with a longer timeout.
- Idle, successful completion, abort and recovered tool failures are quiet.
- Windows/Focus Assist may suppress a notification after successful shell
  submission. Right-click for persistent attention-first sessions/counts.
- A failed submission is not acknowledged: at most3 attempts at least5s apart.
  **Refresh now** rearms exhausted **undelivered** episodes only (still respects 5s).
  Acknowledged episodes are never rearmed by Refresh or Explorer re-registration.

See [Status reference](status-reference.md) for precise policy and bounds.

## Editor/log commands are unavailable

Unmapped POSIX/WSL working directories are intentionally not translated to
Windows editor commands. Use the actual terminal; there is no terminal focus or
control. **View Copilot logs** targets Copilot only, never pi conversations.
**Open settings** requires a writable settings path. Failed actions use fixed
safe errors, not private paths or OS exception text.

## Diagnostic probe (checkout-only)

The ZIP ships the GUI, not the probe or compiler. Build a checkout to diagnose
with `tray-goblin-probe`; configuration selection is ALWAYS explicit:

```bash
cargo run --bin tray-goblin-probe -- --config /path/to/synthetic-config.json --all-sessions
```

See [Configuration](configuration.md#explicit-aggregate-probe-example) for an
executable synthetic example, actual output and invalid/conflicting inputs.
Without `--all-sessions`, the legacy MVP probe still emits EXACTLY six fields:

```bash
cargo run --bin tray-goblin-probe -- --session-root tests/fixtures/live-session
```

That fixture must report `working`. The legacy path intentionally reports a
failed tool as `attention_needed`; do not confuse that preserved MVP contract
with the aggregate persisted-error-only alert policy. Recognized aggregate
vocabulary includes `assistant.turn_start`, `assistant.turn_end`,
`session.model_change`, `assistant.message`, `tool.execution_complete`, and
`session.error`; unknown fields/events are ignored safely. Probe config I/O errors
may contain OS text, unlike fixed GUI errors. Review metadata before sharing it.

## Performance and release verification

Use the actual installed collectors and matching build identity:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pwsh -NoProfile -File "$tg\resources\test-attention-windows.ps1" -ExecutablePath "$tg\tray-goblin.exe" -EvidencePath .\windows-attention.json -PollIntervalMs 1000
if ($LASTEXITCODE -ne 0) { throw 'Actual attention field gate failed' }
pwsh -NoProfile -File "$tg\resources\measure-performance.ps1" -Path "$tg\tray-goblin.exe" -DurationSeconds 30 -JsonPath .\windows-performance.json
if ($LASTEXITCODE -ne 0) { throw 'Actual performance gate failed' }
```

`-PollIntervalMs` is the EXPECTED cadence: match the running config's
`pollIntervalMs` or `poll_interval_ms` (omitted/null defaults to 1000ms).
For a nondefault config location, select the actual running file too, for example
ONLY if that tray already runs at 2000ms from this file:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pwsh -NoProfile -File "$tg\resources\test-attention-windows.ps1" -ExecutablePath "$tg\tray-goblin.exe" -EvidencePath .\windows-attention.json -PollIntervalMs 2000 -ConfigurationPath 'D:\TrayGoblinProfile\TrayGoblin\config.json'
if ($LASTEXITCODE -ne 0) { throw 'Actual attention field gate failed' }
```

Neither parameter changes tray cadence or launches it with another config; there
is no implied GUI config flag. Edit the actual config and restart the tray first.
Attention/clear uses 2P+500ms, quiet checks use 3P. Run each whole collector fence;
stop on any native failure before launching downstream collection.

Field collection is INTERACTIVE with genuine observations/timings; both scripts
reject Linux. Performance Path binds the exact measured PID/path/hash and rejects
a mismatched process. Match digest to the field report. 50MB/5% is a Windows gate,
not a Linux estimate; this measurement must pass before promoting a preview build
to stable. M6 is still blocked here. See [Testing](../agent/testing.md#manual-windows-checklist).

## Removing everything

First `pi remove` the SAME persistent package path and reload/quit pi. Then run the
installed uninstaller; configuration is kept unless `-RemoveConfiguration` is
explicit. The separate WSL integration copy is yours to remove after opt-out.
See [Installation](installation.md) and [Privacy](privacy.md).
