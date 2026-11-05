#!/usr/bin/env python3
"""Stdlib-only CI2 verifier. Never executes/extracts downloaded Windows code.

The shell entrypoint has only --repo/--commit/--out-root and --self-test. Short
clock budgets and executable mutants exist only inside the offline test harness.
"""

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import struct
import subprocess
import sys
import tempfile
import time
import zipfile

sys.dont_write_bytecode = True
WORKFLOW = ".github/workflows/windows-test-build.yml"
ARTIFACT = "tray-goblin-windows-x86_64"


class VerificationError(ValueError):
    pass


def require(ok, message):
    if not ok:
        raise VerificationError(message)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key")
        result[key] = value
    return result


def decode(text):
    return json.loads(text, object_pairs_hook=unique_object)


def timestamp(value):
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    require(parsed.tzinfo is not None, "timestamp lacks timezone")
    return parsed


@dataclass(frozen=True)
class Deadlines:
    discovery: float = 120
    watch: float = 1500
    download: float = 120


class GitHub:
    def __init__(self, evidence):
        self.evidence = evidence
        self.serial = 0

    def command(self, args, seconds):
        require(seconds > 0, "discovery deadline exceeded")
        self.serial += 1
        # GNU timeout signals the command's process group; KILL follows TERM
        # after five seconds. No ordinary CLI/environment short-deadline switch.
        result = subprocess.run(
            ["timeout", "--kill-after=5s", f"{max(0.001, seconds):.3f}s", "gh", *args],
            capture_output=True, text=True, check=False,
        )
        prefix = self.evidence / f"{self.serial:02d}"
        prefix.with_suffix(".stdout").write_text(result.stdout)
        prefix.with_suffix(".stderr").write_text(result.stderr)
        prefix.with_suffix(".json").write_text(json.dumps({"command": ["gh", *args], "exit": result.returncode}))
        if result.returncode != 0:
            raise VerificationError(f"GitHub {args[0]} command failed (exit {result.returncode}); inspect retained evidence")
        return result.stdout

    def api(self, endpoint, seconds):
        return decode(self.command(["api", endpoint], seconds))


def check_run(run, repo, sha, run_id):
    require(isinstance(run, dict), "run inspection must be an object")
    require(type(run.get("id")) is int and run["id"] == run_id, "run ID mismatch")
    require(run.get("head_sha") == sha, "run SHA mismatch/missing")
    require(run.get("path") == WORKFLOW, "wrong workflow path")
    require(run.get("event") in ("push", "workflow_dispatch"), "wrong workflow event")
    require(run.get("head_branch") == "main", "wrong source branch")
    require(run.get("repository", {}).get("full_name", "").lower() == repo.lower(), "wrong repository")
    require(run.get("html_url") == f"https://github.com/{repo}/actions/runs/{run_id}", "unexpected run browser URL")
    timestamp(run["created_at"])


def discover(gh, repo, sha, seconds):
    end = time.monotonic() + seconds
    while True:
        remaining = end - time.monotonic()
        runs = decode(gh.command([
            "run", "list", "--repo", repo, "--workflow", Path(WORKFLOW).name,
            "--branch", "main", "--commit", sha, "--limit", "100",
            "--json", "databaseId,headSha,event,status,conclusion,url",
        ], remaining))
        require(isinstance(runs, list), "run listing must be an array")
        candidates = [r for r in runs if isinstance(r, dict) and r.get("headSha") == sha
                      and r.get("event") in ("push", "workflow_dispatch")]
        require(len(candidates) <= 1, "ambiguous exact-SHA runs")
        if candidates:
            run_id = candidates[0].get("databaseId")
            require(type(run_id) is int and run_id > 0, "invalid run ID")
            run = gh.api(f"repos/{repo}/actions/runs/{run_id}", end - time.monotonic())
            check_run(run, repo, sha, run_id)
            require(run.get("conclusion") in (None, "success"), "run failed/cancelled/skipped")
            return run_id
        require(time.monotonic() < end, "no exact-SHA workflow run within discovery deadline")
        time.sleep(min(0.25, max(0, end - time.monotonic())))


