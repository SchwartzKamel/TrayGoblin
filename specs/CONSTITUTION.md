# Constitution: TrayGoblin

> Project-wide principles every spec, plan, and task must honor.

## Principles
1. **Code quality** — Keep the monitor, parser, Windows shell, and packaging concerns separated. Prefer typed data and explicit errors over stringly or silent fallbacks.
2. **Testing** — Every state transition and parser boundary must have deterministic fixture coverage. A Windows release is not shippable until the core tests and cross-build both pass.
3. **Security & privacy** — Read Copilot state files locally and read-only. Never deserialize, display, persist, or log prompt content, assistant content, tool arguments, tool results, tokens, credentials, or repository file contents.
4. **UX & consistency** — Tray state must be understandable from color plus text, not color alone. Errors must be concise and actionable without exposing session content.
5. **Performance** — Poll no faster than once per second by default, avoid full-file rescans after initialization, target under 50 MB working set, and target under 5% idle CPU on Windows 10/11.

## Non-negotiables
- No administrator privileges for install, startup registration, or normal use.
- No dependency on GitHub Actions or another CI service for validation or release publishing.
- The application must continue safely when Copilot CLI adds unknown event fields or event types.
- Release artifacts must include checksums and be built manually from a clean tagged commit.

## Tech constraints
- **Must use:** stable Rust, a native Windows notification-area implementation, JSON configuration, PowerShell installation scripts.
- **Must avoid:** Electron and a resident Node.js sidecar because they put the 50 MB memory budget at unnecessary risk; invasive Copilot plugins or telemetry configuration for the MVP.
- **Deployment target:** Windows 10 and Windows 11, x86-64, distributed as a portable ZIP installed per-user.

## Recording deviations

| Date | Principle | Why we deviated | Scope |
|---|---|---|---|
| 2026-07-28 | Original MVP stack suggested Electron/Tauri plus Node.js | A native Rust tray process better satisfies the stated memory, CPU, privacy, and manual cross-build constraints while preserving the requested behavior | MVP implementation |

**Version**: 1.0.0 | **Ratified**: 2026-07-28 | **Last Amended**: 2026-07-28

