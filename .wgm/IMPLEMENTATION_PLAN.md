# Session attention implementation plan

Full track · Ralph-full · fresh one-task authors/non-author reviewers.
Root AGENTS.md and specs/CONSTITUTION.md govern; NEVER edit root AGENTS.md.
Active spec/glossary: `.wgm/specs/session-attention.md`, `.wgm/specs/CONTEXT.md`.
Root plan/specs remain historical MVP authority. This is the live follow-up state.

## CURRENT checkpoint — 2026-11-04 UTC
**M1–M5 ACCEPTED at implementation tier. M6 BLOCKED/unverified.**
**Ready for real Windows/WSL testing, NOT a published or release-ready build.**
Accepted commits: M1 `0d79111`, M2 `058ef74`, M3 `f4ef6de`, M4 `d94c85c`.
M4 score/helper-scope metadata: `aa617e9`. M5 product matches reviewed4e5e3c1;
all code gates/RUGGED/specPASS/qualityPASS stand. Final full-fleet audit0508 plus
fresh five-role A08 closure0535 leave NO surviving Agent action in scope; AMBER
O01 protected-root authorization/O02 genuine target-host evidence remain explicit.
Fresh M5 validator completed in tier order: tier1=97.35, tier2=98.33,
tier3=97.54; overall97.74. Each prior tier cleared95 before the next opened.
No native Windows evidence is implied by these CLI/offline-host judgments.

## Goal and boundaries
One native Windows tray identifies pi/Copilot attention across Windows/local WSL.
Pi: ALL explicit blocking dialogs including manual, plus final run errors.
Copilot: new persisted errors only; passive input capability unavailable.
Ordinary Idle/completion/abort/recovered tool failures quiet. Metadata/manual labels
only; no prompts/results/args/tokens/credentials/generated names. Native notification
plus attention-first persistent list. No daemon/server/dashboard/remoteSSH collector,
automatic terminal focus/control or implicit WSL discovery/launch. Explicit UNC
reads can wake WSL; removing that source and restarting disables those reads.

## Contract
- One repository writer at a time; authors never stage/commit/review/open holdouts.
  Parent owns acceptance/commits; validator alone opens holdouts tier1→2→3 at95.
- Done requires EXACT task gate0 plus current ruggedRUGGED and BOTH fresh non-author
  PASS at an immutable candidate, and M5's final audit/dispositions. Report/exit0
  alone is not PASS. Verify actual symbols/files/counts/source+artifact hashes.
- No product edits during reviews; no edits/builds during guarded audit. Never edit
  an active shell helper. Preserve reports/refs before retry; clear only stale run
  exit signals. No reset/stash/history rewrite, broad cleanup or user-file discard.
- Purpose-commented tests; no weakening/skips/deletions. Bugs assertion-fail on
  rejected source, pass fixed, then COMPLETE gate0. Separate rejected/fixed targets;
  contaminated shared-target M3 attempt is retained but excluded. Bounded failure
  requires wonder/reflect/new signal or safe exact WIP stop, not silent waiver.
- Synthetic fixtures/isolated HOME/XDG, verified public caches only; no user data,
  credentials or model/network calls in tests. Format only touched files.
- Core, actual offline pi1.4 host, OCI CLI, Windows compilation and actual Windows
  operation are DISTINCT. No Linux/mock substitute for M6. No publication delegated.
- External evidence root (RUN): `/home/lafiamafia/.cache/traygoblin-wgm/attention-20261103`.
  Authors: fresh pi/github-copilot/gpt-6.1-sol xhigh via bundled loop/no stored
  session/extensions/MCP. Reviews/audits: fresh CopilotCLI contexts/lenses, model
  family unqueried. No cross-family/tooling-independence/billing/speedup claim.

## Tasks

### M1 — Publish real pi attention status
- **objective:** Content-free publisher through actual installed pi loader/runner.
- **files/areas:** `integrations/pi/`, `tests/pi-{status,host}.test.mjs`, `scripts/test-pi-host.mjs`.
- **validation:** `node --test tests/pi-status.test.mjs tests/pi-host.test.mjs`
- **acceptance:** R1–R3.16KiB scalar protocol; independent lifecycle/dialog overlay,
  same restored error ID; ambiguous ID-less hooks latch one actionable tracking
  fault until reload.2s heartbeat,32 pending writes+reserved close, atomic ownership
  cleanup/privacy traps. No sensitive modeled fields or idle/busy inference.