def inspect_artifact(doc, repo, sha, run_id, run):
    require(isinstance(doc, dict) and doc.get("total_count") == 1, "absent/ambiguous artifact")
    assets = doc.get("artifacts")
    require(isinstance(assets, list) and len(assets) == 1, "absent/ambiguous artifact list")
    asset = assets[0]
    require(isinstance(asset, dict), "artifact must be an object")
    require(asset.get("name") == ARTIFACT, "wrong artifact name")
    require(asset.get("expired") is False, "artifact expired or expiration missing")
    require(timestamp(asset["expires_at"]) > datetime.now(timezone.utc), "artifact stale")
    require(timestamp(asset["created_at"]) >= timestamp(run["created_at"]), "artifact predates run")
    require(type(asset.get("id")) is int and asset["id"] > 0, "invalid artifact ID")
    require(type(asset.get("size_in_bytes")) is int and asset["size_in_bytes"] > 0, "empty artifact")
    origin = asset.get("workflow_run", {})
    require(origin.get("id") == run_id and origin.get("head_sha") == sha
            and origin.get("head_branch") == "main", "artifact source identity mismatch")
    require(asset.get("archive_download_url") == f"https://api.github.com/repos/{repo}/actions/artifacts/{asset['id']}/zip", "unexpected artifact API URL")
    return asset


def check_pe(data):
    # Decode actual DOS/COFF/PE32+ headers, not a filename or 'file' string.
    require(len(data) >= 64 and data[:2] == b"MZ", "missing DOS executable header")
    offset = struct.unpack_from("<I", data, 0x3C)[0]
    require(64 <= offset <= len(data) - 94, "PE header out of bounds")
    require(data[offset:offset + 4] == b"PE\0\0", "missing PE signature")
    machine, sections = struct.unpack_from("<HH", data, offset + 4)
    optional_size, flags = struct.unpack_from("<HH", data, offset + 20)
    require(machine == 0x8664 and sections > 0 and flags & 2, "not an x64 executable")
    optional = offset + 24
    require(optional_size >= 70 and optional + optional_size <= len(data), "truncated optional PE header")
    require(struct.unpack_from("<H", data, optional)[0] == 0x20B, "not PE32+")
    require(struct.unpack_from("<H", data, optional + 68)[0] == 2, "not a Windows GUI executable")


def verify_payload(download, sha):
    entries = list(download.iterdir())
    require(len(entries) == 2 and all(p.is_file() and not p.is_symlink() for p in entries), "download must contain exactly two regular sibling files")
    archives = [p for p in entries if re.fullmatch(r"tray-goblin-\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?-windows-x86_64\.zip", p.name)]
    require(len(archives) == 1, "missing/ambiguous expected ZIP basename")
    archive = archives[0]
    checksum = download / (archive.name + ".sha256")
    require(checksum in entries, "missing sibling checksum basename")
    require(0 < archive.stat().st_size < 100 * 1024 * 1024, "empty/oversized ZIP")
    require(checksum.stat().st_size < 1024, "oversized checksum")
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    require(checksum.read_text(encoding="ascii") == f"{digest}  {archive.name}\n", "checksum digest/path mismatch")
    stem = archive.stem
    files = {
        "LICENSE", "README.txt", "install.ps1", "uninstall.ps1", "tray-goblin.exe",
        "integrations/pi/README.md", "integrations/pi/package.json", "integrations/pi/tray-goblin.js",
        "resources/SETUP.md", "resources/config.example.json", "resources/build.json",
        "resources/test-attention-windows.ps1", "resources/measure-performance.ps1",
    }
    dirs = {"", "integrations/", "integrations/pi/", "resources/"}
    expected = {f"{stem}/{entry}" for entry in files | dirs}
    with zipfile.ZipFile(archive) as zipped:
        infos = zipped.infolist()
        require(len(infos) == len(expected) and {i.filename for i in infos} == expected, "ZIP path/persistent-resource allowlist mismatch")
        require(sum(i.file_size for i in infos) < 200 * 1024 * 1024, "oversized ZIP payload")
        require(all(not stat.S_ISLNK(i.external_attr >> 16) and not i.flag_bits & 1 for i in infos), "symlink/encrypted ZIP entry")
        require(zipped.testzip() is None, "ZIP CRC/integrity failure")
        require(all(zipped.getinfo(f"{stem}/{entry}").file_size > 0 for entry in files), "empty persistent payload resource")
        executable = zipped.read(f"{stem}/tray-goblin.exe")
        check_pe(executable)
        require(zipped.getinfo(f"{stem}/resources/build.json").file_size < 65536, "oversized manifest")
        manifest = decode(zipped.read(f"{stem}/resources/build.json"))
        require(isinstance(manifest, dict) and manifest.get("buildCommit") == sha, "manifest source SHA mismatch")
        require(manifest.get("worktreeDirty") is False, "manifest source is not clean")
        require(manifest.get("published") is False, "manifest must be unpublished")
        exe_digest = hashlib.sha256(executable).hexdigest()
        require(manifest.get("executableSha256") == exe_digest, "manifest executable digest mismatch")
    return digest, exe_digest


