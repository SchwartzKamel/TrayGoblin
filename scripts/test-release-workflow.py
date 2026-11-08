#!/usr/bin/env python3
"""Syntax/security policy for the optional write-token release boundary."""

import copy
import importlib.util
from pathlib import Path
import sys

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location(
    "ci_policy", Path(__file__).with_name("test-ci-workflow.py")
)
ci = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ci)
Error = ci.PolicyError
parse = ci.parse
eq = ci.require_equal

RUN = "set -euo pipefail\npython3 scripts/publish-release.py\n"
ENV = {
    "GH_TOKEN": "${{ github.token }}",
    "RELEASE_REPO": "${{ github.repository }}",
    "RELEASE_TAG": "${{ inputs.tag }}",
    "RELEASE_BUILD_RUN_ID": "${{ inputs.build_run_id }}",
    "RELEASE_MODE": "${{ inputs.mode }}",
    "RELEASE_FIELD_EVIDENCE": "${{ inputs.field_evidence }}",
    "RELEASE_PERFORMANCE_EVIDENCE": "${{ inputs.performance_evidence }}",
    "RELEASE_ACCEPTANCE_EVIDENCE": "${{ inputs.acceptance_evidence }}",
    "RELEASE_OUT_ROOT": "${{ runner.temp }}/tray-goblin-releases",
    "PYTHONDONTWRITEBYTECODE": "1",
}


def validate(doc):
    eq(set(doc), {"name", "on", "permissions", "concurrency", "jobs"}, "root schema")
    eq(doc["name"], "Manual Releases", "name")
    # Closed input schema: input values may ONLY cross through environment data.
    eq(
        doc["on"],
        {
            "workflow_dispatch": {
                "inputs": {
                    "tag": {
                        "description": "Existing annotated tag (for example v0.1.0)",
                        "required": "true",
                        "type": "string",
                    },
                    "build_run_id": {
                        "description": "Successful Windows test build run ID for the tag commit",
                        "required": "true",
                        "type": "string",
                    },
                    "mode": {
                        "description": "Draft preparation or gated public publishing",
                        "required": "true",
                        "type": "choice",
                        "default": "draft",
                        "options": ["draft", "prerelease", "stable"],
                    },
                    "field_evidence": {
                        "description": "Actual collector JSON under .wgm/release-evidence/ (public modes)",
                        "required": "false",
                        "type": "string",
                        "default": "",
                    },
                    "performance_evidence": {
                        "description": "Matching Windows process JSON under .wgm/release-evidence/",
                        "required": "false",
                        "type": "string",
                        "default": "",
                    },
                    "acceptance_evidence": {
                        "description": "Complete manual build and Windows M6 acceptance JSON under .wgm/release-evidence/",
                        "required": "false",
                        "type": "string",
                        "default": "",
                    },
                }
            }
        },
        "dispatch inputs/triggers",
    )
    eq(doc["permissions"], {}, "default no privileges")
    eq(
        doc["concurrency"],
        {"group": "release-${{ inputs.tag }}", "cancel-in-progress": "false"},
        "noncanceling tag concurrency",
    )
    eq(set(doc["jobs"]), {"release"}, "single job")
    job = doc["jobs"]["release"]
    eq(
        set(job),
        {"if", "runs-on", "timeout-minutes", "permissions", "defaults", "steps"},
        "job schema",
    )
    eq(
        job["if"],
        "${{ github.ref == 'refs/heads/main' }}",
        "trusted main dispatch source",
    )
    eq(job["runs-on"], "ubuntu-24.04", "runner")
    eq(job["timeout-minutes"], "35", "bounded timeout")
    eq(
        job["permissions"],
        {"contents": "write", "actions": "read"},
        "least release privileges",
    )
    eq(job["defaults"], {"run": {"shell": "bash"}}, "shell")
    eq(
        job["steps"],
        [
            {
                "name": "Checkout trusted dispatch source",
                "uses": ci.CHECKOUT,
                "with": {
                    "persist-credentials": "false",
                    "fetch-depth": "0",
                    "fetch-tags": "true",
                },
            },
            {"name": "Prepare or publish verified Release", "env": ENV, "run": RUN},
        ],
        "complete checkout/data-only execution policy",
    )


