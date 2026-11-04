// Audit A11/A12 and final A02/A05 regression controls. Execute EXACT published
// and actual packaged fences. Offline seams prove control flow, never a build,
// Windows observation, network provisioning or publication.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import * as fs from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve, basename } from "node:path";
import { createHash } from "node:crypto";

const args = process.argv.slice(2);
const option = (name, fallback) => args.includes(name) ? args[args.indexOf(name) + 1] : fallback;
const sourceRoot = resolve(option("--source-root", resolve(import.meta.dirname, "..")));
const documentPath = resolve(args[0] && !args[0].startsWith("--") ? args[0] : join(sourceRoot, "docs/manual-release.md"));
const onlyFences = args.includes("--failfast-only");
const auditOnly = args.includes("--audit-fences-only");
const auditTarget = option("--audit-target", "all");
const archivePath = resolve(option("--archive", join(sourceRoot, "dist/tray-goblin-0.1.0-windows-x86_64.zip")));
const document = fs.readFileSync(documentPath, "utf8");
const evidenceBase = resolve(process.env.TRAY_GOBLIN_EVIDENCE_DIR ?? tmpdir());
const repositoryRoot = resolve(import.meta.dirname, "..");
assert.ok(evidenceBase !== repositoryRoot && !evidenceBase.startsWith(repositoryRoot + "/"), "evidence must be external");
fs.mkdirSync(evidenceBase, { recursive: true });
const root = fs.mkdtempSync(join(evidenceBase, "release-doc-controls-"));
const home = join(root, "home");
fs.mkdirSync(home);
const realGit = execFileSync("which", ["git"], { encoding: "utf8" }).trim();
const realBash = execFileSync("which", ["bash"], { encoding: "utf8" }).trim();
const realPwsh = execFileSync("which", ["pwsh"], { encoding: "utf8" }).trim();
const env = { PATH: process.env.PATH, HOME: home, USERPROFILE: home,
  XDG_CONFIG_HOME: join(home, "config"), GIT_CONFIG_NOSYSTEM: "1", GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_AUTHOR_NAME: "Synthetic Control", GIT_AUTHOR_EMAIL: "synthetic@example.invalid",
  GIT_COMMITTER_NAME: "Synthetic Control", GIT_COMMITTER_EMAIL: "synthetic@example.invalid" };
const report = { documentPath, documentSha256: createHash("sha256").update(document).digest("hex"),
  tier: "offline-release-doc-control-flow-not-publication", annotatedTag: null, fences: [], canaryKilled: false };
