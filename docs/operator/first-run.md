# First run

**Audience:** Operators

This page describes what TrayGoblin does the first time it starts, and how to confirm it is
working correctly in under a minute.

## What happens at startup

1. TrayGoblin reads `%APPDATA%\TrayGoblin\config.json` if that file exists. A missing file is
   normal on a first run and means the documented defaults are used.
2. It resolves the Copilot session folder: the `sessionRoot` value from configuration when set,
   otherwise `%USERPROFILE%\.copilot\session-state`.
3. It registers a notification-area icon and starts a timer at the configured polling interval
   (1,000 ms by default).
4. It polls immediately, so the first state appears without waiting a full interval.

TrayGoblin has no window, no console, and no dashboard. The icon, its tooltip, and its right-click
menu are the entire interface.

## The first state you will see

With Copilot CLI closed, the tray shows **Idle**. That is correct: Idle means "no Copilot turn is
in progress", including the case where no Copilot session is active at all.

## Confirm the magic moment

1. Open a terminal in a repository and start GitHub Copilot CLI.
2. Send a prompt.
3. Within two polling intervals the icon changes to **Working**.
4. When the turn finishes, the icon returns to **Idle** and the tooltip gains a
   `Last turn: <duration>` line.

Hover the icon at each step. A Working tooltip looks like this:

```text
TrayGoblin — Working
Model: gpt-5.6-sol
Repo: octo-org/content-free-demo
Last turn: 2.4 s
```

Only lines whose values are known are shown. When no repository is available, the tooltip shows a
`Dir:` line with the active directory instead. See
[Status reference](status-reference.md) for every line and state.

## The right-click menu

| Command | What it does |
|---|---|
| **Refresh now** | Polls immediately instead of waiting for the next timer tick |
| **Open in VS Code** | Opens the active Copilot directory in VS Code |
| **View Copilot logs** | Opens the active Copilot session folder, falling back to the session folder root |
| **Open settings** | Creates `config.json` with documented defaults if it is missing, then opens it |
| **Quit** | Exits TrayGoblin; the Startup shortcut still launches it at your next sign-in |

If a command has nothing to open — for example **Open in VS Code** before any Copilot session has
reported a directory — the tooltip shows a short, content-free explanation such as
`Open in VS Code is unavailable: no active Copilot directory is known`. No path, session
identifier, or operating-system message is ever shown.

## Startup behaviour

The installer adds `TrayGoblin.lnk` to your per-user Startup folder, so TrayGoblin starts at sign
in. If Windows Explorer is still starting when TrayGoblin launches, the notification-area icon is
re-registered automatically on the next timer tick and whenever Explorer restarts, so a missing
icon recovers by itself.

## Next steps

- [Status reference](status-reference.md) — what each state and tooltip line means.
- [Configuration](configuration.md) — change the polling interval or point at another session
  folder.
- [Privacy](privacy.md) — exactly what is read.
- [Troubleshooting](troubleshooting.md) — no icon, a stuck state, or a startup error dialog.
