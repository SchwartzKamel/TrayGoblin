# MVP: Windows Copilot Status Bar

**Audience:** Operators and Agents

> **Architecture decision:** The implementation uses one native Rust process rather than the original Electron/Tauri and Node.js sketch. This is the accepted decision in [`specs/adr/0001-native-rust-tray.md`](../specs/adr/0001-native-rust-tray.md), chosen to protect the memory, CPU, privacy, and manual-release constraints. The implemented design is described in [`architecture.md`](architecture.md).

This page is the product definition. For usage, read the operator guides — [Installation](operator/installation.md), [First run](operator/first-run.md), [Status reference](operator/status-reference.md), [Configuration](operator/configuration.md), [Privacy](operator/privacy.md), and [Troubleshooting](operator/troubleshooting.md) — all indexed in [`README.md`](README.md).

## Overview
A lightweight Windows notification-area application that displays real-time, content-free status from GitHub Copilot CLI. It provides at-a-glance Copilot activity without requiring a dashboard or terminal switch.

## Goals
- Persistent tray icon showing Copilot activity and model state  
- Real-time updates on model, last completed turn duration, and active directory
- Seamless integration with PowerShell and VS Code  
- Minimal resource footprint  
- A provider boundary that can support other AI CLIs after the MVP

## Architecture

### High-Level Description
- GitHub Copilot CLI writes per-session metadata and events below `%USERPROFILE%\.copilot\session-state`.
- A platform-neutral Rust monitor reads only event type, timestamp, model, success, repository, and directory fields.
- A native Windows tray shell polls the monitor every second and updates the icon, tooltip, and menu.
- The application never models prompt content, assistant content, tool arguments, tool results, tokens, credentials, or repository files.

### Components
- Core: Rust session discovery, workspace parsing, event parsing, and status state machine
- Shell: Windows-native notification-area icon, tooltip, timer, and context menu
- Diagnostic path: content-free JSON probe used by deterministic fixture tests
- Config: JSON for refresh rate and session-root override
- Startup: PowerShell installer that copies the app per-user and creates a Startup shortcut

## Core Features

### Status Flow
1. Copilot CLI appends content-bearing events to its local session file.
2. TrayGoblin deserializes only a safe subset of metadata and ignores unknown fields and event types.
3. The monitor derives Idle, Working, or Attention needed plus model, repository, directory, and turn duration.
4. The Windows shell updates the icon and tooltip within two polling intervals.

### Feature List
- Tray icon with color and text states:
  - Idle  
  - Working
  - Attention needed
- Tooltip showing:
  - Current model  
  - Last completed turn duration
  - Active directory or repo  
- Auto-refresh every second by default (configurable 500–10,000 ms)
- Right-click menu:
  - Refresh now
  - Open in VS Code
  - View Copilot logs
  - Open settings  
  - Quit

## MVP Scope
- Single tray icon bound to Copilot CLI  
- Read-only polling of Copilot CLI local session state
- Tooltip with model, last turn duration, and active directory or repository
- Manual refresh action in tray menu  
- PowerShell installer script (`install.ps1`) to set up and add to startup

## Future Enhancements
See [`specs/tray-status.md`](../specs/tray-status.md#out-of-scope-this-pass) for the authoritative
follow-up list. Keeping the list in one place prevents the product overview from drifting away from
the executable scope.

## Tech Stack
- Stable Rust for the monitor and native Windows tray shell
- JSON for configuration and diagnostic probe output
- GNU Windows cross-toolchain for manual x86-64 release builds
- PowerShell for installation, startup registration, uninstall, and Windows performance measurement
- ZIP plus SHA-256 checksum for preview distribution

## Testing

Two layers, kept distinct — see [`agent/testing.md`](agent/testing.md) for the commands.

**Deterministic, on any host with the toolchain**
- Unit tests for session selection, parsing, configuration, state transitions, actions, and privacy boundaries
- Content-free Copilot session fixtures for offline testing, including a degraded future-format fixture
- Cross-build validation that produces a Windows PE executable
- Reproducible packaging with checksum verification
- Documentation validation through `bash scripts/check-docs.sh`

**Manual, on Windows 10 and Windows 11 only**
- Tray icon appearance, tooltip rendering, menu commands, and startup registration
- Scripted pass/fail measurement of CPU and memory usage with `scripts/measure-performance.ps1`

## Success Criteria
- Tray icon updates within 2 seconds of Copilot CLI state change  
- No more than 50 MB working set during normal idle operation
- Less than 5% CPU usage while idle
- No admin privileges required for normal use after installation
- A manually built, checksummed Windows x86-64 preview ready to publish from a clean tagged commit
  after the Operator Windows checklist passes, following [`manual-release.md`](manual-release.md)
- The Windows performance measurement must pass before promoting a preview build to stable