def verify(repo, sha, out_root, deadlines=Deadlines()):
    require(re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9-]*/[A-Za-z0-9][A-Za-z0-9_.-]*", repo) is not None, "--repo must be owner/repo")
    require(re.fullmatch(r"[0-9a-fA-F]{40}", sha) is not None, "--commit must be 40 hex characters")
    sha = sha.lower()
    root = Path(out_root).expanduser().resolve()
    root.mkdir(parents=True, exist_ok=True)
    # This owned child is NEVER reused or removed, even on errors/interruption.
    owned = Path(tempfile.mkdtemp(prefix="tray-goblin-ci-", dir=root))
    evidence = owned / "evidence"
    download = owned / "download"
    evidence.mkdir()
    download.mkdir()
    print(f"Owned download/evidence directory: {owned}", file=sys.stderr, flush=True)
    gh = GitHub(evidence)
    run_id = discover(gh, repo, sha, deadlines.discovery)
    gh.command(["run", "watch", str(run_id), "--repo", repo, "--exit-status"], deadlines.watch)
    # A successful watch is not sufficient: inspect the exact completed run again.
    run = gh.api(f"repos/{repo}/actions/runs/{run_id}", deadlines.discovery)
    check_run(run, repo, sha, run_id)
    require(run.get("status") == "completed" and run.get("conclusion") == "success", "run not completed successfully")
    asset = inspect_artifact(gh.api(f"repos/{repo}/actions/runs/{run_id}/artifacts?per_page=100", deadlines.discovery), repo, sha, run_id, run)
    gh.command(["run", "download", str(run_id), "--repo", repo, "--name", ARTIFACT, "--dir", str(download)], deadlines.download)
    digest, exe_digest = verify_payload(download, sha)
    # Browser route uses inspected API artifact/run IDs, never an invented run.
    artifact_url = f"{run['html_url']}/artifacts/{asset['id']}"
    record = {"runUrl": run["html_url"], "artifactUrl": artifact_url, "artifactId": asset["id"],
              "sourceSHA": sha, "zipSha256": digest, "executableSha256": exe_digest,
              "downloadDirectory": str(download)}
    (evidence / "verified.json").write_text(json.dumps(record, indent=2) + "\n")
    print("Verified Windows test artifact (not release or Windows runtime evidence)")
    print(f"Run: {record['runUrl']}\nArtifact: {artifact_url}\nSource SHA: {sha}\nZIP SHA-256: {digest}\nPE SHA-256: {exe_digest}")
    return record


