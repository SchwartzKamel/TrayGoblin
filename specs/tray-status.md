# Spec: Copilot tray status

> Conforms to `specs/CONSTITUTION.md`.

## JTBD (job to be done)
A Windows developer running GitHub Copilot CLI wants to know whether Copilot is working, which model and repository are active, and how long the last turn took without switching terminals or opening a dashboard.

## User-visible success criteria
- Installing the release ZIP creates a per-user startup entry and launches a single tray icon without requiring elevation.
- Within two seconds of a Copilot turn starting or ending, the icon and tooltip show the corresponding Working or Idle state.
- The tooltip shows the active model, last completed turn duration, and active repository or directory when those values are available.
- A right-click menu supports Refresh now, Open in VS Code, View Copilot logs, Open settings, and Quit.
- Missing, malformed, locked, or newer Copilot state files produce a safe status instead of a crash or content leak.
- A manually built x86-64 Windows release archive and SHA-256 checksum are published from a tagged commit.

## Magic moment
- **The whoa:** Submit a Copilot prompt, watch the tray turn amber within two seconds, then see it return green with the model, repository, and completed-turn duration without leaving the editor.
- **Demo path:** Install TrayGoblin, start Copilot CLI in a repository, send a prompt, hover the tray while it works, hover again after completion, and use Open in VS Code.
- **Smallest end-to-end slice:** A content-free fixture that moves from `assistant.turn_start` to `assistant.turn_end` drives the same monitor used by the tray from Working to Idle and records a duration.
- **Merely functional vs magical:** A static “Copilot process running” light is insufficient; the app must show live turn state and useful context.

## Acceptance criteria → backpressure

| Criterion (EARS) | How it is verified |
|---|---|
| When an active session emits `assistant.turn_start`, TrayGoblin shall report Working within two polling intervals. | `cargo test --lib monitor::tests::turn_start_sets_generating` |
| When that turn emits `assistant.turn_end`, TrayGoblin shall report Idle and the completed wall-clock duration. | `cargo test --lib monitor::tests::turn_end_sets_latency` |
| When model metadata appears in `session.model_change` or `assistant.message`, TrayGoblin shall show the newest non-empty model name. | `cargo test --lib events::tests::tracks_latest_model` |
| While an active session has workspace metadata, TrayGoblin shall show repository and active directory without reading conversation content. | `cargo test --lib session::tests::reads_workspace_metadata_only` |
| If an event is unknown, malformed, or contains extra fields, then TrayGoblin shall ignore unsupported content and remain operational. | `cargo test --lib events::tests::unknown_future_event_is_non_fatal` |
| For every event fixture, TrayGoblin shall deserialize no prompt, response, tool argument, tool result, token, credential, or repository-content field. | `cargo test --lib events::tests::does_not_model_sensitive_fields` |
| If the current turn contains a failed tool execution or the selected state file cannot be read, then TrayGoblin shall report Attention needed with a content-free reason. | `cargo test --lib monitor::tests::failed_tool_sets_safe_error` |
| When the user selects Refresh now, the tray shall poll immediately rather than waiting for the timer. | `cargo test --lib app::tests::manual_refresh_requests_poll` |
| When the user selects Open in VS Code, View Copilot logs, or Open settings, TrayGoblin shall launch the corresponding local target or report a safe error. | `cargo test --lib actions::tests` |
| When TrayGoblin is installed, the installer shall copy files per-user and create a startup shortcut without elevation. | `pwsh -NoProfile -File scripts/test-installer.ps1` |
| The release process shall produce a Windows PE executable, ZIP archive, and matching SHA-256 checksum without CI. | `bash scripts/package-release.sh 0.1.0` |
| While idle on Windows 10/11, TrayGoblin shall not exceed 50 MB working set or 5% CPU during the 30-second measurement window. | `pwsh -NoProfile -File scripts/measure-performance.ps1 -DurationSeconds 30` |

## Holdout scenarios
- **Files:** `scenarios/active-turn.yaml`, `scenarios/degraded-state.yaml`
- **Holdout rule:** implementation work must not read these files; only validation and review judge against them.

## Assumptions
- Copilot CLI 1.0.76 stores `workspace.yaml`, `events.jsonl`, and active `inuse.*.lock` markers below `%USERPROFILE%\.copilot\session-state`.
- The event format is internal and may evolve, so the parser intentionally selects only type, timestamp, model, and success fields while ignoring everything else.
- “Latency” means last completed turn duration, not network-only latency or time-to-first-token.
- The default polling interval is 1,000 ms; configuration accepts 500–10,000 ms.
- If several sessions are active, the most recently modified lock marker wins.
- The first public preview targets Windows x86-64. ARM64, signing, MSI/MSIX, and Store distribution are follow-ups.
- A native Rust tray shell is an intentional deviation from the original Electron/Tauri suggestion to protect the performance budget.

## Out of scope (this pass)
- Token visualization, cost accounting, prompt or response display, desktop notifications, themes, and multi-CLI providers.
- Restarting or controlling Copilot CLI.
- VS Code extension-state integration.
- Automatic updates, code signing, MSI/MSIX, Windows Store publication, and ARM64 artifacts.
