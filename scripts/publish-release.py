#!/usr/bin/env python3
"""Optional manual Releases: existing annotated tag + exact CI bytes, draft first.

Never builds, extracts or runs downloaded code. Collector JSON is trusted reviewed
repository DATA, not proof obtainable from mocks. Public modes require actual M6.
Short deadlines and gh/Git fakes are private offline self-test controls only.
"""

import argparse
import copy
from dataclasses import dataclass
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import time
import zipfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("ci_verify", HERE / "verify-ci-build.py")
ci = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = ci
spec.loader.exec_module(ci)
require = ci.require
decode = ci.decode
Error = ci.VerificationError
RELEASE_WORKFLOW = ".github/workflows/release.yml"
EVIDENCE_AREA = ".wgm/release-evidence"
MARKER = "<!-- tray-goblin-release-v1 "


@dataclass(frozen=True)
class Deadlines:
    discovery: float = 120
    watch: float = 1500
    transfer: float = 120


class GitHub:
    """Retain only sanitized operation/exit evidence, never gh stdout/stderr/token.

    gh receives the scoped token as environment, never argv. Even an API error
    that echoes secrets or report content cannot enter the console/evidence.
    """

    def __init__(self, evidence):
        self.evidence = evidence
        self.serial = 0

    def command(self, args, seconds):
        require(seconds > 0, "deadline exceeded")
        result = subprocess.run(
            ["timeout", "--kill-after=5s", f"{seconds:.3f}s", "gh", *args],
            capture_output=True,
            check=False,
        )
        self.serial += 1
        (self.evidence / f"command-{self.serial:03d}.json").write_text(
            json.dumps(
                {
                    "operation": args[0],
                    "exit": result.returncode,
                    "deadlineSeconds": seconds,
                }
            )
            + "\n"
        )
        require(
            result.returncode == 0,
            "GitHub command failed; owned directory and draft retained",
        )
        require(len(result.stdout) <= 2 * 1024 * 1024, "oversized API response")
        return result.stdout.decode("utf-8")

    def api(self, endpoint, seconds=120):
        return decode(self.command(["api", endpoint], seconds))


def git(*args):
    # Read-only operations only; no shell, credentials or unbounded subprocesses.
    env = {k: v for k, v in os.environ.items() if k not in ("GH_TOKEN", "GITHUB_TOKEN")}
    result = subprocess.run(
        ["timeout", "--kill-after=5s", "120s", "git", *args],
        env=env,
        capture_output=True,
        check=False,
    )
    require(
        result.returncode == 0,
        "required Git metadata unavailable (no tag/branch fallback)",
    )
    require(len(result.stdout) <= 1024 * 1024, "oversized Git metadata")
    return result.stdout.decode("utf-8").strip()


def validate_args(repo, tag, run_id, mode):
    require(
        re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9-]*/[A-Za-z0-9][A-Za-z0-9_.-]*", repo),
        "invalid repository",
    )
    # Semver, no leading zeros in numeric version/prerelease identifiers.
    match = re.fullmatch(
        r"v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?",
        tag,
    )
    require(match, "tag must be vSemVer")
    if match[4]:
        require(
            all(
                not (p.isdigit() and len(p) > 1 and p[0] == "0")
                for p in match[4].split(".")
            ),
            "invalid semver prerelease",
        )
    require(type(run_id) is int and run_id > 0, "invalid source run ID")
    require(mode in ("draft", "prerelease", "stable"), "invalid mode")
    require(
        mode != "stable" or not match[4], "stable mode refuses a prerelease version"
    )


def tag_identity(gh, repo, tag, seconds):
    ref = "refs/tags/" + tag
    require(git("cat-file", "-t", ref) == "tag", "existing annotated tag required")
    sha = git("rev-parse", "--verify", ref + "^{commit}")
    require(re.fullmatch(r"[0-9a-f]{40}", sha), "tag did not peel to commit")
    # Check remote annotated tag too: never implicitly create/move a remote ref.
    remote = gh.api(f"repos/{repo}/git/ref/tags/{tag}", seconds)
    obj = remote.get("object", {})
    require(
        obj.get("type") == "tag"
        and obj.get("sha") == git("rev-parse", "--verify", ref),
        "remote annotated tag mismatch",
    )
    peeled = gh.api(f"repos/{repo}/git/tags/{obj['sha']}", seconds).get("object", {})
    require(
        peeled.get("type") == "commit" and peeled.get("sha") == sha,
        "remote tag commit mismatch",
    )
    import tomllib

    cargo = tomllib.loads(git("show", sha + ":Cargo.toml"))
    require(
        cargo.get("package", {}).get("version") == tag[1:],
        "Cargo version at tag mismatch",
    )
    return sha


def source_run(gh, repo, sha, run_id, seconds):
    run = gh.api(f"repos/{repo}/actions/runs/{run_id}", seconds)
    ci.check_run(run, repo, sha, run_id)
    require(
        run.get("status") == "completed" and run.get("conclusion") == "success",
        "source build must already be successful",
    )
    return run


def repository_file(path, checkout, suffix=".json", maximum=256 * 1024):
    require(
        isinstance(path, str) and len(path) <= 256 and path, "missing M6 evidence path"
    )
    parts = path.split("/")
    require(
        parts[:2] == EVIDENCE_AREA.split("/")
        and len(parts) >= 3
        and all(
            p and p not in (".", "..") and re.fullmatch(r"[A-Za-z0-9_.-]+", p)
            for p in parts
        )
        and path.endswith(suffix),
        "evidence must be inside explicit repository evidence area",
    )
    root = Path(checkout).resolve()
    current = root
    for part in parts:
        current = current / part
        require(not current.is_symlink(), "symlink evidence rejected")
    require(
        current.is_file() and 0 < current.stat().st_size <= maximum,
        "missing/oversized evidence",
    )
    return current


def repository_json(path, checkout):
    # No evaluation, imports, subprocesses or logging of report contents.
    raw = repository_file(path, checkout).read_bytes()
    require(len(raw) <= 256 * 1024, "oversized evidence")
    report = decode(raw.decode("utf-8-sig"))
    require(isinstance(report, dict), "evidence must be JSON object")
    return report, hashlib.sha256(raw).hexdigest()


def number(value, low=0, high=1e12):
    return type(value) in (int, float) and math.isfinite(value) and low <= value <= high


def text(value):
    return isinstance(value, str) and 0 < len(value.strip()) <= 4096


def windows_path(value):
    return (
        text(value)
        and re.match(r"^[A-Za-z]:[\\/]", value) is not None
        and value.lower().endswith("tray-goblin.exe")
    )


def report_identity(report, sha, exe):
    require(
        report.get("buildSha") == sha
        and report.get("executableSha256") == exe
        and report.get("worktreeDirty") is False,
        "M6 clean source/executable identity mismatch",
    )
    require(
        type(report.get("pid")) is int
        and report["pid"] > 0
        and windows_path(report.get("executablePath")),
        "M6 process identity missing",
    )
    start = ci.timestamp(report["startedUtc"])
    finish = ci.timestamp(report["finishedUtc"])
    require(
        ci.timestamp(report["processStartUtc"]) <= start <= finish,
        "M6 invalid chronology",
    )
    require(report.get("passed") is True, "M6 failing/partial report")
    return start, finish


def expected_cases(poll):
    """Derive all 55 collector case IDs/bounds from its native/WSL dialog matrix.

    Self-test compares this matrix to actual current Observe-Case declarations,
    so collector changes require a deliberate policy update, not passed:true.
    """
    cases = {}
    deadline = 2 * poll + 500
    for origin in ("native", "wsl"):
        for dialog in (
            "confirm",
            "select",
            "input",
            "editor",
            "custom",
            "manual-settings",
            "manual-compaction",
        ):
            for transition in ("attention", "clear"):
                cases[f"{origin}-{dialog}-{transition}"] = (deadline, 0)
        for suffix in (
            "final-error",
            "new-work-clear",
            "copilot-persisted-error",
            "copilot-new-turn-clear",
        ):
            cases[f"{origin}-{suffix}"] = (deadline, 0)
        for suffix in (
            "error-deduplicated",
            "completion-quiet",
            "abort-recovered-quiet",
        ):
            cases[f"{origin}-{suffix}"] = (0, 3 * poll)
        cases[f"{origin}-error-dialog"] = (0, 0)
    for name in (
        "same-directory-identity",
        "suppression-persistent-list",
        "unc-wake-observed",
        "source-removal-restart-no-reads",
        "clean-quit",
    ):
        cases[name] = (0, 0)
    for name in ("pi-reload", "wsl-recovery"):
        cases[name] = (deadline, 0)
    for name in ("pi-crash-stale", "wsl-shutdown-after-read"):
        cases[name] = (10000 + 2 * poll, 0)
    for name in ("explorer-no-replay", "native-isolation"):
        cases[name] = (0, 3 * poll)
    return cases