# Purpose: controlled gh responses exercise the REAL command/inspection/download
# orchestration offline. These are not GitHub boundary availability evidence.
GH_STUB = r'''#!/usr/bin/env python3
import json, os, pathlib, shutil, sys, time
args = sys.argv[1:]
case = os.environ['CI_STUB_CASE']
sha = 'a' * 40
repo = 'owner/repo'
run = {'id': 17, 'head_sha': sha, 'path': '.github/workflows/windows-test-build.yml',
       'event': 'push', 'head_branch': 'main', 'repository': {'full_name': repo},
       'html_url': 'https://github.com/owner/repo/actions/runs/17',
       'created_at': '2026-01-01T00:00:00Z', 'status': 'completed', 'conclusion': 'success'}
if args[:2] == ['run', 'list']:
    rows = [{'databaseId': 17, 'headSha': sha, 'event': 'push'}]
    if case == 'missing-run': rows = []
    if case == 'wrong-sha': rows[0]['headSha'] = 'b' * 40
    if case == 'missing-sha': rows[0].pop('headSha')
    if case == 'ambiguous-run': rows *= 2
    if case == 'wrong-event': rows[0]['event'] = 'release'
    print(json.dumps(rows))
elif args[:2] == ['run', 'watch']:
    if case in ('failed-watch', 'cancelled-watch'): sys.exit(1)
    if case == 'watch-timeout': time.sleep(60)
    pathlib.Path(os.environ['CI_STUB_STATE']).touch()
elif args[0] == 'api':
    if 'artifacts?' in args[1]:
        asset = {'id': 23, 'name': 'tray-goblin-windows-x86_64', 'expired': False,
                 'size_in_bytes': 1234, 'created_at': '2026-01-01T00:00:01Z',
                 'expires_at': '2099-01-01T00:00:00Z',
                 'workflow_run': {'id': 17, 'head_sha': sha, 'head_branch': 'main'},
                 'archive_download_url': 'https://api.github.com/repos/owner/repo/actions/artifacts/23/zip'}
        if case == 'expired': asset['expired'] = True
        if case == 'stale': asset['expires_at'] = '2000-01-01T00:00:00Z'
        if case == 'wrong-asset-sha': asset['workflow_run']['head_sha'] = 'b' * 40
        assets = [asset]
        if case == 'ambiguous-asset': assets *= 2
        if case == 'missing-asset': assets = []
        print(json.dumps({'total_count': len(assets), 'artifacts': assets}))
    else:
        if case == 'inspect-wrong-sha': run['head_sha'] = 'b' * 40
        if case == 'inspect-missing-sha': run.pop('head_sha')
        if case == 'wrong-workflow': run['path'] = '.github/workflows/release.yml'
        if case == 'failed-run': run['conclusion'] = 'failure'
        if case == 'cancelled-run': run['conclusion'] = 'cancelled'
        if pathlib.Path(os.environ['CI_STUB_STATE']).exists():
            if case == 'failed-after-watch': run['conclusion'] = 'failure'
            if case == 'cancelled-after-watch': run['conclusion'] = 'cancelled'
            if case == 'incomplete-after-watch': run['status'] = 'in_progress'
        print(json.dumps(run))
        if case == 'inspection-error': sys.exit(1)
        if case == 'malformed-inspection': print('not json')
elif args[:2] == ['run', 'download']:
    dest = pathlib.Path(args[args.index('--dir') + 1])
    if case in ('partial-timeout', 'partial-success'):
        (dest / 'interrupted.zip').write_bytes(b'partial retained bytes')
        if case == 'partial-timeout': time.sleep(60)
    elif case == 'download-error': sys.exit(1)
    else:
        for source in pathlib.Path(os.environ['CI_STUB_PAYLOAD']).iterdir():
            shutil.copyfile(source, dest / source.name)
else:
    sys.exit(99)
'''


def fixture(directory, sha):
    # Synthetic non-runnable PE bytes isolate structural header/manifest checks.
    pe = bytearray(512)
    pe[:2] = b"MZ"
    struct.pack_into("<I", pe, 0x3C, 64)
    pe[64:68] = b"PE\0\0"
    struct.pack_into("<HH", pe, 68, 0x8664, 1)
    struct.pack_into("<HH", pe, 84, 240, 2)
    struct.pack_into("<H", pe, 88, 0x20B)
    struct.pack_into("<H", pe, 156, 2)
    manifest = {"buildCommit": sha, "worktreeDirty": False, "published": False,
                "executableSha256": hashlib.sha256(pe).hexdigest()}
    stem = "tray-goblin-0.1.0-windows-x86_64"
    payload = {"LICENSE": b"fixture", "README.txt": b"fixture", "install.ps1": b"fixture",
               "uninstall.ps1": b"fixture", "tray-goblin.exe": bytes(pe),
               "integrations/pi/README.md": b"fixture", "integrations/pi/package.json": b"{}",
               "integrations/pi/tray-goblin.js": b"fixture", "resources/SETUP.md": b"fixture",
               "resources/config.example.json": b"{}", "resources/build.json": json.dumps(manifest).encode(),
               "resources/test-attention-windows.ps1": b"fixture", "resources/measure-performance.ps1": b"fixture"}
    archive = directory / f"{stem}.zip"
    with zipfile.ZipFile(archive, "w") as zipped:
        for name in ("", "integrations/", "integrations/pi/", "resources/"):
            zipped.writestr(f"{stem}/{name}", b"")
        for name, data in payload.items():
            zipped.writestr(f"{stem}/{name}", data)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    (directory / f"{stem}.zip.sha256").write_text(f"{digest}  {stem}.zip\n")
    return payload, digest


