#!/usr/bin/env python3
"""REL2 read-only REAL GitHub Release boundary; never dispatch/create/publish/tag."""

import argparse
import importlib.util
import json
from pathlib import Path
import re
import sys
import tempfile
import time

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location(
    "publish_release", Path(__file__).with_name("publish-release.py")
)
p = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = p
spec.loader.exec_module(p)


def draft_record(release, repo, tag):
    p.require(
        isinstance(release, dict) and release.get("draft") is True,
        "actual draft Release required",
    )
    body = release.get("body", "")
    p.require(
        isinstance(body, str) and body.count(p.MARKER) == 1 and body.endswith(" -->\n"),
        "owned Release notes required",
    )
    record = p.decode(body.split(p.MARKER, 1)[1][:-5])
    p.require(
        isinstance(record, dict)
        and set(record)
        == {
            "owner",
            "repo",
            "tag",
            "sourceSHA",
            "buildRunId",
            "mode",
            "executableSha256",
            "assets",
            "evidence",
        },
        "invalid Release ownership schema",
    )
    p.require(
        record["owner"] == "tray-goblin-release-v1"
        and record["repo"] == repo
        and record["tag"] == tag
        and record["mode"] == "draft"
        and record["evidence"] == {},
        "wrong Release ownership/mode",
    )
    p.validate_args(repo, tag, record["buildRunId"], "draft")
    archive = f"tray-goblin-{tag[1:]}-windows-x86_64.zip"
    p.require(
        set(record["assets"]) == {archive, archive + ".sha256"}
        and all(re.fullmatch(r"[0-9a-f]{64}", v) for v in record["assets"].values())
        and re.fullmatch(r"[0-9a-f]{64}", record["executableSha256"])
        and re.fullmatch(r"[0-9a-f]{40}", record["sourceSHA"]),
        "invalid Release source/digests",
    )
    p.require(
        p.check_release(release, record) == set(record["assets"]),
        "both actual Release assets required",
    )
    return record


def trusted_dispatch_source(gh, repo, seconds):
    # Trust local clean canonical main, pinned to the remote ref; never logs.
    p.require(p.git("status", "--porcelain") == "", "dispatch source tree dirty")
    p.require(p.git("symbolic-ref", "--short", "HEAD") == "main", "canonical main required")
    sha = p.git("rev-parse", "--verify", "HEAD")
    p.require(re.fullmatch(r"[0-9a-f]{40}", sha)
              and sha == p.git("rev-parse", "--verify", "refs/heads/main")
              == p.git("rev-parse", "--verify", "refs/remotes/origin/main"), "local main/remote pin mismatch")
    p.require(p.git("remote", "get-url", "origin") in
              (f"https://github.com/{repo}.git", f"https://github.com/{repo}", f"git@github.com:{repo}.git"), "noncanonical origin")
    remote = gh.api(f"repos/{repo}/git/ref/heads/main", seconds)
    p.require(remote.get("object", {}).get("type") == "commit"
              and remote["object"]["sha"] == sha, "remote main moved from approved source")
    workflow = gh.api(f"repos/{repo}/actions/workflows/release.yml", seconds)
    p.require(workflow.get("path") == p.RELEASE_WORKFLOW and workflow.get("state") == "active"
              and type(workflow.get("id")) is int and workflow["id"] > 0, "approved workflow unavailable")
    return sha, workflow["id"]


