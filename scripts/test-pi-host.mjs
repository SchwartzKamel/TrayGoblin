import assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import nativeFs from "node:fs/promises";
import { syncBuiltinESMExports } from "node:module";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import http from "node:http";
import https from "node:https";
import net from "node:net";

const [packageRoot, extensionPath, mode] = process.argv.slice(2);
assert.ok(packageRoot && extensionPath, "package and extension paths are required");
assert.ok(mode === undefined || mode === "startup-recovery" || mode === "startup-order");
assert.notEqual(process.env.HOME, undefined);
assert.equal(process.env.HOME, process.env.USERPROFILE);
assert.equal(process.env.PI_CODING_AGENT_DIR, join(process.env.HOME, "agent"));
const manifest = JSON.parse(await fs.readFile(join(packageRoot, "package.json"), "utf8"));
assert.equal(manifest.name, "@earendil-works/pi-coding-agent");
assert.equal(manifest.version, "1.4.0", "host validation is pinned, not skipped");

const evidence = {
  hostVersion: manifest.version,
  extensionsLoaded: 0,
  uiStarts: 0,
  uiEnds: 0,
  sensitiveReads: 0,
  sessionIdReads: 0,
  modelIdReads: 0,
  busyMetadataReads: 0,
  networkCalls: 0,
  heartbeatObserved: false,
  replacementObserved: false,
  cleanupObserved: false,
  finalEpisode: 0,
  ioFailures: [],
  pendingShutdownObserved: false,
};
const forbidden = () => {
  evidence.sensitiveReads++;
  throw new Error("forbidden synthetic sensitive access");
};
const noNetwork = () => {
  evidence.networkCalls++;
  throw new Error("network is forbidden in offline host validation");
};
globalThis.fetch = noNetwork;
http.request = noNetwork;
http.get = noNetwork;
https.request = noNetwork;
https.get = noNetwork;
net.Socket.prototype.connect = noNetwork;

// Fault injection still runs the runtime-loaded artifact, not a second publisher factory.
const realIO = {
  writeFile: nativeFs.writeFile,
  rename: nativeFs.rename,
  unlink: nativeFs.unlink,
};
let fault;
let blockedWrite;
nativeFs.writeFile = async (path, ...args) => {
  if (String(path).startsWith(process.env.TRAY_GOBLIN_STATUS_DIR)) {
    if (fault === "write") throw new Error("SYNTHETIC_PRIVATE_WRITE");
    if (blockedWrite) {
      blockedWrite.entered.resolve();
      await blockedWrite.release.promise;
    }
  }
  return realIO.writeFile(path, ...args);
};
nativeFs.rename = async (from, ...args) => {
  if (fault === "rename" && String(from).startsWith(process.env.TRAY_GOBLIN_STATUS_DIR)) {
    throw new Error("SYNTHETIC_PRIVATE_RENAME");
  }
  return realIO.rename(from, ...args);
};
nativeFs.unlink = async (path, ...args) => {
  if (fault === "delete" && String(path).endsWith(".json")) {
    throw new Error("SYNTHETIC_PRIVATE_DELETE");
  }
  return realIO.unlink(path, ...args);
};
syncBuiltinESMExports();