def self_test():
    external = os.environ.get("TRAY_GOBLIN_EVIDENCE_DIR")
    if external:
        Path(external).mkdir(parents=True, exist_ok=True)
    test_root = Path(tempfile.mkdtemp(prefix="ci-verifier-self-test-", dir=external))
    stub_bin = test_root / "bin"
    stub_bin.mkdir()
    stub = stub_bin / "gh"
    stub.write_text(GH_STUB)
    stub.chmod(0o755)
    payload_dir = test_root / "payload"
    payload_dir.mkdir()
    sha = "a" * 40
    payload, expected_digest = fixture(payload_dir, sha)
    caller = test_root / "caller"
    caller.mkdir()
    sentinel = caller / "sentinel"
    sentinel.write_bytes(b"caller-owned sentinel must survive\n")
    original = sentinel.read_bytes()
    env = {**os.environ, "PATH": f"{stub_bin}:{os.environ['PATH']}", "CI_STUB_PAYLOAD": str(payload_dir),
           "PYTHONDONTWRITEBYTECODE": "1"}
    helper = Path(__file__).resolve()
    wrapper = helper.with_suffix(".sh")
    controls = 0

    def exercise(case, module=helper, expect_success=False, live_entry=False):
        nonlocal controls
        before = set(caller.glob("tray-goblin-ci-*"))
        # Private in-memory test call; the public shell CLI retains fixed budgets.
        code = ("import importlib.util,sys; "
                "s=importlib.util.spec_from_file_location('ci_verify',sys.argv[1]); "
                "m=importlib.util.module_from_spec(s); sys.modules[s.name]=m; s.loader.exec_module(m); "
                "m.verify('owner/repo','a'*40,sys.argv[2],m.Deadlines(2,2,2))")
        command = (["bash", str(wrapper), "--repo", "owner/repo", "--commit", sha, "--out-root", str(caller)]
                   if live_entry else [sys.executable, "-B", "-c", code, str(module), str(caller)])
        start = time.monotonic()
        state = test_root / f"stub-state-{module.stem}-{case}-{controls}"
        result = subprocess.run(command, env={**env, "CI_STUB_CASE": case, "CI_STUB_STATE": str(state)}, capture_output=True, text=True, timeout=7)
        elapsed = time.monotonic() - start
        label = f"{module.stem}-{case}"
        (test_root / f"{label}.stdout").write_text(result.stdout)
        (test_root / f"{label}.stderr").write_text(result.stderr)
        (test_root / f"{label}.result.json").write_text(json.dumps({
            "case": case, "exit": result.returncode, "elapsedSeconds": elapsed,
            "expectedSuccess": expect_success, "command": command,
        }, indent=2) + "\n")
        require(sentinel.exists() and sentinel.read_bytes() == original, "self-test: caller sentinel deleted/changed")
        after = set(caller.glob("tray-goblin-ci-*"))
        new = after - before
        require(len(new) == 1, "self-test: new owned child not created")
        owned = new.pop()
        require((result.returncode == 0) == expect_success, f"self-test: {case} wrong exit status")
        if expect_success:
            record = decode((owned / "evidence/verified.json").read_text())
            require(record["zipSha256"] == expected_digest and record["sourceSHA"] == sha, "self-test: successful bytes/source not verified")
            require("https://github.com/owner/repo/actions/runs/17/artifacts/23" in result.stdout, "self-test: missing inspected artifact URL")
        else:
            require("https://github.com/" not in result.stdout + result.stderr and "Verified Windows test artifact" not in result.stdout,
                    "self-test: premature success URL/claim")
            require(not (owned / "evidence/verified.json").exists(), "self-test: false verification record")
        if case == "partial-timeout":
            require(2 <= elapsed < 7, "self-test: interrupted download was not bounded")
            require((owned / "download/interrupted.zip").read_bytes() == b"partial retained bytes", "self-test: partial evidence not retained")
        controls += 1
        return owned

    # Selection, terminal-status, inspection and asset failures must all fail hard.
    for case in ("missing-run", "wrong-sha", "missing-sha", "ambiguous-run", "wrong-event",
                 "inspect-wrong-sha", "inspect-missing-sha", "wrong-workflow", "failed-run", "cancelled-run",
                 "failed-watch", "cancelled-watch", "watch-timeout", "failed-after-watch", "cancelled-after-watch",
                 "incomplete-after-watch", "inspection-error", "malformed-inspection",
                 "expired", "stale", "ambiguous-asset", "missing-asset", "wrong-asset-sha", "download-error", "partial-success"):
        exercise(case)
    partial = exercise("partial-timeout")
    retry = exercise("success", expect_success=True, live_entry=True)
    require(partial != retry and (partial / "download/interrupted.zip").exists(), "self-test: retry reused/deleted partial child")
    require((retry / "download" / next(payload_dir.glob("*.zip")).name).read_bytes() == next(payload_dir.glob("*.zip")).read_bytes(), "self-test: retry bytes differ")

    # Executable counterfactuals prove the harness detects destructive cleanup,
    # swallowed gh inspection exit status and success-shaped partial downloads.
    source = helper.read_text()
    mutants = [
        ("deleting-caller", "root.mkdir(parents=True, exist_ok=True)", "shutil.rmtree(root)\n    root.mkdir(parents=True, exist_ok=True)", "success", True),
        ("swallowing-inspection", "if result.returncode != 0:", "if False:", "inspection-error", False),
        ("accepting-partial", "digest, exe_digest = verify_payload(download, sha)", "digest, exe_digest = 'unsafe', 'unsafe'", "partial-success", False),
    ]
    killed = 0
    for name, old, new, case, success in mutants:
        require(source.split("\n# Purpose: controlled gh responses", 1)[0].count(old) == 1, "self-test: mutant seam drift")
        mutated = test_root / f"{name}.py"
        mutated.write_text(source.replace(old, new, 1))
        # Separate caller protects original interrupted-download/retry evidence.
        saved_caller, saved_sentinel = caller, sentinel
        caller = test_root / name
        caller.mkdir()
        sentinel = caller / "sentinel"
        sentinel.write_bytes(original)
        try:
            exercise(case, module=mutated, expect_success=success)
        except VerificationError as exc:
            expected_failure = ("caller sentinel deleted/changed" if name == "deleting-caller" else f"{case} wrong exit status")
            require(expected_failure in str(exc), f"self-test: {name} rejected for unrelated failure")
            killed += 1
        else:
            raise VerificationError(f"self-test: unsafe mutant survived: {name}")
        finally:
            caller, sentinel = saved_caller, saved_sentinel

    # Byte/path/manifest/PE counterfactuals protect downloaded content independently
    # of gh stubs. Rebuild sibling checksum so malformed payload is not rejected
    # merely due to an obsolete hash; ZIP structure/PE/JSON must reject it.
    byte_controls = 0
    edits = {
        "wrong-build-sha": ("resources/build.json", json.dumps({"buildCommit": "b" * 40, "worktreeDirty": False, "published": False, "executableSha256": hashlib.sha256(payload['tray-goblin.exe']).hexdigest()}).encode()),
        "dirty-build": ("resources/build.json", json.dumps({"buildCommit": sha, "worktreeDirty": True, "published": False, "executableSha256": hashlib.sha256(payload['tray-goblin.exe']).hexdigest()}).encode()),
        "published-build": ("resources/build.json", json.dumps({"buildCommit": sha, "worktreeDirty": False, "published": True, "executableSha256": hashlib.sha256(payload['tray-goblin.exe']).hexdigest()}).encode()),
        "wrong-exe-hash": ("resources/build.json", json.dumps({"buildCommit": sha, "worktreeDirty": False, "published": False, "executableSha256": '0' * 64}).encode()),
        "not-pe": ("tray-goblin.exe", b"not an executable"),
        "wrong-machine": ("tray-goblin.exe", payload["tray-goblin.exe"][:68] + b"\x4c\x01" + payload["tray-goblin.exe"][70:]),
        "console-pe": ("tray-goblin.exe", payload["tray-goblin.exe"][:156] + b"\x03\x00" + payload["tray-goblin.exe"][158:]),
        "missing-pi": ("integrations/pi/tray-goblin.js", None),
        "missing-setup": ("resources/SETUP.md", None),
        "traversal-entry": ("../escape", b"unsafe"),
    }
    for name, (entry, data) in edits.items():
        directory = test_root / name
        directory.mkdir()
        archive = directory / "tray-goblin-0.1.0-windows-x86_64.zip"
        with zipfile.ZipFile(next(payload_dir.glob("*.zip"))) as original_zip, zipfile.ZipFile(archive, "w") as zipped:
            for info in original_zip.infolist():
                if info.filename.endswith("/" + entry):
                    if data is not None:
                        zipped.writestr(info, data)
                else:
                    zipped.writestr(info, original_zip.read(info))
            if entry == "../escape":
                zipped.writestr(entry, data)
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        (directory / (archive.name + ".sha256")).write_text(f"{digest}  {archive.name}\n")
        try:
            verify_payload(directory, sha)
        except VerificationError:
            byte_controls += 1
        else:
            raise VerificationError(f"unsafe payload counterfactual accepted: {name}")
    # Download layout/checksum/CRC controls cover success-shaped incomplete or
    # misdirected bytes, independently of the interrupted gh download control.
    for name in ("wrong-checksum", "checksum-path", "missing-checksum", "nested-download", "truncated-zip", "corrupt-crc"):
        directory = test_root / name
        directory.mkdir()
        for source_file in payload_dir.iterdir():
            shutil.copyfile(source_file, directory / source_file.name)
        archive = next(directory.glob("*.zip"))
        checksum = directory / (archive.name + ".sha256")
        if name == "wrong-checksum":
            checksum.write_text(f"{'0' * 64}  {archive.name}\n")
        elif name == "checksum-path":
            checksum.write_text(f"{expected_digest}  ../{archive.name}\n")
        elif name == "missing-checksum":
            checksum.unlink()
        elif name == "nested-download":
            nested = directory / "unexpected"
            nested.mkdir()
            archive.rename(nested / archive.name)
        else:
            data = bytearray(archive.read_bytes())
            if name == "truncated-zip":
                data = data[:100]
            else:
                with zipfile.ZipFile(archive) as zipped:
                    info = zipped.getinfo(archive.stem + "/LICENSE")
                    offset = info.header_offset
                    name_size, extra_size = struct.unpack_from("<HH", data, offset + 26)
                    data[offset + 30 + name_size + extra_size] ^= 1
            archive.write_bytes(data)
            checksum.write_text(f"{hashlib.sha256(data).hexdigest()}  {archive.name}\n")
        try:
            verify_payload(directory, sha)
        except (VerificationError, zipfile.BadZipFile):
            byte_controls += 1
        else:
            raise VerificationError(f"unsafe download counterfactual accepted: {name}")
    (test_root / "summary.json").write_text(json.dumps({
        "evidenceTier": "offline controlled stubs, not GitHub boundary evidence",
        "commandControls": controls, "executableMutantsRejected": killed,
        "payloadCounterfactualsRejected": byte_controls,
        "interruptedOwnedChild": str(partial), "differentVerifiedRetryChild": str(retry),
        "syntheticZipSha256": expected_digest, "callerSentinelUnchanged": sentinel.read_bytes() == original,
    }, indent=2) + "\n")
    print(f"CI verifier OFFLINE self-test passed: {controls} command controls, {killed} executable mutants rejected, {byte_controls} payload counterfactuals rejected.")
    print(f"Evidence: {test_root}\nInterrupted owned child retained: {partial}\nDifferent verified retry child: {retry}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo")
    parser.add_argument("--commit")
    parser.add_argument("--out-root")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        require(not any((args.repo, args.commit, args.out_root)), "--self-test cannot be combined with live flags")
        self_test()
    else:
        require(all((args.repo, args.commit, args.out_root)), "--repo, --commit and --out-root are required")
        verify(args.repo, args.commit, args.out_root)


if __name__ == "__main__":
    try:
        main()
    except (VerificationError, OSError, ValueError, KeyError, TypeError, struct.error, zipfile.BadZipFile, subprocess.SubprocessError) as exc:
        # Never echo raw gh output (which may contain success-shaped URLs) on error.
        print(f"error: CI verification failed: {exc}", file=sys.stderr)
        sys.exit(1)
