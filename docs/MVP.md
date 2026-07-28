# MVP: Windows Copilot Status Bar

## Overview
A lightweight Windows system tray application that displays real-time status information from GitHub Copilot CLI and related AI CLIs. It integrates into the Windows notification area and provides at-a-glance status for Copilot activity.

## Goals
- Persistent tray icon showing Copilot activity and model state  
- Real-time updates on token usage, latency, and active directory  
- Seamless integration with PowerShell and VS Code  
- Minimal resource footprint  
- Extensible for other AI CLIs (Claude, Gemini, Cursor)

## Architecture

### High-Level Description
- GitHub Copilot CLI produces status output (model, tokens, latency, etc.).  
- A Node.js status poller reads this output at a fixed interval.  
- The poller sends structured status data to the tray UI via IPC or WebSocket.  
- The tray UI (Electron or Tauri) updates the Windows notification area icon and tooltip.

### Components
- Frontend: Electron or Tauri tray application  
- Backend: Node.js status poller for Copilot CLI  
- Communication: JSON IPC or WebSocket between poller and tray UI  
- Config: YAML/JSON for refresh rate, CLI paths, theme  
- Startup: PowerShell installer to register app on login

## Core Features

### Status Flow
1. Copilot CLI emits status (model, tokens, latency).  
2. Poller parses and normalizes the status output.  
3. Poller sends a compact JSON payload to the tray UI.  
4. Tray UI updates:
   - Icon state (idle, generating, error)  
   - Tooltip text (model, tokens, latency, directory)  

### Feature List
- Tray icon with color-coded states:
  - Idle  
  - Generating  
  - Error  
- Tooltip showing:
  - Current model  
  - Token usage  
  - Latency  
  - Active directory or repo  
- Auto-refresh every 3–5 seconds  
- Right-click menu:
  - Restart Copilot CLI  
  - Open VS Code  
  - View logs  
  - Open settings  
- Plugin-style extension for other CLIs (Claude, Gemini, Cursor)

## MVP Scope
- Single tray icon bound to Copilot CLI  
- Basic polling of Copilot CLI status via subprocess or log/API  
- Tooltip with model and latency  
- Manual refresh action in tray menu  
- PowerShell installer script (`install.ps1`) to set up and add to startup

## Future Enhancements
- Token/session visualization (progress bar or gauge)  
- Multi-CLI support with selectable active backend  
- Custom themes (light/dark, accent colors)  
- Desktop notifications for errors, rate limits, or disconnections  
- Optional integration with VS Code Copilot extension state

## Tech Stack
- Electron or Tauri for tray UI  
- Node.js + TypeScript for poller and IPC  
- electron-builder or tauri build for packaging  
- PowerShell for installation and startup registration

## Testing
- Unit tests for polling logic and status parsing  
- Mocked Copilot CLI outputs for offline testing  
- Manual tray interaction tests on Windows 10 and Windows 11  
- Basic performance checks for CPU and memory usage

## Success Criteria
- Tray icon updates within 2 seconds of Copilot CLI state change  
- Under 50MB RAM usage during normal operation  
- Under 5% CPU usage while idle  
- No admin privileges required for normal use after installation
