# Documentation standards

**Audience:** Agents

Rules for writing and reviewing documentation in this repository, and what
`bash scripts/check-docs.sh` enforces automatically.

## Audiences

Every page is written for one of two audiences, named exactly:

- **Operators** — people who install, run, configure, and troubleshoot TrayGoblin on their own
  machine.
- **Agents** — people and automated agents who change this repository.

Do not invent other audience names, and never label an audience as anything else — the checker
rejects any other value. A reference page that genuinely serves both uses
`**Audience:** Operators and Agents`.

Declare the audience on its own line immediately below the page title:

```markdown
# Configuration

**Audience:** Operators
```

Directory placement follows the audience:

| Directory | Required label |
|---|---|
| `docs/operator/` | `Operators` |
| `docs/agent/` | `Agents` |
| `docs/` (top level) | Any of the three values |

## Action language

Where a responsibility differs by audience, name the owner in the sentence: "the operator verifies
the checksum on Windows", "the agent tags the validated commit". Instructions inside an operator
page are addressed to the operator; instructions inside an agent page are addressed to the agent.
Never present a manual Windows result as if a deterministic command had produced it.

## Accuracy rules

1. **Describe implemented behaviour only.** If something is planned, say who owns it and where —
   for example "owned by task TZ in `IMPLEMENTATION_PLAN.md`".
2. **Keep the deterministic and manual layers separate.** Deterministic checks run on any host with
   the toolchain; tray interaction and performance measurement require real Windows.
3. **State the promotion gate.** Any page that discusses releases must say that the Windows
   performance measurement must pass before promoting a preview build to stable.
4. **Never claim CI.** No hosted pipeline builds, validates, or publishes this project.
5. **Use canonical vocabulary** from [`specs/CONTEXT.md`](../../specs/CONTEXT.md): **Idle**,
   **Working**, **Attention needed**, Active session, Active directory, Turn latency. Avoid the
   network-flavoured latency phrasing listed there, and avoid informal synonyms for Working.
6. **Stay content-free.** Documentation must not include real prompt, response, tool, token, or
   credential data, or machine-specific absolute paths from a contributor's home directory.
7. **Version references must match `Cargo.toml`.** Archive names and packaging commands use the
   current package version.

## Link rules

- Use relative links between repository files. `docs/README.md` is the index and must link every
  page under `docs/` outside `docs/audit/`.
- Anchors must match a real heading in the target file, using GitHub's slug rules: lowercase,
  punctuation dropped, spaces replaced by hyphens.
- Do not link to external URLs for anything a reader needs in order to complete a documented
  procedure; the checker performs no network access and cannot validate those.

## Files that must exist

`README.md`, `docs/README.md`, `docs/MVP.md`, `docs/architecture.md`, `docs/manual-release.md`,
`scripts/check-docs.sh`, the six operator guides, and the four agent guides. Removing or renaming
one is a failing change until the checker's required-file list and the index are updated with it.

## Shared-state and protected files

- `AGENTS.md` is protected operational guidance. Do not edit it without explicit operator approval
  that names the file and scope; documentation pages link to it instead.
- `IMPLEMENTATION_PLAN.md` is shared lifecycle state. The active task updates its own status and
  validation notes after the gate passes, but documentation work does not redefine other tasks.
- Requirements under `specs/` change only when implementation or holdout evidence exposes a real
  contradiction. Holdout files under `scenarios/` remain validation-only.

## What the checker verifies

`bash scripts/check-docs.sh` is deterministic and offline. It validates:

| Check | Failure looks like |
|---|---|
| Required files exist | A required page was deleted or renamed |
| Audience labels | A missing, misspelled, or directory-mismatched `**Audience:**` line |
| Forbidden vocabulary | A non-canonical audience name, a non-canonical latency term, or a contributor's absolute home path |
| Internal links | A relative link to a missing file, or an anchor with no matching heading |
| Index completeness | A page under `docs/` that `docs/README.md` does not link |
| Required commands and phrases | A page that lost the command or gate statement it must contain |
| Script references | A documented `scripts/…` path that does not exist and is not a declared planned script |
| Version consistency | A documented archive name or packaging command that disagrees with `Cargo.toml` |
| Stale status claims | Unfinished-work wording, or a task described as pending that `IMPLEMENTATION_PLAN.md` marks done |
| Shell syntax | `bash -n` failing on a shipped shell script; `shellcheck` also runs when installed and is reported as skipped when not |

Run it before handing over any documentation change:

```bash
bash scripts/check-docs.sh
```

## Adding a page

1. Put it in `docs/operator/` or `docs/agent/` unless it is a shared reference.
2. Add the title and the `**Audience:**` line.
3. Link it from [`docs/README.md`](../README.md), in both the "Start here" table and the matching
   section list.
4. If it is a required page, add it to the required-file list in `scripts/check-docs.sh` and to the
   list above.
5. Run `bash scripts/check-docs.sh`.

## Related

- [Development](development.md)
- [Testing](testing.md)
- [Documentation index](../README.md)
