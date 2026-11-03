# Session attention vocabulary

> Extends `../../specs/CONTEXT.md`; this follow-up supersedes newest-session-only
> vocabulary only where explicitly noted in `session-attention.md`.

| Term | Meaning | Canonical name |
|---|---|---|
| Source | One default-supplied or explicitly configured provider/path/environment | `Source` / Source |
| Provider | Agent CLI family, not the model vendor | `pi` or `copilot` |
| Session instance | One live publisher/runtime, distinct even for the same directory/session | Session instance |
| Attention episode | A new explicit input requirement or supported terminal error, with stable identity for deduplication | Attention episode |
| Input required | Supported explicit blocking UI during agent work; not inferred from idle time | Input required |
| Terminal error | Final pi run error or new persisted Copilot session.error, not a failed tool that the agent recovers from | Session error |
| Unavailable | Stale publisher or unreadable/stalled source; not proof of an agent input request | Unavailable |
| Quiet completion | Successful completion without notification | Idle |
| Notification request | A content-free policy action awaiting successful native adapter submission | Notification request |

Unknown Copilot input capability means unavailable detection, never no-input-needed.
WSL is local Linux under Windows, not remote SSH monitoring.
