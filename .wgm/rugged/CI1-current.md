# CI1 independent implementation review

**Verdict: RUGGED**

This passes **CI1 offline policy/build/verifier readiness only**, not GitHub artifact availability, Windows operation, performance, M6 or release readiness. Review date: 2026-11-05 UTC. Immutable candidate `21959f353a5c3d801ddf4c8d041e5a17d1e47b1f` resolves from `refs/wgm/candidates/CI1`; base: `c0d64e56e1e56254dc976bc37c5918d37dccc799`. I am a fresh, independent non-author context, using the discoverable rugged companion in **review**, not plan mode.

## Context and bottlenecks

Actual operators are the Windows/WSL user downloading a test ZIP while signed in, the Linux maintainer, and an Ubuntu 24.04 GitHub-hosted runner depending on public apt/Rust/Cargo/artifact services. No additional operator training or staffing is assumed.

**Intrinsic design constraint:** Cross-compilation and structural verification cannot establish native notification delivery or performance. **User capacity:** Actions authentication, the outer artifact ZIP versus inner installable ZIP, and expiration require explicit instructions and a local fallback. **Operational stress, dominant:** Dependency/network failure, cancellation, missing/expired/wrong artifacts and interrupted downloads must not become success claims or destroy caller data.

## Scope and simplification

Read root `AGENTS.md`, the constitution, CI spec and prior plan review completely. Inspected the bounded `CI1.diff`, all 13 allowlisted candidate blobs through `git show`, and connected canonical commands. Independently matched all 13 blobs to the frozen gate checkout. Runtime, tests, assets and integrations have identical candidate/base tree IDs; protected instructions and canonical validation/packaging scripts have identical blob IDs.

Keep one canonical build/package path, two official pinned actions, read-only repository permission, and no extra jobs, services, cache actions or publishing machinery. Accept authenticated, unsigned, expiring **test** artifacts rather than anonymous release assets. Retaining failed download children costs disk space but preserves ownership and repair evidence; retry allocates another child instead of destructive cleanup. Stable/public tooling remains network-dependent rather than hermetically pinned: visible failure is acceptable, availability is not presumed.

## Field evidence and certainty

Let `RUN` denote `/home/lafiamafia/.cache/traygoblin-wgm/ci-20261104`. Both authoritative exact gates completed with **exit 0**: `RUN/CI1.gate.log`/`.exit` and `RUN/CI1.parent-gate.log`/`.exit`. I awaited the parent rather than duplicating builds. `CI1.parent-gate.sh` confirms isolated HOME/XDG, explicit public Cargo/Rustup caches, tooling-only PyYAML visibility and the unchanged command:

```bash
actionlint .github/workflows/windows-test-build.yml && python3 scripts/test-ci-workflow.py && bash scripts/verify-ci-build.sh --self-test && bash scripts/check-docs.sh && bash scripts/validate-toolchain.sh && bash scripts/package-release.sh 0.1.0
```

| Claim | Evidence and checks | Certainty reached |
|---|---|---|
| Closed workflow authority and complete command bodies | `.github/workflows/windows-test-build.yml:3-59`; `test-ci-workflow.py:53-109,126-165`, `parse`/`validate`: one approved workflow, **25 unsafe policy + 6 unsafe parser controls rejected** in both gates. Main push/manual triggers, branch cancellation, Ubuntu24.04, 20-minute job, exactly `contents: read`, no escalation, persisted checkout credentials, user secrets, release/tag/deploy commands or broad uploads. | Rung 4: executed policy controls, source/failure traces inspected. |
| Official action pins and packaging layout | GitHub's official repositories independently resolve checkout `11d5960a326750d5838078e36cf38b85af677262` and upload-artifact `ea165f8d65b6e75b540449e92b4886f43607fa02`. Inspected pinned `action.yml` inputs and upload README's common-root layout. Upload is explicitly `success()`, only `dist/*.zip` + `dist/*.zip.sha256`, missing-files error, 14-day retention. | Rung 3: actual public identity/input inspection and walked upload contract; hosted execution not claimed. |
| Local build/package readiness | Both gates: **141 Rust tests**, no failures/ignored; **427 docs checks**, zero skipped; **44 injected release-doc failures**, canonical fences and swallowed-error controls; actual x64 Windows GUI PE, reproducible double packing, **17 ZIP entries**, CRC and sibling SHA256. Independently ran the candidate's `check_pe` and payload checks against retained bytes. | Rung 4: executed canonical evidence inspected; candidate verifier functions independently exercised. |
| Fail-closed verifier orchestration | `verify-ci-build.py:69-142,200-231`, `GitHub.command`/`discover`/`check_run`/`inspect_artifact`/`verify`: **27 command controls + 3 executable mutants + 16 payload controls** in both gates. Inspected all author command outcomes and intended mutant deaths, not just totals. | Rung 4: real orchestration through controlled gh plus inspected executable counterfactuals. |
| Download identity/content | `verify_payload`/`check_pe`, lines 145-197: exact two regular siblings, filename-bound checksum, 17-entry persistent-resource allowlist, CRC, size/symlink/encryption checks, DOS/PE/AMD64/PE32+/GUI headers, exact source SHA, `worktreeDirty=false`, `published=false`, executable digest. Controls cover SHA/dirty/published/digest, non-PE/wrong-machine/console, missing pi/setup, traversal, checksum/path/layout/truncation/CRC. | Rung 4: candidate functions exercised against actual archive and retained successful synthetic retry. |

