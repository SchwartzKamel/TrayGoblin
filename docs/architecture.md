# Architecture

**Audience:** Operators and Agents

TrayGoblin is one native Rust process. The Copilot session parser, monitor, and tray decision logic
are platform-neutral and host-testable; only the Win32 notification-area shell is Windows-only.
This split is the accepted decision in
[`specs/adr/0001-native-rust-tray.md`](../specs/adr/0001-native-rust-tray.md), taken to protect the
memory, CPU, privacy, and manual-release constraints in
[`specs/CONSTITUTION.md`](../specs/CONSTITUTION.md).

## Process model

- One process, one thread. The Windows shell runs a standard Win32 message loop. Its `WM_TIMER`
  wakes at the configured interval below one second and once per second for slower settings;
  `app::App` enforces the configured poll due time.
- No background service, no sidecar, no embedded browser, no network client.
- Release builds use the Windows GUI subsystem, so there is no console window; startup failures are
  reported through a content-free message box.
- Configuration is read once at startup. Changing it requires restarting TrayGoblin.

## Data flow

```text
Copilot CLI  ──writes──▶  %USERPROFILE%\.copilot\session-state\<session>\
                              inuse.<pid>.lock
                              workspace.yaml
                              events.jsonl
                                     │  read-only, appended bytes only
                                     ▼
                         monitor::SessionMonitor
                            selects newest active session
                            parses an allow-list of fields
                                     │
                                     ▼
                         status::StatusSnapshot   (content-free)
                                     │
                    ┌────────────────┴─────────────────┐
                    ▼                                  ▼
              app::App (decisions)            bin/tray-goblin-probe
         icon variant, tooltip, actions        one JSON snapshot, exits
                    │
                    ▼
              tray (Windows only)
        Shell_NotifyIcon, menu, WM_TIMER
```

## Module map

| Module | Responsibility | Platform |
|---|---|---|
| `src/config.rs` | JSON configuration: `pollIntervalMs` (500–10,000, default 1,000) and `sessionRoot`; unknown keys ignored; out-of-range values are typed errors | Neutral |
| `src/events.rs` | Parses one JSONL event line into an allow-listed `SessionEvent`; unknown types, unknown fields, and malformed lines are non-fatal | Neutral |
| `src/session.rs` | Parses `workspace.yaml` into repository name and active directory only, tolerating nested and aliased shapes | Neutral |
| `src/monitor.rs` | Selects the active session, caches per-file byte offsets, applies events to a turn state machine, produces a `StatusSnapshot` | Neutral |
| `src/status.rs` | The content-free snapshot type and the `Idle` / `Generating` / `Error` state enum | Neutral |
| `src/icon.rs` | Renders 32×32 tray icons from reviewable text pixel maps in `assets/`, one silhouette and palette per state | Neutral |
| `src/actions.rs` | Menu vocabulary and pure launch-target selection; typed, content-free action errors; settings-file creation | Neutral |
| `src/app.rs` | Decisions: when to poll, which icon variant, what the tooltip says, which effects the shell must perform | Neutral |
| `src/tray.rs` | Win32 window, `Shell_NotifyIcon` registration and retry, timer, context menu, process launching | Windows only |
| `src/main.rs` | Starts the tray on Windows; on other hosts prints how to use the probe instead | Both |
| `src/bin/tray-goblin-probe.rs` | Diagnostic binary that prints one snapshot as JSON using an explicit field allow-list | Neutral |

The dependency direction is one-way: `tray` depends on `app`, `app` depends on `monitor`,
`monitor` depends on `events`, `session`, and `status`. Nothing platform-neutral depends on
Windows.

## Session selection

A Copilot session folder is *active* when it contains an `inuse.*.lock` marker. The monitor picks
the session whose newest lock marker has the most recent modification time, breaking ties by path
so selection is deterministic. When the selected session changes, cached offsets and turn state are
cleared and rebuilt from the new session, so re-activating an earlier session can never combine a
reset state machine with an end-of-file offset.

## Incremental reading

Each poll opens `events.jsonl`, seeks to the cached offset for that path, and reads only the
appended bytes.

- The offset advances only through the last complete newline, so a record that Copilot is still
  writing is retried on the next poll rather than skipped.
