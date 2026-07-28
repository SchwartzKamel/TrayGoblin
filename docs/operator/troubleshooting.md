# Troubleshooting

**Audience:** Operators

Symptom-first fixes. Every diagnostic below is content-free: nothing here asks you to share prompt
or response data, and TrayGoblin itself cannot produce it.

## No icon appears after installing

1. Confirm the executable exists at `%LOCALAPPDATA%\Programs\TrayGoblin\tray-goblin.exe`.
2. Check whether the icon is hidden in the notification-area overflow. Windows hides new icons by
   default: open **Taskbar settings → Other system tray icons** and enable TrayGoblin.
3. If Windows Explorer was restarting when TrayGoblin started, the icon re-registers by itself on
   the next timer tick; wait a few seconds.
4. Start the executable manually and watch for a startup dialog.

## A dialog says TrayGoblin could not start

The dialog names the cause without exposing file contents:

| Message | Cause | Fix |
|---|---|---|
| `the settings file is invalid; use 500-10000 for pollIntervalMs` | `config.json` has an out-of-range interval or invalid JSON | Correct or delete `%APPDATA%\TrayGoblin\config.json`; see [Configuration](configuration.md) |
| `the settings file could not be read` | The configuration file exists but cannot be opened | Close any editor holding it, check permissions, or delete the file to fall back to defaults |
| `no Copilot session folder is configured and none could be determined` | Neither `sessionRoot` nor `%USERPROFILE%` could be resolved | Set `sessionRoot` explicitly in `config.json` |

## The state never leaves Idle while Copilot is working

1. Confirm Copilot CLI is writing state: `%USERPROFILE%\.copilot\session-state` should contain a
   session folder with an `inuse.*.lock` marker and an `events.jsonl` file.
2. If you set `sessionRoot`, confirm it points at the folder that actually holds those session
   folders, not at one session folder.
3. If several Copilot sessions are open, remember that only the session with the most recently
   modified lock marker is observed.
4. Choose **Refresh now**; if that changes nothing, the observed session has no open turn.

## The state is stuck on Working

TrayGoblin reports Working while a turn has started and not ended. If the Copilot process was
killed mid-turn, the session file may never receive its `assistant.turn_end` event. Starting a new
turn, or a new Copilot session, clears the state.

## Attention needed will not clear

Attention needed persists deliberately until the next turn starts, so a failure is not missed.

- `Reason: a tool execution failed in the last turn` — check the Copilot CLI terminal for the
  failure and start another turn.
- `Reason: Copilot session state is unreadable` — the selected session's `events.jsonl` cannot be
  read. Confirm the file still exists and is readable, and that no backup or antivirus tool is
  locking it.

## The tooltip shows no model, repository, or last-turn duration

Each line appears only when the value is known.

- The model line appears after the session reports a model.
- The repository line requires repository metadata in the session's `workspace.yaml`; otherwise a
  `Dir:` line is shown instead.
- The last-turn duration appears after the first turn in the observed session completes, and it
  requires timestamps on both the start and end events.

## A menu command does nothing, or reports it is unavailable

| Message | Meaning |
|---|---|
| `Open in VS Code is unavailable: no active Copilot directory is known` | No observed session has reported a directory yet |
| `View Copilot logs is unavailable: no session folder is configured` | No session folder could be resolved |
| `Open settings is unavailable: no settings location could be determined` | `%APPDATA%` could not be resolved |
| `Open settings failed: the settings file is not writable` | The configuration folder or file is not writable |
| `<command> could not be started` | Windows refused to launch the target; for VS Code, confirm `code` is installed and on `PATH` |

## After a Copilot CLI update, the tray stopped reacting

Copilot's session format is internal and may change. TrayGoblin is built so that unknown event
types, unknown fields, and malformed lines are never fatal — the tray keeps running and simply
stops learning from records it does not recognize. That is why a format change shows up as a state
that no longer moves rather than a crash.

The commands below come from a repository checkout with stable Rust installed; they are not part of
the installed portable payload. To confirm and report a format change:

1. Run the diagnostic probe against your own session folder:

   ```bash
   cargo run --bin tray-goblin-probe -- --session-root "C:/Users/<you>/.copilot/session-state"
   ```

   The probe prints one content-free JSON snapshot. If it reports `idle` while Copilot is clearly
   mid-turn, the event vocabulary has probably changed.
2. Compare against a known-good fixture, which must always report `working`:

   ```bash
   cargo run --bin tray-goblin-probe -- --session-root tests/fixtures/live-session
   ```

3. Open an issue that includes only: your Copilot CLI version, your TrayGoblin version, the probe
   output above, and the *event type names* you see in `events.jsonl`. Do not attach the file
   itself — it contains prompt and response content that TrayGoblin never reads.

The event types TrayGoblin currently recognizes are `assistant.turn_start`, `assistant.turn_end`,
`session.model_change`, `assistant.message`, and `tool.execution_complete`.

## TrayGoblin uses more memory or CPU than expected

The budget is under 50 MB working set and under 5% CPU while idle. From a repository checkout on
Windows, an operator can verify it directly against the running process:

```powershell
pwsh -NoProfile -File scripts/measure-performance.ps1 -DurationSeconds 30
```

The script samples the running process and exits non-zero when either budget is exceeded. A build
that fails this measurement must not be promoted from preview to stable; see
[Release responsibilities](../agent/release-responsibilities.md).

## Removing everything

See the uninstall section of [Installation](installation.md). Configuration is preserved unless you
pass `-RemoveConfiguration`.

## Related

- [Status reference](status-reference.md)
- [Configuration](configuration.md)
- [Privacy](privacy.md)