def discover_release_run(gh, repo, release, record, deadlines):
    approved_sha, workflow_id = trusted_dispatch_source(gh, repo, deadlines.discovery)
    end = time.monotonic() + deadlines.discovery
    while time.monotonic() < end:
        pages = p.decode(gh.command(
            ["api", f"repos/{repo}/actions/workflows/{workflow_id}/runs?event=workflow_dispatch&per_page=100",
             "--paginate", "--slurp"], end - time.monotonic()))
        p.require(isinstance(pages, list) and len(pages) <= 10
                  and all(isinstance(page, dict) and isinstance(page.get("workflow_runs"), list)
                          and len(page["workflow_runs"]) <= 100 for page in pages), "invalid/bounded release run pages")
        rows = [row for page in pages for row in page["workflow_runs"]]
        # Successful completed candidates across ALL pages first. In-flight jobs
        # have no completed relationship proof, so poll, never watch unrelated jobs.
        for row in rows:
            if not isinstance(row, dict) or row.get("event") != "workflow_dispatch" or row.get("status") != "completed" or row.get("conclusion") != "success":
                continue
            run_id = row.get("id")
            p.require(type(run_id) is int and run_id > 0, "invalid release run ID")
            run = gh.api(f"repos/{repo}/actions/runs/{run_id}", deadlines.discovery)
            trusted = (
                run.get("id") == run_id
                and run.get("head_branch") == "main"
                and run.get("head_sha") == approved_sha
                and run.get("workflow_id") == workflow_id
                and run.get("path") == p.RELEASE_WORKFLOW
                and run.get("event") == "workflow_dispatch"
                and run.get("status") == "completed"
                and run.get("conclusion") == "success"
                and run.get("repository", {}).get("full_name", "").lower()
                == repo.lower()
                and run.get("html_url")
                == f"https://github.com/{repo}/actions/runs/{run_id}"
            )
            if not trusted:
                continue
            jobs = gh.api(
                f"repos/{repo}/actions/runs/{run_id}/jobs?per_page=100",
                deadlines.discovery,
            )
            p.require(
                jobs.get("total_count") == 1
                and len(jobs.get("jobs", [])) == 1
                and jobs["jobs"][0].get("name") == "release"
                and jobs["jobs"][0].get("conclusion") == "success",
                "actual release job not successful",
            )
            # Dispatch source can be newer than immutable tag. Match actual job
            # completion to the independently verified Release/API/asset IDs.
            log = gh.command(
                ["run", "view", str(run_id), "--repo", repo, "--log"],
                deadlines.discovery,
            )
            expected = f"Verified owner-visible draft Release: {release['html_url']}"
            identity = f"Release ID: {release['id']}; asset IDs: {[a['id'] for a in release['assets']]}; source SHA: {record['sourceSHA']}"
            if (
                expected in log
                and identity in log
                and record["executableSha256"] in log
                and record["assets"][
                    f"tray-goblin-{record['tag'][1:]}-windows-x86_64.zip"
                ]
                in log
            ):
                p.require(trusted_dispatch_source(gh, repo, deadlines.discovery) == (approved_sha, workflow_id), "approved source changed during discovery")
                return run
        time.sleep(min(0.25, max(0, end - time.monotonic())))
    raise p.Error(
        "no successful actual release workflow run matching draft/source/digests"
    )


def verify(repo, tag, out_root, deadlines=p.Deadlines()):
    p.validate_args(repo, tag, 1, "draft")
    root = Path(out_root).resolve()
    root.mkdir(parents=True, exist_ok=True)
    owned = Path(tempfile.mkdtemp(prefix="tray-goblin-release-verification-", dir=root))
    evidence = owned / "evidence"
    evidence.mkdir()
    print(
        f"Owned Release download/evidence retained: {owned}",
        file=sys.stderr,
        flush=True,
    )
    gh = p.GitHub(evidence)
    release = p.find_release(gh, repo, tag, deadlines.discovery)
    record = draft_record(release, repo, tag)
    sha = p.tag_identity(gh, repo, tag, deadlines.discovery)
    p.require(sha == record["sourceSHA"], "Release/tag source mismatch")
    source = p.source_run(gh, repo, sha, record["buildRunId"], deadlines.discovery)
    run = discover_release_run(gh, repo, release, record, deadlines)
    download = owned / "release-assets"
    p.download_assets(gh, record, download, record["assets"], deadlines.transfer)
    archive = f"tray-goblin-{tag[1:]}-windows-x86_64.zip"
    p.require(
        p.ci.verify_payload(download, sha)
        == (record["assets"][archive], record["executableSha256"]),
        "permanent Release payload mismatch",
    )
    current = p.inspected_release(gh, repo, release["id"], deadlines.discovery)
    p.require(
        p.check_release(current, record) == set(record["assets"])
        and current["assets"] == release["assets"],
        "draft changed during verification",
    )
    result = {
        "releaseUrl": current["html_url"],
        "releaseId": current["id"],
        "assetIds": [a["id"] for a in current["assets"]],
        "releaseRunId": run["id"],
        "releaseRunUrl": run["html_url"],
        "dispatchSourceSHA": run["head_sha"],
        "sourceBuildRunId": source["id"],
        "sourceSHA": sha,
        "zipSha256": record["assets"][archive],
        "executableSha256": record["executableSha256"],
    }
    (evidence / "verified.json").write_text(json.dumps(result, indent=2) + "\n")
    print(
        f"Verified actual owner-visible DRAFT Release (sign-in required): {current['html_url']}"
    )
    print(
        f"Release ID: {current['id']}; asset IDs: {result['assetIds']}; release workflow run: {run['id']}"
    )
    print(
        f"Source SHA: {sha}; ZIP SHA-256: {result['zipSha256']}; EXE SHA-256: {result['executableSha256']}"
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--draft", action="store_true", required=True)
    parser.add_argument("--out-root", default=".wgm/release-downloads")
    args = parser.parse_args()
    verify(args.repo, args.tag, args.out_root)


if __name__ == "__main__":
    try:
        main()
    except (
        p.Error,
        OSError,
        ValueError,
        KeyError,
        TypeError,
        p.struct.error,
        p.zipfile.BadZipFile,
        p.subprocess.SubprocessError,
        RecursionError,
    ):
        print(
            "error: actual draft Release verification failed; inspect retained sanitized evidence",
            file=sys.stderr,
        )
        sys.exit(1)