- **status:** done
- **evidence:** Commit0d79111/runtimec41f964;33/33 (25 core,8 actual offline host),
  ruggedRUGGED/specPASS/qualityPASS. RUN/M1.R4 before/after/cap-close mutants;
  historical policy/metadata reservations retained, explicitly superseded by approved
  all-dialog/latched-fault policy. M4 adds optional manual labels/stronger privacy.

### M2 — Concurrent content-free sources and aggregate probe
- **objective:** Independent bounded pi/Copilot state/config and legacy compatibility.
- **files/areas:** `src/{config,monitor,events,attention,lib}.rs`, `src/bin/tray-goblin-probe.rs`, `tests/attention_monitor.rs`, `tests/fixtures/attention/`, `docs/operator/configuration.md`.
- **validation:** `cargo test --workspace`
- **acceptance:** R4–R5,R7 parsing,R9.16 sources,64 live rows/source,4096 membership,
 16KiB+1 pi reads/10s expiry; UUID regular files/no symlink escape. Stale residue
  cannot consume live capacity; explicit sources replace defaults/empty disables;
  historical/tool-only Copilot errors quiet. Old probe exactly SIX fields.
- **status:** done
- **evidence:** Commit058ef74;Rust93,ruggedRUGGED/specPASS/qualityPASS; publisher→
  parser/probe and read-cap/directory-pinning/saturation controls RUN/M2.R1.
  No blocking reservations; core evidence is not Windows field evidence.

### M3 — Controller, source isolation and stable provenance
- **objective:** Bounded low-noise delivery/fixed workers with recovery-safe identities.
- **files/areas:** `src/{attention,attention_worker,monitor,lib}.rs`, `tests/{attention_controller,attention_monitor}.rs`, `docs/architecture.md`, `docs/operator/configuration.md`.
- **validation:** `cargo test --test attention_monitor --test attention_controller && cargo test --workspace`
- **acceptance:** R6–R7. Coalesce/tick,3 attempts >=5s apart from completion,
  success-only ack/Refresh undelivered-only;16 fixed owned workers/bounded slots,
  no UI I/O/join/replacement. Drain/stop<250ms, other progress<1s,10 stalls/no growth,
  actual-time expiry. Checked non-recycling process-lifetime Copilot IDs; known
  parser retention by typed COMPLETE membership, never selected rows; source/partial
  loss preserves all budgets.64 parsers/4096 histories/source/<=4 reason slots,
  visible overflow; mixed64 stress. M1 lifecycle/queue/privacy unchanged.
- **status:** done
- **evidence:** Commitf4ef6de matches e1221e2b77d8ea9bc453151505c7b0288f09d770;
  exact52 targeted/136 workspace,0 ignored,ruggedRUGGED/specPASS/qualityPASS.
  RUN/M3.R4 new-error20fails/20passes; R4.1 partial-scan controls20fails/20passes,
  full34 suite10/10. Current `.wgm/rugged/M3-current.md`. Tier1=93.37 lacked M5
  continuous demo, NOT an M3 defect. No blocking reservations.

### M4 — Native Windows notifications and usable identity
- **objective:** Connect tested workers/controller to existing native tray/balloons/menu.
- **files/areas:** `Cargo.toml` (existing windows-sys flags only), `src/{tray,app,actions,icon,attention,attention_worker,monitor}.rs`, `tests/{attention_controller,attention_monitor}.rs`, `integrations/pi/{tray-goblin.js,README.md}`, `tests/pi-{status,host}.test.mjs`, `scripts/test-pi-host.mjs`, `docs/operator/{status-reference,first-run}.md`.
- **validation:** `node --test tests/pi-status.test.mjs tests/pi-host.test.mjs && cargo test --workspace && cargo build --release --target x86_64-pc-windows-gnu --bin tray-goblin`
- **acceptance:** R8 implementation tier. UI requests/drains/stops only; epoch
  freshness/controller/actual completion separate from uptime cadence. Engine/ack
  survives Explorer; bounded UTF16 NIF_INFO/tooltip/menu, persistent attention-first
  identity/capabilities/diagnostics/safe actions. Optional64UTF8-byte manual labels,
  unlabeled v1 compatibility/native singleton. No native-delivery claim.