def field_gate(report, sha, exe):
    require(
        set(report)
        == {
            "evidenceTier",
            "startedUtc",
            "finishedUtc",
            "buildSha",
            "worktreeDirty",
            "executableSha256",
            "pid",
            "executablePath",
            "processStartUtc",
            "windows",
            "wslVersion",
            "sourceKinds",
            "sourceCount",
            "syntheticSessionCount",
            "loadEvidence",
            "pollIntervalMs",
            "timingMethod",
            "cases",
            "completed",
            "passed",
            "uncWakeObservation",
        },
        "M6 field collector schema mismatch",
    )
    require(
        report.get("evidenceTier") == "interactive-windows-wsl-observations",
        "M6 requires actual field collector tier",
    )
    start, finish = report_identity(report, sha, exe)
    require(report.get("completed") is True, "M6 incomplete field report")
    windows = report.get("windows", {})
    require(
        isinstance(windows, dict)
        and all(text(windows.get(k)) for k in ("caption", "version", "build"))
        and supported_windows(windows),
        "M6 Windows version missing",
    )
    require(
        text(report.get("wslVersion")) and "WSL" in report["wslVersion"].upper(),
        "M6 WSL version missing",
    )
    require(
        isinstance(report.get("sourceKinds"), list)
        and len(report["sourceKinds"]) == 2
        and set(report["sourceKinds"]) == {"pi", "copilot"}
        and type(report.get("sourceCount")) is int
        and 2 <= report["sourceCount"] <= 16,
        "M6 source info missing",
    )
    require(
        type(report.get("syntheticSessionCount")) is int
        and 2 <= report["syntheticSessionCount"] <= 1024
        and report.get("loadEvidence") == "operator-counted-actual-live-menu-rows",
        "M6 actual load evidence missing",
    )
    poll = report.get("pollIntervalMs")
    require(type(poll) is int and 500 <= poll <= 10000, "M6 cadence invalid")
    require(
        report.get("timingMethod")
        == "stopwatch-before-trigger-to-human-observation-conservative-upper-bound",
        "M6 timing method invalid",
    )
    require(
        report.get("uncWakeObservation") in ("WOKE", "STAYED-STOPPED"),
        "M6 UNC observation missing/failing",
    )
    expected = expected_cases(poll)
    cases = report.get("cases")
    require(
        isinstance(cases, list) and len(cases) == len(expected),
        "M6 missing/extra field cases",
    )
    seen = set()
    previous = start
    for case in cases:
        require(
            isinstance(case, dict)
            and set(case)
            == {
                "id",
                "startedUtc",
                "observedUtc",
                "pid",
                "executablePath",
                "executableSha256",
                "operatorObserved",
                "elapsedMs",
                "maximumMs",
                "minimumMs",
                "timingPassed",
                "passed",
            },
            "invalid field case schema",
        )
        name = case.get("id")
        require(
            name in expected and name not in seen, "M6 unknown/duplicate field case"
        )
        seen.add(name)
        maximum, minimum = expected[name]
        require(
            case.get("maximumMs") == maximum
            and case.get("minimumMs") == minimum
            and type(case.get("maximumMs")) is int
            and type(case.get("minimumMs")) is int,
            "M6 case timing bounds changed",
        )
        elapsed = case.get("elapsedMs")
        require(
            number(elapsed, minimum)
            and (maximum == 0 or elapsed <= maximum)
            and case.get("timingPassed") is True
            and case.get("passed") is True
            and case.get("operatorObserved") == "PASS",
            "M6 contradictory/failing case",
        )
        require(
            type(case.get("pid")) is int
            and case["pid"] > 0
            and case.get("executablePath") == report["executablePath"]
            and case.get("executableSha256") == exe,
            "M6 case executable identity mismatch",
        )
        triggered, observed = (
            ci.timestamp(case["startedUtc"]),
            ci.timestamp(case["observedUtc"]),
        )
        require(
            previous <= triggered <= observed <= finish
            and elapsed <= (observed - triggered).total_seconds() * 1000 + 10,
            # observedUtc includes typing PASS AFTER the stopwatch stops.
            # Never subtract that human time from collector elapsedMs.
            "M6 contradictory case chronology",
        )
        previous = observed
    require(seen == set(expected), "M6 missing expected cases")


def performance_gate(report, sha, exe):
    require(
        set(report)
        == {
            "evidenceTier",
            "startedUtc",
            "finishedUtc",
            "buildSha",
            "worktreeDirty",
            "executablePath",
            "executableSha256",
            "pid",
            "processStartUtc",
            "windowsVersion",
            "process",
            "durationSeconds",
            "samples",
            "logicalProcessors",
            "peakWorkingSetMb",
            "maxWorkingSetMb",
            "workingSetPassed",
            "averageCpuPercent",
            "maxIdleCpuPercent",
            "cpuPassed",
            "passed",
        },
        "M6 resource collector schema mismatch",
    )
    require(
        report.get("evidenceTier") == "windows-process-resource-measurement",
        "M6 requires actual Windows resource collector tier",
    )
    start, finish = report_identity(report, sha, exe)
    require(
        text(report.get("windowsVersion"))
        and re.fullmatch(r"Microsoft Windows NT 10\.0\.([0-9]+)\.[0-9]+", report["windowsVersion"])
        and int(report["windowsVersion"].split(".")[2]) >= 10240
        and report.get("process") == "tray-goblin",
        "M6 Windows process version missing",
    )
    duration = report.get("durationSeconds")
    require(
        number(duration, 30, 3665)
        and abs((finish - start).total_seconds() - duration) <= 5,
        "M6 resource sampling must be at least 30 seconds",
    )
    require(
        type(report.get("samples")) is int
        and report["samples"] >= 2
        and type(report.get("logicalProcessors")) is int
        and report["logicalProcessors"] > 0,
        "M6 raw sampling identity missing",
    )
    # Current collector gates RAW metrics before rounding. Those flags are valid
    # only with these budgets; rounded metrics alone never authorize publishing.
    for value, budget, flag, limit in (
        ("peakWorkingSetMb", "maxWorkingSetMb", "workingSetPassed", 50),
        ("averageCpuPercent", "maxIdleCpuPercent", "cpuPassed", 5),
    ):
        require(
            number(report.get(budget), 1, limit)
            and type(report[budget]) is int
            and number(report.get(value), 0, report[budget])
            and report.get(flag) is True,
            "M6 resource raw-comparison flag/budget/metric failed",
        )


def supported_windows(windows):
    if not isinstance(windows, dict) or set(windows) != {"caption", "version", "build"}:
        return False
    version = windows.get("version", "")
    build = windows.get("build", "")
    caption = windows.get("caption", "")
    return (
        isinstance(version, str) and isinstance(build, str) and isinstance(caption, str)
        and re.fullmatch(r"10\.0\.[0-9]+", version) is not None
        and re.fullmatch(r"[0-9]{5}", build) is not None
        and version == "10.0." + build
        and int(build) >= 10240
        and re.search(r"\bWindows " + ("11" if int(build) >= 22000 else "10") + r"\b", caption) is not None
    )


def same_host(field, perf):
    require(
        all(field[k] == perf[k] for k in ("pid", "executablePath", "processStartUtc"))
        and perf["windowsVersion"].startswith("Microsoft Windows NT " + field["windows"]["version"] + "."),
        "field/resource host or process provenance mismatch",
    )


CHECKLIST = {
    "install": {"standard-user", "files-localappdata", "startup-shortcut"},
    "icon": {"registered", "explorer-restart-restored"},
    "legibility": {"idle-working-attention-shape-color", "high-contrast"},
    "magic-moment": {"blocking-and-manual-dialog", "other-session-progress", "bounded-attention-clear", "quiet-completion-abort-recovery", "copilot-error-not-completion"},
    "tooltip": {"aggregate-list-agreement", "same-directory-identity", "submenu-model-directory-run-start-duration", "bounded-shell-text"},
    "menu": {"attention-first-persistent-identity-metadata", "suppressed-notifications-list", "refresh", "vscode", "copilot-logs", "settings", "quit"},
    "configuration": {"out-of-range-content-free-dialog"},
    "startup": {"sign-out-sign-in-autostart"},
    "uninstall": {"files-shortcut-removed", "configuration-kept", "remove-configuration"},
    "field": {"all-55-native-wsl-cases", "file-access-trace-no-reads"},
    "performance": {"same-exe-host-30s-50mb-5percent"},
}


def participant(value):
    return isinstance(value, str) and re.fullmatch(r"[a-z0-9][a-z0-9._@-]{0,127}", value) is not None


def manual_commands(tag):
    version = tag[1:]
    return [
        ["git", "checkout", "--detach", tag],
        ["git", "status", "--porcelain"],
        ["git", "rev-parse", "--verify", "HEAD"],
        ["git", "rev-parse", "--verify", "refs/tags/" + tag + "^{commit}"],
        ["git", "rev-parse", "--verify", "refs/heads/main"],
        ["git", "rev-parse", "--verify", "refs/remotes/origin/main"],
        ["git", "remote", "get-url", "origin"],
        ["git", "cat-file", "-t", "refs/tags/" + tag],
        ["cargo", "test", "--workspace"],
        ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"],
        ["bash", "scripts/validate-toolchain.sh"],
        ["bash", "scripts/demo-attention.sh", "--container", "podman"],
        ["bash", "scripts/demo.sh", version],
        ["bash", "scripts/check-docs.sh"],
        ["pwsh", "-NoProfile", "-File", "scripts/test-installer.ps1"],
        ["node", "scripts/test-release-docs.mjs"],
        ["bash", "scripts/package-release.sh", version],
    ]


