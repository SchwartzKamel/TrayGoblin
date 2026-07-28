# CONTEXT — domain glossary

## Terms

| Term | Means | Canonical name (code / UI) | Not to be confused with |
|---|---|---|---|
| Session root | Copilot CLI's local directory containing per-session state | `session_root` / **Session folder** | Copilot process logs |
| Copilot session | One UUID-named directory under the session root | `CopilotSession` / **Session** | One model request |
| Active session | A session with an `inuse.*.lock` marker; newest lock wins if several exist | `active_session` / **Active session** | Most recently modified inactive session |
| Turn | Work bounded by `assistant.turn_start` and `assistant.turn_end` | `Turn` / **Turn** | Individual tool execution |
| Generating | An active turn has started and has not ended | `StatusState::Generating` / **Working** | Merely having Copilot CLI open |
| Idle | No turn is active, including when no Copilot process is active | `StatusState::Idle` / **Idle** | Disconnected or failed |
| Error | State data is unreadable or a tool execution failed in the current turn | `StatusState::Error` / **Attention needed** | An unknown event type |
| Turn latency | Wall-clock duration of the most recently completed turn | `last_turn_duration` / **Last turn** | Network time-to-first-token |
| Status snapshot | Content-free model shown by the tray at one instant | `StatusSnapshot` | Raw Copilot event |

## Avoid (non-canonical aliases)

- "request latency" -> use **Turn latency**.
- "busy" or "thinking" -> use **Generating** in code and **Working** in UI.
- "workspace" when referring to the observed directory -> use **Active directory**.

## Open questions

None for the MVP.