- **status:** done
- **evidence:** Commitd94c85c matches5894cd119b80fc47bace1f36002574cc38248828;
  Node36 (26 core/10 actual host),Rust141 (68+38+19+16),0 skipped/ignored,
  Windows PE32+ x86-64 GUI/import checks; ruggedRUGGED/specPASS/qualityPASS.
  Four driver controls20/20/full38 suite5/5; separately compiled dispatch/clock/ack
  mutants assertion-fail; privacy mutant hits real poisoned getters. Actual
  `AttentionRuntime::cycle`, epoch completion, NIF_INFO and safe actions verified.
  RUN/M4.author/REPORT.md and current `.wgm/rugged/M4-current.md`.
  Spec reservations:none within tier; quality retains unverified M6 boundary.
  Fresh judge tier1=94.12 (87.67/94.67/94/96.25/98),10 independent file-free controls
  plus fixture CLI; `continuous-demo-missing`, tiers2/3 unopened. RUN/M4.judge.log.

### M5 — Package, documentation and offline attention demo
- **objective:** Installable unpublished artifact, continuous actual-host→Rust demo/setup.
- **files/areas:** `scripts/{demo-attention.sh,demo-attention-host.mjs,test-release-docs.mjs,test-attention-windows.ps1,package-release.sh,check-docs.sh,measure-performance.ps1,test-installer.ps1,test-pi-host.mjs}`, `src/bin/tray-goblin-attention-demo.rs`, `scripts/demo.sh` (package allowlist only), `install.ps1`, `uninstall.ps1`, `integrations/pi/`, `assets/`, `README.md`, `docs/`, `.wgm/docs/audit/`, `.wgm/deferred-work.md`. Extra helpers declared/allowlisted BEFORE freeze.
- **validation:** `bash scripts/demo-attention.sh --container podman && bash scripts/demo.sh 0.1.0 && bash scripts/check-docs.sh`
- **acceptance:** R9–R10. Exact package allowlist/checksum/persistent resources,
  no silent pi settings edits. TWO concurrent installed pi1.4 publishers in same
  directory→shipped Rust parser/workers/controller/actual submit. Identity/privacy,
  heartbeat/dialogs/errors/restoration/dedupe/quiet completion; old MVP unchanged.
  OCI separate cached bookworm-slim/nonroot/network-none/read-only binary+fixtures,
  real JSON/cleanup. Windows collectors reject Linux, bind cadence/PID/path/digest.
  Indexed accurate setup/limitations/UNC/suppression docs; final qualitative audit.
- **status:** done
- **acceptance record — 2026-11-04 UTC:** Parent accepts M5 after exact full chain exit0, current runtime RUGGED + both fresh non-author product PASS at4e5e3c1, metadata author gate0 with107 source/87 product identity checks, independent scoped specPASS/qualityPASS at5ab2703, and complete four-persona+writer A08 closure report0535. A08 fixed/closed; A01–A07 independently closed; no Agent action survives the bounded delta. Overall docs AMBER is preserved for O01/O02, not relabeled GREEN. Parent verified no runtime/resources/tests/docs-fleet changes against reviewed product and inspected actual reports/counts. Raw scoped reports RUN/M5.A08.{spec,quality,audit}.log; exact gate RUN/M5.A08/gate.exit0. Counts:14 real-host phases/5 adapter requests, restricted OCI CLI JSON/cleanup,141 Rust tests,56 installer passes/one explicit Windows shortcut skip,408 docs checks. Prior standalone Node36 gate is M4 evidence, not invented new M5 execution. No native Windows/WSL/resources proof. Exactly one task advanced, M5; M6 stays blocked.
- **CURRENT exact evidence:** Candidate4e5e3c1f3106e4f958fc9af15afd9cdc707dd794;
  gate0, fresh scoped ruggedRUGGED/specPASS/qualityPASS after full R3 reviews at
  b0b244f. All nine delta paths inspected; runtime/control/package trees unchanged.
  RUN/M5.validation.* and M5.parent-gate-R4/:14 actual-host phases/5 adapter requests,
  zero sensitive/network/busy reads; separate restricted OCI JSON/absence.
  Rust141/0 ignored; installer56pass/0fail/1 Windows-only shortcut SKIP;
  Windows cross-build/package, exact MVP/allowlist/checksum/two-pack; docs408/0 skipped.
 12 Bash+2 PowerShell canonical fences+4 audit fences,44 injected failures/run,
  success/canary controls including actual ZIP; old-source behavioral failures
  retained. No fresh M5 standalone Node36 run claimed. No M5 holdout score.
