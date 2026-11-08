Candidate: **`6527fa9d38829091e44e4983b0571bb038b15489`**
Fresh non-author review; same configured model/tools, independent context/lens only. Scope: the five permitted DOC1 paths, scenarios excluded before diff inspection.

### 1. Context
Operators are release owners provisioning tools, recovering failed hosted jobs, and collecting genuine Windows evidence. Offline checks, authenticated draft verification, and public authorization must remain distinct.

### 2. Bottleneck
- **Design:** immutable draft ownership and evidence-before-mutation admission.
- **User capacity:** missing tooling setup and ambiguous collector sequencing.
- **Operational stress:** ephemeral runner scratch, generic failures, and errors occurring before evidence allocation.

DOC1 addresses these instruction risks without changing production behavior.

### 3. Simplify
A1–A6 are substantively corrected: one linked tooling recipe; embedded performance-first installed-archive M6 pair with immediate exits; honest hosted/local recovery; fresh-version public bindings before initial dispatch; opening REL2 pointer; syntax-first preallocation recovery.

Trade-off: additional explicit instructions, but no new pipeline machinery or policy exception.

### 4. Field-test
Independently ran `E/docfix-rugged-evidence/check.py`: **exit 0**. Inspected author baseline/fixed raw proofs and exact complete gate command/log/exit—not rerun builds or runtime QA.

Full check stdout:
```text
candidate 6527fa9d38829091e44e4983b0571bb038b15489
PASS five-path diff only
PASS pinned diff matches Git
PASS plan proven M6/opening bytes (parent status metadata separate)
PASS proof bytes .wgm/specs/release-publishing.md
PASS proof bytes docs/agent/development.md
PASS proof bytes docs/agent/testing.md
PASS proof bytes docs/manual-release.md
PASS governance/runtime/CI/scripts/collectors/Cargo/assets object identities
PASS author changed only five doc paths
PASS current protected bytes match intake
PASS immutable annotated tag
PASS original 12 Bash + 2 PS + JSON schema bytes/order
PASS M6 exact manual installed performance-first pair
PASS M6 command bound to retained executed proof
PASS published recipe bound to actual public-tool execution
RESERVATION GitHub anchor mismatch .wgm/IMPLEMENTATION_PLAN.md -> ../docs/manual-release.md#step-8-operator-verification-on-windows
RESERVATION GitHub anchor mismatch docs/agent/testing.md -> ../manual-release.md#step-8-operator-verification-on-windows
PASS scoped local links/anchors 53
PASS baseline assertion result
PASS fixed assertion result
PASS exact M6 offline none
PASS exact M6 offline performance
PASS exact M6 offline field
PASS real public recipe exit0
PASS recipe failure seam python3
PASS recipe failure seam actionlint
PASS recipe failure seam timeout
PASS recipe failure seam sha256sum
PASS recipe failure seam gh
PASS whole exact REL1 gate exit0
PASS complete gate controls/docs/Rust141/GUI PE receipts
PASS draft-only bindings and blocked M6 retained
PASS retained original Release ZIP/EXE identities
PASS retained original CI ZIP/EXE identities
CHECK EXIT 0 — offline DOC1 evidence only; no Windows/public inference
```

Evidence reaches **rung 4 for DOC1 instruction/control-flow claims**: baseline assertion failure with 11 failed expectations; fixed zero failures; exact published tooling recipe success plus five exit-23 seams; real outer PowerShell with native-stub collector success/failure traces. These are **offline controls, not Windows evidence**.

Exact retained gate:
```text
actionlint .github/workflows/release.yml && python3 scripts/test-release-workflow.py && python3 scripts/publish-release.py --self-test && python3 scripts/test-ci-workflow.py && bash scripts/check-docs.sh && bash scripts/validate-toolchain.sh
```
`REL2-docfix/gate-v2.{command,log,exit}` establishes the complete chain **0**: publisher **259+7+2**, docs **462/0 skipped**, canonical fences/44 injections, Rust **141**, Windows x64 GUI PE.

**Reservations:** Step-8 fragments pass the repository’s whitespace-collapsing anchor convention but differ from the independent GitHub-style slug calculation; rendered navigation was not tested. Nonblocking here because M6 embeds the complete canonical command. Initial reviewer checks also exposed snapshot/status, slug, and ZIP-root assumptions; corrected only the external checker, not repository files.

Protected governance/source identities remain unchanged. Local annotated tag still peels to **`9fc0c55…`**; original retained CI/Release ZIP and EXE digests match. Frozen draft **437995827**, asset IDs **722702070/722702122**, remain the recorded identities—not freshly queried remote state.

### 5. Verdict — **RUGGED**
**For bounded DOC1 acceptance only.** A1–A6 can close without another persona audit. Existing actual-draft acceptance stands; **M6/public remains BLOCKED**. No promotion bypass, public authorization, tag rewrite, version change, or artifact-source fix is supplied or implied.

**ONE next action:** Parent mirrors this accepted DOC1 report and records scoped A1–A6 closure against the pinned SHA, preserving the stated reservations and M6 block.