def acceptance_gate(path, checkout, repo, tag, run_id, sha, archive, exe, collectors, field, perf):
    a, digest = repository_json(path, checkout)
    require(set(a) == {"schema", "identity", "createdUtc", "author", "manual", "targets", "reviews", "operator"}, "acceptance schema mismatch")
    identity = {"repo": repo, "tag": tag, "sourceRun": run_id, "sourceSHA": sha,
                "archiveSha256": archive, "executableSha256": exe, **collectors}
    require(a["schema"] == "tray-goblin-acceptance-v1" and a["identity"] == identity and participant(a["author"]), "acceptance binding mismatch")
    created = ci.timestamp(a["createdUtc"])
    from datetime import datetime, timezone
    require(0 <= (datetime.now(timezone.utc) - created).total_seconds() <= 7 * 86400, "acceptance stale/future")
    m = a["manual"]
    require(set(m) == {"sourceSHA", "tag", "worktreeDirty", "canonicalMainSHA", "builder", "startedUtc", "finishedUtc", "commands", "archivePath"}
            and m["sourceSHA"] == m["canonicalMainSHA"] == sha and m["tag"] == tag
            and m["worktreeDirty"] is False and participant(m["builder"]), "manual clean tagged canonical source missing")
    begin, finish = ci.timestamp(m["startedUtc"]), ci.timestamp(m["finishedUtc"])
    require(begin <= finish <= created, "manual chronology invalid")
    commands = m["commands"]
    require(isinstance(commands, list) and len(commands) == len(manual_commands(tag)), "manual validation incomplete")
    for c, argv in zip(commands, manual_commands(tag)):
        require(set(c) == {"argv", "exit", "startedUtc", "finishedUtc", "result", "resultSha256"}
                and c["argv"] == argv and type(c["exit"]) is int and c["exit"] == 0
                and isinstance(c["result"], str) and len(c["result"]) <= 4096
                and c["resultSha256"] == hashlib.sha256(c["result"].encode()).hexdigest(), "manual command/result failed")
        result = c["result"]
        if argv[:2] == ["git", "status"]:
            require(result == "", "manual source dirty")
        elif argv[:2] == ["git", "rev-parse"]:
            require(result.strip() == sha, "manual canonical tag identity mismatch")
        elif argv[:2] == ["git", "remote"]:
            require(result.strip() in (f"https://github.com/{repo}.git", f"https://github.com/{repo}", f"git@github.com:{repo}.git"), "manual canonical origin mismatch")
        elif argv[:2] == ["git", "cat-file"]:
            require(result.strip() == "tag", "manual annotated tag missing")
        else:
            require(text(result), "manual command result missing")
        if argv[:2] == ["bash", "scripts/package-release.sh"]:
            require("Reproducible: two independent packing runs matched." in result and archive in result, "manual reproducibility result missing")
        start, end = ci.timestamp(c["startedUtc"]), ci.timestamp(c["finishedUtc"])
        require(begin <= start <= end <= finish, "manual command chronology invalid")
        begin = end
    manual_zip = repository_file(m["archivePath"], checkout, ".zip", 100 * 1024 * 1024)
    require(manual_zip.name == f"tray-goblin-{tag[1:]}-windows-x86_64.zip", "manual archive name mismatch")
    repository_file(m["archivePath"] + ".sha256", checkout, ".sha256")
    require(ci.verify_payload(manual_zip.parent, sha) == (archive, exe), "manual/CI bytes not equivalent")
    targets = a["targets"]
    require(isinstance(targets, list) and len(targets) == 2, "both Windows targets required")
    seen, operators, target_finish = set(), set(), finish
    for t in targets:
        require(set(t) == {"os", "architecture", "windows", "observer", "startedUtc", "finishedUtc", "checks"}
                and t["os"] in ("Windows10", "Windows11") and t["os"] not in seen
                and t["architecture"] == "x64" and supported_windows(t["windows"])
                and (int(t["windows"]["build"]) >= 22000) == (t["os"] == "Windows11")
                and participant(t["observer"]), "supported target observation missing")
        seen.add(t["os"]); operators.add(t["observer"])
        start, end = ci.timestamp(t["startedUtc"]), ci.timestamp(t["finishedUtc"])
        require(finish <= start <= end <= created, "target chronology invalid")
        target_finish = max(target_finish, end)
        checks = t["checks"]
        require(isinstance(checks, dict) and set(checks) == set(CHECKLIST), "full 11-item checklist required")
        for name, facets in CHECKLIST.items():
            items = checks[name]
            require(isinstance(items, dict) and set(items) == facets, "checklist observation coverage missing")
            for observation in items.values():
                require(set(observation) == {"outcome", "observedUtc", "observation"}
                        and observation["outcome"] == "PASS" and text(observation["observation"])
                        and len(observation["observation"].strip()) >= 20
                        and start <= ci.timestamp(observation["observedUtc"]) <= end, "dated actual checklist observation missing/failing")
    matching = next((t for t in targets if t["windows"] == field["windows"]), None)
    require(matching is not None, "collector host not among observed targets")
    require(ci.timestamp(matching["startedUtc"]) <= ci.timestamp(field["startedUtc"])
            <= ci.timestamp(field["finishedUtc"]) <= ci.timestamp(matching["finishedUtc"])
            and ci.timestamp(matching["startedUtc"]) <= ci.timestamp(perf["startedUtc"])
            <= ci.timestamp(perf["finishedUtc"]) <= ci.timestamp(matching["finishedUtc"]), "collector/target chronology mismatch")
    coverage = {"manual-source-validation-repro", "manual-ci-byte-equivalence", "Windows10-all11", "Windows11-all11", "field-55-wsl-privacy", "performance-supported-same-host-budgets"}
    reviews = a["reviews"]
    require(isinstance(reviews, list) and len(reviews) == 3, "fresh independent reviews missing")
    roles, reviewers = set(), set()
    for r in reviews:
        require(set(r) == {"role", "reviewer", "reviewedUtc", "verdict", "identity", "coverage", "findings"}
                and r["role"] in ("spec", "quality", "rugged") and r["role"] not in roles
                and participant(r["reviewer"]) and r["reviewer"] not in {a["author"], m["builder"], *operators, *reviewers}
                and r["verdict"] == "ACCEPT" and r["identity"] == identity
                and isinstance(r["coverage"], dict) and set(r["coverage"]) == coverage
                and all(text(v) and len(v.strip()) >= 20 for v in r["coverage"].values())
                and r["findings"] == [] and target_finish <= ci.timestamp(r["reviewedUtc"]) <= created
                and (ci.timestamp(r["reviewedUtc"]) - target_finish).total_seconds() <= 7 * 86400,
                "fresh independent M6 review coverage missing/failing")
        roles.add(r["role"]); reviewers.add(r["reviewer"])
    o = a["operator"]
    require(set(o) == {"identity", "name", "acceptedUtc", "decision", "statement"}
            and o["identity"] == identity and o["name"] in operators
            and o["decision"] == "AUTHORIZE-PUBLIC-PRERELEASE-AND-STABLE"
            and text(o["statement"]) and len(o["statement"].strip()) >= 20
            and max(ci.timestamp(r["reviewedUtc"]) for r in reviews) <= ci.timestamp(o["acceptedUtc"]) <= created
            and (created - target_finish).total_seconds() <= 7 * 86400, "explicit operator acceptance missing")
    return digest


def evidence_gate(field_path, performance_path, checkout, sha, exe, acceptance_path, repo, tag, run_id, archive):
    field, fd = repository_json(field_path, checkout)
    perf, pd = repository_json(performance_path, checkout)
    field_gate(field, sha, exe)
    performance_gate(perf, sha, exe)
    same_host(field, perf)
    collectors = {"fieldSha256": fd, "performanceSha256": pd}
    ad = acceptance_gate(acceptance_path, checkout, repo, tag, run_id, sha, archive, exe, collectors, field, perf)
    return {**collectors, "acceptanceSha256": ad}


def ownership(repo, tag, sha, run_id, mode, download, exe, evidence):
    return {
        "owner": "tray-goblin-release-v1",
        "repo": repo,
        "tag": tag,
        "sourceSHA": sha,
        "buildRunId": run_id,
        "mode": mode,
        "executableSha256": exe,
        "assets": {
            p.name: hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(download.iterdir())
        },
        "evidence": evidence,
    }


def notes(record):
    gate = (
        "UNTESTED on actual Windows/WSL; UNPUBLISHED draft preparation. M6 actual field AND performance evidence is still required; this is not release readiness."
        if record["mode"] == "draft"
        else "Complete digest-bound manual validation/build/byte-equivalence, both Windows target checklists, same-host field/resource reports, independent M6 reviews and explicit operator acceptance validated before preparation."
    )
    return (
        f"# TrayGoblin {record['tag']} — {record['mode']}\n\n{gate}\n\n"
        "Unsigned Windows x86-64 portable ZIP; local per-user install, no administrator rights. No signing or auto-update. Copilot input detection unavailable; UNC reads may wake WSL; Windows may suppress notifications (use persistent list).\n\n"
        "DRAFT assets require owner/collaborator GitHub sign-in; no anonymous-download promise. Release assets persist independently of Actions artifact retention. build.json published=false records BUILD TIME and is not rewritten on publication.\n\n"
        f"Source commit: {record['sourceSHA']}\nSource build run: {record['buildRunId']}\nExecutable SHA-256: {record['executableSha256']}\n\n"
        + MARKER
        + json.dumps(record, sort_keys=True, separators=(",", ":"))
        + " -->\n"
    )


def find_release(gh, repo, tag, seconds):
    pages = decode(
        gh.command(
            ["api", f"repos/{repo}/releases?per_page=100", "--paginate", "--slurp"],
            seconds,
        )
    )
    require(
        isinstance(pages, list)
        and len(pages) <= 10
        and all(isinstance(p, list) for p in pages),
        "invalid/oversized release list",
    )
    rows = [
        r
        for page in pages
        for r in page
        if isinstance(r, dict) and r.get("tag_name") == tag
    ]
    require(len(rows) <= 1, "ambiguous release for tag")
    return rows[0] if rows else None


def check_release(release, record):
    require(
        isinstance(release, dict) and release.get("draft") is True,
        "published release refused before mutation",
    )
    require(type(release.get("id")) is int and release["id"] > 0, "invalid Release ID")
    require(
        release.get("tag_name") == record["tag"]
        and release.get("target_commitish") == record["sourceSHA"]
        and release.get("body") == notes(record)
        and release.get("name") == f"TrayGoblin {record['tag']} ({record['mode']})"
        and release.get("prerelease") is False,
        "different/unowned draft (tag/commit/mode/notes)",
    )
    # Draft html_url may be /releases/tag/untagged-...; do not invent a tag URL.
    url = release.get("html_url")
    require(
        text(url)
        and url.startswith(f"https://github.com/{record['repo']}/releases/")
        and not any(c.isspace() for c in url),
        "unexpected Release URL",
    )
    return check_assets(release, record)


def check_assets(release, record):
    assets = release.get("assets")
    require(
        isinstance(assets, list) and len(assets) <= 2, "unknown/extra Release assets"
    )
    seen, ids = set(), set()
    for asset in assets:
        name = asset.get("name")
        require(
            name in record["assets"] and name not in seen,
            "unknown/duplicate draft asset",
        )
        require(
            type(asset.get("id")) is int
            and asset["id"] > 0
            and asset["id"] not in ids
            and asset.get("state") == "uploaded",
            "incomplete draft asset; retain and inspect",
        )
        if asset.get("digest") is not None:
            require(
                asset["digest"] == "sha256:" + record["assets"][name],
                "different draft asset digest",
            )
        seen.add(name)
        ids.add(asset["id"])
    return seen


def inspected_release(gh, repo, release_id, seconds):
    return gh.api(f"repos/{repo}/releases/{release_id}", seconds)


def download_assets(gh, record, destination, names, seconds):
    destination.mkdir()
    args = [
        "release",
        "download",
        record["tag"],
        "--repo",
        record["repo"],
        "--dir",
        str(destination),
    ]
    for name in sorted(names):
        args += ["--pattern", name]
    gh.command(args, seconds)
    files = list(destination.iterdir())
    require(
        {p.name for p in files} == set(names)
        and len(files) == len(names)
        and all(p.is_file() and not p.is_symlink() for p in files),
        "missing/unknown Release download",
    )
    for p in files:
        require(
            0 < p.stat().st_size < 100 * 1024 * 1024
            and hashlib.sha256(p.read_bytes()).hexdigest() == record["assets"][p.name],
            "Release asset digest mismatch",
        )


