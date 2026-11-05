#!/usr/bin/env python3
"""Offline, fail-closed policy for the optional build; PyYAML is tooling only."""

import copy
from pathlib import Path
import sys

sys.dont_write_bytecode = True
import yaml
from yaml.events import AliasEvent

CHECKOUT = "actions/checkout@11d5960a326750d5838078e36cf38b85af677262"
UPLOAD = "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02"
TOOLS = """sudo apt-get update
sudo apt-get install --no-install-recommends -y gcc-mingw-w64-x86-64 zip unzip file python3-yaml
"""
RUST = """rustup toolchain install stable --profile minimal --target x86_64-pc-windows-gnu
rustup default stable
"""
BUILD = """set -euo pipefail
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export HOME
HOME="$(mktemp -d "${RUNNER_TEMP}/tray-goblin-home.XXXXXX")"
export XDG_CONFIG_HOME="$HOME/.config" XDG_CACHE_HOME="$HOME/.cache" XDG_DATA_HOME="$HOME/.local/share" XDG_STATE_HOME="$HOME/.local/state"
unset CARGO_TARGET_DIR
python3 scripts/test-ci-workflow.py
bash scripts/validate-toolchain.sh
version="$(python3 -c 'import tomllib; print(tomllib.load(open("Cargo.toml", "rb"))["package"]["version"])')"
bash scripts/package-release.sh "$version"
"""


class PolicyError(ValueError):
    pass


class StrictLoader(yaml.BaseLoader):
    """String scalars (including 'on'), no executable tags or duplicate keys."""

    def construct_mapping(self, node, deep=False):
        result = {}
        for key_node, value_node in node.value:
            if not isinstance(key_node, yaml.ScalarNode):
                raise PolicyError("non-scalar key")
            key = self.construct_object(key_node, deep=deep)
            if key in result:
                raise PolicyError("duplicate key")
            result[key] = self.construct_object(value_node, deep=deep)
        return result


def parse(text):
    # Bound input and reject aliases/explicit tags before construction: no alias
    # expansion/cycles, YAML object constructors, or silently overwritten policy.
    if len(text.encode("utf-8")) > 65536:
        raise PolicyError("oversized workflow")
    depth = 0
    for event in yaml.parse(text, Loader=yaml.BaseLoader):
        if isinstance(event, AliasEvent) or getattr(event, "tag", None):
            raise PolicyError("aliases/explicit tags forbidden")
        if isinstance(event, (yaml.MappingStartEvent, yaml.SequenceStartEvent)):
            depth += 1
            if depth > 20:
                raise PolicyError("excessive nesting")
        elif isinstance(event, (yaml.MappingEndEvent, yaml.SequenceEndEvent)):
            depth -= 1
    return yaml.load(text, Loader=StrictLoader)


def require_equal(actual, expected, label):
    if actual != expected:
        raise PolicyError(f"{label} differs from approved policy")


def validate(doc):
    # Closed schemas prohibit extra jobs, permissions, services, environments,
    # secrets, arbitrary shell steps and third-party actions, not just bad words.
    if not isinstance(doc, dict):
        raise PolicyError("workflow must be a mapping")
    require_equal(set(doc), {"name", "on", "permissions", "concurrency", "jobs"}, "root keys")
    require_equal(doc["name"], "Windows test build", "name")
    require_equal(doc["on"], {"push": {"branches": ["main"]}, "workflow_dispatch": ""}, "triggers")
    require_equal(doc["permissions"], {"contents": "read"}, "token permissions")
    require_equal(doc["concurrency"], {"group": "windows-test-build-${{ github.ref }}", "cancel-in-progress": "true"}, "branch concurrency")
    jobs = doc["jobs"]
    if not isinstance(jobs, dict):
        raise PolicyError("jobs must be a mapping")
    require_equal(set(jobs), {"windows-test-build"}, "jobs")
    job = jobs["windows-test-build"]
    if not isinstance(job, dict):
        raise PolicyError("job must be a mapping")
    require_equal(set(job), {"runs-on", "timeout-minutes", "defaults", "env", "steps"}, "job keys")
    require_equal(job["runs-on"], "ubuntu-24.04", "runner")
    require_equal(job["timeout-minutes"], "20", "timeout")
    require_equal(job["defaults"], {"run": {"shell": "bash"}}, "shell")
    require_equal(job["env"], {"PYTHONDONTWRITEBYTECODE": "1"}, "environment")
    expected = [
        {"name": "Checkout source", "uses": CHECKOUT, "with": {"persist-credentials": "false"}},
        {"name": "Install public build tools", "run": TOOLS},
        {"name": "Provision stable Rust", "run": RUST},
        {"name": "Test and package Windows target", "run": BUILD},
        {"name": "Upload test ZIP and checksum", "if": "${{ success() }}", "uses": UPLOAD,
         "with": {"name": "tray-goblin-windows-x86_64", "path": "dist/*.zip\ndist/*.zip.sha256\n",
                  "retention-days": "14", "if-no-files-found": "error"}},
    ]
    # Exact command bodies are intentional: canonical scripts cannot be replaced
    # by echoes/no-op builds or appended publication/secret-exfiltration commands.
    require_equal(job["steps"], expected, "complete step/action/input/command policy")