const installed = (relative) => import(pathToFileURL(join(packageRoot, "dist", relative)));
const { loadExtensions } = await installed("core/extensions/loader.js");
const { ExtensionRunner } = await installed("core/extensions/runner.js");
const { SessionManager } = await installed("core/session-manager.js");
const extensionPaths = [resolve(extensionPath)];
if (mode === "startup-order") {
  const earlierPath = join(process.cwd(), "earlier-confirm.mjs");
  await fs.writeFile(earlierPath, `
export default function earlierConfirm(pi) {
  pi.on("before_agent_start", async (_event, ctx) => {
    await ctx.ui.confirm("synthetic title", "synthetic");
  });
}
`);
  extensionPaths.unshift(earlierPath);
}
const loaded = await loadExtensions(extensionPaths, process.cwd());
assert.deepEqual(loaded.errors, []);
assert.equal(loaded.extensions.length, extensionPaths.length);
await assert.rejects(fs.stat(process.env.TRAY_GOBLIN_STATUS_DIR), { code: "ENOENT" });
evidence.extensionsLoaded = loaded.extensions.length;
const session = SessionManager.inMemory(process.cwd());
const manager = new Proxy(session, {
  get(target, key) {
    if (key !== "getSessionId") return forbidden();
    return () => { evidence.sessionIdReads++; return target.getSessionId(); };
  },
});
let modelId = "synthetic-model";
const model = new Proxy({}, {
  get(_target, key) {
    if (key !== "id") return forbidden();
    evidence.modelIdReads++;
    return modelId;
  },
});
const runner = new ExtensionRunner(
  loaded.extensions, loaded.runtime, process.cwd(), manager,
  new Proxy({}, { get: forbidden }),
);
let hostBusy = false;
runner.bindCore(new Proxy({}, { get: () => forbidden }), new Proxy({
  getModel: () => model,
  isIdle: () => {
    evidence.busyMetadataReads++;
    return !hostBusy;
  },
}, { get: (target, key) => target[key] ?? forbidden }));
const errors = [];
runner.onError((error) => errors.push(error));

// Poison sensitive event/context fields without replacing the real dispatch or UI wrapping.
const originalContext = runner.createContext.bind(runner);
runner.createContext = () => {
  const ctx = originalContext();
  for (const key of ["modelRegistry", "scopedModels", "thinkingLevel", "signal"]) {
    Object.defineProperty(ctx, key, { get: forbidden });
  }
  for (const key of ["getContextUsage", "getSystemPrompt", "compact"]) ctx[key] = forbidden;
  return ctx;
};
if (mode === "startup-order") {
  const publisherExtension = loaded.extensions.at(-1);
  const handlers = publisherExtension.handlers.get("before_agent_start");
  publisherExtension.handlers.set("before_agent_start", handlers.map((handler) =>
    (event, ctx) => handler(new Proxy(event, {
      get(_target, key) {
        if (key !== "type") return forbidden();
        return event.type;
      },
    }), ctx),
  ));
}
const originalEmit = runner.emit.bind(runner);
let promptEnded;
runner.emit = async (event) => {
  if (event.type === "ui_prompt_start" || event.type === "ui_prompt_end") {
    evidence[event.type === "ui_prompt_start" ? "uiStarts" : "uiEnds"]++;
    Object.defineProperty(event, "title", { get: forbidden });
  }
  const result = await originalEmit(event);
  if (event.type === "ui_prompt_end") promptEnded?.resolve();
  return result;
};
function event(type, fields = {}) {
  const value = { type, ...fields };
  for (const key of ["messages", "message", "toolResults", "input", "result", "tokens", "credentials"]) {
    Object.defineProperty(value, key, { get: forbidden });
  }
  return value;
}
function deferred() {
  let resolve;
  let reject;
  const promise = new Promise((done, fail) => { resolve = done; reject = fail; });
  return { promise, resolve, reject };
}
async function status() {
  const files = (await fs.readdir(process.env.TRAY_GOBLIN_STATUS_DIR))
    .filter((name) => name.endsWith(".json"));
  assert.equal(files.length, 1);
  const value = JSON.parse(await fs.readFile(join(process.env.TRAY_GOBLIN_STATUS_DIR, files[0]), "utf8"));
  assert.equal(value.sessionId, session.getSessionId());
  assert.equal(value.directory, process.cwd());
  assert.equal(value.model, modelId);
  return value;
}
async function waitFor(predicate) {
  const deadline = Date.now() + 5000;
  while (Date.now() < deadline) {
    const value = await status();
    if (predicate(value)) return value;
    await new Promise((done) => setTimeout(done, 5));
  }
  assert.fail("publisher did not reach expected state before deadline");
}
let dialog;
let synchronousRejection = false;
let fastResolution = false;
runner.setUIContext({
  ...runner.getUIContext(),
  confirm: () => {
    if (synchronousRejection) throw new Error("synthetic dialog rejection");
    if (fastResolution) return Promise.resolve(true);
    return dialog.promise;
  },
  select: () => dialog.promise,
  input: () => dialog.promise,
  editor: () => dialog.promise,
  custom: async () => runner.getUIContext().confirm("nested synthetic title", "synthetic"),
}, "tui");
async function boundary(outcome) {
  await runner.emitBoundary({ type: "agent_before_settle", outcome }, () =>
    new Proxy({}, { get: (_target, key) => key === "then" ? undefined : forbidden() }));
}
async function settle(outcome, aborted = false) {
  await boundary(outcome);
  await runner.emit(event("agent_settled", { aborted }));
}
async function prompt(kind, { reject = false, idle = false } = {}) {
  dialog = deferred();
  const previous = await status();
  const args = kind === "custom" ? [forbidden] : ["synthetic title", "synthetic"];
  const waiting = runner.getUIContext()[kind](...args);
  const observed = waiting.then(
    () => "resolved",
    () => "rejected",
  );
  if (idle) {
    await new Promise((done) => setImmediate(done));
    assert.equal((await status()).state, previous.state);
    assert.equal((await status()).episode, previous.episode);
  } else {
    const input = await waitFor((value) => value.state === "input_required");
    assert.equal(input.episode, previous.episode + 1);
    await new Promise((done) => setTimeout(done, 20));
    assert.equal((await status()).episode, input.episode);
  }
  if (reject) dialog.reject(new Error("synthetic rejected result"));
  else dialog.resolve(true);
  assert.equal(await observed, reject ? "rejected" : "resolved");
  await waitFor((value) => value.state === (idle ? previous.state : "working"));
}