- **Proof/provenance:** RUN/M5.{finalize,cadence-fix,docs-fix,status-fix}/REPORT.md;
  M5.author/REPORT.md is absent (first author timed out), not fabricated.
  Status-fix artifact manifest actually has12 entries, docs-fix6: parent rechecked
  all12; prior review13-count claim is a preserved measurement disagreement, NOT
  a missing artifact. Three14-digest journeys/artifact hashes verified. ZIP
  `9485a19ea81fa03763a11dcf8802132389db9b215d72a31d80ccd0a3e3f5e1a5`;
  PE `f8acac734ebf670cec55017a952664ff0dcb12a362659ea2a17c1503a56f3602`.
  Manifest honestly says pre-freezeaa617e98/dirty/unpublished; no official release.
- **final independent validation / artifact record:** Fresh tiered judge reports97.35/98.33/97.54, overall97.74, source/runtime/digests inspected and ten file-free policy tests independently passed. Supplied actual-host/OCI/compiler evidence stays attributed, not called independent rerun. Boundary-combination partials remain documented; no substantiated new product blocker. Parent then reran COMPLETE exact chain from CLEAN commitfe6e301: exit0,141 Rust,56 installer/one explicit Windows shortcut skip,408 docs,14 real-host phases/5 adapter requests and separate restricted OCI; sibling checksum verified. Archive `dist/tray-goblin-0.1.0-windows-x86_64.zip`, SHA256a24f507f3faa928f40a69cd9b2eb255ac9946fa3fc38f8f3cee2f2ae061feba9; PE SHA256f8acac734ebf670cec55017a952664ff0dcb12a362659ea2a17c1503a56f3602. Packaged resources/build.json truthfully records cleanfe6e301 and published=false. Later commits are agent metadata only; product bytes remain identical to reviewed4e5e3c1. Old9485/35fdd hashes are historical prior archive generations, not the current artifact. Exact parent log RUN/final-clean-gate.log/exit0; judge RUN/M5.judge.log.
- **CURRENT audit disposition:** Full-fleet0508 AMBER originally retained A08;
  `.wgm/docs/audit/2026-11-04T0535Z_attention-a08-closure.md` independently verifies
  A08 FIXED/CLOSED at5ab2703, with no Agent actions surviving. Complete five-role
  context/lens review; model/tool independence not claimed. Both scoped non-author
  PASS reports independently verified exact16-line delta and unchanged107/87 maps.
  Structural --check-report0 was run before indexing; qualitative AMBER remains
  solely O01 protected root/O02 genuine Windows/WSL/resources evidence. Historical
  severity/count/execution disagreements are retained, not a waiver.
- **Stop/remediation history — 2026-11-04 checkpoint15694c3:** Bounded wonder/reflect/regeneration rounds exhausted;
  do NOT extend another blind loop or waive A08. This coordinator checkpoint makes
  STOP/current next-action explicit but is NOT independent closure of A08. One
  precise next authorization: fresh metadata-only author verifies/aligns operative
  NEXT to the authoritative checkpoint, preserves dated history, reruns exact gate;
  fresh scoped non-author review and five-role audit close A08 before M5 done.
  External prompt RUN/M5.A08.plan-fix.prompt. Then parent accepts/commits and ONLY
  THEN runs judge-one.sh M5; tier1>=95 before2, then3. No new product feature work.

