# Status reference

**Audience:** Operators

TrayGoblin reports exactly three states. Each state has its own icon shape *and* its own color, so
the tray remains readable with a high-contrast theme or color-vision deficiency. The tooltip always
repeats the state as text.

## The three states

| Tray state | Icon | Means | Derived from |
|---|---|---|---|
| **Idle** | Green, calm silhouette | No Copilot turn is in progress. This includes having no active Copilot session at all. | No turn is open in the selected session, or no active session exists |
| **Working** | Amber, active silhouette | A Copilot turn has started and has not finished. | An `assistant.turn_start` event with no matching `assistant.turn_end` yet |
| **Attention needed** | Red, alert silhouette | The last turn had a failed tool execution, or Copilot's session state could not be read. | A failed `tool.execution_complete` inside the active turn, or an unreadable events file |

Idle is not "disconnected" and not "failed". TrayGoblin never claims to know whether Copilot CLI
itself is running — it reports whether a turn is in progress.

## How a session is selected

Copilot CLI keeps one UUID-named session directory inside the **Session folder**. A session is
*active* when it contains an `inuse.*.lock` marker. TrayGoblin selects the session with the most recently
modified lock marker; if you run several Copilot sessions at once, the newest one wins and the
others are ignored. When the selected session changes, TrayGoblin rebuilds its view of that
session's state from the beginning of the new session's events.

## Tooltip lines

The tooltip is built only from content-free values, in this order:

| Line | Shown when | Example |
|---|---|---|
| `TrayGoblin — <state>` | Always | `TrayGoblin — Working` |
| `Model: <name>` | A model name has been observed | `Model: gpt-5.6-sol` |
| `Repo: <owner/name>` | The session's workspace metadata names a repository | `Repo: octo-org/content-free-demo` |
| `Dir: <path>` | No repository is known but an active directory is | `Dir: C:/src/demo` |
| `Last turn: <duration>` | At least one turn has completed in this session | `Last turn: 2.4 s` |
| `Reason: <text>` | The state is Attention needed | `Reason: a tool execution failed in the last turn` |
| Action message | The last menu command could not run | `Open in VS Code is unavailable: no active Copilot directory is known` |

Durations under a minute are shown with one decimal (`2.4 s`); longer turns use minutes and
seconds (`1 m 05 s`). Windows silently truncates long tooltips, so TrayGoblin truncates at 127
characters itself and marks the cut with `…`. The state line is always kept.

## Attention needed, in detail

There are two reasons, and the tooltip names which one applies:

| Reason line | Cause | What to do |
|---|---|---|
| `a tool execution failed in the last turn` | Copilot reported a failed tool execution while the turn was open | Check the Copilot CLI terminal; the state clears when the next turn starts |
| `Copilot session state is unreadable` | The selected session's `events.jsonl` could not be read — deleted, locked, or permission-denied | See [Troubleshooting](troubleshooting.md) |

Attention needed persists until the next turn starts, so a failure that happens while you are away
from the machine is still visible when you return.

## Timing

- The default polling interval is 1,000 ms; the accepted range is 500–10,000 ms.
- A state change is expected to appear within two polling intervals — two seconds at the default
  cadence.
- **Refresh now** polls immediately instead of waiting for the next tick.
- After the first poll, only newly appended event bytes are read, so a long-running session does
  not get slower to observe.

## Values that are never shown

Prompts, assistant responses, tool arguments, tool results, token counts, credentials, and
repository file contents are never parsed, stored, logged, or displayed. See
[Privacy](privacy.md).

## Checking state without the tray

The repository ships a content-free diagnostic probe used by the deterministic tests. It is run from
a repository checkout with stable Rust installed, not from the installed payload. It prints one JSON
snapshot from a session folder and exits:

```bash
cargo run --bin tray-goblin-probe -- --session-root tests/fixtures/live-session
```

```json
{
  "state": "working",
  "model": "gpt-5.6-sol",
  "repository": "octo-org/content-free-demo",
  "active_directory": "C:/fixture/content-free-demo",
  "last_turn_duration_ms": null,
  "attention_reason": null
}
```

`state` maps to the tray states as `idle`, `working`, and `attention_needed`; `attention_reason` is
`tool_failed` or `state_unavailable`. Point `--session-root` at your own session folder to inspect
a live machine, or at the bundled fixtures to see known-good output.

## Related

- [First run](first-run.md)
- [Configuration](configuration.md)
- [Troubleshooting](troubleshooting.md)
- [Architecture](../architecture.md) for how the state machine is implemented
