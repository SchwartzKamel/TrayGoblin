# TrayGoblin

TrayGoblin is a lightweight Windows notification-area companion for GitHub Copilot CLI. It observes Copilot's local session state read-only and shows whether the active turn is **Working**, **Idle**, or needs attention, together with the model, repository, and last completed turn duration.

> **Status:** Full-track implementation is starting at T1. No installable release exists yet; follow `IMPLEMENTATION_PLAN.md` for the authoritative task state.

## Product direction

- Native Rust process targeting Windows 10/11 x86-64
- One-second default refresh with a two-second state-update bound
- Content-free parsing: prompts, responses, tool arguments, results, tokens, and credentials are never modeled
- Per-user PowerShell install and startup registration with no administrator privileges
- Manual local validation, packaging, tagging, and GitHub release publishing

The original concept is in [`docs/MVP.md`](docs/MVP.md). The accepted architecture is recorded in [`specs/adr/0001-native-rust-tray.md`](specs/adr/0001-native-rust-tray.md), and executable requirements live in [`specs/tray-status.md`](specs/tray-status.md).

## Start here

1. Read [`specs/CONSTITUTION.md`](specs/CONSTITUTION.md) for non-negotiables.
2. Read [`specs/CONTEXT.md`](specs/CONTEXT.md) for canonical status terms.
3. Start with T1 in [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md).
4. Use [`AGENTS.md`](AGENTS.md) for build and validation commands as each task creates them.

## Repository map

| Path | Purpose |
|---|---|
| `docs/MVP.md` | Product overview and concrete MVP scope |
| `specs/` | Constitution, glossary, acceptance spec, and architecture decision |
| `scenarios/` | Holdout user journeys used only during validation and review |
| `IMPLEMENTATION_PLAN.md` | Ordered build tasks and current status |
| `AGENTS.md` | Lean operational build and validation guide |
| `docs/audit/` | WGM documentation-audit paper trail |

## License

[MIT](LICENSE)