## Optional CI follow-up — user approved
Current source is pushed: remote main=dbe78f7 at intake, existing14 commits landed.
Standard track; fresh one-task authors/non-author reviewers. Governing spec:
`.wgm/specs/ci-test-artifacts.md`. Optional CI cannot replace manual release/M6.
CI Plan-exit: RUGGED (fresh discoverable companion,2026-11-04), report
`.wgm/rugged/ci-plan-current.md`; initial UNKNOWN interruption-test gap closed by
named self-test/deadlines/owned-partial/retry controls. Constitution/coverage/
consistency PASS, no placeholder tasks, first CI1 bounded, CI2 is exact live demo.
Readiness98/100; prior goal/scope explicitly approved, no further material unknown.

### CI1 — Configure optional Windows-target test builds
- **objective:** Add a pinned, read-only GitHub workflow that runs canonical core/cross-build/package scripts and uploads Windows test ZIPs.
- **files/areas:** `.github/workflows/windows-test-build.yml`, `scripts/test-ci-workflow.py`, `scripts/verify-ci-build.sh`, `README.md`, `docs/agent/development.md`, `docs/agent/release-responsibilities.md`, `docs/operator/installation.md`. CI1 author declares tightly coupled `scripts/verify-ci-build.py` (stdlib live verification and isolated offline controls), `docs/agent/testing.md` (stale M5 inventory) and `docs/agent/documentation-standards.md` (obsolete no-hosted-pipeline rule), plus `docs/README.md` and `docs/manual-release.md` (active index/status and stale no-CI sentence only; release fences unchanged); no runtime/installer/packager or protected root AGENTS changes.
- **validation:** `actionlint .github/workflows/windows-test-build.yml && python3 scripts/test-ci-workflow.py && bash scripts/verify-ci-build.sh --self-test && bash scripts/check-docs.sh && bash scripts/validate-toolchain.sh && bash scripts/package-release.sh 0.1.0`
- **acceptance:** C1–C3. Ubuntu24.04 builds Windows x86-64 GNU using existing scripts; push main/manual triggers, contents:read only, pinned checkout/upload SHAs, no persisted credentials/secrets/releases, success-only nonempty ZIP+SHA upload retained14days. Semantic policy/mutants and YAML syntax checked. Live verifier created with exact commit/run/artifact/path/checksum/PE/clean manifest validation; no destructive download directory reuse. Docs clarify signed-in test-artifact downloads, current M1–M5 acceptance and no CI/release dependency. M6 stays unverified.
- **status:** done
- **parent acceptance — 2026-11-05 UTC:** Immutable candidate `21959f353a5c3d801ddf4c8d041e5a17d1e47b1f`, `refs/wgm/candidates/CI1`; independent EXACT gate0 (`RUN/CI1.parent-gate.log`/`.exit`, measured65s), current `.wgm/rugged/CI1-current.md` RUGGED, fresh non-author specPASS and qualityPASS (`RUN/CI1.{spec,quality}.log`). Parent verified all13 candidate blobs,12 protected/runtime/package identities, runner141/0 failed/0 ignored, docs427/0 skipped, policy25+6, verifier27+3 executable mutants+16 payload controls, actual x64GUI PE/17 ZIP entries/CRC+sibling checksum. Partial2.607796s/gh124/verifier1/no URL/sentinel unchanged/owned partial retained/different retry; raw parent controls copied intact from `/tmp` into RUN for durable retention. Quality additionally tested TERM-resistant parent+descendant timeout5.203s. Reviews are fresh context/lens independence, model/tooling-family independence unqueried; no author active during review. Reservations: hosted/public tooling and actual exact-SHA download remain CI2, local archive is dirty/base-SHA/unpublished, no Windows/M6/release proof. Final Standard docs audit remains the CI2 handoff obligation, not a CI/release prerequisite. Parent will commit only declared CI1 paths plus this acceptance/current rugged record; product/workflow bytes must equal candidate.
- **CI1 author gate — 2026-11-04:** Exact validation above exit0; `RUN=/home/lafiamafia/.cache/traygoblin-wgm/ci-20261104`, `CI1.gate.log`/`CI1.gate.exit`. actionlint1.7.12; PyYAML6.0.3 BaseLoader policy:1 approved/25 unsafe policy/6 unsafe input controls. Verifier OFFLINE:27 command controls,3 executable mutants rejected for intended failures,16 payload counterfactuals. Partial download gh exit124/verifier exit1 in2.531s, no success URL, sentinel unchanged, owned partial retained at `RUN/ci-verifier-self-test-1ew2zy0z/caller/tray-goblin-ci-qvdq40na`; byte-verified retry in DIFFERENT child `tray-goblin-ci-mte8pujq`. Logs/summary retained there; these stubs are NOT GitHub boundary evidence. Docs427 checks/0 skips (44 injected release-doc failures plus success/swallowed-error controls); Rust141 passed/0 failed/0 ignored; actual Windows GUI PE cross-build passed; canonical package twice matched,17 entries/CRC+sibling checksum verified. `RUN/CI1.artifact.json` and `RUN/artifact/` retain ZIP SHA256 `fa89da55fd8ca0cbfa80ba7362e658b4ea3b8560761c34d78555b129c1b975da`, PE SHA256 `f8acac734ebf670cec55017a952664ff0dcb12a362659ea2a17c1503a56f3602`. Manifest truthfully records author HEAD `c0d64e56e1e56254dc976bc37c5918d37dccc799`, worktreeDirty=true/published=false; NOT clean release/live-CI identity proof. Gate used synthetic isolated HOME/XDG, original public Cargo/Rustup caches and only verified public PyYAML module; initial isolated-HOME module-not-found attempt retained as `CI1.gate.initial-pyyaml.*`, resolved by tooling-only module visibility (no runtime dependency). Files are exactly declared areas above; active stale CI/M5 docs swept, release fences unchanged. Root AGENTS/runtime/installer/packager/existing tests unchanged; no bytecode junk, staging/commit/push/trigger/release or independent review claimed. Parent freeze/fresh reviews/acceptance still required; CI1 remains in_progress, CI2 pending and M6 blocked.
- **validation-signal follow-up from initial UNKNOWN plan review:** Verifier self-test above MUST include partial-download interruption with2s test-only deadline/nonzero within7s/no success URL, unchanged caller sentinel, retained owned partial directory/evidence, successful retry into DIFFERENT owned child. Production discovery120s/watch1500s/download120s kill-after5; wrong/missing/failed/cancelled/expired/ambiguous inspection cases fail. Offline controlled stubs do not replace actual CI2. Initial plan report retained `.wgm/rugged/ci-plan-initial.md`; revised exact design gets companion rerun before author.

