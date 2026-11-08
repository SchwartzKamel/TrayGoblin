# REL2 actual DRAFT stress review — non-author

## Scope and identity binding

**Reviewed:** actual authenticated, owner-only GitHub DRAFT acceptance—not pipeline reimplementation, runtime QA, public release readiness, or REL2 completion.

| Identity | Bound value |
|---|---|
| Metadata candidate | `6033ad42d0d915f9a6768212be356e050bac84b5` |
| Candidate tree | `37c4807fc80ecc851455e198076ac1008d70e79d` |
| Base / tagSource | `9fc0c55ca4c8c2ebabee49b371216eda1afe51c9` |
| Approved dispatchSource | `9fc0c55ca4c8c2ebabee49b371216eda1afe51c9` |
| Annotated `v0.1.0` tag object | `cbb9ec713f2fc6e36aba6faf3ccf2413c66ffdc9` |
| Source build / artifact | `42440178931` / `13601357116` |
| Failed / successful Release runs | `42440507514` / `42440682582` |
| Release / asset IDs | `437995827` / `722702070`, `722702122` |

The metadata candidate is **not** the build, tag, or dispatch source. Their current equality to `9fc0c55…` does not collapse those distinct identities.

Evidence aliases:

- **E:** `/home/lafiamafia/.cache/traygoblin-wgm/release-ci-20261105/C1-completion`
- **O:** `E/rel2-rugged-evidence`
- **Checkout:** `E/REL2-review-checkout`

I read the installed rugged skill completely, the protected governing documents, release specification, candidate diff, frozen state/hash manifest, and prior C1 review. Every diff used the sole allowlist `.wgm/IMPLEMENTATION_PLAN.md`, with scenario exclusions supplied beforehand. No holdout was opened.

The independently regenerated diff matches `E/rel2-candidate.diff` exactly; scoped whitespace checking passed. Its SHA-256 is:

`9b198eb5f6a77acdf6dbaf39c69cbdd3c7bc11711b830319fbb6c2eed106eae4`

Protected governing bytes match base, candidate, and both worktrees. The shipped verifier/publisher and CI workflows examined are byte-identical to accepted `e99bac1`. Accepted C1 mechanical/runtime boundaries were not reopened.

This is a fresh non-author context/lens using the same configured model/tool setup—not model-family independence.

## Context

The actual operator is an authenticated Linux coordinator preparing downloads for an authorized GitHub owner/collaborator. The relevant environment is real GitHub Actions, Release APIs, and authenticated downloads—not anonymous access or Windows application execution.

Observed constraints include:

- A real workflow failure leaving an owned draft with zero assets.
- Recovery requiring exact source, ownership, notes, mode, and byte correspondence.
- Actions artifact expiry, distinct from Release asset storage.
- A metadata-dirty main worktree unsuitable for live verification.

The approved sparse checkout was clean, on canonical `main`, with local `main`, `origin/main`, actual remote main, workflow identity, and annotated tag checked by shipped functions. Staffing, on-call capacity, and worst-night operator behavior are not established; none is invented here.

## Bottleneck decomposition

| Bucket | Finding |
|---|---|
| **Intrinsic design constraint** | Release preparation is multi-step and can leave an incomplete draft. Build success alone cannot establish Release success. |
| **User capacity** | The operator must distinguish metadata from immutable source and inspect ownership before retrying; generic failure output does not explain the narrower cause. |
| **Operational stress** | The first actual run failed after draft creation. A single matching dispatch recovered that same draft without substituting local rebuilt bytes. |

**Dominant risk:** confusing partial preparation or successful build CI with verified Release acceptance. The evidence directly addresses that risk.

## Simplify

Retain the existing reduction: **one manual dispatch, exact CI artifact reuse, draft-first preparation, and matching-owner recovery**.

- Avoid rebuilding/repacking, checksum rewriting, hand uploads, retagging, or destructive rollback.
- Keep the incomplete draft and its ownership/evidence rather than deleting the recovery anchor.
- Emit success only after both downloaded assets verify.

**Accepted trade-offs:** owner authentication, a manual diagnosis/retry step, and an incomplete draft remaining visible after failure. Sanitized errors protect confidentiality but sacrifice diagnostic specificity. No new mechanism or runtime change is justified by this metadata-only scope.

## Actual field stress and verification

### Fresh production verification

From the approved clean checkout, I ran:

```bash
bash scripts/verify-release.sh \
  --repo SchwartzKamel/TrayGoblin \
  --tag v0.1.0 --draft \
  --out-root "$O/downloads"
```

**Exit: 0**, during **2026-11-08 03:00:48–03:01:00 UTC**.

The shipped verifier independently established the draft, annotated tag/source, successful source build, approved matching Release workflow, asset identities, downloaded payload, and unchanged final draft.

A separate reviewer probe then made **30 actual read-only GitHub operations**, calling shipped functions rather than rebuilding their acceptance logic. **Exit: 0**, during **03:02:43–03:03:10 UTC**.

### Actual API and downloaded bytes

Current API state:

- Release `437995827`: `draft=true`, `prerelease=false`, `published_at=null`.
- Exact tag/source and generated ownership/body match.
- Both assets are uploaded; individual asset API responses agree with the Release asset list.