**Walked interruption/recovery:** Production discovery/watch/download budgets are **120/1500/120 seconds**, GNU timeout **kill-after 5 seconds** (`Deadlines`, lines 57-61; `command`, 69-84). Additional inspection API calls are separately bounded; these are not one aggregate wall-clock deadline. `gh` help confirms the actual list JSON fields/filters, watch `--exit-status`, and exact-run single-artifact download into the requested directory.

The two-second offline partial-download control returned gh **124**, verifier **1**, in **2.530871 seconds author / 2.607796 seconds parent**, both below seven seconds. Independently inspected empty failure stdout/no GitHub URL, unchanged sentinel bytes and retained partial bytes. Author owned partial/retry children are `tray-goblin-ci-qvdq40na` / `tray-goblin-ci-mte8pujq` under `RUN/ci-verifier-self-test-1ew2zy0z/caller`; parent children are `tray-goblin-ci-9zw5_vkj` / `tray-goblin-ci-a5slhtf1` under `/tmp/ci-verifier-self-test-sy7d5w04/caller`. Both retries used different children. Inspection errors stop before download; success-shaped partial download stops at the sibling-file check; destructive cleanup, swallowed inspection exits and bypassed payload verification mutants hit their intended assertions. No verified record or browser URL is emitted before all checks pass.

## Reservations and boundaries

Actual archive ZIP SHA256 is `fa89da55fd8ca0cbfa80ba7362e658b4ea3b8560761c34d78555b129c1b975da`; PE SHA256 is `f8acac734ebf670cec55017a952664ff0dcb12a362659ea2a17c1503a56f3602`. Its manifest honestly identifies the **base author HEAD, dirty=true, published=false**. I confirmed the live payload checker rejects that dirty archive. It is not clean candidate-SHA or GitHub identity proof.

Clean hosted packaging is structurally possible: event-SHA checkout, preserved compiler-cache paths before HOME replacement, system PyYAML installation, disabled bytecode, ignored target/dist output, and unchanged HEAD-derived manifest logic (`package-release.sh:118-130`; `.gitignore:1-3`). The initial isolated-HOME missing-PyYAML failure is retained, not concealed. This compatibility trace stops at **rung 3**; actual Ubuntu-hosted install/build/upload/download remains CI2, not an empirical CI1 claim. Public service failures terminate the fail-fast steps and suppress upload; real service availability and TERM-resistant download recovery were not reproduced here.

Operator instructions correctly distinguish signed-in Actions downloads, outer/inner ZIPs, checksums, expiration/local fallback and no verified download until CI2 (`docs/operator/installation.md:18-57`). README and release responsibilities preserve manual clean-tag release and genuine M6 Windows/WSL/resource boundaries. No rung-5 GitHub or Windows result is claimed; the broader live/field gates remain blocking.

No holdout directories were opened, other roles delegated, issues harvested, product/docs/protected instructions edited, or staging/commit/push/release performed. The only review write is this report.

**One next action:** After parent acceptance and its authorized push, obtain CI2's actual exact-HEAD run/download evidence with `bash scripts/verify-ci-build.sh --repo SchwartzKamel/TrayGoblin --commit "$(git rev-parse HEAD)" --out-root .wgm/ci-downloads`, retaining real run/artifact identity and verified digests.
