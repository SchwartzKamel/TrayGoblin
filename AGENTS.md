# AGENTS.md

> These commands are forward contracts fulfilled by T1–TZ in `IMPLEMENTATION_PLAN.md`. Before a task creates its named script, fixture, or manifest, follow the plan rather than expecting the command to exist.

## Build & run
```bash
cargo test --workspace
bash scripts/validate-toolchain.sh
cargo run --bin tray-goblin-probe -- --session-root tests/fixtures/live-session
```

The GUI target is Windows x86-64. Build it locally with the repository scripts; do not treat CI as a release prerequisite.
Architecture details are planned for `docs/architecture.md` in T6; until then, `specs/adr/0001-native-rust-tray.md` is authoritative.

## Validate (backpressure)
```bash
bash scripts/demo.sh 0.1.0
bash scripts/check-docs.sh
```

- A plan task is not done until its exact validation command exits zero.
- Run Windows performance measurement with `scripts/measure-performance.ps1` before promoting a preview to stable.

## Operational notes
- Copilot state defaults to `%USERPROFILE%\.copilot\session-state`.
- TrayGoblin must never read or model event content, arguments, results, tokens, or credentials; `events::tests::does_not_model_sensitive_fields` is the deterministic contract.
- Release artifacts are built and published manually from a clean tagged commit.
- The reproducible release procedure is a required T6 deliverable at `docs/manual-release.md`.

## Codebase patterns
- Keep platform-neutral parsing and monitoring in the library; keep notification-area code Windows-only.
- Unknown Copilot fields and event types are expected compatibility input, not fatal errors.
- Tests use content-free fixtures and explain which state transition they protect.