| Asset | ID | Bytes |
|---|---:|---:|
| ZIP | `722702070` | `978029` |
| ZIP checksum | `722702122` | `103` |

I freshly downloaded the exact source-run artifact and compared both sibling files byte-for-byte with the fresh Release download and both retained parent downloads.

```text
ZIP SHA-256
952159b3be5f27028b664efdf4c9ed69e513aa13959d092b5aa10a2ccabcb5f5

EXE SHA-256
3faeb2070dbb320d1cc8f0031cc60f6a1976392a7202d279b9da352ee3bc6fc9

Checksum-file SHA-256
813e13d9237a0f9cddd73d2ce254642225c6d4f63115c8d33378b451e4f7252c
```

Shipped payload validation passed: **17 entries**, CRC integrity, persistent-resource allowlist, x86-64 PE32+ **GUI** headers, exact source manifest, `worktreeDirty=false`, and build-time `published=false`.

Actual owner URL:

https://github.com/SchwartzKamel/TrayGoblin/releases/tag/untagged-8e2016c23150cf410382

This is an authenticated draft URL, not an anonymous-download promise.

### Actual failure and recovery

Fresh GitHub run/job APIs confirm:

- `42440507514`: completed **failure**, same approved main/source/workflow.
- `42440682582`: completed **success**, same approved main/source/workflow.
- Exactly those two same-source dispatches are present; both have `run_attempt=1`.
- Actual logs and retained dispatch commands show identical tag/build/mode inputs.

Freshly retrieved failed and successful job logs are byte-identical to the retained logs. The failed log records exit **1**, retains the owned-directory reference, and contains no success claim.

The frozen post-failure API records the same draft with **zero assets**. The retained real-code diagnosis exited **0**; its exact-body digest agrees with notes regenerated from freshly downloaded source bytes:

`c926f22f927b1385158ab59ab6b29ccecb1094228d8b8285ebda534cf65ec94a`

The historical zero-asset state was inspected, not recreated. The later real workflow uploaded both assets and logged the exact Release/source/digests. No reviewer dispatch or mutation occurred.

**Narrower first-failure cause remains UNKNOWN.** The generic sanitized log does not establish eventual consistency or another specific explanation.

### Manual-source evidence—not CI equivalence

Both retained clean-source gates exited **0**, with **22 individually recorded zero-status commands** each:

- Pre-tag: **02:47:26–02:49:21 UTC**
- Detached annotated-tag: **02:51:41–02:54:35 UTC**

Logs substantiate workspace tests141, strict Clippy, formatting, toolchain/GUI cross-build, real offline pi-host journey, separate restricted OCI, docs452, installer56, release-document controls, and packaging. The Windows Startup-shortcut skip remains explicit.

Each phase’s two packing runs match its own ZIP/checksum bytes. Both local ZIPs **and EXEs differ from CI**. This establishes neither manual/CI byte equivalence nor independent compilation reproducibility.

### Certainty rung reached

| Load-bearing claim | Rung |
|---|---|
| Candidate/protected/frozen-byte correspondence | **4** — independent fail-loud checks; all **23** frozen manifest entries match |
| Actual draft/tag/run/asset/download identity | **5** — fresh production verification against the running GitHub system |
| CI-to-Release exact byte correspondence | **5** — fresh real downloads checked by shipped payload functions |
| Same-draft failure/recovery | **5** — actual incident/retry evidence corroborated by fresh run APIs and exact logs; no reinjection |
| Historical zero-asset ownership diagnosis | **4** — inspected real-code execution, exits, hashes, and regenerated expected body |
| Manual gates and own-environment two-pack equality | **4** — inspected recorded executions and independently checked bytes |
| Windows/M6/public readiness | **Not established or admitted** |

## Verdict

**RUGGED**

For the bounded **actual owner-only DRAFT acceptance tier**, deterministic evidence shows a real incomplete preparation remained owned and recoverable, one matching retry succeeded, and authenticated downloads contain the exact successful source-CI bytes. This is not a REL2 completion or public-release verdict.

## Reservations and preservation

- First-failure diagnosis remains generic; recurrence frequency is not established.
- The hosted checkout **Node20→24 deprecation warning** is retained.
- Public/M6 acceptance remains blocked. No live field, performance, or acceptance fixtures were fabricated.
- Windows10/11 operation, Startup/sign-in/uninstall, resources, and independent/operator acceptance are not supplied by Linux tests.
- Release storage is distinct from Actions retention; continued availability after artifact expiry was not observed in this review.
- Remaining REL2 adjudication is still owed; the plan correctly remains `in_progress`.
- Both repository HEADs, index hashes, plan bytes, and statuses remained unchanged during the probe. The main worktree remains metadata-dirty; the review checkout remains clean.
- No builds, repairs/resets, repository edits, staging/commits, new authors, dispatches, tags, or Release mutations were performed.

Proof artifacts are confined to **`O/`**: production command/log/exit records, `verify-actual.py`, API/recovery summaries, manual-byte proofs, preservation snapshots, and **`proof.sha256` covering 72 files**. Final frozen-manifest and proof-hash checks both exited **0**.

## One next action

**Submit candidate `6033ad42…` with this frozen-plus-fresh evidence to the pending REL2 acceptance gate, preserving the immutable tag, separate source identities, and owner-only draft boundary.**