- If the file has shrunk since the last poll — rotation or truncation — the monitor resets the turn
  state and re-reads from the start.
- Full-file rescans after initialization are therefore avoided, which is what keeps the CPU budget
  reachable at a one-second cadence.

## Turn state machine

| Event | Effect |
|---|---|
| `assistant.turn_start` | Turn becomes active, start timestamp recorded, any previous failure cleared |
| `assistant.turn_end` | Turn closes; last-turn duration is the wall-clock difference when both timestamps are present |
| `session.model_change`, `assistant.message` | Newest non-empty model name is retained |
| `tool.execution_complete` with `success: false` during an active turn | Turn is marked failed |
| Anything else | Ignored |

The snapshot state is then derived: failed turn → `Error` (**Attention needed**), active turn →
`Generating` (**Working**), otherwise `Idle`. An unreadable events file for an otherwise active
session also yields `Error`, with the `StateUnavailable` reason instead of `ToolFailed`. A failure
stays visible until the next turn starts.

Internal names come from [`specs/CONTEXT.md`](../specs/CONTEXT.md); the user-visible labels are
**Idle**, **Working**, and **Attention needed**.

## Privacy boundary

The boundary is the parser, not the display layer:

- `events.rs` and `session.rs` deserialize an allow-list of fields — event type, timestamp, model,
  success, repository, active directory. Sensitive fields are never named in any struct, so they
  cannot be held even transiently in the program's model.
- `tray-goblin-probe` serializes through its own `ProbeSnapshot` type, so the JSON wire shape is an
  intentional allow-list rather than whatever the core struct happens to derive.
- Action and startup errors are typed enums whose `Display` text names the action and the missing
  prerequisite only — never a path, a session identifier, or an operating-system message.
- `events::tests::does_not_model_sensitive_fields` is the deterministic contract that fails the
  build if this ever regresses.

## Failure handling

| Failure | Behaviour |
|---|---|
| No session folder exists | `Idle`, no error dialog |
| No active session | `Idle` |
| `events.jsonl` unreadable | **Attention needed**, reason `state_unavailable` |
| Malformed or unknown event line | Ignored; polling continues |
| Unknown configuration key | Ignored |
| Out-of-range or invalid configuration | Explicit startup error; TrayGoblin does not start with a surprising cadence |
| Notification-area registration fails | Retried on every timer tick and on Explorer's `TaskbarCreated` broadcast |
| A menu target cannot be launched | Content-free message added to the tooltip; the tray keeps running |

## Performance design

| Budget | How the design meets it |
|---|---|
| Under 50 MB working set | One native process, no webview or scripting runtime, small dependency set |
| Under 5% idle CPU | One timer tick per interval; incremental reads; the tray is redrawn only when the rendered view actually changes |
| State visible within two seconds | 1,000 ms default interval, immediate poll at startup and on **Refresh now** |

Budgets are asserted on Windows with `scripts/measure-performance.ps1`; that measurement must pass
before promoting a preview build to stable.

## Validation boundary

| Deterministic on Linux or any host | Manual on Windows |
|---|---|
| Parser, monitor, configuration, icon, action, and app-decision tests | Tray icon appearance and legibility |
| Fixture-driven probe journeys | Tooltip rendering and truncation in the real shell |
| Windows x86-64 cross-build producing a PE executable | Menu commands launching VS Code, the session folder, and settings |
| Reproducible packaging and checksum generation | Startup shortcut behaviour after sign-in |
| Installer and uninstaller static and sandbox checks | Working-set and CPU measurement |

See [Testing](agent/testing.md) for the exact commands on each side of that line.

## Deliberate non-goals

Restarting or controlling Copilot CLI, reading repository files, a settings window, desktop
notifications, themes, token or cost visualization, auto-update, code signing, MSI/MSIX, and ARM64
artifacts are all out of scope for this pass. See
[`specs/tray-status.md`](../specs/tray-status.md#out-of-scope-this-pass).

## Related

- [MVP](MVP.md) — product scope
- [Manual release](manual-release.md) — how the executable becomes a published artifact
- [Development](agent/development.md) — how to work on the code
- [Privacy](operator/privacy.md) — the operator-facing summary of the same boundary