def prepare(
    repo,
    tag,
    run_id,
    mode,
    field_path,
    performance_path,
    out_root,
    checkout=".",
    deadlines=Deadlines(),
    acceptance_path=None,
):
    validate_args(repo, tag, run_id, mode)
    root = Path(out_root).resolve()
    root.mkdir(parents=True, exist_ok=True)
    owned = Path(tempfile.mkdtemp(prefix="tray-goblin-release-", dir=root))
    evidence_dir = owned / "evidence"
    evidence_dir.mkdir()
    download = owned / "ci-bytes"
    download.mkdir()
    print(f"Owned Release directory retained: {owned}", file=sys.stderr, flush=True)
    gh = GitHub(evidence_dir)
    sha = tag_identity(gh, repo, tag, deadlines.discovery)
    run = source_run(gh, repo, sha, run_id, deadlines.discovery)
    artifact = ci.inspect_artifact(
        gh.api(
            f"repos/{repo}/actions/runs/{run_id}/artifacts?per_page=100",
            deadlines.discovery,
        ),
        repo,
        sha,
        run_id,
        run,
    )
    gh.command(
        [
            "run",
            "download",
            str(run_id),
            "--repo",
            repo,
            "--name",
            ci.ARTIFACT,
            "--dir",
            str(download),
        ],
        deadlines.transfer,
    )
    zip_digest, exe = ci.verify_payload(download, sha)
    require(
        (download / f"tray-goblin-{tag[1:]}-windows-x86_64.zip").is_file(),
        "artifact version/tag mismatch",
    )
    acceptance_path = acceptance_path if acceptance_path is not None else os.environ.get("RELEASE_ACCEPTANCE_EVIDENCE", "")
    def public_gate():
        return evidence_gate(field_path, performance_path, checkout, sha, exe,
                             acceptance_path, repo, tag, run_id, zip_digest)
    gates = {} if mode == "draft" else public_gate()
    record = ownership(repo, tag, sha, run_id, mode, download, exe, gates)
    # No reports or secrets serialized. Retain immutable local ownership/notes.
    (evidence_dir / "ownership.json").write_text(json.dumps(record, indent=2) + "\n")
    body = owned / "release-notes.md"
    body.write_text(notes(record))
    release = find_release(gh, repo, tag, deadlines.discovery)
    if release is None:
        gh.command(
            [
                "release",
                "create",
                tag,
                "--repo",
                repo,
                "--verify-tag",
                "--target",
                sha,
                "--draft",
                "--title",
                f"TrayGoblin {tag} ({mode})",
                "--notes-file",
                str(body),
            ],
            deadlines.transfer,
        )
        release = find_release(gh, repo, tag, deadlines.discovery)
    existing = check_release(release, record)
    release_id = release["id"]
    (evidence_dir / "draft.json").write_text(
        json.dumps(
            {
                "releaseId": release_id,
                "presentAssetIds": [a["id"] for a in release["assets"]],
            }
        )
        + "\n"
    )
    # Existing bytes are downloaded BEFORE any upload; hashes, not filenames or
    # optional API digest alone, establish whether a partial draft belongs to us.
    if existing:
        download_assets(
            gh, record, owned / "existing-assets", existing, deadlines.transfer
        )
    for serial, name in enumerate(sorted(record["assets"])):
        current = inspected_release(gh, repo, release_id, deadlines.discovery)
        actual = check_release(current, record)
        require(actual == existing, "draft changed concurrently; refuse mutation")
        if name not in existing:
            gh.command(
                ["release", "upload", tag, str(download / name), "--repo", repo],
                deadlines.transfer,
            )
            existing.add(name)
        (evidence_dir / f"upload-{serial}.json").write_text(
            json.dumps(
                {
                    "releaseId": release_id,
                    "verifiedNamesBeforeNextStep": sorted(existing),
                }
            )
            + "\n"
        )
    release = inspected_release(gh, repo, release_id, deadlines.discovery)
    require(
        check_release(release, record) == set(record["assets"]),
        "both Release assets required",
    )
    permanent = owned / "release-assets"
    download_assets(gh, record, permanent, record["assets"], deadlines.transfer)
    require(
        ci.verify_payload(permanent, sha) == (zip_digest, exe),
        "Release bytes differ from measured CI bytes",
    )
    require(
        tag_identity(gh, repo, tag, deadlines.discovery) == sha,
        "tag changed during preparation",
    )
    release = inspected_release(gh, repo, release_id, deadlines.discovery)
    require(
        check_release(release, record) == set(record["assets"]),
        "draft changed during final asset verification",
    )
    if mode != "draft":
        # Recheck evidence/draft immediately before the sole publish call.
        # Never patch body/tag/assets and never edit published state.
        require(
            public_gate() == gates,
            "evidence changed before publish",
        )
        require(
            check_release(
                inspected_release(gh, repo, release_id, deadlines.discovery), record
            )
            == set(record["assets"]),
            "draft changed before publish",
        )
        gh.command(
            [
                "api",
                f"repos/{repo}/releases/{release_id}",
                "--method",
                "PATCH",
                "-F",
                "draft=false",
                "-F",
                f"prerelease={'true' if mode == 'prerelease' else 'false'}",
            ],
            deadlines.transfer,
        )
        release = inspected_release(gh, repo, release_id, deadlines.discovery)
        require(
            release.get("draft") is False
            and release.get("prerelease") is (mode == "prerelease")
            and release.get("body") == notes(record)
            and release.get("tag_name") == tag
            and release.get("target_commitish") == sha
            and check_assets(release, record) == set(record["assets"]),
            "published state not confirmed",
        )
        # A final download proves permanent bytes after publication too.
        download_assets(
            gh, record, owned / "published-assets", record["assets"], deadlines.transfer
        )
    result = {
        "releaseId": release_id,
        "releaseUrl": release["html_url"],
        "assetIds": [a["id"] for a in release["assets"]],
        "sourceArtifactId": artifact["id"],
        "zipSha256": zip_digest,
        "ownership": record,
    }
    (evidence_dir / "verified.json").write_text(json.dumps(result, indent=2) + "\n")
    print(
        f"Verified {'owner-visible draft' if mode == 'draft' else mode} Release: {release['html_url']}"
    )
    print(
        f"Release ID: {release_id}; asset IDs: {result['assetIds']}; source SHA: {sha}; ZIP SHA-256: {zip_digest}; EXE SHA-256: {exe}"
    )
    return owned


# Purpose: isolated gh/Git stubs exercise actual command orchestration and asset
# bytes. They never discover user auth/config, execute PE or call GitHub services.
STUB = r"""#!/usr/bin/env python3
import hashlib, json, os, pathlib, shutil, sys, time
c = json.loads(pathlib.Path(os.environ['RELEASE_STUB_CONFIG']).read_text())
a = sys.argv[1:]
if pathlib.Path(sys.argv[0]).name == 'git':
    if c.get('gitError'): sys.exit(1)
    if a[:2] == ['cat-file', '-t']: print(c.get('tagType', 'tag'))
    elif a[:2] == ['rev-parse', '--verify']:
        print(c.get('peel', 'a'*40) if a[2].endswith('^{commit}') else ('d'*40 if a[2] in ('HEAD','refs/heads/main','refs/remotes/origin/main') else 'c'*40))
    elif a[0] == 'status': print('')
    elif a[0] == 'symbolic-ref': print('main')
    elif a[0] == 'remote': print('https://github.com/owner/repo.git')
    elif a[0] == 'show': print('[package]\nversion="' + c.get('version', '0.1.0') + '"')
    else: sys.exit(98)
    sys.exit()
state = pathlib.Path(c['state'])
assetdir = state.parent / 'assets'
assetdir.mkdir(exist_ok=True)
log = state.parent / 'calls.jsonl'
with log.open('a') as f: f.write(json.dumps(a) + '\n')
r = json.loads(state.read_text()) if state.exists() else c.get('release')
def save(): state.write_text(json.dumps(r))
def flag(key): return a[a.index(key)+1]
if c.get('apiError') and a[0] == 'api':
    print(os.environ.get('GH_TOKEN', ''))
    print('https://github.com/owner/repo/releases/tag/false-success', file=sys.stderr)
    sys.exit(1)
if a[0] == 'api':
    e = a[1]
    if '--method' in a:
        assert r['draft'] is True
        r['draft'] = False
        r['prerelease'] = 'prerelease=true' in a
        save(); print(json.dumps(r))
    elif '/git/ref/heads/main' in e: print(json.dumps({'object': {'type': 'commit', 'sha': 'd'*40}}))
    elif '/actions/workflows/release.yml' in e: print(json.dumps({'id':7,'path':'.github/workflows/release.yml','state':'active'}))
    elif '/actions/workflows/7/runs?' in e:
        print(json.dumps(c.get('releasePages', [{'workflow_runs':[{'id':29,'event':'workflow_dispatch','status':'completed','conclusion':'success'}]}])))
    elif '/git/ref/' in e: print(json.dumps({'object': {'type': c.get('remoteType','tag'), 'sha': 'c'*40}}))
    elif '/git/tags/' in e: print(json.dumps({'object': {'type': 'commit', 'sha': c.get('remoteSHA','a'*40)}}))
    elif 'artifacts?' in e: print(json.dumps(c['artifact']))
    elif '/actions/runs/29/jobs?' in e:
        print(json.dumps({'total_count':1,'jobs':[{'name':'release','conclusion':'success'}]}))
    elif '/actions/runs/29' in e:
        print(json.dumps({'id':c.get('releaseRunId',29),'head_sha':c.get('dispatchSHA','d'*40),'head_branch':c.get('dispatchBranch','main'),'workflow_id':c.get('workflowId',7),'path':c.get('releaseWorkflow','.github/workflows/release.yml'),
              'event':'workflow_dispatch','status':'completed','conclusion':'success',
              'repository':{'full_name':'owner/repo'},'html_url':'https://github.com/owner/repo/actions/runs/29'}))
    elif '/actions/runs/' in e: print(json.dumps(c['run']))
    elif '/releases?' in e: print(json.dumps([[r] if r else []]))
    elif '/releases/' in e: print(json.dumps(r))
    else: sys.exit(97)
elif a[:2] == ['run','list']:
    print(json.dumps([{'databaseId':29,'event':'workflow_dispatch','status':'completed','conclusion':'success'}]))
elif a[:2] == ['run','view']:
    if c.get('wrongReleaseLog'): print('unrelated successful run')
    else:
        record = json.loads(r['body'].split('<!-- tray-goblin-release-v1 ')[1][:-5])
        archive = 'tray-goblin-0.1.0-windows-x86_64.zip'
        print('Verified owner-visible draft Release: '+r['html_url'])
        print(f"Release ID: {r['id']}; asset IDs: {[x['id'] for x in r['assets']]}; source SHA: {record['sourceSHA']}; ZIP SHA-256: {record['assets'][archive]}; EXE SHA-256: {record['executableSha256']}")
elif a[:2] == ['run','download']:
    dest = pathlib.Path(flag('--dir'))
    for src in pathlib.Path(c['payload']).iterdir(): shutil.copyfile(src, dest / src.name)
elif a[:2] == ['release','create']:
    assert '--draft' in a and '--verify-tag' in a and '--notes-file' in a
    r = {'id': 41, 'tag_name': a[2], 'target_commitish': flag('--target'), 'draft': True, 'prerelease': False,
         'name': flag('--title'), 'body': pathlib.Path(flag('--notes-file')).read_text(),
         'html_url': 'https://github.com/owner/repo/releases/tag/untagged-owned', 'assets': []}
    save(); print(r['html_url'])
elif a[:2] == ['release','upload']:
    assert '--clobber' not in a and r['draft'] is True
    src = pathlib.Path(a[3]); name = src.name
    assert not any(x['name'] == name for x in r['assets'])
    if c.get('interrupt') and name.endswith('.sha256'):
        (src.parent.parent / 'evidence/upload-partial.bin').write_bytes(b'owned interrupted upload evidence')
        time.sleep(60)
    if c.get('uploadError'): sys.exit(1)
    if c.get('omitSecond') and name.endswith('.sha256'): sys.exit()
    shutil.copyfile(src, assetdir / name)
    r['assets'].append({'id': 51+len(r['assets']), 'name': name, 'state': 'uploaded', 'size': src.stat().st_size,
                        'digest': 'sha256:' + hashlib.sha256(src.read_bytes()).hexdigest()})
    save()
elif a[:2] == ['release','download']:
    if c.get('changeAcceptanceOnDownload'):
        pathlib.Path(c['changeAcceptanceOnDownload']).write_text('{"passed":false}')
    dest = pathlib.Path(flag('--dir'))
    for i, value in enumerate(a):
        if value == '--pattern':
            name = a[i+1]
            shutil.copyfile(assetdir / name, dest / name)
            if c.get('badDownload'): (dest / name).write_bytes(b'corrupt permanent asset')
else: sys.exit(96)
"""