def main():
    workflow = Path(__file__).resolve().parents[1] / ".github/workflows/windows-test-build.yml"
    doc = parse(workflow.read_text(encoding="utf-8"))
    validate(doc)
    controls = []

    def mutant(purpose, change):
        changed = copy.deepcopy(doc)
        change(changed)
        controls.append((purpose, changed))

    def steps(d):
        return d["jobs"]["windows-test-build"]["steps"]

    # Each unsafe counterfactual protects a distinct authority/build boundary.
    mutant("mutable checkout pin", lambda d: steps(d)[0].update(uses="actions/checkout@v4"))
    mutant("missing upload pin", lambda d: steps(d)[4].update(uses="actions/upload-artifact"))
    mutant("write token", lambda d: d["permissions"].update(contents="write"))
    mutant("extra token privilege", lambda d: d["permissions"].update({"actions": "write"}))
    mutant("job privilege escalation", lambda d: d["jobs"]["windows-test-build"].update(permissions={"contents": "write"}))
    mutant("persisted credentials", lambda d: steps(d)[0]["with"].update({"persist-credentials": "true"}))
    mutant("implicit credentials", lambda d: steps(d)[0].pop("with"))
    for trigger in ("pull_request_target", "pull_request", "release", "schedule"):
        mutant(f"inappropriate {trigger} trigger", lambda d, t=trigger: d["on"].update({t: ""}))
    mutant("arbitrary push branch", lambda d: d["on"]["push"].update(branches=["*"]))
    mutant("release creation", lambda d: steps(d).insert(4, {"run": "gh release create v0.1.0"}))
    mutant("secret in shell", lambda d: steps(d)[3].update(run=BUILD + "echo '${{ secrets.TOKEN }}'\n"))
    mutant("secret in env", lambda d: d["jobs"]["windows-test-build"]["env"].update(TOKEN="${{ secrets.TOKEN }}"))
    mutant("always upload", lambda d: steps(d)[4].update({"if": "${{ always() }}"}))
    mutant("implicit upload condition", lambda d: steps(d)[4].pop("if"))
    mutant("missing files accepted", lambda d: steps(d)[4]["with"].update({"if-no-files-found": "warn"}))
    mutant("broad upload path", lambda d: steps(d)[4]["with"].update(path="dist/**"))
    mutant("checksum omitted", lambda d: steps(d)[4]["with"].update(path="dist/*.zip\n"))
    mutant("wrong retention", lambda d: steps(d)[4]["with"].update({"retention-days": "90"}))
    mutant("additional job", lambda d: d["jobs"].update(extra={"runs-on": "ubuntu-24.04", "steps": [{"run": "true"}]}))
    mutant("fake build", lambda d: steps(d)[3].update(run="echo build passed\n"))
    mutant("no HOME isolation", lambda d: steps(d)[3].update(run=BUILD.replace('HOME="$(mktemp -d "${RUNNER_TEMP}/tray-goblin-home.XXXXXX")"\n', "")))
    mutant("excessive deadline", lambda d: d["jobs"]["windows-test-build"].update({"timeout-minutes": "60"}))
    for purpose, changed in controls:
        try:
            validate(changed)
        except PolicyError:
            continue
        raise PolicyError(f"unsafe counterfactual accepted: {purpose}")

    # Parser controls guard inputs rather than relying on safe-looking text.
    bad_inputs = ["on: {}\non: {}\n", "a: &a [*a]\n", "!!python/object/apply:os.system ['false']",
                  "---\non: {}\n---\non: {}\n", "a: [", "a: " + "[" * 21 + "0" + "]" * 21]
    for text in bad_inputs:
        try:
            validate(parse(text))
        except (PolicyError, yaml.YAMLError):
            continue
        raise PolicyError("unsafe parser counterfactual accepted")
    print(f"CI workflow policy passed: 1 approved workflow, {len(controls)} unsafe policy counterfactuals rejected, {len(bad_inputs)} unsafe input controls rejected.")


if __name__ == "__main__":
    try:
        main()
    except (PolicyError, yaml.YAMLError, OSError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        sys.exit(1)
