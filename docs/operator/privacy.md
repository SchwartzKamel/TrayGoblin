# Privacy

**Audience:** Operators

TrayGoblin is a passive, local, read-only observer. This page distinguishes raw-byte scanning
from the allow-listed metadata it models and exposes.

## The content-free contract

TrayGoblin never deserializes into modeled fields, persists, logs, or displays:

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

Session files can contain sensitive fields alongside metadata. Readers temporarily buffer and
scan raw bytes (including unknown fields) before allow-listed parsing; that is not a promise
that sensitive bytes never enter memory. Those fields are skipped, not turned into content
values in the data model. Repository file contents are not read.

## Legacy MVP metadata

From the Copilot session folder (`%USERPROFILE%\.copilot\session-state` by default), TrayGoblin
models only:

| Source | Fields used |
|---|---|
| `inuse.*.lock` markers | Existence and modification time, to pick the active session |
| `events.jsonl` | Event type, event timestamp, model name, and a tool execution's success flag |
| `workspace.yaml` | Repository name and active directory path |

Everything else on those lines — including every field TrayGoblin does not recognize — is ignored
and never enters the program's data model.

The default probe has exactly six output fields: `state`, `model`, `repository`,
`active_directory`, `last_turn_duration_ms`, and `attention_reason`. This is the legacy snapshot,
not an inventory of the aggregate attention model.

## Aggregate tray/probe metadata

The current tray and explicit `--all-sessions` probe use this separate allow-list:

| Scope | Modeled/output metadata |
|---|---|
| Source | Configured `id`, `label`, `kind`, `path`, optional `wsl_distro`; `capabilities.input_detection` and `session_errors` |
| Session | `session_id`, optional manual `label`, `instance_id`, `model`, `directory`, `state`, `attention` (`id`, `reason`, `notify`), `run_started_at_ms`, `last_run_duration_ms` |
| Health | `unavailable_pi_statuses` and fixed `diagnostics` codes |

The pi v1 reader additionally validates publication version/provider, episode and publication
timestamp and optional explicit manual label. Membership identities, publisher
freshness deadlines and controller delivery bookkeeping are internal, not extra
JSON fields. See [Configuration](configuration.md) for explicit probe loading and
limits. Native wiring is accepted at implementation/compilation tier; actual
Windows/WSL delivery and resource evidence remains blocked M6.

## Where data goes

- **No telemetry or outbound service.** TrayGoblin has no network client, crash reporting or
  update check. Configured filesystem roots can include UNC/WSL paths; those reads may use the
  OS filesystem transport and may wake WSL.
- **No observed session data is persisted by the tray/probe.** The only file the GUI writes is your own
  `%APPDATA%\TrayGoblin\config.json`, and only when you choose **Open settings** and the file does
  not exist yet. The separate opt-in [pi publisher](../../integrations/pi/README.md) writes its
  content-free status files; it is not tray/probe persistence. The installer persists
  integration code/README/resources below its root, without changing pi settings.
- **All access is read-only.** Copilot's session files are opened for reading; TrayGoblin never
  writes, moves, truncates, or deletes them, and never changes Copilot CLI configuration or
  telemetry settings.

## Error messages are content-free too

GUI startup/action failures name the action and the missing prerequisite, never a path, a
session identifier, or an operating-system message. For example:

```text
Open in VS Code is unavailable: no active Copilot directory is known
Open settings failed: the settings file is not writable
Copilot session state is unreadable
```

Because release builds run without a console, startup failures appear in a small dialog using the
same content-free wording.

The CLI probe has a different diagnostic surface: config I/O errors use
`error: cannot read config: <OS error>`, and invalid JSON can report parser location/type details.
It does not echo the config document, but OS-supplied text is not covered by the fixed GUI-message
guarantee. Aggregate JSON intentionally includes configured paths and source/session identities;
review probe output before sharing it.

## What is visible on screen

The tooltip/menu/notification can show provider/source, optional manual label,
session/instance IDs, model, directory and available timing. The legacy probe
also includes repository metadata. If you share your screen, treat the directory and repository lines as you would your
editor's title bar. There is no way for prompt or response text to appear there.

## Confirming it yourself

The first three checks are run from a repository checkout with stable Rust installed.

- **Read the code:** the parsers live in `src/events.rs` and `src/session.rs`, and the field
  allow-list for diagnostic output is in `src/bin/tray-goblin-probe.rs`.
- **Run the privacy test:** it fails if a sensitive field is ever added to the model.

  ```bash
  cargo test --lib events::tests::does_not_model_sensitive_fields
  ```

- **Inspect the legacy snapshot:** the diagnostic probe prints the six fields listed above,
  not the aggregate model's complete inventory.

  ```bash
  cargo run --bin tray-goblin-probe -- --session-root tests/fixtures/live-session
  ```

- **Check for network access:** monitor `tray-goblin.exe` with Resource Monitor or a firewall
  prompt; it never initiates a connection.

## Related

- [Status reference](status-reference.md) for the exact values shown
- [Configuration](configuration.md) for the only file TrayGoblin writes
- [Architecture](../architecture.md) for where the privacy boundary is enforced in code