def control_reports(sha, exe):
    # Positive SHAPE control only. Never written to the live repository evidence
    # area. Owner review of genuine collection remains an external M6 obligation.
    from datetime import datetime, timedelta, timezone

    instant = datetime.now(timezone.utc) - timedelta(hours=2)

    def utc(value):
        return value.isoformat()

    identity = {
        "buildSha": sha,
        "worktreeDirty": False,
        "executableSha256": exe,
        "pid": 123,
        "executablePath": "C:\\fixture\\tray-goblin.exe",
        "processStartUtc": utc(instant - timedelta(minutes=1)),
        "passed": True,
    }
    f = {
        **identity,
        "evidenceTier": "interactive-windows-wsl-observations",
        "startedUtc": utc(instant),
        "completed": True,
        "windows": {
            "caption": "Microsoft Windows 11",
            "version": "10.0.22631",
            "build": "22631",
        },
        "wslVersion": "WSL version: 2.6.0",
        "sourceKinds": ["pi", "copilot"],
        "sourceCount": 4,
        "syntheticSessionCount": 4,
        "loadEvidence": "operator-counted-actual-live-menu-rows",
        "pollIntervalMs": 1000,
        "timingMethod": "stopwatch-before-trigger-to-human-observation-conservative-upper-bound",
        "uncWakeObservation": "WOKE",
        "cases": [],
    }
    for name, (maximum, minimum) in expected_cases(1000).items():
        elapsed = max(100, minimum)
        observed = instant + timedelta(milliseconds=elapsed)
        f["cases"].append(
            {
                "id": name,
                "startedUtc": utc(instant),
                "observedUtc": utc(observed),
                "pid": 123,
                "executablePath": identity["executablePath"],
                "executableSha256": exe,
                "operatorObserved": "PASS",
                "elapsedMs": elapsed,
                "maximumMs": maximum,
                "minimumMs": minimum,
                "timingPassed": True,
                "passed": True,
            }
        )
        instant = observed
    f["finishedUtc"] = utc(instant)
    perf_start = ci.timestamp(f["startedUtc"]) - timedelta(seconds=30)
    perf = {
        **identity,
        "evidenceTier": "windows-process-resource-measurement",
        "startedUtc": utc(perf_start),
        "finishedUtc": f["startedUtc"],
        "windowsVersion": "Microsoft Windows NT 10.0.22631.0",
        "process": "tray-goblin",
        "durationSeconds": 30,
        "samples": 31,
        "logicalProcessors": 8,
        "peakWorkingSetMb": 20,
        "maxWorkingSetMb": 50,
        "workingSetPassed": True,
        "averageCpuPercent": 0.25,
        "maxIdleCpuPercent": 5,
        "cpuPassed": True,
    }
    return f, perf


def control_acceptance(sha, exe, archive, field, perf, area, payload):
    # SYNTHETIC offline admission control, never live evidence or a creation API.
    from datetime import timedelta
    utc = lambda value: value.isoformat()
    start = min(ci.timestamp(field["startedUtc"]), ci.timestamp(perf["startedUtc"]))
    begin, finish = start - timedelta(minutes=30), start - timedelta(minutes=1)
    end = ci.timestamp(perf["finishedUtc"]) + timedelta(minutes=30)
    identity = {"repo": "owner/repo", "tag": "v0.1.0", "sourceRun": 17, "sourceSHA": sha,
                "archiveSha256": archive, "executableSha256": exe,
                "fieldSha256": hashlib.sha256((area / "field.json").read_bytes()).hexdigest(),
                "performanceSha256": hashlib.sha256((area / "perf.json").read_bytes()).hexdigest()}
    commands = []
    for argv in manual_commands("v0.1.0"):
        result = "OFFLINE synthetic command outcome; not actual manual validation."
        if argv[:2] == ["git", "status"]: result = ""
        if argv[:2] == ["git", "rev-parse"]: result = sha
        if argv[:2] == ["git", "remote"]: result = "https://github.com/owner/repo.git"
        if argv[:2] == ["git", "cat-file"]: result = "tag"
        if argv[:2] == ["bash", "scripts/package-release.sh"]:
            result = "Reproducible: two independent packing runs matched. SHA-256 " + archive
        commands.append({"argv": argv, "exit": 0, "startedUtc": utc(begin), "finishedUtc": utc(begin),
                         "result": result, "resultSha256": hashlib.sha256(result.encode()).hexdigest()})
    manual = area / "manual-bytes"
    manual.mkdir(exist_ok=True)
    for src in payload.iterdir(): shutil.copyfile(src, manual / src.name)
    targets = []
    for os_name, build in (("Windows10", "19045"), ("Windows11", "22631")):
        targets.append({"os": os_name, "architecture": "x64", "windows": {"caption": "Microsoft " + os_name.replace("Windows", "Windows "), "version": "10.0." + build, "build": build},
                        "observer": "fixture-operator", "startedUtc": utc(start), "finishedUtc": utc(end),
                        "checks": {k: {facet: {"outcome": "PASS", "observedUtc": utc(start), "observation": "OFFLINE synthetic observation, not Windows evidence: " + facet}
                                       for facet in facets} for k, facets in CHECKLIST.items()}})
    coverage = {k: "OFFLINE synthetic review of " + k + "; not actual independent acceptance."
                for k in ("manual-source-validation-repro", "manual-ci-byte-equivalence", "Windows10-all11", "Windows11-all11", "field-55-wsl-privacy", "performance-supported-same-host-budgets")}
    return {"schema": "tray-goblin-acceptance-v1", "identity": identity,
            "createdUtc": utc(end + timedelta(minutes=3)), "author": "fixture-author",
            "manual": {"sourceSHA": sha, "tag": "v0.1.0", "worktreeDirty": False, "canonicalMainSHA": sha,
                       "builder": "fixture-builder", "startedUtc": utc(begin), "finishedUtc": utc(finish), "commands": commands,
                       "archivePath": EVIDENCE_AREA + "/manual-bytes/" + next(payload.glob("*.zip")).name},
            "targets": targets,
            "reviews": [{"role": role, "reviewer": "fixture-reviewer-" + role, "reviewedUtc": utc(end + timedelta(minutes=1)),
                         "verdict": "ACCEPT", "identity": identity, "coverage": coverage, "findings": []} for role in ("spec", "quality", "rugged")],
            "operator": {"identity": identity, "name": "fixture-operator", "acceptedUtc": utc(end + timedelta(minutes=2)),
                         "decision": "AUTHORIZE-PUBLIC-PRERELEASE-AND-STABLE", "statement": "OFFLINE synthetic authorization, not permission to publish."}}


