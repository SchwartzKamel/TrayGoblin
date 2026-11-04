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
assert.ok(mode === undefined || [
  "attention-peer", "startup-recovery", "startup-order", "dialog-policy", "prompt-order",
  "old-debt-new-dialog-idle", "old-debt-new-dialog-working", "old-debt-new-dialog-error",
].includes(mode));
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
  manualLabel: null,
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
if (mode === "dialog-policy" || mode === "prompt-order" || mode?.startsWith("old-debt-new-dialog-")) {
  const earlierPath = join(process.cwd(), "earlier-ui-observer.mjs");
  await fs.writeFile(earlierPath, `
export default function earlierUIObserver(pi) {
  pi.on("ui_prompt_start", () => globalThis.trayGoblinTestStartObserver?.());
  pi.on("session_before_compact", async (_event, ctx) => {
    await ctx.ui.confirm("synthetic compaction title", "synthetic");
    return { cancel: true };
  });
}
`);
  extensionPaths.unshift(earlierPath);
}
const loaded = await loadExtensions(extensionPaths, process.cwd());
assert.deepEqual(loaded.errors, []);
assert.equal(loaded.extensions.length, extensionPaths.length);
// Standalone controls own a missing root; concurrent peers share the first
// publisher's already-created root, but still start no resources in factories.
if (mode !== "attention-peer") {
  await assert.rejects(fs.stat(process.env.TRAY_GOBLIN_STATUS_DIR), { code: "ENOENT" });
}
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
let promptStarted;
let promptEndDispatched;
runner.emit = async (event) => {
  if (event.type === "ui_prompt_start" || event.type === "ui_prompt_end") {
    evidence[event.type === "ui_prompt_start" ? "uiStarts" : "uiEnds"]++;
    Object.defineProperty(event, "title", { get: forbidden });
  }
  const dispatch = originalEmit(event);
  if (event.type === "ui_prompt_end") promptEndDispatched?.resolve();
  const result = await dispatch;
  if (event.type === "ui_prompt_start") promptStarted?.resolve();
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
  if (mode !== "attention-peer") assert.equal(files.length, 1);
  const values = await Promise.all(files.map(async (name) =>
    JSON.parse(await fs.readFile(join(process.env.TRAY_GOBLIN_STATUS_DIR, name), "utf8"))));
  const owned = values.filter((value) => value.sessionId === session.getSessionId());
  assert.equal(owned.length, 1);
  const [value] = owned;
  assert.equal(value.sessionId, session.getSessionId());
  assert.equal(value.directory, process.cwd());
  assert.equal(value.model, modelId);
  // The actual loaded extension may publish exactly one new optional scalar.
  const allowed = ["version", "provider", "sessionId", "instanceId", "model", "directory",
    "state", "attentionReason", "episode", "updatedAt", "runStartedAt", "lastRunDurationMs"];
  if (Object.hasOwn(value, "label")) {
    assert.equal(typeof value.label, "string");
    assert.ok(Buffer.byteLength(value.label) <= 64);
    assert.doesNotMatch(value.label, /[\u0000-\u001f\u007f-\u009f\ud800-\udfff]/u);
    allowed.push("label");
  }
  assert.deepEqual(Object.keys(value).sort(), allowed.sort());
  evidence.manualLabel = value.label ?? null;
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
let allocatedEpisode = 0;
async function prompt(kind, { reject = false } = {}) {
  dialog = deferred();
  const previous = await status();
  const args = kind === "custom" ? [forbidden] : ["synthetic title", "synthetic"];
  promptEnded = deferred();
  const waiting = runner.getUIContext()[kind](...args);
  const observed = waiting.then(
    () => "resolved",
    () => "rejected",
  );
  const input = await waitFor((value) => value.state === "input_required");
  assert.equal(input.attentionReason, null);
  allocatedEpisode = Math.max(allocatedEpisode, previous.episode) + 1;
  assert.equal(input.episode, allocatedEpisode);
  assert.equal(input.runStartedAt, previous.runStartedAt);
  await new Promise((done) => setTimeout(done, 20));
  assert.equal((await status()).episode, input.episode);
  if (reject) dialog.reject(new Error("synthetic rejected result"));
  else dialog.resolve(true);
  assert.equal(await observed, reject ? "rejected" : "resolved");
  await promptEnded.promise;
  promptEnded = undefined;
  const restored = await waitFor((value) => value.state === previous.state);
  assert.equal(restored.attentionReason, previous.attentionReason);
  if (previous.state === "error") assert.equal(restored.episode, previous.episode);
  assert.equal(restored.runStartedAt, previous.runStartedAt);
  assert.equal(restored.lastRunDurationMs, previous.lastRunDurationMs);
}

async function exerciseLifecycle() {
  await runner.emit(event("session_start"));
  assert.equal((await status()).state, "idle");
  const first = await status();
  await waitFor((value) => value.updatedAt > first.updatedAt);
  evidence.heartbeatObserved = true;
  await prompt("confirm");

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
  assert.equal((await status()).episode, 4);

  await runner.emit(event("agent_start"));
  await settle("error");
  const failed = await status();
  assert.equal(failed.state, "error");
  assert.equal(failed.episode, 5);
  await runner.emit(event("agent_end"));
  await prompt("select");
  assert.equal((await status()).state, "error");
  await waitFor((value) => value.updatedAt > failed.updatedAt);
  assert.equal((await status()).state, "error");

  await runner.emit(event("agent_start"));
  assert.equal((await status()).state, "working");
  await settle("error", true);
  assert.equal((await status()).state, "idle");
  assert.equal((await status()).episode, failed.episode);
  await runner.emit(event("agent_start"));
  await settle("aborted");
  assert.equal((await status()).state, "idle");

  await runner.emit(event("agent_start"));
  synchronousRejection = true;
  assert.throws(() => runner.getUIContext().confirm("synthetic", "synthetic"), /synthetic dialog rejection/);
  synchronousRejection = false;
  await waitFor((value) => value.state === "working" && value.episode === 7);
  allocatedEpisode = 7;
  await settle("completed");
  evidence.finalEpisode = (await status()).episode;
  await prompt("editor");

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
  await prompt("confirm");
  assert.equal((await status()).episode, 1);
  evidence.startupOrder = { waitingEpisodes: [], clearedEpisodes: [] };
  for (const reject of [false, true]) {
    const previous = await status();
    assert.equal(previous.state, "idle");
    hostBusy = true;
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
      assert.equal(input.runStartedAt, null, "UI cannot infer an undelivered run start");
      evidence.startupOrder.waitingEpisodes.push(input.episode);
      if (reject) dialog.reject(new Error("synthetic dialog rejection"));
      else dialog.resolve(true);
      await starting;
      const cleared = await waitFor((value) => value.state === "working");
      assert.equal(cleared.episode, input.episode);
      assert.ok(Number.isSafeInteger(cleared.runStartedAt));
      evidence.startupOrder.clearedEpisodes.push(cleared.episode);
      await settle("completed");
      hostBusy = false;
      assert.equal((await status()).state, "idle");
      allocatedEpisode = input.episode;
      await prompt("confirm");
      assert.equal((await status()).episode, input.episode + 1);
    } finally {
      dialog.resolve(true);
      await starting;
    }
  }

  hostBusy = true;
  fastResolution = true;
  blockedWrite = { entered: deferred(), release: deferred() };
  // Wait for hook delivery, not its blocked publication; completion is checked after release.
  promptEndDispatched = deferred();
  const starting = runner.emitBeforeAgentStart(
    { messageType: "user", prompt: "synthetic" },
    { cwd: process.cwd(), selectedTools: [] },
  );
  try {
    await blockedWrite.entered.promise;
    await promptEndDispatched.promise;
  } finally {
    blockedWrite.release.resolve();
    await starting;
    blockedWrite = undefined;
    fastResolution = false;
    promptEndDispatched = undefined;
  }
  await waitFor((value) => value.state === "working" && value.episode === 6);
  evidence.startupOrder.fastEndCleared = true;
  await settle("error");
  hostBusy = false;
  const failed = await status();
  assert.equal(failed.state, "error");
  assert.equal(failed.episode, 7);
  await prompt("confirm");
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

// Real manual-compaction dispatch and all UI kinds must restore Idle/error without consulting isIdle.
async function exerciseDialogPolicy() {
  const { AgentSession } = await installed("core/agent-session.js");
  // Only the installed getters are exercised here, not the model-backed compaction algorithm.
  const syntheticSession = Object.create(AgentSession.prototype);
  syntheticSession._isAgentRunActive = false;
  await runner.emit(event("session_start"));
  evidence.dialogPolicy = { restoredErrorEpisodes: [], idleRestored: false, overlapControls: 0 };
  for (const underlying of ["idle", "error"]) {
    if (underlying === "error") {
      await runner.emit(event("agent_start"));
      await settle("error");
    }
    const previous = await status();
    syntheticSession._compactionAbortController = new AbortController();
    assert.equal(syntheticSession.isStreaming, false);
    assert.equal(syntheticSession.isIdle, false);
    hostBusy = !syntheticSession.isIdle;
    dialog = deferred();
    promptEnded = deferred();
    const compacting = runner.emit(event("session_before_compact"));
    try {
      const input = await waitFor((value) => value.state === "input_required");
      assert.equal(input.runStartedAt, null, "manual compaction must not invent an agent run");
      assert.equal(input.lastRunDurationMs, previous.lastRunDurationMs);
      assert.ok(input.episode > previous.episode);
      dialog.resolve(true);
      assert.deepEqual(await compacting, { cancel: true });
      await promptEnded.promise;
      const restored = await status();
      assert.equal(restored.state, underlying);
      assert.equal(restored.runStartedAt, null);
      assert.equal(restored.lastRunDurationMs, previous.lastRunDurationMs);
      if (underlying === "error") {
        assert.equal(restored.episode, previous.episode);
        evidence.dialogPolicy.restoredErrorEpisodes.push(restored.episode);
      } else {
        evidence.dialogPolicy.idleRestored = true;
      }
    } finally {
      dialog.resolve(true);
      await compacting;
      syntheticSession._compactionAbortController = undefined;
      hostBusy = false;
      promptEnded = undefined;
    }
    assert.equal(syntheticSession.isIdle, true);
    allocatedEpisode = Math.max(allocatedEpisode, previous.episode) + 1;
    for (const kind of ["confirm", "select", "input", "editor", "custom"]) {
      await prompt(kind, { reject: true });
      if (underlying === "error") {
        evidence.dialogPolicy.restoredErrorEpisodes.push((await status()).episode);
      }
    }
  }
  // The actual wrapper coalesces overlapping/nested UI; either completion order must stay balanced.
  for (const outerFirst of [true, false]) {
    const previous = await status();
    const starts = evidence.uiStarts;
    const ends = evidence.uiEnds;
    const outer = deferred();
    const inner = deferred();
    dialog = outer;
    const first = runner.getUIContext().confirm("synthetic", "synthetic");
    dialog = inner;
    const second = runner.getUIContext().select("synthetic", ["synthetic"]);
    await waitFor((value) => value.state === "input_required");
    const input = await status();
    const [early, late] = outerFirst ? [outer, inner] : [inner, outer];
    early.resolve(true);
    await (outerFirst ? first : second);
    assert.equal((await status()).state, "input_required");
    assert.equal((await status()).episode, input.episode);
    promptEnded = deferred();
    late.resolve(true);
    await Promise.all([first, second]);
    await promptEnded.promise;
    promptEnded = undefined;
    assert.equal((await status()).state, previous.state);
    assert.equal((await status()).episode, previous.episode);
    assert.equal(evidence.uiStarts - starts, 1);
    assert.equal(evidence.uiEnds - ends, 1);
    evidence.dialogPolicy.overlapControls++;
  }
  assert.equal(evidence.busyMetadataReads, 0, "publisher must never use generic busy metadata");
  await runner.emit(event("session_shutdown"));
  evidence.cleanupObserved = (await fs.readdir(process.env.TRAY_GOBLIN_STATUS_DIR)).length === 0;
  assert.deepEqual(errors, []);
}

// Inverted ID-less hooks must latch a distinct fault, never restore false health or resurrect input.
async function exercisePromptOrder() {
  evidence.promptOrder = { invertedDialogs: 0, faultReasons: [], freshControls: 0 };
  for (const underlying of ["working", "error", "idle"]) {
    await runner.emit(event("session_start"));
    await runner.emit(event("agent_start"));
    if (underlying !== "working") await settle(underlying === "error" ? "error" : "completed");
    const previous = await status();
    const delayed = [];
    let entered;
    globalThis.trayGoblinTestStartObserver = () => {
      const gate = deferred();
      delayed.push(gate);
      entered.resolve();
      return gate.promise;
    };
    let faultEpisode;
    // Repeated unmatched ends must retain one fault episode, not accumulate debt or IDs.
    for (let i = 0; i < 2; i++) {
      entered = deferred();
      dialog = deferred();
      promptEnded = deferred();
      const waiting = runner.getUIContext().confirm("synthetic", "synthetic");
      await entered.promise;
      const beforeEnd = await status();
      assert.equal(beforeEnd.state, i === 0 ? underlying : "error");
      if (i > 0) {
        assert.equal(beforeEnd.attentionReason, "ui_tracking_unavailable");
        assert.equal(beforeEnd.episode, faultEpisode);
      }
      dialog.resolve(true);
      await waiting;
      await promptEnded.promise;
      const failed = await status();
      assert.equal(failed.state, "error");
      assert.equal(failed.attentionReason, "ui_tracking_unavailable");
      faultEpisode ??= previous.episode + 1;
      assert.equal(failed.episode, faultEpisode);
      evidence.promptOrder.invertedDialogs++;
    }
    // Reverse releases exercise start/start ordering independently of dialog completion order.
    for (const gate of delayed.toReversed()) {
      promptStarted = deferred();
      gate.resolve();
      await promptStarted.promise;
      const restored = await status();
      assert.equal(restored.state, "error", "late start cannot resurrect closed attention");
      assert.equal(restored.attentionReason, "ui_tracking_unavailable");
      assert.equal(restored.episode, faultEpisode, "late delivery cannot allocate a new fault");
      assert.equal(restored.runStartedAt, previous.runStartedAt);
      assert.equal(restored.lastRunDurationMs, previous.lastRunDurationMs);
    }
    delete globalThis.trayGoblinTestStartObserver;
    promptStarted = undefined;
    promptEnded = undefined;
    evidence.promptOrder.faultReasons.push((await status()).attentionReason);
    await runner.emit(event("session_shutdown"));
    await runner.emit(event("session_start"));
    assert.equal((await status()).state, "idle");
    assert.equal((await status()).attentionReason, null);
    allocatedEpisode = 0;
    await prompt("confirm");
    await prompt("input", { reject: true });
    evidence.promptOrder.freshControls += 2;
    await runner.emit(event("session_shutdown"));
  }
  assert.equal(evidence.busyMetadataReads, 0);
  assert.deepEqual(errors, []);
  await runner.emit(event("session_shutdown"));
  evidence.cleanupObserved = (await fs.readdir(process.env.TRAY_GOBLIN_STATUS_DIR)).length === 0;
}

// This recreates the review's real-wrapper old-debt/NEW-open-dialog schedule for each base state.
async function exerciseOldDebtNewDialog(underlying) {
  await runner.emit(event("session_start"));
  if (underlying !== "idle") await runner.emit(event("agent_start"));
  if (underlying === "error") await settle("error");
  const previous = await status();
  const delayed = [];
  let entered;
  globalThis.trayGoblinTestStartObserver = () => {
    const gate = deferred();
    delayed.push(gate);
    entered.resolve();
    return gate.promise;
  };
  for (let i = 0; i < 2; i++) {
    entered = deferred();
    dialog = deferred();
    promptEnded = deferred();
    const old = runner.getUIContext().confirm("synthetic old", "synthetic");
    await entered.promise;
    dialog.resolve(true);
    await old;
    await promptEnded.promise;
  }
  delete globalThis.trayGoblinTestStartObserver;
  promptStarted = deferred();
  dialog = deferred();
  const newDialog = dialog;
  const waiting = runner.getUIContext().confirm("synthetic NEW", "synthetic");
  let finished = false;
  const observed = waiting.then(() => { finished = true; });
  await promptStarted.promise;
  const faultState = await status();
  const faultEpisode = previous.episode + 1;
  function assertFault(value) {
    assert.equal(value.state, "error", "tracking fault must already be visible while NEW dialog waits");
    assert.equal(value.attentionReason, "ui_tracking_unavailable");
    assert.equal(value.episode, faultEpisode, "one stable integration-fault episode until reload");
    assert.equal(value.instanceId, previous.instanceId);
  }
  try {
    assert.equal(finished, false);
    assertFault(faultState);
    assert.equal(faultState.runStartedAt, previous.runStartedAt);
    assert.equal(faultState.lastRunDurationMs, previous.lastRunDurationMs);
    for (const gate of delayed.toReversed()) {
      promptStarted = deferred();
      gate.resolve();
      await promptStarted.promise;
      assertFault(await status());
      assert.equal(finished, false);
    }
    const beforeHeartbeat = await status();
    const heartbeat = await waitFor((value) => value.updatedAt > beforeHeartbeat.updatedAt);
    assertFault(heartbeat);
    for (const outcome of ["error", "completed", "aborted"]) {
      await runner.emit(event("agent_start"));
      assertFault(await status());
      await settle(outcome);
      assertFault(await status());
      assert.equal(finished, false);
    }
    promptEnded = deferred();
    newDialog.resolve(true);
    await observed;
    await promptEnded.promise;
    assertFault(await status());
    // Another delivered start and rejected end cannot clear or renew the fault.
    dialog = deferred();
    promptStarted = deferred();
    promptEnded = deferred();
    const rejected = runner.getUIContext().input("synthetic", "synthetic");
    const rejection = assert.rejects(rejected, /synthetic rejection/);
    await promptStarted.promise;
    assertFault(await status());
    dialog.reject(new Error("synthetic rejection"));
    await rejection;
    await promptEnded.promise;
    const closed = await status();
    assertFault(closed);
    assertFault(await waitFor((value) => value.updatedAt > closed.updatedAt));
    await runner.emit(event("session_shutdown"));
    await runner.emit(event("session_start"));
    const reloaded = await status();
    assert.notEqual(reloaded.instanceId, previous.instanceId);
    assert.equal(reloaded.state, "idle");
    assert.equal(reloaded.attentionReason, null);
    assert.equal(reloaded.episode, 0);
    allocatedEpisode = 0;
    await prompt("confirm");
    await runner.emit(event("agent_start"));
    await settle("error");
    assert.equal((await status()).attentionReason, "run_error");
    allocatedEpisode = (await status()).episode;
    await prompt("select");
    assert.equal((await status()).attentionReason, "run_error");
    evidence.oldDebtNewDialog = {
      underlying, faultEpisode, newDialogWaiting: true, stableFault: true,
      heartbeatObserved: true, reloadRecovered: true,
      state: faultState.state, attentionReason: faultState.attentionReason,
    };
  } finally {
    for (const gate of delayed) gate.resolve();
    newDialog.resolve(true);
    await observed;
    delete globalThis.trayGoblinTestStartObserver;
    promptStarted = undefined;
    promptEnded = undefined;
  }
  assert.deepEqual(errors, []);
  await runner.emit(event("session_shutdown"));
  evidence.cleanupObserved = (await fs.readdir(process.env.TRAY_GOBLIN_STATUS_DIR)).length === 0;
}

// The continuous demo reuses this exact real-host bootstrap in TWO live child
// processes. IPC drives lifecycle metadata only; UI events still originate in
// the installed runner's blocking wrapper, never hand-written protocol files.
async function exerciseAttentionPeer() {
  assert.equal(typeof process.send, "function");
  await runner.emit(event("session_start"));
  process.send({ ready: await status(), pid: process.pid, hostVersion: manifest.version });
  let waiting;
  for await (const message of peerMessages()) {
    try {
      switch (message.command) {
        case "run": await runner.emit(event("agent_start")); break;
        case "open":
          assert.equal(waiting, undefined);
          dialog = deferred();
          promptEnded = deferred();
          waiting = runner.getUIContext().confirm("SYNTHETIC_PRIVATE_TITLE", "SYNTHETIC_PRIVATE_ARGS");
          await waitFor((value) => value.state === "input_required");
          break;
        case "close":
          assert.ok(waiting);
          dialog.resolve(true);
          await waiting;
          await promptEnded.promise;
          waiting = undefined;
          promptEnded = undefined;
          break;
        case "tentative-error":
          await runner.emit(event("tool_execution_end", { isError: true }));
          await runner.emit(event("agent_end"));
          await boundary("error");
          break;
        case "complete": await settle("completed"); break;
        case "error": await settle("error"); break;
        case "abort": await settle("aborted"); break;
        case "stop":
          assert.equal(waiting, undefined);
          await runner.emit(event("session_shutdown"));
          assert.deepEqual(errors, []);
          assert.equal(evidence.sensitiveReads, 0);
          assert.equal(evidence.networkCalls, 0);
          process.send({ id: message.id, evidence, stopped: true });
          process.disconnect();
          return;
        default: assert.fail("unknown peer command");
      }
      process.send({ id: message.id, status: await status(), evidence });
    } catch (error) {
      process.send({ id: message.id, failure: String(error) });
      throw error;
    }
  }
}
async function* peerMessages() {
  const { on } = await import("node:events");
  for await (const [message] of on(process, "message")) yield message;
}

try {
  if (mode === "attention-peer") await exerciseAttentionPeer();
  else if (mode === "startup-recovery") await exerciseStartupRecovery();
  else if (mode === "startup-order") await exerciseStartupOrder();
  else if (mode === "dialog-policy") await exerciseDialogPolicy();
  else if (mode === "prompt-order") await exercisePromptOrder();
  else if (mode?.startsWith("old-debt-new-dialog-")) await exerciseOldDebtNewDialog(mode.slice("old-debt-new-dialog-".length));
  else await exerciseLifecycle();
  process.stdout.write(JSON.stringify(evidence) + "\n");
} finally {
  fault = undefined;
  for (const [name, implementation] of Object.entries(realIO)) nativeFs[name] = implementation;
  syncBuiltinESMExports();
  await runner.emit(event("session_shutdown"));
}