def main():
    path = Path(__file__).resolve().parents[1] / ".github/workflows/release.yml"
    doc = parse(path.read_text())
    validate(doc)
    controls = []

    def mutant(purpose, change):
        altered = copy.deepcopy(doc)
        change(altered)
        controls.append((purpose, altered))

    def job(d):
        return d["jobs"]["release"]

    def checkout(d):
        return job(d)["steps"][0]

    def step(d):
        return job(d)["steps"][1]

    # Purpose-commented unsafe counterfactuals protect actual write authority,
    # not a keyword inventory. Every mutated schema must really fail validation.
    for trigger in (
        "push",
        "pull_request",
        "pull_request_target",
        "release",
        "schedule",
    ):
        mutant(
            "automatic/untrusted " + trigger,
            lambda d, t=trigger: d["on"].update({t: ""}),
        )
    mutant(
        "mutable action pin", lambda d: checkout(d).update(uses="actions/checkout@v4")
    )
    mutant(
        "persist write credentials",
        lambda d: checkout(d)["with"].update({"persist-credentials": "true"}),
    )
    mutant(
        "shallow unpeeled tag",
        lambda d: checkout(d)["with"].update({"fetch-depth": "1"}),
    )
    mutant(
        "tag instead of trusted script source",
        lambda d: checkout(d)["with"].update(ref="${{ inputs.tag }}"),
    )
    mutant("untrusted branch write job", lambda d: job(d).pop("if"))
    mutant("unbounded job", lambda d: job(d).update({"timeout-minutes": "360"}))
    mutant(
        "cancel partial upload",
        lambda d: d["concurrency"].update({"cancel-in-progress": "true"}),
    )
    mutant(
        "broad default authority", lambda d: d["permissions"].update(contents="write")
    )
    mutant(
        "excess action authority",
        lambda d: job(d)["permissions"].update(actions="write"),
    )
    mutant(
        "extra privilege", lambda d: job(d)["permissions"].update({"packages": "write"})
    )
    mutant("extra writer job", lambda d: d["jobs"].update(extra=copy.deepcopy(job(d))))
    mutant(
        "public default",
        lambda d: d["on"]["workflow_dispatch"]["inputs"]["mode"].update(
            default="stable"
        ),
    )
    mutant(
        "secret substituted in shell",
        lambda d: step(d).update(run=RUN + "echo '${{ github.token }}'\n"),
    )
    for name in (
        "tag",
        "build_run_id",
        "mode",
        "field_evidence",
        "performance_evidence",
        "acceptance_evidence",
    ):
        mutant(
            "shell injection " + name,
            lambda d, n=name: step(d).update(
                run=RUN + "echo '${{ inputs." + n + " }}'\n"
            ),
        )
    mutant("missing acceptance input", lambda d: d["on"]["workflow_dispatch"]["inputs"].pop("acceptance_evidence"))
    mutant("missing acceptance binding", lambda d: step(d)["env"].pop("RELEASE_ACCEPTANCE_EVIDENCE"))
    mutant(
        "ambient secret instead of scoped token",
        lambda d: step(d)["env"].update(GH_TOKEN="${{ secrets.PAT }}"),
    )
    mutant(
        "write-token rebuild",
        lambda d: step(d).update(run=RUN + "cargo build --release\n"),
    )
    mutant(
        "Actions artifact mistaken for Release",
        lambda d: job(d)["steps"].append({"uses": ci.UPLOAD}),
    )
    mutant(
        "auto tag/create",
        lambda d: step(d).update(
            run=RUN + "git tag v0.1.0\ngh release create v0.1.0\n"
        ),
    )
    mutant(
        "different script/noop", lambda d: step(d).update(run="echo release verified\n")
    )
    for purpose, changed in controls:
        try:
            validate(changed)
        except (Error, KeyError, TypeError):
            continue
        raise Error("unsafe counterfactual accepted: " + purpose)
    bad = [
        "on: {}\non: {}\n",
        "a: &a [*a]\n",
        "!!python/object/apply:os.system ['false']",
        "---\non: {}\n---\non: {}\n",
        "a: [",
        "a: " + "[" * 21 + "0" + "]" * 21,
    ]
    for value in bad:
        try:
            validate(parse(value))
        except (Error, ci.yaml.YAMLError, KeyError, TypeError):
            continue
        raise Error("unsafe YAML accepted")
    print(
        f"Release workflow policy passed: 1 approved, {len(controls)} unsafe policy counterfactuals rejected, {len(bad)} parser controls rejected."
    )


if __name__ == "__main__":
    try:
        main()
    except (Error, ci.yaml.YAMLError, OSError, TypeError, KeyError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        sys.exit(1)