async function exerciseLifecycle() {
  await runner.emit(event("session_start"));
  assert.equal((await status()).state, "idle");
  const first = await status();
  await waitFor((value) => value.updatedAt > first.updatedAt);
  evidence.heartbeatObserved = true;
  await prompt("confirm", { idle: true });

  await runner.emit(event("before_agent_start"));
  assert.equal((await status()).state, "working");
  await runner.emit(event("agent_start"));
  await prompt("confirm");
  await prompt("input", { reject: true });
  await prompt("custom");
  await runner.emit(event("tool_execution_end", { isError: true }));
  await runner.emit(event("agent_end"));
  await boundary("error");
  assert.equal((await status()).state, "working");
  // Continuation/recovery starts another low-level run; the tentative error never alerts.
  await runner.emit(event("agent_start"));
  await settle("completed");
  assert.equal((await status()).state, "idle");
  assert.equal((await status()).episode, 3);

  await runner.emit(event("agent_start"));
  await settle("error");
  const failed = await status();
  assert.equal(failed.state, "error");
  assert.equal(failed.episode, 4);
  await runner.emit(event("agent_end"));
  await prompt("select", { idle: true });
  assert.equal((await status()).state, "error");
  await waitFor((value) => value.updatedAt > failed.updatedAt);
  assert.equal((await status()).state, "error");

  await runner.emit(event("agent_start"));
  assert.equal((await status()).state, "working");
  await settle("error", true);
  assert.equal((await status()).state, "idle");
  assert.equal((await status()).episode, 4);
  await runner.emit(event("agent_start"));
  await settle("aborted");
  assert.equal((await status()).state, "idle");

  await runner.emit(event("agent_start"));
  synchronousRejection = true;
  assert.throws(() => runner.getUIContext().confirm("synthetic", "synthetic"), /synthetic dialog rejection/);
  synchronousRejection = false;
  await waitFor((value) => value.state === "working" && value.episode === 5);
  await settle("completed");
  evidence.finalEpisode = (await status()).episode;
  await prompt("editor", { idle: true });

  modelId = "synthetic-new-model";
  await runner.emit(event("model_select"));
  assert.equal((await status()).model, modelId);
  const old = await status();
  await runner.emit(event("session_shutdown"));
  await runner.emit(event("session_shutdown"));
  assert.deepEqual(await fs.readdir(process.env.TRAY_GOBLIN_STATUS_DIR), []);
  await runner.emit(event("session_start"));
  assert.notEqual((await status()).instanceId, old.instanceId);
  assert.equal((await status()).episode, 0);
  evidence.replacementObserved = true;

  fault = "write";
  await runner.emit(event("agent_start"));
  assert.equal((await status()).state, "idle");
  fault = undefined;
  await runner.emit(event("agent_start"));
  assert.equal((await status()).state, "working");
  fault = "rename";
  await runner.emit(event("model_select"));
  fault = undefined;
  await settle("completed");

  blockedWrite = { entered: deferred(), release: deferred() };
  const pendingWrite = runner.emit(event("agent_start"));
  await blockedWrite.entered.promise;
  let closed = false;
  const pendingShutdown = runner.emit(event("session_shutdown")).then(() => { closed = true; });
  await new Promise((done) => setImmediate(done));
  assert.equal(closed, false);
  blockedWrite.release.resolve();
  await Promise.all([pendingWrite, pendingShutdown]);
  blockedWrite = undefined;
  assert.deepEqual(await fs.readdir(process.env.TRAY_GOBLIN_STATUS_DIR), []);
  evidence.pendingShutdownObserved = true;

  await runner.emit(event("session_start"));
  const deleting = await status();
  fault = "delete";
  await runner.emit(event("session_shutdown"));
  assert.equal((await status()).instanceId, deleting.instanceId);
  fault = undefined;
  await realIO.unlink(join(process.env.TRAY_GOBLIN_STATUS_DIR, `${deleting.instanceId}.json`));
  await runner.emit(event("session_start"));
  await runner.emit(event("session_shutdown"));
  evidence.ioFailures = errors.map((error) => error.error);
  assert.deepEqual(evidence.ioFailures, [
    "TrayGoblin: write-failed", "TrayGoblin: rename-failed", "TrayGoblin: delete-failed",
  ]);
  for (const error of errors) {
    assert.doesNotMatch(error.error, /PRIVATE/);
  }
  evidence.cleanupObserved = (await fs.readdir(process.env.TRAY_GOBLIN_STATUS_DIR)).length === 0;
  assert.equal(evidence.sensitiveReads, 0);
  assert.equal(evidence.networkCalls, 0);
}