def self_test():
    external = os.environ.get("TRAY_GOBLIN_EVIDENCE_DIR")
    require(
        Deadlines() == Deadlines(120, 1500, 120), "self-test: production deadline drift"
    )
    root = Path(tempfile.mkdtemp(prefix="release-offline-controls-", dir=external))
    bin_dir = root / "bin"
    bin_dir.mkdir()
    for tool in ("gh", "git"):
        path = bin_dir / tool
        path.write_text(STUB)
        path.chmod(0o755)
    payload = root / "payload"
    payload.mkdir()
    sha = "a" * 40
    original_payload, zip_digest = ci.fixture(payload, sha)
    exe = hashlib.sha256(original_payload["tray-goblin.exe"]).hexdigest()
    f, perf = control_reports(sha, exe)
    caller = root / "caller"
    caller.mkdir()
    sentinel = caller / "sentinel"
    sentinel.write_bytes(b"caller-owned unknown file must remain intact")
    original = sentinel.read_bytes()
    secret = "OFFLINE_SECRET_MUST_NEVER_BE_SERIALIZED"
    # Deliberately no ambient auth/config/session environment in fake children.
    env = {
        "PATH": f"{bin_dir}:/usr/bin:/bin",
        "HOME": str(root / "home"),
        "USERPROFILE": str(root / "home"),
        **{k: str(root / "home" / k) for k in ("XDG_CONFIG_HOME", "XDG_CACHE_HOME", "XDG_DATA_HOME", "XDG_STATE_HOME")},
        "PYTHONDONTWRITEBYTECODE": "1",
        "GH_TOKEN": secret,
    }
    Path(env["HOME"]).mkdir()
    count = 0
    config_base = {
        "payload": str(payload),
        "run": {
            "id": 17,
            "head_sha": sha,
            "path": ci.WORKFLOW,
            "event": "push",
            "head_branch": "main",
            "repository": {"full_name": "owner/repo"},
            "html_url": "https://github.com/owner/repo/actions/runs/17",
            "created_at": "2026-01-01T00:00:00Z",
            "status": "completed",
            "conclusion": "success",
        },
        "artifact": {
            "total_count": 1,
            "artifacts": [
                {
                    "id": 23,
                    "name": ci.ARTIFACT,
                    "expired": False,
                    "size_in_bytes": 1234,
                    "created_at": "2026-01-01T00:00:01Z",
                    "expires_at": "2099-01-01T00:00:00Z",
                    "workflow_run": {"id": 17, "head_sha": sha, "head_branch": "main"},
                    "archive_download_url": "https://api.github.com/repos/owner/repo/actions/artifacts/23/zip",
                }
            ],
        },
    }
    tested = []

    def exercise(
        label,
        change=None,
        field=f,
        resource=perf,
        acceptance=True,
        mode="draft",
        success=False,
        session=None,
        module=None,
        tag="v0.1.0",
        forbid_mutation=False,
        live_entry=False,
    ):
        nonlocal count
        session = session or root / f"case-{count}"
        session.mkdir(exist_ok=True)
        checkout = session / "checkout"
        area = checkout / EVIDENCE_AREA
        area.mkdir(parents=True, exist_ok=True)
        if field is not None:
            (area / "field.json").write_text(json.dumps(field))
        if resource is not None:
            (area / "perf.json").write_text(json.dumps(resource))
        if field is not None and resource is not None and acceptance is not None:
            a = control_acceptance(sha, exe, zip_digest, field, resource, area, payload)
            if callable(acceptance): acceptance(a)
            (area / "acceptance.json").write_text(json.dumps(a))
        c = copy.deepcopy(config_base)
        c["state"] = str(session / "state.json")
        if change:
            change(c, session)
        config = session / "config.json"
        config.write_text(json.dumps(c))
        before = set(caller.iterdir())
        code = (
            "import importlib.util,sys; s=importlib.util.spec_from_file_location('release_test',sys.argv[1]); "
            "m=importlib.util.module_from_spec(s);sys.modules[s.name]=m;s.loader.exec_module(m); "
            "m.prepare('owner/repo',sys.argv[4],17,sys.argv[5],'.wgm/release-evidence/field.json',"
            "'.wgm/release-evidence/perf.json',sys.argv[2],sys.argv[3],m.Deadlines(2,2,2))"
        )
        command = [
            sys.executable,
            "-B",
            "-c",
            code,
            str(module or Path(__file__)),
            str(caller),
            str(checkout),
            tag,
            mode,
        ]
        child_env = {**env, "RELEASE_STUB_CONFIG": str(config),
                     "RELEASE_ACCEPTANCE_EVIDENCE": EVIDENCE_AREA + "/acceptance.json"}
        if live_entry:
            command = [sys.executable, "-B", str(Path(__file__))]
            child_env.update(
                RELEASE_REPO="owner/repo",
                RELEASE_TAG=tag,
                RELEASE_BUILD_RUN_ID="17",
                RELEASE_MODE=mode,
                RELEASE_OUT_ROOT=str(caller),
            )
        start = time.monotonic()
        result = subprocess.run(
            command,
            env=child_env,
            capture_output=True,
            timeout=7,
        )
        elapsed = time.monotonic() - start
        # Secrets from an erroring gh MUST have been suppressed by the real code.
        require(
            secret.encode() not in result.stdout + result.stderr,
            "self-test: secret leaked",
        )
        (session / f"{label}.stdout").write_bytes(result.stdout)
        (session / f"{label}.stderr").write_bytes(result.stderr)
        (session / f"{label}.result.json").write_text(
            json.dumps(
                {
                    "command": command,
                    "exit": result.returncode,
                    "elapsedSeconds": elapsed,
                }
            )
            + "\n"
        )
        calls = (
            [
                decode(line)
                for line in (session / "calls.jsonl").read_text().splitlines()
            ]
            if (session / "calls.jsonl").exists()
            else []
        )
        require(sentinel.read_bytes() == original, "self-test: caller sentinel changed")
        children = set(caller.iterdir()) - before
        require(len(children) <= 1, "self-test: unexpected local ownership")
        owned = next(iter(children)) if children else None
        require(
            (result.returncode == 0) == success, f"self-test: wrong exit for {label}"
        )
        if success:
            verified = decode((owned / "evidence/verified.json").read_text())
            require(
                verified["zipSha256"] == zip_digest
                and verified["ownership"]["executableSha256"] == exe,
                "self-test: wrong verified bytes",
            )
            require(
                b"Verified " in result.stdout
                and b"https://github.com/owner/repo/releases/" in result.stdout,
                "self-test: missing actual Release success URL",
            )
            require(
                sum("--method" in a for a in calls) == (mode != "draft"),
                "self-test: forbidden/missing publish mutation",
            )
        else:
            require(
                b"https://github.com/" not in result.stdout + result.stderr
                and b"Verified " not in result.stdout,
                "self-test: premature success URL",
            )
            require(
                not owned or not (owned / "evidence/verified.json").exists(),
                "self-test: false verified record",
            )
            require(
                not any("--method" in a for a in calls),
                "self-test: forbidden publish mutation",
            )
        if forbid_mutation:
            require(
                not any(
                    a[:2] in (["release", "create"], ["release", "upload"])
                    or "--method" in a
                    for a in calls
                ),
                "self-test: forbidden Release mutation",
            )
        count += 1
        tested.append(label)
        return session, owned, calls, elapsed

    # Tag/source/artifact gates must reject BEFORE any Release mutation.
    changes = {
        "missing-tag": lambda c, s: c.update(gitError=True),
        "unpeeled-tag-object": lambda c, s: c.update(peel="c" * 40),
        "lightweight-tag": lambda c, s: c.update(tagType="commit"),
        "wrong-remote-tag": lambda c, s: c.update(remoteSHA="b" * 40),
        "missing-remote-annotated-tag": lambda c, s: c.update(remoteType="commit"),
        "version-mismatch": lambda c, s: c.update(version="0.2.0"),
        "source-mismatch": lambda c, s: c["run"].update(head_sha="b" * 40),
        "wrong-workflow": lambda c, s: c["run"].update(path=RELEASE_WORKFLOW),
        "failed-source": lambda c, s: c["run"].update(conclusion="failure"),
        "incomplete-source": lambda c, s: c["run"].update(status="in_progress"),
        "wrong-artifact-sha": lambda c, s: c["artifact"]["artifacts"][0][
            "workflow_run"
        ].update(head_sha="b" * 40),
        "expired-artifact": lambda c, s: c["artifact"]["artifacts"][0].update(
            expired=True
        ),
        "stale-artifact": lambda c, s: c["artifact"]["artifacts"][0].update(
            expires_at="2000-01-01T00:00:00Z"
        ),
        "wrong-artifact": lambda c, s: c["artifact"]["artifacts"][0].update(
            name="unknown"
        ),
        "missing-artifact": lambda c, s: c["artifact"].update(
            total_count=0, artifacts=[]
        ),
        "ambiguous-artifact": lambda c, s: c["artifact"].update(
            total_count=2, artifacts=c["artifact"]["artifacts"] * 2
        ),
        "api-error-with-secret": lambda c, s: c.update(apiError=True),
    }
    for label, change in changes.items():
        _, _, calls, _ = exercise(label, change)
        require(
            not any(a[0] == "release" for a in calls),
            "self-test: invalid source reached Release mutation",
        )
    exercise("invalid-semver", tag="v01.0.0")
    exercise("missing-tag-value", tag="")
    exercise("shell-tag-data", tag="v0.1.0;echo injected")

    # Each expected case is indispensable, even with completed/passed true.
    for name in expected_cases(1000):
        altered = copy.deepcopy(f)
        altered["cases"] = [case for case in altered["cases"] if case["id"] != name]
        _, _, calls, _ = exercise("missing-" + name, field=altered, mode="prerelease")
        require(
            not any(a[0] == "release" for a in calls),
            "self-test: partial evidence reached mutation",
        )
    field_changes = {
        "fake-tier": lambda r: r.update(evidenceTier="offline-synthetic-fixture"),
        "partial": lambda r: r.update(completed=False),
        "dirty": lambda r: r.update(worktreeDirty=True),
        "wrong-build": lambda r: r.update(buildSha="b" * 40),
        "wrong-exe": lambda r: r.update(executableSha256="0" * 64),
        "missing-windows": lambda r: r.update(windows={}),
        "missing-wsl": lambda r: r.update(wslVersion=""),
        "missing-source": lambda r: r.update(sourceKinds=["pi"]),
        "fake-load": lambda r: r.update(loadEvidence="synthetic-capability-answer"),
        "wrong-cadence": lambda r: r.update(pollIntervalMs=1),
        "unc-fail": lambda r: r.update(uncWakeObservation="FAIL"),
        "observed-fail": lambda r: r["cases"][0].update(operatorObserved="FAIL"),
        "case-flag-fail": lambda r: r["cases"][0].update(passed=False),
        "timing-flag-fail": lambda r: r["cases"][0].update(timingPassed=False),
        "elapsed-too-slow": lambda r: r["cases"][0].update(elapsedMs=2501),
        "changed-maximum": lambda r: r["cases"][0].update(maximumMs=999999),
        "wrong-case-digest": lambda r: r["cases"][0].update(executableSha256="0" * 64),
        "duplicate-case": lambda r: r["cases"].__setitem__(
            1, copy.deepcopy(r["cases"][0])
        ),
        "contradictory-time": lambda r: r["cases"][0].update(
            observedUtc=r["processStartUtc"]
        ),
        "quiet-too-short": lambda r: next(
            c for c in r["cases"] if c["minimumMs"]
        ).update(elapsedMs=100),
    }
    perf_changes = {
        "fake-tier": lambda r: r.update(evidenceTier="linux-synthetic-resource"),
        "dirty": lambda r: r.update(worktreeDirty=True),
        "wrong-source": lambda r: r.update(buildSha="b" * 40),
        "wrong-exe": lambda r: r.update(executableSha256="0" * 64),
        "short-window": lambda r: r.update(durationSeconds=29.99),
        "no-samples": lambda r: r.update(samples=1),
        "no-processors": lambda r: r.update(logicalProcessors=0),
        "wrong-os": lambda r: r.update(windowsVersion="Linux"),
        "memory-budget": lambda r: r.update(maxWorkingSetMb=51),
        "cpu-budget": lambda r: r.update(maxIdleCpuPercent=6),
        "memory-excess": lambda r: r.update(peakWorkingSetMb=50.01),
        "cpu-excess": lambda r: r.update(averageCpuPercent=5.01),
        "raw-memory-fail": lambda r: r.update(workingSetPassed=False),
        "raw-cpu-fail": lambda r: r.update(cpuPassed=False),
        "overall-fail": lambda r: r.update(passed=False),
        "nan": lambda r: r.update(averageCpuPercent=float("nan")),
    }
    for kind, source, mutations in (
        ("field", f, field_changes),
        ("performance", perf, perf_changes),
    ):
        for label, mutate in mutations.items():
            altered = copy.deepcopy(source)
            mutate(altered)
            _, _, calls, _ = exercise(
                kind + "-" + label,
                field=altered if kind == "field" else f,
                resource=altered if kind == "performance" else perf,
                mode="stable",
            )
            require(
                not any(a[0] == "release" for a in calls),
                "self-test: invalid evidence reached mutation",
            )
    exercise("missing-field", field=None, mode="prerelease")
    exercise("missing-performance", resource=None, mode="stable")
    acceptance_changes = {
        "absent": None,
        "partial": lambda a: a["targets"][0]["checks"].pop("tooltip"),
        "failed-install": lambda a: a["targets"][0]["checks"]["install"]["standard-user"].update(outcome="FAIL"),
        "failed-startup": lambda a: a["targets"][1]["checks"]["startup"]["sign-out-sign-in-autostart"].update(outcome="FAIL"),
        "failed-uninstall": lambda a: a["targets"][0]["checks"]["uninstall"]["remove-configuration"].update(outcome="FAIL"),
        "no-reviews": lambda a: a.update(reviews=[]),
        "non-independent": lambda a: a["reviews"][0].update(reviewer=a["author"]),
        "duplicate-reviewer": lambda a: a["reviews"][1].update(reviewer=a["reviews"][0]["reviewer"]),
        "reviewer-identity-whitespace": lambda a: a["reviews"][0].update(reviewer=" " + a["author"]),
        "reviewer-identity-case": lambda a: a["reviews"][0].update(reviewer=a["author"].upper()),
        "generic-review": lambda a: a["reviews"][0].update(coverage={}),
        "failed-review": lambda a: a["reviews"][0].update(verdict="REJECT"),
        "failed-validation": lambda a: a["manual"]["commands"][8].update(exit=1),
        "missing-validation": lambda a: a["manual"].update(commands=[]),
        "dirty-source": lambda a: a["manual"].update(worktreeDirty=True),
        "wrong-canonical": lambda a: a["manual"].update(canonicalMainSHA="b" * 40),
        "one-os": lambda a: a["targets"].pop(),
        "wrong-arch": lambda a: a["targets"][0].update(architecture="x86"),
        "undated": lambda a: a["targets"][0]["checks"]["startup"]["sign-out-sign-in-autostart"].update(observedUtc=""),
        "no-operator": lambda a: a["operator"].update(decision="NOT-ACCEPTED"),
        "failed-repro-result": lambda a: a["manual"]["commands"][-1].update(result="Failed reproducibility"),
        "wrong-command-result-digest": lambda a: a["manual"]["commands"][8].update(resultSha256="0" * 64),
        "stale-acceptance": lambda a: a.update(createdUtc="2020-01-01T00:00:00Z"),
        "stale-review": lambda a: a["reviews"][0].update(reviewedUtc="2020-01-01T00:00:00Z"),
        "wrong-run": lambda a: a["identity"].update(sourceRun=99),
        "wrong-sha": lambda a: a["identity"].update(sourceSHA="b" * 40),
        "wrong-archive": lambda a: a["identity"].update(archiveSha256="0" * 64),
        "wrong-exe": lambda a: a["identity"].update(executableSha256="0" * 64),
        "wrong-field-digest": lambda a: a["identity"].update(fieldSha256="0" * 64),
        "wrong-perf-digest": lambda a: a["identity"].update(performanceSha256="0" * 64),
    }
    for mode in ("prerelease", "stable"):
        for target in (0, 1):
            for item in CHECKLIST:
                exercise(f"acceptance-{mode}-target{target}-missing-{item}",
                         acceptance=lambda a, t=target, k=item: a["targets"][t]["checks"].pop(k),
                         mode=mode, forbid_mutation=True)
        def manual_mismatch(c, session):
            manual = session / "checkout" / EVIDENCE_AREA / "manual-bytes"
            archive = next(manual.glob("*.zip"))
            with zipfile.ZipFile(archive, "a") as z: z.comment = b"different manual ZIP bytes"
            (manual / (archive.name + ".sha256")).write_text(hashlib.sha256(archive.read_bytes()).hexdigest() + "  " + archive.name + "\n")
        exercise("manual-ci-byte-mismatch-" + mode, manual_mismatch, mode=mode, forbid_mutation=True)
        exercise("acceptance-recheck-" + mode,
                 lambda c, session: c.update(changeAcceptanceOnDownload=str(session / "checkout" / EVIDENCE_AREA / "acceptance.json")), mode=mode)
        for label, change in acceptance_changes.items():
            exercise("acceptance-" + mode + "-" + label, acceptance=change, mode=mode, forbid_mutation=True)
        for label, update in {
            "windows7": {"windowsVersion": "Microsoft Windows NT 6.1.7601.0"},
            "wrong-build": {"windowsVersion": "Microsoft Windows NT 10.0.19045.0"},
            "wrong-pid": {"pid": 456},
            "wrong-process-start": {"processStartUtc": "2020-01-01T00:00:00Z"},
            "wrong-host-path": {"executablePath": "C:\\other\\tray-goblin.exe"},
        }.items():
            altered = {**perf, **update}
            exercise("provenance-" + mode + "-" + label, resource=altered, mode=mode, forbid_mutation=True)
    for mode in ("draft", "prerelease", "stable"):
        exercise(
            "positive-shape-only-" + mode,
            mode=mode,
            success=True,
            live_entry=(mode == "draft"),
        )

    # Byte counterfactuals use REAL ZIP/manifest/PE bytes with updated sibling SHA;
    # corruption must hit the intended deeper validation, not just stale checksum.
    payload_mutations = {
        "wrong-build": (
            "resources/build.json",
            json.dumps(
                {
                    "buildCommit": "b" * 40,
                    "worktreeDirty": False,
                    "published": False,
                    "executableSha256": exe,
                }
            ).encode(),
        ),
        "dirty-build": (
            "resources/build.json",
            json.dumps(
                {
                    "buildCommit": sha,
                    "worktreeDirty": True,
                    "published": False,
                    "executableSha256": exe,
                }
            ).encode(),
        ),
        "published-build": (
            "resources/build.json",
            json.dumps(
                {
                    "buildCommit": sha,
                    "worktreeDirty": False,
                    "published": True,
                    "executableSha256": exe,
                }
            ).encode(),
        ),
        "wrong-manifest-exe": (
            "resources/build.json",
            json.dumps(
                {
                    "buildCommit": sha,
                    "worktreeDirty": False,
                    "published": False,
                    "executableSha256": "0" * 64,
                }
            ).encode(),
        ),
        "not-pe": ("tray-goblin.exe", b"not executable"),
        "x86-pe": (
            "tray-goblin.exe",
            original_payload["tray-goblin.exe"][:68]
            + b"\x4c\x01"
            + original_payload["tray-goblin.exe"][70:],
        ),
        "console-pe": (
            "tray-goblin.exe",
            original_payload["tray-goblin.exe"][:156]
            + b"\x03\x00"
            + original_payload["tray-goblin.exe"][158:],
        ),
        "missing-resource": ("resources/SETUP.md", None),
        "traversal": ("../escape", b"unsafe"),
    }
    for label, (entry, data) in payload_mutations.items():

        def alter(c, session, entry=entry, data=data):
            bad = session / "payload"
            bad.mkdir()
            archive = bad / next(payload.glob("*.zip")).name
            with (
                zipfile.ZipFile(next(payload.glob("*.zip"))) as src,
                zipfile.ZipFile(archive, "w") as dst,
            ):
                for info in src.infolist():
                    if info.filename.endswith("/" + entry):
                        if data is not None:
                            dst.writestr(info, data)
                    else:
                        dst.writestr(info, src.read(info))
                if entry.startswith("../"):
                    dst.writestr(entry, data)
            (bad / (archive.name + ".sha256")).write_text(
                hashlib.sha256(archive.read_bytes()).hexdigest()
                + "  "
                + archive.name
                + "\n"
            )
            c["payload"] = str(bad)

        _, _, calls, _ = exercise("payload-" + label, alter)
        require(
            not any(a[0] == "release" for a in calls),
            "self-test: corrupt payload reached mutation",
        )
    for label in ("corrupt-zip", "bad-sha", "missing-sha", "wrong-version"):

        def alter(c, session, label=label):
            bad = session / "payload"
            shutil.copytree(payload, bad)
            archive = next(bad.glob("*.zip"))
            checksum = bad / (archive.name + ".sha256")
            if label == "corrupt-zip":
                archive.write_bytes(b"corrupt ZIP")
                checksum.write_text(
                    hashlib.sha256(archive.read_bytes()).hexdigest()
                    + "  "
                    + archive.name
                    + "\n"
                )
            elif label == "bad-sha":
                checksum.write_text("0" * 64 + "  " + archive.name + "\n")
            elif label == "missing-sha":
                checksum.unlink()
            else:
                new = bad / archive.name.replace("0.1.0", "0.2.0")
                archive.rename(new)
                checksum.unlink()
                (bad / (new.name + ".sha256")).write_text(
                    hashlib.sha256(new.read_bytes()).hexdigest()
                    + "  "
                    + new.name
                    + "\n"
                )
            c["payload"] = str(bad)

        exercise(label, alter)

    # Seed owned drafts to protect existing published/different/unknown state.
    base_record = ownership("owner/repo", "v0.1.0", sha, 17, "draft", payload, exe, {})

    def seed(c, session):
        c["release"] = {
            "id": 41,
            "tag_name": "v0.1.0",
            "target_commitish": sha,
            "draft": True,
            "prerelease": False,
            "name": "TrayGoblin v0.1.0 (draft)",
            "body": notes(base_record),
            "html_url": "https://github.com/owner/repo/releases/tag/untagged-owned",
            "assets": [],
        }

    for label, mutate in {
        "already-published": lambda r: r.update(draft=False),
        "different-notes": lambda r: r.update(body="unowned draft"),
        "different-commit": lambda r: r.update(target_commitish="b" * 40),
        "different-mode": lambda r: r.update(name="TrayGoblin v0.1.0 (stable)"),
        "unknown-asset": lambda r: r["assets"].append(
            {"id": 51, "name": "unknown.txt", "state": "uploaded"}
        ),
        "different-asset": lambda r: r["assets"].append(
            {
                "id": 51,
                "name": next(payload.glob("*.zip")).name,
                "state": "uploaded",
                "digest": "sha256:" + "0" * 64,
            }
        ),
    }.items():

        def change(c, s, mutate=mutate):
            seed(c, s)
            mutate(c["release"])

        _, _, calls, _ = exercise(label, change)
        require(
            not any(
                a[:2] in (["release", "upload"], ["release", "create"])
                or "--method" in a
                for a in calls
            ),
            "self-test: protected draft/published mutation",
        )
    # A server-side success with a missing second asset is still a hard failure.
    exercise(
        "missing-second-asset",
        lambda c, s: c.update(omitSecond=True),
        mode="prerelease",
    )
    exercise(
        "corrupt-permanent-download",
        lambda c, s: c.update(badDownload=True),
        mode="stable",
    )

    # Matching metadata without an API digest cannot hide different existing bytes.
    def wrong_existing(c, session):
        seed(c, session)
        name = next(payload.glob("*.zip")).name
        c["release"]["assets"] = [
            {"id": 51, "name": name, "state": "uploaded", "digest": None}
        ]
        asset_dir = session / "assets"
        asset_dir.mkdir()
        (asset_dir / name).write_bytes(b"unknown different draft bytes")

    exercise(
        "wrong-existing-bytes-without-api-digest", wrong_existing, forbid_mutation=True
    )
    exercise("upload-api-failure", lambda c, s: c.update(uploadError=True))
    session, partial, _, elapsed = exercise(
        "interrupted-second-upload", lambda c, s: c.update(interrupt=True)
    )
    require(
        2 <= elapsed <= 7
        and (partial / "evidence/upload-partial.bin").read_bytes()
        == b"owned interrupted upload evidence",
        "self-test: bounded partial evidence missing",
    )
    require(
        len(decode((session / "state.json").read_text())["assets"]) == 1,
        "self-test: first permanent asset not retained",
    )
    (session / "assets/unknown-local-sentinel").write_bytes(b"unknown local file")
    _, retry, calls, _ = exercise("matching-draft-retry", success=True, session=session)
    require(
        retry != partial
        and (partial / "evidence/upload-partial.bin").exists()
        and (session / "assets/unknown-local-sentinel").read_bytes()
        == b"unknown local file",
        "self-test: retry overwrote unknown/partial files",
    )
    require(
        sum(a[:2] == ["release", "upload"] for a in calls) == 3,
        "self-test: retry must upload only missing asset (including prior calls)",
    )

    # Path/JSON policy is DATA-only and bounded; no report content appears in errors.
    path_controls = 0
    checkout = root / "path-checkout"
    area = checkout / EVIDENCE_AREA
    area.mkdir(parents=True)
    (area / "good.json").write_text(json.dumps(f))
    (area / "link.json").symlink_to(area / "good.json")
    (area / "big.json").write_bytes(b" " * (256 * 1024 + 1))
    (area / "duplicate.json").write_text('{"passed":true,"passed":false}')
    for path in (
        "",
        "/tmp/report.json",
        ".wgm/release-evidence/../good.json",
        ".wgm/release-evidence/link.json",
        ".wgm/release-evidence/big.json",
        ".wgm/release-evidence/duplicate.json",
        "tests/fixtures/fake.json",
    ):
        try:
            repository_json(path, checkout)
        except (Error, ValueError, OSError):
            path_controls += 1
        else:
            raise Error("self-test: unsafe evidence path/JSON accepted")

    # Executable counterfactuals prove controls kill a bad gate/mutation, not just
    # count reassuring strings. Mutants are isolated files, never live code edits.
    mutants = [
        (
            "bypass-public-evidence",
            'gates = {} if mode == "draft" else public_gate()',
            "gates = {}",
            "missing-field-mutant",
            {"field": None, "mode": "prerelease", "forbid_mutation": True},
        ),
        (
            "allow-published",
            'isinstance(release, dict) and release.get("draft") is True',
            "isinstance(release, dict)",
            "published-mutant",
            {},
        ),
    ]
    killed = 0
    for name, old, new, label, kwargs in mutants:
        source = (
            Path(__file__).read_text().split("# Purpose: isolated gh/Git stubs", 1)[0]
        )
        require(source.count(old) == 1, "self-test: mutant seam drift")
        mutant = root / (name + ".py")
        mutant.write_text(
            source.replace(old, new).replace(
                "HERE = Path(__file__).resolve().parent",
                "HERE = Path(" + repr(str(HERE)) + ")",
            )
        )
        if name == "allow-published":

            def change(c, s):
                seed(c, s)
                c["release"]["draft"] = False

            kwargs["change"] = change
            kwargs["forbid_mutation"] = True
        try:
            exercise(label, module=mutant, **kwargs)
        except Error as exc:
            require(
                "wrong exit" in str(exc) or "forbidden" in str(exc),
                "self-test: unrelated mutant failure",
            )
            killed += 1
        else:
            raise Error("self-test: unsafe executable mutant survived")

    # Compare matrix IDs to the actual collector source (not just our own count).
    collector = (HERE / "test-attention-windows.ps1").read_text()
    literal_ids = set(re.findall(r"^Observe-Case '([^']+)'", collector, re.M))
    require(
        literal_ids == set(list(expected_cases(1000))[44:]),
        "self-test: collector literal case policy drift",
    )
    require(
        "@('native', 'wsl')" in collector
        and "@('confirm', 'select', 'input', 'editor', 'custom', 'manual-settings', 'manual-compaction')"
        in collector,
        "self-test: collector matrix drift",
    )
    for suffix in (
        "final-error",
        "error-deduplicated",
        "error-dialog",
        "new-work-clear",
        "completion-quiet",
        "abort-recovered-quiet",
        "copilot-persisted-error",
        "copilot-new-turn-clear",
    ):
        require(f'"$origin-{suffix}"' in collector, "self-test: collector suffix drift")
    # Exercise the exact REL2 shell boundary with gh/Git stubs only, including
    # its separate dispatch SHA. This proves wiring, NOT actual GitHub availability.
    helper_spec = importlib.util.spec_from_file_location(
        "release_verification", HERE / "verify-release.py"
    )
    helper = importlib.util.module_from_spec(helper_spec)
    helper_spec.loader.exec_module(helper)
    live_release = decode((session / "state.json").read_text())
    require(
        helper.draft_record(live_release, "owner/repo", "v0.1.0") == base_record,
        "self-test: REL2 ownership parse mismatch",
    )
    for label, change in (
        ("REL2-success", {}),
        ("REL2-unrelated-running-first", {"releasePages": [{"workflow_runs": [{"id":99,"event":"workflow_dispatch","status":"in_progress","conclusion":None},{"id":29,"event":"workflow_dispatch","status":"completed","conclusion":"success"}]}]}),
        ("REL2-paginated-running-first", {"releasePages": [{"workflow_runs": [{"id":99,"event":"workflow_dispatch","status":"in_progress","conclusion":None}]},{"workflow_runs": [{"id":29,"event":"workflow_dispatch","status":"completed","conclusion":"success"}]}]}),
        ("REL2-wrong-workflow", {"releaseWorkflow": ci.WORKFLOW}),
        ("REL2-unrelated-run", {"wrongReleaseLog": True}),
        ("REL2-nonmain", {"dispatchBranch": "untrusted-branch"}),
        ("REL2-missing-branch", {"dispatchBranch": None}),
        ("REL2-invalid-sha", {"dispatchSHA": "not-a-sha"}),
        ("REL2-copied-log-wrong-source", {"dispatchSHA": "e" * 40}),
        ("REL2-row-id-mismatch", {"releaseRunId": 99}),
        ("REL2-workflow-id", {"workflowId": 8}),
        ("REL2-corrupt-asset", {"badDownload": True}),
    ):
        cfg = decode((session / "config.json").read_text())
        cfg.update(change)
        config_path = session / (label + ".config.json")
        config_path.write_text(json.dumps(cfg))
        out = root / label
        out.mkdir()
        if label == "REL2-success":
            command = [
                "bash",
                str(HERE / "verify-release.sh"),
                "--repo",
                "owner/repo",
                "--tag",
                "v0.1.0",
                "--draft",
                "--out-root",
                str(out),
            ]
        else:
            code = (
                "import importlib.util,sys;s=importlib.util.spec_from_file_location('rel2',sys.argv[1]);"
                "m=importlib.util.module_from_spec(s);s.loader.exec_module(m);"
                "m.verify('owner/repo','v0.1.0',sys.argv[2],m.p.Deadlines(2,2,2))"
            )
            command = [
                sys.executable,
                "-B",
                "-c",
                code,
                str(HERE / "verify-release.py"),
                str(out),
            ]
        before_calls = (session / "calls.jsonl").read_text().splitlines()
        start = time.monotonic()
        result = subprocess.run(
            command,
            env={**env, "RELEASE_STUB_CONFIG": str(config_path)},
            capture_output=True,
            timeout=7,
        )
        (out / "stdout.log").write_bytes(result.stdout)
        (out / "stderr.log").write_bytes(result.stderr)
        (out / "result.json").write_text(
            json.dumps(
                {
                    "command": command,
                    "exit": result.returncode,
                    "elapsedSeconds": time.monotonic() - start,
                }
            )
            + "\n"
        )
        expected_success = label in ("REL2-success", "REL2-unrelated-running-first", "REL2-paginated-running-first")
        require(
            (result.returncode == 0) == expected_success,
            "self-test: REL2 wrong exit",
        )
        if expected_success:
            verified = decode(
                next(
                    out.glob(
                        "tray-goblin-release-verification-*/evidence/verified.json"
                    )
                ).read_text()
            )
            require(
                verified["dispatchSourceSHA"] == "d" * 40
                and verified["sourceSHA"] == sha
                and verified["zipSha256"] == zip_digest
                and b"https://github.com/owner/repo/releases/" in result.stdout,
                "self-test: REL2 wrong actual source/Release identity",
            )
        else:
            require(
                b"https://github.com/" not in result.stdout + result.stderr
                and not list(out.glob("*/evidence/verified.json")),
                "self-test: REL2 false success",
            )
        calls = [
            decode(line)
            for line in (session / "calls.jsonl")
            .read_text()
            .splitlines()[len(before_calls) :]
        ]
        require(
            not any(
                a[:2] in (["release", "create"], ["release", "upload"], ["run", "watch"])
                or "--method" in a
                for a in calls
            ),
            "self-test: REL2 mutation or unrelated watch",
        )
        tested.append(label)
    for path in root.rglob("*"):
        if path.is_file() and not path.is_symlink():
            require(
                secret.encode() not in path.read_bytes(), "self-test: secret serialized"
            )
    (root / "summary.json").write_text(
        json.dumps(
            {
                "tier": "offline-controls-NOT-live-M6-or-Release-evidence",
                "cases": tested,
                "commandControls": count,
                "pathControls": path_controls,
                "executableMutantsRejected": killed,
                "zipSha256": zip_digest,
                "executableSha256": exe,
                "partialOwnedChild": str(partial),
                "newRetryChild": str(retry),
                "callerSentinelIntact": sentinel.read_bytes() == original,
            },
            indent=2,
        )
        + "\n"
    )
    print(
        f"Release OFFLINE self-test passed: {count} command controls, {path_controls} path/JSON controls, {killed} executable mutants rejected; 55 mandatory field cases."
    )
    print(f"Offline evidence retained: {root}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    # Dispatch inputs are environment DATA, never shell syntax/CLI code.
    run_id = os.environ.get("RELEASE_BUILD_RUN_ID", "")
    require(re.fullmatch(r"[1-9][0-9]{0,19}", run_id), "source run ID required")
    prepare(
        os.environ.get("RELEASE_REPO", ""),
        os.environ.get("RELEASE_TAG", ""),
        int(run_id),
        os.environ.get("RELEASE_MODE", "draft"),
        os.environ.get("RELEASE_FIELD_EVIDENCE", ""),
        os.environ.get("RELEASE_PERFORMANCE_EVIDENCE", ""),
        os.environ.get("RELEASE_OUT_ROOT", tempfile.gettempdir()),
    )


if __name__ == "__main__":
    try:
        main()
    except (
        Error,
        OSError,
        ValueError,
        KeyError,
        TypeError,
        struct.error,
        zipfile.BadZipFile,
        subprocess.SubprocessError,
        RecursionError,
    ):
        # Fixed failure text: do not print API output, reports, paths or secrets.
        print(
            "error: Release preparation/verification failed; retain owned directory/draft and inspect sanitized evidence",
            file=sys.stderr,
        )
        sys.exit(1)
