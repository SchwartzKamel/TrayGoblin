// Continuous offline journey: two ACTUAL installed pi runners keep the shipped
// publisher alive together while the shipped Rust dispatcher consumes their files.
import assert from "node:assert/strict";
import { spawn, fork, execFileSync, spawnSync } from "node:child_process";
import { once } from "node:events";
import * as fs from "node:fs/promises";
import { createHash } from "node:crypto";
import { dirname, join, resolve } from "node:path";
import { createInterface } from "node:readline";

const [payload, driverPath, probePath, evidencePath, engine] = process.argv.slice(2);
assert.ok(payload && driverPath && probePath && evidencePath);
assert.equal(engine, "podman");
const repo = resolve(import.meta.dirname, "..");
const packageRoot = process.env.PI_CODING_AGENT_PACKAGE ??
  resolve(dirname(process.execPath), "../lib/node_modules/@earendil-works/pi-coding-agent");
await fs.mkdir(evidencePath, { recursive: true });
const work = await fs.mkdtemp(join(evidencePath, "journey-"));
const workspace = join(work, "same-working-directory");
const statusDir = join(work, "status");
await fs.mkdir(workspace);
const extension = join(payload, "integrations/pi/tray-goblin.js");
// Standalone instructions/links must resolve in the INSTALLED payload after
// extraction is removed, with no checkout or distribution-relative dependency.
for (const path of ["resources/SETUP.md", "integrations/pi/README.md"]) {
  const absolute = join(payload, path);
  const text = await fs.readFile(absolute, "utf8");
  for (const [, target] of text.matchAll(/\]\(([^)#]+)(?:#[^)]*)?\)/g)) {
    if (!/^https?:/.test(target)) assert.equal((await fs.stat(resolve(dirname(absolute), target))).isFile(), true);
  }
}
const buildIdentity = JSON.parse(await fs.readFile(join(payload, "resources/build.json"), "utf8"));
assert.equal(buildIdentity.published, false);
assert.equal(buildIdentity.executableSha256,
  createHash("sha256").update(await fs.readFile(join(payload, "tray-goblin.exe"))).digest("hex"));
const sha = async (path) => createHash("sha256").update(await fs.readFile(path)).digest("hex");
const evidence = { utc: new Date().toISOString(), buildCommit: execFileSync("git", ["rev-parse", "HEAD"], { cwd: repo, encoding: "utf8" }).trim(),
  tier: "offline-real-pi-host-to-shipped-rust-not-windows", phases: [], peers: [],
  digests: {}, commands: [], oci: null };
for (const path of [extension, join(payload, "tray-goblin.exe"), join(payload, "resources/build.json"), driverPath, probePath, join(repo, "scripts/test-pi-host.mjs"), import.meta.filename,
  ...["app", "attention", "attention_worker", "monitor", "config"].map((name) => join(repo, `src/${name}.rs`)),
  join(packageRoot, "dist/core/extensions/runner.js"), join(packageRoot, "dist/core/extensions/loader.js")]) {
  evidence.digests[path] = await sha(path);
}
const children = [];
function timed(promise) {
  return Promise.race([promise, new Promise((_, reject) => {
    const timer = setTimeout(() => reject(new Error("demo boundary timed out")), 12000);
    timer.unref();
  })]);
}
async function peer(label) {
  const home = join(work, label);
  await fs.mkdir(home);
  const args = [packageRoot, extension, "attention-peer"];
  const env = { PATH: dirname(process.execPath), HOME: home, USERPROFILE: home,
    XDG_CONFIG_HOME: join(home, "config"), XDG_DATA_HOME: join(home, "data"),
    XDG_CACHE_HOME: join(home, "cache"), XDG_STATE_HOME: join(home, "state"),
    PI_CODING_AGENT_DIR: join(home, "agent"), TRAY_GOBLIN_STATUS_DIR: statusDir, TRAY_GOBLIN_LABEL: label };
  const child = fork(join(repo, "scripts/test-pi-host.mjs"), args, { cwd: workspace, env, silent: true });
  children.push(child);
  let stderr = "";
  child.stderr.on("data", (value) => { stderr += value; assert.ok(stderr.length < 65536); });
  child.stdout.on("data", () => {});
  const closed = once(child, "exit");
  const ready = await timed(once(child, "message").then(([m]) => m));
  assert.ok(ready.ready && ready.pid === child.pid);
  assert.equal(ready.hostVersion, "1.4.0");
  evidence.commands.push({ executable: process.execPath, args: [join(repo, "scripts/test-pi-host.mjs"), ...args], cwd: workspace, home });
  let id = 0;
  return { child, ready: ready.ready, async command(command) {
    assert.equal(child.exitCode, null, "both actual publishers must remain alive together");
    const response = timed(once(child, "message").then(([m]) => m));
    child.send({ command, id: ++id });
    const message = await response;
    assert.equal(message.id, id);
    assert.equal(message.failure, undefined);
    assert.equal(message.evidence.sensitiveReads, 0);
    assert.equal(message.evidence.networkCalls, 0);
    assert.equal(message.evidence.busyMetadataReads, 0);
    if (command === "stop") {
      const [code] = await timed(closed);
      assert.equal(code, 0);
      assert.equal(stderr, "");
      assert.equal(message.evidence.uiStarts, message.evidence.uiEnds);
      evidence.peers.push({ pid: child.pid, home, sessionId: ready.ready.sessionId, instanceId: ready.ready.instanceId, ...message.evidence });
      await assert.rejects(fs.stat(join(home, "agent/auth.json")), { code: "ENOENT" });
      await assert.rejects(fs.stat(join(home, "agent/settings.json")), { code: "ENOENT" });
    }
    return message.status;
  } };
}
let driver;
try {
  // Start concurrently, with one shared cwd and status source, NOT two sequential recordings.
  const [a, b] = await Promise.all([peer("review"), peer("build")]);
  assert.notEqual(a.ready.sessionId, b.ready.sessionId);
  assert.notEqual(a.ready.instanceId, b.ready.instanceId);
  assert.equal(a.ready.directory, b.ready.directory);
  const configPath = join(work, "config.json");
  await fs.writeFile(configPath, JSON.stringify({ sources: [{ id: "live-pi", label: "Offline pi", kind: "pi", path: statusDir }] }));
  driver = spawn(driverPath, [configPath], { env: { PATH: process.env.PATH, HOME: work, USERPROFILE: work }, stdio: ["pipe", "pipe", "pipe"] });
  children.push(driver);
  const driverExit = once(driver, "exit");
  const lines = createInterface({ input: driver.stdout })[Symbol.asyncIterator]();
  let driverErrors = "";
  driver.stderr.on("data", (v) => { driverErrors += v; assert.ok(driverErrors.length < 65536); });
  evidence.commands.push({ executable: driverPath, args: [configPath] });
  const expectation = (p, state, episode = null) => ({ session_id: p.ready.sessionId, instance_id: p.ready.instanceId,
    label: p.ready.label, directory: workspace, state, episode });
  async function observe(phase, sessions, submissions, quiet_ms = 80, attributedTo = submissions ? [a] : []) {
    for (const p of [a, b]) {
      assert.equal(p.child.exitCode, null);
      assert.equal(p.child.signalCode, null);
      assert.equal(p.child.connected, true);
      process.kill(p.child.pid, 0); // OS liveness probe, not process control.
    }
    driver.stdin.write(JSON.stringify({ phase, sessions, submissions, quiet_ms }) + "\n");
    const line = await timed(lines.next());
    assert.equal(line.done, false, driverErrors);
    const result = JSON.parse(line.value);
    assert.equal(result.phase, phase);
    assert.equal(result.sourceReads, evidence.phases.length + 1, "each step requires an actual completed source read");
    assert.equal(result.adapterRequests.length, submissions);
    assert.equal(result.submissionFailed, false);
    for (const request of result.adapterRequests) {
      assert.equal(request.accepted, true);
      assert.equal(request.title, `TrayGoblin — ${attributedTo.length} need attention`);
      const identified = attributedTo.find((p) => request.body.includes(`pi | Offline pi | ${p.ready.label} |`)
        && request.body.includes(p.ready.sessionId.slice(0, 12)) && request.body.includes(p.ready.instanceId.slice(0, 8)));
      assert.ok(identified, "actual submitted balloon must identify the eligible peer, not merely a correct snapshot");
      const state = sessions.find((s) => s.session_id === identified.ready.sessionId).state;
      assert.ok(request.body.includes(state === "error" ? "Session error" : "Input required"));
    }
    result.livePublisherPids = [a.child.pid, b.child.pid];
    evidence.phases.push(result);
    await fs.writeFile(join(evidencePath, "journey.json"), JSON.stringify(evidence, null, 2));
    return result;
  }
  await observe("concurrent-idle", [expectation(a, "idle"), expectation(b, "idle")], 0);
  // A manual idle dialog is approved attention; a second same-directory session keeps working.
  await b.command("run");
  const input = await a.command("open");
  assert.equal(input.runStartedAt, null);
  await observe("manual-dialog-and-other-work", [expectation(a, "input_required", input.episode), expectation(b, "working")], 1);
  await observe("same-dialog-deduplicated", [expectation(a, "input_required", input.episode), expectation(b, "working")], 0, 2100);
  const heartbeat = JSON.parse(await fs.readFile(join(statusDir, `${a.ready.instanceId}.json`), "utf8"));
  assert.ok(heartbeat.updatedAt > input.updatedAt, "actual 2s heartbeat while both publishers remain alive");
  assert.equal(heartbeat.episode, input.episode);
  assert.equal(heartbeat.state, "input_required");
  evidence.heartbeat = { before: input.updatedAt, after: heartbeat.updatedAt, instanceId: heartbeat.instanceId };
  const bInput = await b.command("open");
  await observe("independent-second-peer-dialog", [expectation(a, "input_required", input.episode), expectation(b, "input_required", bInput.episode)], 1, 80, [b]);
  await b.command("close");
  await observe("second-peer-restores-working", [expectation(a, "input_required", input.episode), expectation(b, "working")], 0);

  // OCI is a separate CLI-only boundary. Only read-only binary+synthetic fixture
  // mounts enter cached Debian; neither SDK nor user state enters the container.
  const fixture = join(work, "oci");
  await fs.mkdir(fixture, { mode: 0o755 });
  await fs.mkdir(join(fixture, "pi"), { mode: 0o755 });
  for (const p of [a, b]) await fs.copyFile(join(statusDir, `${p.ready.instanceId}.json`), join(fixture, "pi", `${p.ready.instanceId}.json`));
  await fs.writeFile(join(fixture, "config.json"), JSON.stringify({ sources: [{ id: "oci-pi", label: "OCI synthetic", kind: "pi", path: "/fixtures/pi" }] }));
  for (const f of await fs.readdir(join(fixture, "pi"))) await fs.chmod(join(fixture, "pi", f), 0o444);
  await fs.chmod(join(fixture, "config.json"), 0o444);
  const storage = process.env.TRAY_GOBLIN_PODMAN_ROOT ? ["--root", process.env.TRAY_GOBLIN_PODMAN_ROOT, "--runroot", process.env.TRAY_GOBLIN_PODMAN_RUNROOT] : [];
  assert.ok(storage.length === 0 || storage.at(-1));
  const image = "docker.io/library/debian:bookworm-slim";
  const imageId = execFileSync(engine, [...storage, "image", "inspect", "--format", "{{.Id}}", image], { encoding: "utf8" }).trim();
  // Podman inspect reports a full content ID, with or without the OCI prefix
  // depending on its version. Preserve the EXACT engine-reported identity.
  assert.match(imageId, /^(sha256:)?[0-9a-f]{64}$/);
  const containerName = `traygoblin-oci-${process.pid}-${Date.now()}`;
  const args = [...storage, "run", "--name", containerName, "--rm", "--timeout", "20", "--pull", "never", "--network", "none", "--read-only", "--read-only-tmpfs=false",
    "--user", "65534:65534", "--cap-drop", "all", "--security-opt", "no-new-privileges",
    "--volume", `${resolve(probePath)}:/probe:ro`, "--volume", `${fixture}:/fixtures:ro`, imageId,
    "/probe", "--config", "/fixtures/config.json", "--all-sessions"];
  const ociRaw = execFileSync(engine, args, { encoding: "utf8", timeout: 30000 });
  const oci = JSON.parse(ociRaw);
  assert.deepEqual(Object.keys(oci), ["sources"]);
  assert.equal(oci.sources.length, 1);
  const source = oci.sources[0];
  assert.deepEqual(Object.keys(source).sort(), ["source", "capabilities", "sessions", "unavailable_pi_statuses", "diagnostics"].sort());
  assert.deepEqual(source.capabilities, { input_detection: true, session_errors: true });
  assert.deepEqual(source.diagnostics, []);
  assert.equal(source.unavailable_pi_statuses, 0);
  assert.deepEqual(source.source, { id: "oci-pi", label: "OCI synthetic", kind: "pi", path: "/fixtures/pi", wsl_distro: null });
  assert.equal(source.sessions.length, 2, "empty successful container output is not evidence");
  for (const [p, state] of [[a, "input_required"], [b, "working"]]) {
    const row = source.sessions.find((s) => s.session_id === p.ready.sessionId);
    assert.ok(row);
    assert.deepEqual(Object.keys(row).sort(), ["session_id", "label", "instance_id", "model", "directory", "state", "attention", "run_started_at_ms", "last_run_duration_ms"].sort());
    assert.equal(row.instance_id, p.ready.instanceId);
    assert.equal(row.label, p.ready.label);
    assert.equal(row.directory, workspace);
    assert.equal(row.model, "synthetic-model");
    assert.equal(row.state, state);
    if (p === b) assert.equal(row.attention, null);
  }
  assert.deepEqual(Object.keys(source.source).sort(), ["id", "label", "kind", "path", "wsl_distro"].sort());
  assert.deepEqual(source.sessions.find((s) => s.session_id === a.ready.sessionId).attention, { id: input.episode, reason: "input_required", notify: true });
  const cleanup = spawnSync(engine, [...storage, "container", "exists", containerName], { encoding: "utf8" });
  assert.equal(cleanup.error, undefined);
  assert.equal(cleanup.status, 1, "engine must confirm the --rm container is absent, not just uninspectable");
  evidence.oci = { engine, cleanup: { command: [engine, ...storage, "container", "exists", containerName], exit: cleanup.status }, engineVersion: execFileSync(engine, [...storage, "--version"], { encoding: "utf8" }).trim(), image, imageId, args, result: oci, tier: "cli-only-not-pi-host-or-windows" };
  await fs.writeFile(join(evidencePath, "oci.json"), JSON.stringify(evidence.oci, null, 2));
  await a.command("close");
  await observe("manual-close-restores-idle", [expectation(a, "idle"), expectation(b, "working")], 0);
  await b.command("tentative-error");
  await observe("nonfinal-error-stays-working", [expectation(a, "idle"), expectation(b, "working")], 0);
  await b.command("run");
  await b.command("complete");
  await observe("recovered-tool-and-success-quiet", [expectation(a, "idle"), expectation(b, "idle")], 0, 3000);
  const [aTogether, bTogether] = await Promise.all([a.command("open"), b.command("open")]);
  await observe("two-actual-episodes-coalesced", [expectation(a, "input_required", aTogether.episode), expectation(b, "input_required", bTogether.episode)], 1, 80, [a, b]);
  await Promise.all([a.command("close"), b.command("close")]);
  await observe("both-manual-dialogs-restore-idle", [expectation(a, "idle"), expectation(b, "idle")], 0);
  await a.command("run");
  const failed = await a.command("error");
  assert.equal(failed.attentionReason, "run_error");
  await observe("final-settle-error", [expectation(a, "error", failed.episode), expectation(b, "idle")], 1);
  const overlay = await a.command("open");
  assert.ok(overlay.episode > failed.episode);
  await observe("dialog-over-error", [expectation(a, "input_required", overlay.episode), expectation(b, "idle")], 1);
  const restored = await a.command("close");
  assert.equal(restored.episode, failed.episode);
  await observe("same-error-restored-without-replay", [expectation(a, "error", failed.episode), expectation(b, "idle")], 0);
  await a.command("run");
  await a.command("abort");
  await observe("new-run-clears-and-abort-quiet", [expectation(a, "idle"), expectation(b, "idle")], 0);
  // Both publishers remain alive until ALL observations above are acknowledged.
  await Promise.all([a.command("stop"), b.command("stop")]);
  assert.deepEqual(await fs.readdir(statusDir), []);
  driver.stdin.end();
  const [code] = await timed(driverExit);
  assert.equal(code, 0, driverErrors);
  assert.equal(driverErrors, "");
  const serialized = JSON.stringify(evidence);
  assert.doesNotMatch(serialized, /SYNTHETIC_PRIVATE_(TITLE|ARGS)|SENSITIVE_SENTINEL/);
  assert.equal(evidence.phases.reduce((n, p) => n + p.adapterRequests.length, 0), 5);
  evidence.finishedUtc = new Date().toISOString();
  await fs.writeFile(join(evidencePath, "journey.json"), JSON.stringify(evidence, null, 2));
  console.log(`Continuous real-host→shipped Rust journey: ${evidence.phases.length} phases, 5 actual adapter requests; evidence ${evidencePath}`);
} finally {
  for (const child of children) if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
}