// A failed first write must recover through the loaded host without another session_start.
async function exerciseStartupRecovery() {
  fault = "write";
  await runner.emit(event("session_start"));
  assert.deepEqual(errors.map(({ event, error }) => ({ event, error })), [
    { event: "session_start", error: "TrayGoblin: write-failed" },
  ]);
  assert.deepEqual(await fs.readdir(process.env.TRAY_GOBLIN_STATUS_DIR), []);
  fault = undefined;
  await runner.emit(event("agent_start"));
  const recovered = await status();
  assert.equal(recovered.state, "working");
  dialog = deferred();
  const waiting = runner.getUIContext().confirm("synthetic title", "synthetic");
  const input = await waitFor((value) => value.state === "input_required");
  try {
    const heartbeat = await waitFor((value) => value.updatedAt > input.updatedAt);
    assert.equal(heartbeat.instanceId, recovered.instanceId);
    assert.equal(heartbeat.state, "input_required");
    assert.equal(heartbeat.episode, input.episode);
    evidence.startupRecovery = {
      initialError: errors[0].error,
      recoveredAt: recovered.updatedAt,
      inputAt: input.updatedAt,
      heartbeatAt: heartbeat.updatedAt,
      episode: heartbeat.episode,
    };
    evidence.heartbeatObserved = true;
  } finally {
    dialog.resolve(true);
    await waiting;
  }
  await waitFor((value) => value.state === "working");
  await runner.emit(event("session_shutdown"));
  evidence.cleanupObserved = (await fs.readdir(process.env.TRAY_GOBLIN_STATUS_DIR)).length === 0;
  assert.equal(errors.length, 1);
  assert.equal(evidence.sensitiveReads, 0);
  assert.equal(evidence.networkCalls, 0);
}

