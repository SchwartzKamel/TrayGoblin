# C1 rugged review — non-author

## Candidate and scope

- **Mode:** review; bounded mechanical lint fix.
- **Candidate:** `1e9e35f3ac96167917900894d5ea908d2b178ac8`
- **Base:** `d67a6f57913d9c450265a4ffc8a554f1ed3f439d`
- **Candidate tree:** `5300dd4a32b03975223631ee68da1b1667a3a8f6`
- **Reviewed diff paths only:** `src/monitor.rs`, `.wgm/IMPLEMENTATION_PLAN.md`.
- Scenario and holdout exclusions were supplied **before every diff**. No scenario was opened.
- Governing root `AGENTS.md` and `specs/CONSTITUTION.md` were read completely and treated read-only.
- I did not author this change. No builds, product/metadata edits, staging, commits, or network operations were performed.

Evidence aliases:

- **E:** `/home/lafiamafia/.cache/traygoblin-wgm/release-ci-20261105/C1-completion`
- **A:** `/home/lafiamafia/.cache/traygoblin-wgm/release-ci-20261105/REL2.C1`
- **V:** `E/c1-rugged-evidence`

No prior C1-specific review was found among the existing review records. Previously accepted source/CI pipeline design was not reopened.

## Context

The actual operator is the parent coordinator on **Linux**, preparing an **unsigned, owner-only DRAFT**. The immediate acceptance environment is stable Rust/Clippy **1.95.0**, with deterministic compiler and exact-gate evidence—not Windows field operation.

The shared worktree remains on `main` at the base commit; the two permitted working files match their immutable candidate blobs. This review does not establish clean-source manual/repro acceptance, actual DRAFT/download acceptance, or Windows M6 readiness. Public release remains **BLOCKED**.

## Bottleneck decomposition

| Bucket | Finding |
|---|---|
| **Intrinsic design constraint** | Production items following the test module trigger `clippy::items_after_test_module` under `-D warnings`. This is the immediate C1 blocker. |
| **User capacity** | The coordinator must bind acceptance to the new immutable source, rather than reuse the earlier successful build or confuse author evidence with parent acceptance. |
| **Operational stress** | An isolated environment initially lacked PyYAML. The exact chain failed visibly and retained that failure; it did not silently skip validation. |

**Dominant C1 bottleneck:** compiler-enforced source ordering. No runtime redesign is justified by this scope.

## Simplify

The candidate uses the smallest relevant reduction: move the unchanged production suffix before the unchanged test module.

- No lint suppression, weakened assertion, skip, or new runtime mechanism is introduced.
- The compiler’s requested ordering replaces any need for a waiver or special-case gate.
- **Accepted trade-off:** a relocation diff and changed compiler source positions; production bytes and test bodies remain unchanged.
- No resource or child-process lifecycle is changed, so speculative cleanup changes and broad runtime QA are excluded.
- The plan remains `in_progress` and labels author evidence **“not acceptance”**; it does not falsely close downstream release gates.

## Field test and independent verification

### Immutable-byte proof

I inspected `E/candidate.diff` and independently regenerated the diff with the explicit allowlist and exclusions. It matches exactly; the scoped whitespace check passed.

The probe read **the actual candidate Git blob**, not merely the mutable worktree or parent’s assertion, and established:

```text
before = P + T + S
after  = P + S + T
```

All bytes are retained; both files are **48,960 bytes**. The test module occupies candidate lines **937–1244**, with all production preceding it. Its **6 tests and 60 assertion sites** are unchanged. All six named module tests appear as passing in the retained gate log.

| Fingerprint | SHA-256 |
|---|---|
| Test module, excluding separator newline | `6998d08adaacf7b4bce5fd1df514b8e324b9540deb01f690fb0e892af0b140c7` |
| Combined production `P+S` | `b928ce0854f0541fe3ef67c28dbc2324e91077464fa7962945987ff4df3d8e6b` |
| Candidate `src/monitor.rs` | `0e04be66d9433db8291fad0f4e74f0d10ac23bf29155ababed010ab00d70a79b` |
| Scoped candidate diff | `a29e8200e3112fd3024ee5cc079affce41ff8084d431c567749ada7616b4a702` |