### CI2 — Push CI and prove actual artifact download
- **objective:** Prove the optional workflow and artifact on GitHub, not only local syntax/build.
- **files/areas:** `scripts/verify-ci-build.sh`, `.wgm/IMPLEMENTATION_PLAN.md`, `.wgm/rugged/`, `.wgm/docs/audit/`.
- **validation:** `bash scripts/verify-ci-build.sh --repo SchwartzKamel/TrayGoblin --commit "$(git rev-parse HEAD)" --out-root .wgm/ci-downloads`
- **acceptance:** C4–C5. Push without force; exact current main workflow run exits success; one nonexpired artifact downloads to owned child directory. ZIP/checksum and packaged PE x64 GUI/clean source/published=false identity verified. Record exact remote/run SHA, artifact ID/link, command/exit/digest. If a real job fails, retain logs and fix the actual cause. Final metadata push checked again at latest remote HEAD; no publication/Windows-runtime claim. Final Standard docs audit and independent satisfaction check recorded.
- **status:** pending

### M6 — Real Windows/WSL field and resource evidence
- **objective:** Actual delivery/recovery and50MB/5% budget before release readiness.
- **files/areas:** `scripts/test-attention-windows.ps1`, `scripts/measure-performance.ps1`, `.wgm/rugged/`, this plan.
- **validation:** `pwsh -NoProfile -File scripts/test-attention-windows.ps1 -ExecutablePath target/x86_64-pc-windows-gnu/release/tray-goblin.exe -EvidencePath .wgm/windows-attention.json && pwsh -NoProfile -File scripts/measure-performance.ps1 -Path target/x86_64-pc-windows-gnu/release/tray-goblin.exe -DurationSeconds 30 -JsonPath .wgm/windows-performance.json`
- **acceptance:** R11–R12. Genuine native+WSL dialogs/errors/Copilot, quiet3P,
  deadlines/dedupe/suppression/list/reload/crash/Explorer no replay, WSL shutdown/
  native isolation/actual UNC wake/recovery/removal/quit. Real UTC/buildSHA/executable
  digest/PID/path/Windows+WSL versions/source kinds/load/observations; matching
  performance digest. Native installation/Startup/process safety also unverified.
