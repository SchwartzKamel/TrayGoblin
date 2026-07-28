# Privacy

**Audience:** Operators

TrayGoblin is a passive, local, read-only observer. This page states exactly what it reads, what it
refuses to read, and how you can confirm both claims yourself.

## The content-free contract

TrayGoblin never reads, deserializes, stores, logs, or displays:

- prompt text or any other user message content
- assistant responses
- tool arguments
- tool results
- token counts or usage data
- credentials, tokens, or authentication material
- the contents of your repository files

This is a project non-negotiable in [`specs/CONSTITUTION.md`](../../specs/CONSTITUTION.md), not a
best-effort goal, and it is enforced by a test that fails the build if a sensitive field is ever
modeled.

## What it does read

From the Copilot session folder (`%USERPROFILE%\.copilot\session-state` by default), TrayGoblin
reads only:

| Source | Fields used |
|---|---|
| `inuse.*.lock` markers | Existence and modification time, to pick the active session |
| `events.jsonl` | Event type, event timestamp, model name, and a tool execution's success flag |
| `workspace.yaml` | Repository name and active directory path |

Everything else on those lines — including every field TrayGoblin does not recognize — is ignored
and never enters the program's data model.

## Where data goes

- **Nowhere off your machine.** TrayGoblin makes no network requests. It has no telemetry, no
  crash reporting, and no update check.
- **Nothing is persisted.** The only file TrayGoblin writes is your own
  `%APPDATA%\TrayGoblin\config.json`, and only when you choose **Open settings** and the file does
  not exist yet.
- **All access is read-only.** Copilot's session files are opened for reading; TrayGoblin never
  writes, moves, truncates, or deletes them, and never changes Copilot CLI configuration or
  telemetry settings.

## Error messages are content-free too

Failures name the action and the missing prerequisite, never a path, a session identifier, or an
operating-system message. For example:

```text
Open in VS Code is unavailable: no active Copilot directory is known
Open settings failed: the settings file is not writable
Copilot session state is unreadable
```

Because release builds run without a console, startup failures appear in a small dialog using the
same content-free wording.

## What is visible on screen

The tooltip can show a model name, a repository name, an active directory path, and a turn
duration. If you share your screen, treat the directory and repository lines as you would your
editor's title bar. There is no way for prompt or response text to appear there.

## Confirming it yourself

The first three checks are run from a repository checkout with stable Rust installed.

- **Read the code:** the parsers live in `src/events.rs` and `src/session.rs`, and the field
  allow-list for diagnostic output is in `src/bin/tray-goblin-probe.rs`.
- **Run the privacy test:** it fails if a sensitive field is ever added to the model.

  ```bash
  cargo test --lib events::tests::does_not_model_sensitive_fields
  ```

- **Inspect what a snapshot contains:** the diagnostic probe prints the complete set of values
  TrayGoblin can hold at one instant.

  ```bash
  cargo run --bin tray-goblin-probe -- --session-root tests/fixtures/live-session
  ```

- **Check for network access:** monitor `tray-goblin.exe` with Resource Monitor or a firewall
  prompt; it never initiates a connection.

## Related

- [Status reference](status-reference.md) for the exact values shown
- [Configuration](configuration.md) for the only file TrayGoblin writes
- [Architecture](../architecture.md) for where the privacy boundary is enforced in code