### Governing documents

Base, candidate, and worktree bytes are equal for both governing documents:

```text
AGENTS.md
36edeca8ac9bbe76ad61a89ceec87483c1ea13564ddc7ceca2f5088406a3d681

specs/CONSTITUTION.md
351906d8e436fa25d04a2a63d72dcf58cb98c7931163be2e439063022ab533b8
```

### Real compiler and exact-gate evidence

**Baseline:** `A/baseline.log:23` contains the actual ordering error; lines 47–49 identify `items_after_test_module`; line 53 records `rc=101`. The command and exit files agree.

**Fixed exact gate:**

```bash
cargo clippy --all-targets -- -D warnings && actionlint .github/workflows/release.yml && python3 scripts/test-release-workflow.py && python3 scripts/publish-release.py --self-test && python3 scripts/test-ci-workflow.py && bash scripts/check-docs.sh && bash scripts/validate-toolchain.sh
```

The retained command file and executable chain agree exactly. `A/exact-gate.log` contains all seven commands in order and records:

- fixed Clippy success;
- documentation **452 checks, 0 skipped** at line 45;
- **141 Rust tests passed**, including the six unchanged module tests;
- Windows-target PE cross-build validation at line 262;
- final **`rc=0`** at line 263.

The seven TSV zero statuses are **derived from successful completion of the exact `&&` chain**, not seven separately instrumented exit measurements.

Recorded UTC execution windows:

- Baseline: **2026-11-08 02:20:13–02:20:25**
- Successful fixed gate: **2026-11-08 02:22:54–02:27:45**

```text
baseline.log SHA-256
a8bf2359e5613381c562af7938f1c52f650879164856fe13ced5c413499d1f95

exact-gate.log SHA-256
d854f020de11f03d2d7b0d0fd4d00e8a08650287d43630003b8024aa2d843bc7
```

The first fixed attempt remains intact with **exit 1**: Clippy passed, then Python failed on missing `yaml`. The corrected environment records the external PyYAML path and version. This failure was not overwritten or represented as gate success.

### Provenance and preservation

- Independently rechecked **all 47 entries** in `A/evidence.sha256`; every hash matches.
- `E/author-evidence-check.exit` and `E/parent-c1-proof.exit` both equal **0**.
- Parent proof values agree with the independently checked immutable candidate.
- Original manual failure remains intact at:
  `/home/lafiamafia/release-coordinator-r1/manual-source/gate.log`
- Its retained copy is byte-identical, with SHA-256:
  `de67a3857f5bb5ee8720a383af77876dcdd2cfc53e49bf64f049feaabb20237f`.
- HEAD and repository index bytes remained unchanged during the probe.

### Certainty rungs

| Load-bearing claim | Rung reached |
|---|---|
| Exact permutation, test/assertion preservation, production ordering | **4** — independently executed fail-loud byte probe against immutable Git blobs |
| Governing documents and original failure preserved | **4** — independently executed equality/hash checks |
| Baseline failure and fixed compiler/fullgate success | **4** — inspected dated real compiler/test executions, commands, logs, exits, and rechecked hashes |
| Windows field readiness | **Not evaluated** — no rung-5 Windows field claim is made |

## Verdict

**RUGGED**

The bounded C1 ordering fix has deterministic byte-preservation proof and real baseline101/fixed0/exact-fullgate0 evidence. No blocking finding remains within this mechanical review scope.

## Reservations

- This is compiler-mechanical acceptance, not Windows field validation.
- The gate evidence was inspected and independently hash-checked, **not rerun** during review.
- Fresh spec/quality acceptance and clean-source manual/repro proof remain separate obligations.
- M6/public readiness remains blocked.
- Only probe artifacts were written under `V`; `.wgm` was not written.

Independent probe: `V/verify.py`, exit **0**. Full evidence: `V/verification.json`; hashes: `V/probe.sha256`.

## One next action

After the remaining independent acceptance checks, **the parent should run the required fresh clean-source manual/repro gate on the accepted C1 source before any tag or DRAFT operation**.