- **status:** blocked
- **reason:** No actual Windows/WSL runtime. Operator action; no promotion/publication
  until genuine field+performance proof. Linux/mock/syntax/compilation cannot satisfy.

## History, dissent and operative NEXT
All earlier current-looking handoffs are SUPERSEDED by this checkpoint. Full
pre-stop plan retained RUN/M5.plan-before-stop.md and reviewed candidate4e5e3c1.
No tracked history/runtime scratch was deleted; raw reports/refs remain.
- M3 WIP79e9220/e193b6c and plans5122dbd/20a4995 preserve earlier spec dissent despite
  green122 tests; edec9fa preserves later partial-membership quality dissent/132.
  Stronger compiled R4/R4.1 fixes received accepted136; no inference approach reopened.
- M5 R0 timeout1800s preserved RUN/M5.R0/. Empty wrapper and isolated-HOME cache
  failure explicitly excluded; public cached image restored with no pull.
- Rejected25df263: both reviewers CHANGES-REQUESTED on cadence aliases/default.
 33 actual AST controls compile/execute; rejected24 behavior failures include all
  three pinned mismatches; separate fixed56/0/1 and COMPLETE gate0. Ref
  refs/wgm/candidates/M5-before-cadence-fix; RUN/M5.R1-rejected/, M5.cadence-fix/.
- RED0405 audit: genuine A01–A06 repaired by fresh author; actual rejected101442e
  source/ZIP fences execute and assertion-fail on fail-through, not syntax/missing
  markers. Full fresh R3 reviews atb0b244f. Ref M5-before-docs-fix; RUN/M5.docs-fix/.
- AMBER0442: A01–A05 closed; A06/A07 author corrected, fresh R4 scoped reviews at
  4e5e3c1; current AMBER0508 verifies those closed but A08 remains. Reports immutable;
  severity/execution/count dissent preserved, including denied principal replay,
  no verbatim online provisioning claim, and12-versus13 manifest normalization.
- **OPERATIVE NEXT:** CI1 offline acceptance is complete. Parent pushes the accepted
  optional CI commit, then executes CI2's exact real GitHub download gate, fresh
  validator and one final Standard docs audit before handoff. M1–M5/A08 and97.74
  stand; Operator M6 remains required for real Windows/WSL behavior/resources.
  No tag, official release/publication/promotion or protected AGENTS.md edit.

## Telemetry and consented workflow harvest
Full-request wall time, allocated-lane capacity, active-agent time, parked time,
peak concurrency, critical-path duration and total timed/missing turn/lane counts
were not fully instrumented and are UNAVAILABLE, not estimated. Ready-to-test
validation finished2026-11-04T05:47:11Z before this reporting overhead. Measured
partial coordinator windows are6755s and10659s; these are subprocess elapsed
windows, not whole-request wall/active-agent/lane time. Raw runner inventories:
metrics.tsv12/12 timed author invocations,4445s CLI elapsed; strong-author-metrics.tsv
6/6 timed,4786s CLI elapsed (includes timeout). Other parent/reviewer/diagnostic
turns and lane inventory are missing. Do NOT call CLI lifetime agent-hours, billing
or causal speedup. Final exact chain measured104s. Raw files retained externally.
Project consent:true/auto_report:true was honored: a SINGLE pre-anonymized generic
workflow lesson passed courier dry-run/scrub/residual guards and open-issue de-dup,
then published to https://github.com/agent-frontier/wgm/issues/192. No project code,
host identifiers, raw ledger or secrets were reported. The delegated courier's
path-approval denial is recorded separately; authorized parent helper ran the
actual safety gates and publication successfully.