try {
  // Plumbing creates an isolated commit without staging/committing this checkout.
  const synthetic = join(root, "git");
  fs.mkdirSync(synthetic);
  const git = (...a) => execFileSync(realGit, a, { cwd: synthetic, env, encoding: "utf8", stdio: ["pipe", "pipe", "pipe"] }).trim();
  git("init", "--quiet");
  const tree = git("mktree");
  const commit = git("commit-tree", tree, "-m", "synthetic offline release-doc control");
  git("tag", "-a", "v0.1.0", "-m", "synthetic annotated tag", commit);
  const tagObject = git("rev-parse", "refs/tags/v0.1.0");
  assert.notEqual(tagObject, commit);
  const peeled = git("rev-parse", "--verify", "refs/tags/v0.1.0^{commit}");
  assert.equal(peeled, commit);
  if (!onlyFences && !auditOnly) {
    const line = document.match(/^.*git rev-parse[^\n]*v0\.1\.0[^\n]*$/m)?.[0].trim();
    assert.ok(line, "published tag identity command exists");
    const actual = execFileSync(realBash, ["-c", line], { cwd: synthetic, env, encoding: "utf8" }).trim();
    assert.equal(actual, commit, "A11: published annotated-tag identity must be the COMMIT, not tag object");
    assert.ok(document.split("git rev-parse --verify 'refs/tags/v0.1.0^{commit}'").length >= 3, "both tag and notes identities must peel");
  }
  report.annotatedTag = { commit, tagObject, peeled };
  const seams = join(root, "bin");
  fs.mkdirSync(seams);
  const shim = `#!${process.execPath}
const fs = require('node:fs');
const path = require('node:path');
const name = path.basename(process.argv[1]);
const args = process.argv.slice(2);
const log = process.env.CONTROL_LOG;
const prior = fs.existsSync(log) ? fs.readFileSync(log, 'utf8').trim().split('\\n').filter(Boolean) : [];
const step = prior.length + 1;
const failed = step === Number(process.env.FAIL_STEP);
fs.appendFileSync(log, JSON.stringify({name, args, step, failed}) + '\\n');
if (failed) { console.error('INJECTED_NATIVE_EXIT73:' + name + ':' + step); process.exit(73); }
if (name === 'git' && args[0] === 'rev-parse') console.log(process.env.SYNTHETIC_COMMIT);
if (name === 'git' && args[0] === 'status' && process.env.DIRTY_CONTROL) console.log(' M synthetic');
if (name === 'sha256sum') console.log('a'.repeat(64) + '  synthetic.zip');
if (name === 'mkdir') for (const dir of args.filter(v => !v.startsWith('-'))) fs.mkdirSync(dir, {recursive:true});
if (name === 'cat') process.stdout.write(fs.readFileSync(0));
if (name === 'grep') process.exit(process.env.PLACEHOLDER_CONTROL ? 0 : 1);
`;
  for (const name of ["git", "cargo", "bash", "pwsh", "node", "gh", "sha256sum", "mkdir", "rm", "cat", "grep"]) {
    fs.writeFileSync(join(seams, name), shim, { mode: 0o755 });
  }
  const fences = [...document.matchAll(/```bash\n([\s\S]*?)\n```/g)].map((m) => m[1]);
  assert.ok(fences.length >= 12, "all dependent published release fences are exercised");
  function run(code, failStep = 0, extra = {}, shell = realBash) {
    const cwd = fs.mkdtempSync(join(root, "fence-"));
    fs.mkdirSync(join(cwd, "dist"));
    const log = join(cwd, "calls.jsonl");
    // pwsh prepends PSHOME to PATH at startup. Forward the published command to
    // our native PATH seam explicitly, preserving its actual LASTEXITCODE; never
    // accidentally launch a real collector or rewrite the published fence.
    const pwshPrefix = `function pwsh { & '${join(seams, "pwsh").replaceAll("'", "''")}' @args; $global:LASTEXITCODE = $LASTEXITCODE }\n`;
    const shellArgs = shell === realBash ? ["--noprofile", "--norc", "-c", code] : ["-NoProfile", "-NonInteractive", "-Command", pwshPrefix + code];
    const result = spawnSync(shell, shellArgs, { cwd,
      env: { ...env, LOCALAPPDATA: cwd, PATH: `${seams}:${dirname(process.execPath)}:/usr/bin:/bin`, CONTROL_LOG: log,
        FAIL_STEP: String(failStep), SYNTHETIC_COMMIT: commit,
        WINDOWS_RESULT: "synthetic-control-only", PERFORMANCE_RESULT: "synthetic-control-only", ...extra }, encoding: "utf8" });
    assert.equal(result.error, undefined);
    const calls = fs.existsSync(log) ? fs.readFileSync(log, "utf8").trim().split("\n").filter(Boolean).map(JSON.parse) : [];
    return { exit: result.status, calls, stdout: result.stdout, stderr: result.stderr };
  }
  function assertFailure(result, step) {
    assert.ok(result.stderr.includes(`INJECTED_NATIVE_EXIT73:${result.calls.at(-1)?.name}:${step}`),
      "A12: executed failure marker, not a syntax/missing-command failure");
    assert.notEqual(result.exit, 0, "A12: rejected prerequisite must fail the exact fence");
    assert.equal(result.calls.length, step, "A12: NO downstream launch after failed prerequisite");
    assert.equal(result.calls.at(-1).failed, true, "failure marker proves the seam was reached");
  }
  function canonicalControls() {
    for (const [index, code] of fences.entries()) {
      const success = run(code);
      assert.equal(success.exit, 0, `fence ${index} success control: ${success.stderr}`);
      assert.ok(success.calls.length > 0);
      const failures = [];
      // Every native prerequisite, including tag/identity, notes, push/create and
      // view/mkdir/download/checksum, is faulted independently in its own sandbox.
      for (let step = 1; step <= success.calls.length; step++) {
        const failed = run(code, step);
        assertFailure(failed, step);
        failures.push({ step, exit: failed.exit, calls: failed.calls });
      }
      const swallowed = code.replace('|| exit $?', '|| true');
      assert.notEqual(swallowed, code);
      assert.throws(() => assertFailure(run(swallowed, 1), 1), /A12/);
      report.fences.push({ index, code, success, failures, canaryKilled: true });
    }
    // This canary deliberately swallows the FIRST exact fence's native error.
    // The verifier itself must reject it; exit-zero alone is NOT evidence.
    const canary = fences[0].replace('|| exit $?', '|| true');
    assert.notEqual(canary, fences[0]);
    assert.throws(() => assertFailure(run(canary, 1), 1), /A12/);
    report.canaryKilled = true;
    // The actual published Windows commands use native exit checks too. Drive
    // the EXACT fence through real pwsh with fake collector executables; those
    // seams never run collectors or manufacture Windows field measurements.
    const powershell = [...document.matchAll(/```powershell\n([\s\S]*?)\n```/g)].map((m) => m[1]);
    assert.equal(powershell.length, 2, "paired collectors and nondefault-config example both execute");
    report.powershell = [];
    for (const code of powershell) {
      const success = run(code, 0, {}, realPwsh);
      assert.equal(success.exit, 0, success.stderr);
      assert.equal(success.calls.length, code.includes("measure-performance.ps1") ? 2 : 1);
      const failures = [];
      for (let step = 1; step <= success.calls.length; step++) {
        const failed = run(code, step, {}, realPwsh);
        assertFailure(failed, step);
        failures.push(failed);
      }
      const swallowed = code.replace('if ($LASTEXITCODE -ne 0)', 'if ($false)');
      assert.notEqual(swallowed, code);
      assert.throws(() => assertFailure(run(swallowed, 1, {}, realPwsh), 1), /A12/);
      report.powershell.push({ code, success, failures, canaryKilled: true });
    }
    assert.notEqual(run(fences[0], 0, { DIRTY_CONTROL: "1" }).exit, 0, "dirty tree blocks release");
    const notesFence = fences.find((code) => code.includes("cat > release-notes.md"));
    assert.ok(notesFence);
    assert.notEqual(run(notesFence, 0, { PLACEHOLDER_CONTROL: "1" }).exit, 0, "unresolved notes block publish preparation");
    // Execute the PUBLIC explicit-config aggregate recipe as written, against its
    // named content-free fixture. The second exact fence must produce BOTH typed
    // errors and no extra JSON, not silently use GUI/default user configuration.
    const repo = resolve(import.meta.dirname, "..");
    const configuration = fs.readFileSync(join(repo, "docs/operator/configuration.md"), "utf8");
    const examples = [...configuration.matchAll(/```bash\n([\s\S]*?)\n```/g)].map((m) => m[1]);
    assert.equal(examples.length, 2);
    assert.ok(examples[0].includes('--config "$example_dir/config.json" --all-sessions'));
    const actual = spawnSync(realBash, ["--noprofile", "--norc", "-c",
      `trap 'if [[ -n ${'${example_dir:-}'} ]]; then rm -rf "$example_dir"; fi' EXIT\n${examples.join("\n")}`], {
      cwd: repo, env: { ...env, CARGO_HOME: process.env.CARGO_HOME ?? join(process.env.HOME, ".cargo"),
        RUSTUP_HOME: process.env.RUSTUP_HOME ?? join(process.env.HOME, ".rustup"), CARGO_NET_OFFLINE: "true" }, encoding: "utf8" });
    assert.equal(actual.error, undefined);
    assert.equal(actual.status, 1, actual.stderr);
    const aggregate = JSON.parse(actual.stdout);
    assert.equal(aggregate.sources.length, 1);
    assert.equal(aggregate.sources[0].source.id, "fixture-copilot");
    assert.deepEqual(aggregate.sources[0].capabilities, { input_detection: false, session_errors: true });
    assert.equal(aggregate.sources[0].sessions.length, 1);
    assert.equal(aggregate.sources[0].sessions[0].session_id, "session-new");
    assert.equal(aggregate.sources[0].sessions[0].state, "working");
    const errors = actual.stderr.split("\n").filter((line) => line.startsWith("error:"));
    assert.deepEqual(errors, ["error: poll interval 499ms is outside the accepted 500-10000ms range",
      "error: --session-root cannot override explicit sources with --all-sessions"]);
    report.aggregateExample = { exit: actual.status, aggregate, errors };
  }
  if (!auditOnly) canonicalControls();
  // Final-audit duplicates must not silently collect performance after a failed
  // field gate. Extract the actual source/ZIP text, never reconstruct commands.
  const digest = (data) => createHash("sha256").update(data).digest("hex");
  const bashFences = (text) => [...text.matchAll(/```bash\n([\s\S]*?)\n```/g)].map((m) => m[1]);
  const psFences = (text) => [...text.matchAll(/```powershell\n([\s\S]*?)\n```/g)].map((m) => m[1]);
  const selected = (name) => auditTarget === "all" || auditTarget === name;
  assert.ok(["all", "troubleshooting", "setup", "packaged-setup", "configuration"].includes(auditTarget));
  report.auditFences = [];
  function save(name, value) {
    fs.writeFileSync(join(root, name + ".json"), JSON.stringify(value, null, 2));
    fs.writeFileSync(join(root, name + ".stdout.log"), value.stdout);
    fs.writeFileSync(join(root, name + ".stderr.log"), value.stderr);
    fs.writeFileSync(join(root, name + ".exit"), String(value.exit) + "\n");
    return value;
  }
  for (const [name, path] of [["troubleshooting", "docs/operator/troubleshooting.md"],
    ["setup", "assets/attention/SETUP.md"], ["packaged-setup", null]]) {
    if (!selected(name)) continue;
    const text = path ? fs.readFileSync(join(sourceRoot, path), "utf8") :
      execFileSync("unzip", ["-p", archivePath, "tray-goblin-0.1.0-windows-x86_64/resources/SETUP.md"], { encoding: "utf8" });
    const paired = psFences(text).filter((code) => code.includes("test-attention-windows.ps1") && code.includes("measure-performance.ps1"));
    assert.equal(paired.length, 1, `${name}: exactly one actual paired collector fence`);
    const code = paired[0];
    fs.writeFileSync(join(root, name + ".fence"), code);
    const entry = { name, documentSha256: digest(text), code, success: null, failures: [], canaryKilled: false };
    if (!path) {
      entry.archiveSha256 = digest(fs.readFileSync(archivePath));
      assert.equal(text, fs.readFileSync(join(sourceRoot, "assets/attention/SETUP.md"), "utf8"), "actual packaged setup matches selected source");
    }
    report.auditFences.push(entry);
    entry.success = save(name + "-success", run(code, 0, {}, realPwsh));
    assert.equal(entry.success.exit, 0, entry.success.stderr);
    assert.equal(entry.success.calls.length, 2);
    for (let step = 1; step <= 2; step++) {
      const failed = save(name + "-failure-" + step, run(code, step, {}, realPwsh));
      entry.failures.push(failed);
      assertFailure(failed, step);
    }
    const swallowed = code.replace('if ($LASTEXITCODE -ne 0)', 'if ($false)');
    assert.notEqual(swallowed, code);
    fs.writeFileSync(join(root, name + ".canary.fence"), swallowed);
    const canary = save(name + "-canary", run(swallowed, 1, {}, realPwsh));
    assert.throws(() => assertFailure(canary, 1), /A12/);
    entry.canaryKilled = true;
  }
  if (selected("configuration")) {
    // Existing-binary control-flow tier: cargo alone is a seam. The exact named
    // fixture and real probe still run on success/old fences, so stale binaries
    // cannot hide build failure. Xtrace independently detects downstream launch.
    const text = fs.readFileSync(join(sourceRoot, "docs/operator/configuration.md"), "utf8");
    const code = bashFences(text)[0];
    assert.ok(code.includes('--config "$example_dir/config.json" --all-sessions'));
    fs.writeFileSync(join(root, "configuration.fence"), code);
    const entry = { name: "configuration", tier: "offline-existing-binary-control-flow-NOT-build",
      documentSha256: digest(text), code,
      binaryPath: join(repositoryRoot, "target/debug/tray-goblin-probe"),
      binarySha256: digest(fs.readFileSync(join(repositoryRoot, "target/debug/tray-goblin-probe"))),
      fixture: "tests/fixtures/live-session",
      fixtureSha256: Object.fromEntries(fs.readdirSync(join(repositoryRoot, "tests/fixtures/live-session"), { recursive: true })
        .filter((p) => fs.statSync(join(repositoryRoot, "tests/fixtures/live-session", p)).isFile())
        .map((p) => [p, digest(fs.readFileSync(join(repositoryRoot, "tests/fixtures/live-session", p)))])),
      success: null, failures: [], canaryKilled: false };
    report.auditFences.push(entry);
    function configRun(exact, fail) {
      const result = spawnSync(realBash, ["--noprofile", "--norc", "-c",
        `cargo() { if [[ $CONTROL_FAIL == 1 ]]; then echo INJECTED_CARGO_EXIT73 >&2; return 73; fi; echo CONTROL_CARGO_SUCCESS_NO_BUILD >&2; }\n` +
        `trap 'if [[ -n ${'${example_dir:-}'} ]]; then rm -rf "$example_dir"; fi' EXIT\nPS4='+CONTROL: '; set -x\n` + exact],
        { cwd: repositoryRoot, env: { ...env, CONTROL_FAIL: String(fail) }, encoding: "utf8" });
      assert.equal(result.error, undefined);
      return { exit: result.status, stdout: result.stdout, stderr: result.stderr };
    }
    function configFailure(result) {
      assert.ok(result.stderr.includes("INJECTED_CARGO_EXIT73"), "A05: executed cargo failure marker required");
      assert.equal(result.exit, 73, "A05: build exit73 must propagate, not be masked by an existing probe");
      assert.equal(result.stdout, "", "A05: NO downstream probe JSON after failed build");
      assert.ok(!result.stderr.includes("+CONTROL: env -i"), "A05: NO downstream probe launch after failed build");
    }
    entry.success = save("configuration-success", configRun(code, 0));
    assert.equal(entry.success.exit, 0, entry.success.stderr);
    assert.ok(entry.success.stderr.includes("CONTROL_CARGO_SUCCESS_NO_BUILD"));
    assert.ok(entry.success.stderr.includes("+CONTROL: env -i"));
    const aggregate = JSON.parse(entry.success.stdout);
    assert.equal(aggregate.sources[0].source.id, "fixture-copilot");
    assert.equal(aggregate.sources[0].source.path, "tests/fixtures/live-session");
    assert.equal(aggregate.sources[0].sessions[0].session_id, "session-new");
    const failed = save("configuration-failure", configRun(code, 1));
    entry.failures.push(failed);
    configFailure(failed);
    const swallowed = code.replace('|| exit $?', '|| true');
    assert.notEqual(swallowed, code);
    fs.writeFileSync(join(root, "configuration.canary.fence"), swallowed);
    const canary = save("configuration-canary", configRun(swallowed, 1));
    assert.throws(() => configFailure(canary), /A05/);
    entry.canaryKilled = true;
  }
  fs.writeFileSync(join(root, "report.json"), JSON.stringify(report, null, 2));
  const canonicalFailures = report.fences.reduce((n, f) => n + f.failures.length, 0) + (report.powershell ?? []).reduce((n, f) => n + f.failures.length, 0);
  console.log(`Release-doc controls: ${auditOnly ? "audit-only (no build)" : "annotated tag commit verified; " + fences.length + " exact Bash + " + report.powershell.length + " PowerShell canonical fences"}; ${report.auditFences.length} exact audit fences (including actual ZIP when selected); ${canonicalFailures + report.auditFences.reduce((n, f) => n + f.failures.length, 0)} injected failures, success controls and per-fence swallowed-error canaries. Evidence: ${root}`);
} catch (error) {
  fs.writeFileSync(join(root, "rejected.json"), JSON.stringify({ ...report, failure: String(error) }, null, 2));
  throw error;
}