// Earlier awaited startup UI must alert before answer, and fast completion must not strand input.
async function exerciseStartupOrder() {
  await runner.emit(event("session_start"));
  await prompt("confirm", { idle: true });
  assert.equal((await status()).episode, 0);
  evidence.startupOrder = { waitingEpisodes: [], clearedEpisodes: [] };
  for (const reject of [false, true]) {
    const previous = await status();
    assert.equal(previous.state, "idle");
    hostBusy = true;
    assert.equal(runner.createContext().isIdle(), false);
    dialog = deferred();
    let finished = false;
    const starting = runner.emitBeforeAgentStart(
      { messageType: "user", prompt: "synthetic" },
      { cwd: process.cwd(), selectedTools: [] },
    ).then(() => { finished = true; });
    try {
      const input = await waitFor((value) => value.state === "input_required");
      assert.equal(finished, false, "earlier startup handler is still awaiting confirm");
      assert.equal(input.episode, previous.episode + 1);
      assert.equal(input.instanceId, previous.instanceId);
      assert.ok(Number.isSafeInteger(input.runStartedAt));
      evidence.startupOrder.waitingEpisodes.push(input.episode);
      if (reject) dialog.reject(new Error("synthetic dialog rejection"));
      else dialog.resolve(true);
      await starting;
      const cleared = await waitFor((value) => value.state === "working");
      assert.equal(cleared.episode, input.episode);
      assert.equal(cleared.runStartedAt, input.runStartedAt);
      evidence.startupOrder.clearedEpisodes.push(cleared.episode);
      await settle("completed");
      hostBusy = false;
      assert.equal((await status()).state, "idle");
      await prompt("confirm", { idle: true });
      assert.equal((await status()).episode, input.episode);
    } finally {
      dialog.resolve(true);
      await starting;
    }
  }

  hostBusy = true;
  fastResolution = true;
  blockedWrite = { entered: deferred(), release: deferred() };
  promptEnded = deferred();
  const starting = runner.emitBeforeAgentStart(
    { messageType: "user", prompt: "synthetic" },
    { cwd: process.cwd(), selectedTools: [] },
  );
  try {
    await blockedWrite.entered.promise;
    await promptEnded.promise;
  } finally {
    blockedWrite.release.resolve();
    await starting;
    blockedWrite = undefined;
    fastResolution = false;
    promptEnded = undefined;
  }
  await waitFor((value) => value.state === "working" && value.episode === 3);
  evidence.startupOrder.fastEndCleared = true;
  await settle("error");
  hostBusy = false;
  const failed = await status();
  assert.equal(failed.state, "error");
  assert.equal(failed.episode, 4);
  await prompt("confirm", { idle: true });
  assert.equal((await status()).state, "error");
  assert.equal((await status()).episode, failed.episode);
  await runner.emit(event("session_shutdown"));
  evidence.cleanupObserved = (await fs.readdir(process.env.TRAY_GOBLIN_STATUS_DIR)).length === 0;
  assert.deepEqual(errors.map(({ event, error }) => ({ event, error })), [
    { event: "before_agent_start", error: "synthetic dialog rejection" },
  ]);
  assert.equal(evidence.sensitiveReads, 0);
  assert.equal(evidence.networkCalls, 0);
}

try {
  if (mode === "startup-recovery") await exerciseStartupRecovery();
  else if (mode === "startup-order") await exerciseStartupOrder();
  else await exerciseLifecycle();
  process.stdout.write(JSON.stringify(evidence) + "\n");
} finally {
  fault = undefined;
  for (const [name, implementation] of Object.entries(realIO)) nativeFs[name] = implementation;
  syncBuiltinESMExports();
  await runner.emit(event("session_shutdown"));
}
