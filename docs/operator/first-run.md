# First run

**Audience:** Operators

TrayGoblin is one native Windows notification-area icon: no window, console,
dashboard, background service, or terminal control. A second launch in the same
interactive Windows session exits quietly. Settings changes require a restart.

## Startup

1. Read `%APPDATA%\TrayGoblin\config.json` once; a missing file uses defaults.
2. Start independent source workers for native Copilot
   (`%USERPROFILE%\.copilot\session-state`) and pi
   (`%USERPROFILE%\.traygoblin\pi`). A missing implicit pi folder is normal.
3. Register the icon and request the first poll immediately. The timer only
   requests/drains bounded worker snapshots; it never reads provider files or
   waits for a blocked worker. Default polling is 1,000 ms.

An explicit `sources` array replaces both defaults; `[]` disables monitoring.
See [Configuration](configuration.md) for native/custom/WSL examples and limits.
No WSL discovery or launch command is used. Reading an explicitly configured UNC
root can itself wake or keep WSL running. Remove that source and restart the tray
to disable those reads. An inaccessible source degrades visibly without blocking
other sources.

## Give same-repository terminals a manual label

Load the opt-in pi extension with a different label in each terminal. Example
paths assume a checkout at `C:\src\TrayGoblin` or `/home/operator/TrayGoblin`:

```powershell
# Native Windows PowerShell, before starting pi:
$env:TRAY_GOBLIN_LABEL = 'review'
pi --extension C:\src\TrayGoblin\integrations\pi\tray-goblin.js
# Another terminal can use 'build', even in the same working directory.
```

```sh
# Inside WSL (not a Windows path):
TRAY_GOBLIN_LABEL=build pi --extension /home/operator/TrayGoblin/integrations/pi/tray-goblin.js
```

For WSL, explicitly add a pi source such as
`\\wsl.localhost\Ubuntu\home\operator\.traygoblin\pi` to the **Windows** tray's
configuration. The default status folder belongs to the home of the process
running pi, not to the Windows consumer. `TRAY_GOBLIN_STATUS_DIR` can override it.
See the [pi integration](../../integrations/pi/README.md) for persistent opt-in
installation/removal. TrayGoblin never changes pi settings automatically.

Labels are optional user-supplied metadata: at most 64 UTF-8 bytes after control
removal/truncation, then trimmed; blank means unlabeled. They are not pi's automatic
session name, a prompt summary, or a terminal title. Restart pi to change its
inherited environment. Session and instance IDs distinguish unlabeled terminals.

## Confirm the attention journey

1. Start pi with the extension. Ordinary Idle shows no notification.
2. Open an explicit blocking dialog (including a manual extension dialog). A
   single balloon submission identifies the source, manual label and session.
3. Right-click the tray: the session remains at the top as **Input required**
   even if Windows/Focus Assist suppresses the balloon.
4. Answer or cancel the dialog. The underlying Idle/Working/Error state returns.
5. A final pi run error or a new persisted Copilot session error also requests
   attention. Successful completion, ordinary Idle and recovered tool failures
   stay quiet. Passive Copilot input detection is **unavailable**, not inferred.

A successful `Shell_NotifyIconW` submission is **not proof Windows displayed a
notification**. Use the persistent list if notifications are suppressed. Real
Windows/WSL delivery/recovery and resource measurements are separate release
checks; Linux tests/cross-compilation do not substitute for them.

## Right-click menu

All sessions persist in an attention-first list. Each submenu shows content-free
source/session/instance IDs, manual label, model, directory and available timing
(as separate bounded fields).
Choose **Use for Open in VS Code** to select a row; this does not launch anything
or focus/control a terminal. Without a selection the first attention row wins.
Source submenus show provider capabilities, unavailable pi-file counts and fixed
safe diagnostics (including stale/future clocks and tracking faults).

| Command | What it does |
|---|---|
| **Refresh now** | Requests a poll immediately; rearms exhausted *undelivered* episodes, without bypassing the minimum five seconds since submission completion |
| **Open in VS Code** | Opens the selected/default session's native Windows directory |
| **View Copilot logs** | Opens a live Copilot session folder, falling back only to a configured Copilot root; never pi conversations/status |
| **Open settings** | Creates default `config.json` if missing and opens it; restart after edits |
| **Quit** | Stops requests, removes the icon and exits without joining blocked source reads |

Unmapped POSIX/WSL working directories intentionally return a fixed safe action
error: **use the terminal for unmapped or unsafe paths**. There is no implicit
POSIX-to-Windows conversion or WSL/editor command execution. Configured Windows
UNC source roots remain valid for **View Copilot logs**. Action errors never
include OS exception text or private content.

Explorer restarts re-add the icon, not the workers/controller: acknowledged
notifications do not replay, and failed submissions are never acknowledged.
The per-user Startup shortcut still launches TrayGoblin at your next sign-in.

## Next steps

- [Status reference](status-reference.md) — states, counts and delivery limits.
- [Configuration](configuration.md) — sources and polling cadence.
- [Privacy](privacy.md) — prohibited content.
- [Troubleshooting](troubleshooting.md) — startup and unavailable sources.
