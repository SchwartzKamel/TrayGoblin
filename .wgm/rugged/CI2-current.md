# CI2 independent live-evidence review

**Verdict: RUGGED**

Passes **CI2's actual C4-C5 live boundary and bounded metadata accuracy only**, not full-app readiness, M6, Windows operation/performance or release readiness. Review date: 2026-11-05 UTC. Fresh non-author review using the fully read discoverable `/home/lafiamafia/.agents/skills/rugged/SKILL.md`; writers paused.

## Triage and source checks

Immutable candidate `d526ad3e940a3f3268750f8d80ba7688c5a79689` resolves from `refs/wgm/candidates/CI2`; actual pushed source/base and checkout HEAD are `1ab1f7fa107452d27fc4c7eb4268bb118a8d639b`. This is an evidence-record candidate, not the commit compiled by the cited run.

Let `RUN` denote `/home/lafiamafia/.cache/traygoblin-wgm/ci-20261104`, and `OWNED` denote `.wgm/ci-downloads/tray-goblin-ci-ot8bjka4`. Read `RUN/CI2.paths` before the diff: its sole allowlisted path is `.wgm/IMPLEMENTATION_PLAN.md`. Independently generated base-to-candidate diff matches `RUN/CI2.diff` exactly, SHA256 `321bd4c480a33b3b4b16a60f8c7e14f740b3c6b6a7cfc33c704d13aa1ee2f74f`. The delta records live evidence and changes CI2 from pending to in_progress; it does not declare the remaining handoff complete.

Read protected `AGENTS.md`, `specs/CONSTITUTION.md`, `.wgm/specs/ci-test-artifacts.md`, the CI2 entry and prior `.wgm/rugged/CI1-current.md`. Independently checked protected/workflow/verifier/canonical-script blob identities and runtime/assets/integrations/tests tree identities between source and candidate. Accepted CI1 candidate `21959f353a5c3d801ddf4c8d041e5a17d1e47b1f` differs from pushed source only in CI1 plan/review metadata; its implementation is unchanged.

## Context and bottleneck decomposition

Actual operators are the signed-in Windows/WSL multibox user obtaining a test ZIP and the Linux maintainer. Actual build environment is GitHub-hosted Ubuntu 24.04, dependent on public apt, Rust/Cargo and Actions services. No extra staffing, training or always-available services are assumed.

**Intrinsic design constraint:** Cross-compilation, checksums and PE decoding establish package identity/structure, not interactive Windows behavior or trustworthy signing. **User capacity:** Authentication, two ZIP layers and expiration must be legible, with a local build fallback. **Operational stress, dominant:** Public dependency failures, cancellation, expired/wrong artifacts and partial downloads must remain visible failures rather than successful delivery claims.

## Simplification and trade-offs

Retain the existing canonical build/package path, two pinned official actions, read-only repository authority and success-only two-file upload. No release automation, extra jobs/services/cache actions or CI release prerequisite is justified. Accept unsigned, authenticated, expiring test artifacts instead of permanent anonymous release assets; accept public stable-toolchain variability instead of claiming cross-host reproducibility.

Downloads remain atomic at the verification boundary: a fresh owned child, retained failed/partial evidence, a different retry child, and no success record/URL before complete verification. Retention costs disk space but preserves ownership instead of deleting caller data or dropping failed cleanup evidence. Production discovery/watch/download budgets remain 120/1500/120 seconds with kill-after 5 seconds; separate inspection calls mean these are not one aggregate wall-clock deadline.

## Field evidence and certainty

Required evidence is an actual exact-source successful hosted run, API-bound single nonexpired artifact, successful real watch/download, and decoded downloaded payload matching checksum, resource layout, PE and clean/unpublished source manifest. That evidence is present:

```bash
bash scripts/verify-ci-build.sh --repo SchwartzKamel/TrayGoblin --commit "$(git rev-parse HEAD)" --out-root .wgm/ci-downloads
```

`RUN/CI2.live.sh` records this exact command; `CI2.first.sha`, `.log`, `.exit` and `.seconds` bind it to pushed source above, exit **0**, elapsed **67 seconds**. `OWNED/evidence/01-06.json` records real gh list/API/watch/reinspection/artifact/download commands, all exit 0, with raw stdout/stderr siblings and final `verified.json`.

