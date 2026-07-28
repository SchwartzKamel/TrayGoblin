# Configuration

**Audience:** Operators

TrayGoblin is configured by one small JSON file. There is no settings window: the tray's
**Open settings** command creates the file with documented defaults when it is missing and then
opens it in your registered editor.

## Location

```text
%APPDATA%\TrayGoblin\config.json
```

On a non-Windows host used for development, the same resolver falls back to
`$HOME/.config/tray-goblin/config.json`.

The file is optional. When it does not exist, TrayGoblin uses the defaults below.

## Default file

```json
{
  "pollIntervalMs": 1000,
  "sessionRoot": null
}
```

## Keys

| Key | Type | Default | Meaning |
|---|---|---|---|
| `pollIntervalMs` | number | `1000` | How often Copilot session state is polled, in milliseconds. Accepted range is 500–10,000. |
| `sessionRoot` | string or `null` | `null` | Overrides the Copilot session folder. When `null`, `%USERPROFILE%\.copilot\session-state` is used. |

`poll_interval_ms` and `session_root` are accepted as alternative spellings of the same two keys.
Any other key is ignored, so a configuration file written by a future version stays usable.

## Examples

Poll twice per second for a faster tray reaction, at a slightly higher wake-up rate:

```json
{
  "pollIntervalMs": 500
}
```

Poll every five seconds on a battery-conscious machine:

```json
{
  "pollIntervalMs": 5000
}
```

Watch a session folder that is not below your user profile:

```json
{
  "pollIntervalMs": 1000,
  "sessionRoot": "D:/copilot/session-state"
}
```

Use forward slashes, or escape backslashes as `\\`, so the value stays valid JSON.

## Rules that are enforced

- **The interval is never silently clamped.** A value outside 500–10,000 is rejected, and
  TrayGoblin shows the startup error `the settings file is invalid; use 500-10000 for
  pollIntervalMs` instead of starting with a surprising cadence.
- **Invalid JSON is an explicit failure**, reported the same way, without echoing file contents.
- **An unreadable file is an explicit failure.** A file that simply does not exist is not a
  failure — it means defaults.
- **Configuration is read once, at startup.** After editing `config.json`, choose **Quit** from
  the tray menu and start TrayGoblin again by running
  `%LOCALAPPDATA%\Programs\TrayGoblin\tray-goblin.exe`, by opening the `TrayGoblin.lnk` shortcut
  in your Startup folder, or by signing out and back in. **Refresh now** re-polls with the current
  settings; it does not reload the file.

## Choosing an interval

| Interval | Effect |
|---|---|
| 500 ms | Fastest supported reaction, roughly one second worst case. Highest wake-up rate. |
| 1,000 ms (default) | Meets the two-second state-update expectation with a minimal footprint. |
| 2,000–10,000 ms | Fewer wake-ups; state changes may take up to twice the interval to appear. |

Polling only reads bytes appended since the previous poll, so a larger interval saves wake-ups
rather than avoiding re-reads.

## Configuration and uninstall

`uninstall.ps1` keeps `%APPDATA%\TrayGoblin` by default. Pass `-RemoveConfiguration` to delete
`config.json`; see [Installation](installation.md).

## Related

- [Status reference](status-reference.md) for what the polling interval changes
- [Troubleshooting](troubleshooting.md) if TrayGoblin refuses to start after an edit
- [Privacy](privacy.md) for the fields that configuration can and cannot expose
