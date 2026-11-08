## Rugged review — FRAGILE

**Bound candidate:** `f748cead410c1a45daf731f931cdce101972d498`  
**Tree:** `6c2209f92155a4290359284c856b93caee8f96d9`

Current index equals the candidate; worktree equals the index. Index SHA256 remained unchanged. Protected AGENTS/constitution, existing CI, collectors and byte verifier are unchanged.

**Context:** Linux release maintainer; Windows/WSL operator; authenticated GitHub drafts and partially completed uploads. Dominant risk is operational: public admission assumes a careful owner supplies prerequisites the publisher cannot check.

### Blocking findings

1. **Public admission omits required checklist and independent acceptance.**  
   `scripts/publish-release.py:500–505,704–708,795–821` admit publication using only field/performance JSON. Neither collector records standard-user installation, Startup creation/sign-in, uninstall, the complete manual UI checklist, or independent M6 acceptance. These remain explicit prerequisites at `docs/agent/testing.md:173–195` and `docs/manual-release.md:330–331`; missing independent acceptance requires stopping at `docs/manual-release.md:246`.

   **Falsifying control:** Called the unchanged `prepare()` with complete collector-shaped reports but no checklist/acceptance record, plus a separate explicitly failed acceptance file. Both `prerelease` and `stable` exited **0**, issued `PATCH draft=false`, and emitted verified success against the offline transport. No real Windows result was supplied.

   **Required correction:** Fail closed on missing/failing prerequisite acceptance, bound to tag/build, archive/EXE and report digests. A generic `passed:true` or source identity alone must not substitute for the checklist and independent review.

2. **Unsupported Windows performance evidence can authorize stable publication.**  
   `scripts/publish-release.py:466–470` checks only whether `windowsVersion` contains “windows”; it does not establish Windows 10/11. This is weaker than `docs/agent/release-responsibilities.md:48–52` and `docs/manual-release.md:328`.

   **Falsifying control:** Changed only the performance OS to `Microsoft Windows NT 6.1.7601.0`—Windows 7—while retaining matching build/EXE identities and a Windows 11 field report. Both `performance_gate()` and the complete stable `prepare()` accepted it; the latter issued the publish PATCH and exited **0**.

   **Required correction:** Validate supported resource-measurement OS provenance, not a substring, before either public mode.

3. **An unrelated running release job preempts verification of the correct successful job.**  
   `scripts/verify-release.py:94–105` watches each incomplete dispatch before establishing its relationship to the requested draft. Different-tag dispatches can coexist under `.github/workflows/release.yml:31–33`.

   **Falsifying control:** Supplied unrelated running run **99**, followed by successful matching run **29**. Failure from `watch 99 --exit-status` aborted discovery before inspecting run 29. Positive control with run 99 already completed/failed correctly skipped it and matched 29.

   **Required correction:** Inspect matching successful candidates before waiting; unrelated job failure must not block the requested draft’s verification.

### Executed evidence

- Exact offline controls exited **0**: actionlint; release policy **28+6**; publisher self-test **141 command / 7 path-JSON / 2 executable-mutant controls**; existing CI policy **25+6**.
- Interrupted second upload failed in **2.775s**, emitted no success URL, retained the first asset/partial evidence and caller sentinel; matching retry succeeded in a different owned child.
- Tag/source/version, same-byte promotion, protected published/different drafts, mandatory field cases and resource flags were exercised by those controls.
- Author full-gate logs/digests were inspected and matched; not relabeled independent execution or acceptance.

Evidence: `/tmp/TrayGoblin-REL1-rugged-ZreUxQ/` — `offline-controls.log`, `independent-controls.json`, `unsupported-os-public-admission/result.json`, and self-test `summary.json`.

**Certainty:** rung 4 for these code-path findings. No live Release, actual Windows/M6, holdout, model/service, tag, push or dispatch evidence is claimed.

**Highest-leverage next action:** Replace two-report public admission with complete, digest-bound prerequisite and independent-acceptance admission. Preserve draft-only preparation, exact-byte reuse and non-destructive retries; accept additional reviewed evidence rather than an idealized-owner assumption.