| Load-bearing claim | Independently checked evidence | Certainty reached |
|---|---|---|
| Actual pushed source and successful hosted execution | Fresh read-only GitHub branch/run/jobs APIs: main equals source SHA; run [41949924649](https://github.com/SchwartzKamel/TrayGoblin/actions/runs/41949924649), attempt 1, push/main, exact workflow, completed success. Created 01:38:54Z, updated 01:40:04Z. Retained watch succeeds; all job steps succeed on Ubuntu24.04. | Rung 5: retained actual target-service execution, independently API-cross-checked; not a reviewer rerun. |
| Artifact identity and availability at review | Fresh artifacts API matches retained API bytes: exactly one nonexpired `tray-goblin-windows-x86_64`, ID [13380142776](https://github.com/SchwartzKamel/TrayGoblin/actions/runs/41949924649/artifacts/13380142776), exact run/source/main identity, 975742 outer-archive bytes, expires 2026-11-19T01:40:01Z. Actual named gh download exits 0 into OWNED. | Rung 5: real service/upload/download evidence, not controlled stubs. |
| Hosted canonical build and upload | `RUN/CI2.first.actions.log:290,422-537,597-629`: 25 unsafe policy plus 6 parser controls rejected; Rust runner counts 68/0/0/0/38/19/16/0 total 141 passed, zero failed/ignored; two packing runs match; restricted two-file upload finalizes the exact artifact ID. | Rung 5: actual hosted logs and successful job API, not inferred from local gates. |
| Downloaded package identity and safety checks | Independently executed unchanged `check_run`, `inspect_artifact`, `verify_payload` and separate stdlib ZIP/PE/manifest assertions against retained real bytes; exactly two regular siblings, filename-bound SHA256, 17 entries, CRC, nonempty persistent resources, AMD64/PE32+/GUI, exact source SHA, worktreeDirty=false, published=false and executable digest. All 10 static packaged resources byte-match pushed-source blobs. | Rung 4: fresh real-code and independent byte-decoding probes; actual byte origin is established by the live transcript above. |
| Fail-closed interruption and recovery | Accepted CI1 exact gates and both independent PASS reviews retained. Parent offline controls: 27 command cases, 3 executable mutants, 16 payload cases. Fresh inspection confirms unchanged sentinel, retained 22 partial bytes, gh124/verifier1 in 2.607796 seconds, empty failure stdout/no verified record, different retained retry independently payload-verified. `RUN/CI1.quality.log` attributes the real TERM-resistant parent/descendant test, 5.203 seconds. | Rung 4: retained executed controls plus fresh byte checks; TERM result attributed to prior quality reviewer, not rerun or GitHub outage evidence. |

Inner ZIP SHA256: `e95fa745d6004a79f6293f1e3b03796dcc476d0ded68a61fc856ccf211b07f72`.

Packaged PE SHA256: `ab0aeac884c58814a16be31162c2cc1ebc423d5e02e4c7cfe76806e5c5e2699e`.

Outer Actions ZIP digest: `4013274469cd220d24e73827e4dd33560a5f7ae05369c5b576d3e507745709a5`, matching actual upload log and GitHub API. This is distinct from the independently rehashed inner ZIP; no new outer-archive download/re-hash was performed.

## Reservations and field-test gaps

The previous CI1 live-availability gap is closed for this source/run/artifact. No failed live run or source remediation is evidenced or claimed. Public dependency outage behavior remains the unchanged fail-fast/suppressed-upload trace, not a reproduced service outage. Existing offline failure controls are not upgraded into live GitHub failure evidence.

Actual logs retain Rust atomic-method and Actions Node deprecation warnings, including forced Node24 execution of the pinned actions. They did not prevent this run; they are future compatibility exposure, not a reason to invent permanent availability. Hosted/local PE digests differ; matching double packing establishes this build's packaging repeatability, not reproducible compilation across hosts.

Operator instructions (`docs/operator/installation.md:18-44`) preserve signed-in Actions downloads, outer/inner ZIP distinction, 14-day expiry/local fallback and unsigned optional test status. The clean-source manifest is not a clean-tagged release assertion. No Releases publication, Windows setup/Startup/runtime, native/WSL notification behavior, pi-host/model delivery or 50MB/5% performance conclusion follows. Manual clean-tag release and genuine M6 gates remain required; protected root instructions and no-CI-prerequisite policy remain intact.

Final Standard documentation audit and separate authorized satisfaction validation remain handoff obligations; neither is waived or represented as completed here. The final metadata push requires another actual latest-remote-HEAD run/artifact check recorded externally, not a perpetual evidence-commit/build loop. This review promises neither future artifact availability nor success for that later commit.

No root or `.wgm` holdout directory, validator report/prompt, or issue/Hive material was read. No delegation, build/test/download rerun, product/doc edit, staging, commit, push or publication occurred. Only this report is written.

**One next action:** After the authorized final metadata push, verify its actual latest remote HEAD with the exact CI2 command and retain the new run/artifact identity and verified digests externally.
