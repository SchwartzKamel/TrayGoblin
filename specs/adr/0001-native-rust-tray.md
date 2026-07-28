# ADR 0001: Use a native Rust tray process

- **Status:** Accepted
- **Date:** 2026-07-28

## Context
The draft MVP suggested Electron or Tauri with a resident Node.js poller, while also requiring under 50 MB RAM, under 5% idle CPU, no administrator privileges, passive local observation, and manual Windows releases from the current non-Windows environment.

## Decision
Build TrayGoblin as one native Rust process. Keep the Copilot session parser and monitor platform-neutral, and place the notification-area shell behind Windows-only compilation. Read Copilot's existing local session state read-only instead of installing a plugin, changing telemetry settings, or launching a sidecar.

## Alternatives considered
- **Electron plus Node.js poller:** familiar TypeScript stack and straightforward menus, rejected because Chromium and Node make the memory target implausible.
- **Tauri plus Rust or Node sidecar:** lighter than Electron and within the original stack options, rejected for the MVP because a hidden webview adds packaging complexity without providing user value for a tray-only interface.
- **Copilot plugin or OpenTelemetry collector:** structured and extensible, rejected for the MVP because installation would alter Copilot configuration and could conflict with an operator's existing telemetry setup.

## Consequences
- The app should fit the resource budget and can be cross-built as a compact Windows executable.
- Parser fixtures and most behavior can be validated on Linux; the Windows shell still requires Windows or Wine smoke testing.
- There is no settings window in the MVP; settings are a documented JSON file opened from the tray.
- If Copilot changes its internal session format, the provider may need an update, but unknown fields and events remain non-fatal.

