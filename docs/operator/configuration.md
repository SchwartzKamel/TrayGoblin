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
| `sources` | array | omitted | Replaces the aggregate attention sources; see below. Currently used by the aggregate probe/core, not the legacy tray. |

`poll_interval_ms` and `session_root` are accepted as alternative spellings of the same two keys.
Any other key is ignored, so a configuration file written by a future version stays usable.

## Aggregate attention sources (probe/core)

The aggregate path is available through `tray-goblin-probe --all-sessions`.
Native tray integration is a subsequent implementation task; the existing GUI
and default probe retain their original single-session behavior for now.
Without `--all-sessions`, the probe still emits exactly its original six fields.

Missing/legacy configuration supplies two aggregate sources: native Copilot at
`%USERPROFILE%\.copilot\session-state` and pi at
`%USERPROFILE%\.traygoblin\pi` (`$HOME` on development hosts). `sessionRoot`
overrides only the implicit Copilot source. A missing implicit pi directory is
normal when its opt-in publisher is not installed. An explicit `sources` array
replaces **both** defaults; `sources: []` disables aggregate monitoring.

```json
{
  "pollIntervalMs": 1000,
  "sources": [
    {
      "id": "windows-copilot",
      "label": "Windows Copilot",
      "kind": "copilot",
      "path": "C:/Users/operator/.copilot/session-state"
    },
    {
      "id": "ubuntu-pi",
      "label": "Ubuntu pi",
      "kind": "pi",
      "path": "\\\\wsl.localhost\\Ubuntu\\home\\operator\\.traygoblin\\pi",
      "wsl_distro": "Ubuntu"
    }
  ]
}
```

Sources require unique nonempty IDs and labels (at most 128 UTF-8 bytes), a
nonempty path (at most 4096 bytes), and kind `pi` or `copilot`. Optional
`wsl_distro` is a nonempty attribution label (128 bytes), **not** discovery or a
launch command. Control characters are rejected. At most 16 sources are allowed.
Unknown kinds, duplicate IDs and invalid bounds fail configuration explicitly.
`--session-root` may override the implicit Copilot root with `--all-sessions`;
combining it with explicit `sources` is an error. The legacy probe still uses
`sessionRoot`/`--session-root` independently of aggregate source selection.

Only configured roots are polled; no WSL distro or arbitrary home discovery is
performed. Accessing a configured UNC path **may start or keep WSL running**.
Remove that source and restart the application to stop those reads. Source
configuration is immutable until restart.

Aggregate JSON contains `sources`, each with source identity/configuration,
`capabilities`, `sessions`, and fixed content-free `diagnostics`. Copilot input
detection is **unavailable** (`input_detection: false`), never guessed from timeouts.
All active Copilot lock-directories have independent offsets, state and identity.
Historical `session.error` records remain visible but have `attention.notify:
false`; only newly appended errors qualify for alerts. Tool failures alone do
not qualify. A new `assistant.turn_start` clears an error, not Idle or turn end.

Pi consumes the [published v1 protocol](../../integrations/pi/README.md).
Only regular `<UUID>.json` files are opened, never temporary files or symlinks.
Reads stop at 16 KiB plus one overflow-detection byte. A heartbeat more than
10 seconds old, any future timestamp, a regressed publication timestamp,
malformed/partial data or a mismatched instance filename yields Unavailable and
a diagnostic, not inferred input. An exactly 10-second-old heartbeat is fresh.
Restored older error episode IDs are accepted; episode numbers are not freshness
counters. UI tracking faults remain distinctly labeled `ui_tracking_unavailable`.
Directory handles anchor reads during a poll (Linux uses `/proc/self/fd`;
Windows holds directories against rename and rejects reparse points). Descendant
symlinks cannot redirect reads outside the configured root.

Each source exposes at most 64 session instances and examines at most 4096
directory entries per poll. Pi validates freshness before applying the live-session
quota: unavailable publications never displace live attention within that scan.
`unavailable_pi_statuses` counts unavailable regular UUID publications examined
(zero for Copilot); their fixed diagnostics summarize residue even when there is
no spare session-list capacity. Unavailable placeholders occupy only spare slots.
Pi retains at most 64 timestamps and 64 temporary residue names, validates at most
4096 files per poll, and never deletes publisher files. Per-file reads retain the
16 KiB plus one byte limit; only one file is read at a time. Overflow of the scan
or session list is visible, not a guarantee of coverage beyond the scan limit. Copilot
reads at most 256 KiB of appended events per session per poll, retries incomplete
JSONL records, diagnoses/discards overlong records in bounded chunks, and ignores
unknown events/fields. Large historical logs may need multiple polls to initialize,
without replaying historical attention. Source polling here is synchronous:
worker isolation and notification delivery are separate tasks, not claims made
by the aggregate probe.

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
