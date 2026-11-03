import assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import test from "node:test";
import {
  createStatusPublisher,
  HEARTBEAT_MS,
  MAX_STATUS_BYTES,
  statusDirectory,
} from "../integrations/pi/tray-goblin.js";

const keys = [
  "version", "provider", "sessionId", "instanceId", "model", "directory",
  "state", "episode", "updatedAt", "runStartedAt", "lastRunDurationMs",
].sort();

function deferred() {
  let resolve;
  const promise = new Promise((done) => { resolve = done; });
  return { promise, resolve };
}

async function fixture(t, { expectedCloseFailure, ...options } = {}, metadata = {}) {
  const root = await fs.mkdtemp(join(tmpdir(), "traygoblin-pi-status-"));
  const reports = [];
  let clock = 10000;
  let tick;
  let cancelled = 0;
  const publisher = createStatusPublisher({
    sessionId: metadata.sessionId ?? "synthetic-session",
    model: metadata.model ?? "synthetic-model",
    directory: metadata.directory ?? "/synthetic/workspace",
  }, {
    directory: join(root, "status"),
    now: () => clock,
    report: (code) => reports.push(code),
    setInterval(callback, period) {
      assert.equal(period, 2000);
      tick = callback;
      return { unref() {} };
    },
    clearInterval() { cancelled++; },
    ...options,
  });
  t.after(async () => {
    try {
      if (expectedCloseFailure) {
        await assert.rejects(publisher.close(), {
          message: `TrayGoblin: ${expectedCloseFailure}`,
        });
      } else {
        await publisher.close();
      }
    } finally {
      await fs.rm(root, { recursive: true, force: true });
    }
  });
  return {
    publisher, root, reports,
    read: async () => JSON.parse(await fs.readFile(publisher.path, "utf8")),
    setClock: (value) => { clock = value; },
    tick: () => tick(),
    cancelled: () => cancelled,
  };
}

// Default roots are home-relative on native/WSL; overrides need no provider discovery.
test("default and explicit status roots", () => {
  assert.equal(HEARTBEAT_MS, 2000);
  assert.equal(statusDirectory({}, "/synthetic"), "/synthetic/.traygoblin/pi");
  assert.equal(statusDirectory({ WSL_DISTRO_NAME: "Test" }, "/synthetic"), "/synthetic/.traygoblin/pi");
  assert.equal(statusDirectory({ TRAY_GOBLIN_STATUS_DIR: "./synthetic-status" }), resolve("./synthetic-status"));
});

// The consumer contract is an exact allow-list, not a generic event serialization.
test("fixed bounded schema contains only approved metadata", async (t) => {
  const metadata = {
    sessionId: "s".repeat(200),
    model: "m".repeat(200),
    directory: "\u0000\u001b" + "\u{1f600}".repeat(2000),
    get messages() { throw new Error("sensitive metadata read"); },
  };
  const f = await fixture(t, {}, metadata);
  await f.publisher.start();
  const bytes = await fs.readFile(f.publisher.path);
  const status = JSON.parse(bytes);
  assert.deepEqual(Object.keys(status).sort(), keys);
  assert.ok(bytes.length <= MAX_STATUS_BYTES);
  assert.equal(status.version, 1);
  assert.equal(status.provider, "pi");
  assert.equal(status.sessionId.length, 128);
  assert.equal(status.model.length, 128);
  assert.equal(Buffer.byteLength(status.directory), 2048);
  assert.doesNotMatch(status.directory, /[\u0000-\u001f]/u);
  assert.match(status.instanceId, /^[0-9a-f-]{36}$/);
  assert.equal(status.state, "idle");
  assert.equal(status.episode, 0);
  assert.equal(status.updatedAt, 10000);
  assert.equal(status.runStartedAt, null);
  assert.equal(status.lastRunDurationMs, null);
  assert.equal((await fs.readdir(join(f.root, "status"))).length, 1);
});

// Explicit dialogs, not idle UI or tool failures, own input attention episodes.
test("nested input episodes clear on the final dialog end", async (t) => {
  const f = await fixture(t);
  await f.publisher.start();
  await f.publisher.promptStart();
  assert.equal((await f.read()).state, "idle");
  await f.publisher.beginRun();
  await f.publisher.promptStart();
  const first = await f.read();
  assert.equal(first.state, "input_required");
  assert.equal(first.episode, 1);
  await f.publisher.promptStart();
  await f.publisher.promptEnd();
  assert.equal((await f.read()).state, "input_required");
  await f.publisher.promptEnd();
  assert.equal((await f.read()).state, "working");
  await f.publisher.promptStart();
  assert.equal((await f.read()).episode, 2);
  await f.publisher.promptEnd();
});

// Content-free host busy state recognizes startup UI before this extension's run handler.
test("host busy metadata promotes early input but idle dialogs remain quiet", async (t) => {
  const f = await fixture(t);
  await f.publisher.start();
  await f.publisher.promptStart(false);
  await f.publisher.promptEnd();
  assert.equal((await f.read()).state, "idle");
  assert.equal((await f.read()).runStartedAt, null);
  assert.equal((await f.read()).episode, 0);
  await f.publisher.promptStart(true);
  const input = await f.read();
  assert.equal(input.state, "input_required");
  assert.equal(input.episode, 1);
  assert.equal(input.runStartedAt, 10000);
  f.setClock(10500);
  await f.publisher.beginRun();
  assert.equal((await f.read()).state, "input_required");
  assert.equal((await f.read()).runStartedAt, input.runStartedAt);
  await f.publisher.promptEnd();
  assert.equal((await f.read()).state, "working");
  f.publisher.beforeSettle("error");
  await f.publisher.settle(false);
  await f.publisher.promptStart(false);
  await f.publisher.promptEnd();
  assert.equal((await f.read()).state, "error");
  assert.equal((await f.read()).episode, 2);
  assert.equal((await f.read()).lastRunDurationMs, 500);
});

// Prompt state must update before awaiting I/O, or fast-end can race startup and stick waiting.
test("early prompt start and end stay synchronous across delayed publication", async (t) => {
  const entered = deferred();
  const release = deferred();
  const snapshots = [];
  let delayed = false;
  const f = await fixture(t, {
    io: {
      ...fs,
      async writeFile(path, json, options) {
        snapshots.push(JSON.parse(json).state);
        if (delayed) {
          entered.resolve();
          await release.promise;
        }
        return fs.writeFile(path, json, options);
      },
    },
  });
  await f.publisher.start();
  delayed = true;
  const starting = f.publisher.promptStart(true);
  await entered.promise;
  const ending = f.publisher.promptEnd();
  const runHandler = f.publisher.beginRun();
  release.resolve();
  await Promise.all([starting, ending, runHandler]);
  assert.deepEqual(snapshots, ["idle", "input_required", "working", "working"]);
  assert.equal((await f.read()).state, "working");
  assert.equal((await f.read()).episode, 1);
});

// Tentative errors cannot alert until settlement; new work clears persistent errors.
test("settlement, error persistence, recovery, success and abort", async (t) => {
  const f = await fixture(t);
  await f.publisher.start();
  await f.publisher.beginRun();
  f.publisher.beforeSettle("error");
  assert.equal((await f.read()).state, "working");
  f.setClock(10500);
  await f.publisher.settle(false);
  const failed = await f.read();
  assert.equal(failed.state, "error");
  assert.equal(failed.episode, 1);
  assert.equal(failed.runStartedAt, null);
  assert.equal(failed.lastRunDurationMs, 500);
  await f.publisher.settle(false);
  await f.publisher.publish();
  assert.equal((await f.read()).state, "error");
  assert.equal((await f.read()).episode, 1);
  await f.publisher.beginRun();
  assert.equal((await f.read()).state, "working");
  f.publisher.beforeSettle("error");
  await f.publisher.beginRun();
  f.publisher.beforeSettle("completed");
  await f.publisher.settle(false);
  assert.equal((await f.read()).state, "idle");
  assert.equal((await f.read()).episode, 1);
  await f.publisher.beginRun();
  f.publisher.beforeSettle("error");
  await f.publisher.settle(true);
  assert.equal((await f.read()).state, "idle");
  await f.publisher.beginRun();
  f.publisher.beforeSettle("aborted");
  await f.publisher.settle(false);
  assert.equal((await f.read()).state, "idle");
  await f.publisher.beginRun();
  f.publisher.beforeSettle("error");
  await f.publisher.settle(false);
  assert.equal((await f.read()).episode, 2);
});

// Heartbeats refresh every 2s without changing episode identity or clearing errors.
test("heartbeat stays content-free and stops idempotently at close", async (t) => {
  const f = await fixture(t);
  await f.publisher.start();
  await f.publisher.beginRun();
  f.publisher.beforeSettle("error");
  await f.publisher.settle(false);
  f.setClock(12000);
  f.tick();
  await f.publisher.publish();
  assert.equal((await f.read()).updatedAt, 12000);
  assert.equal((await f.read()).state, "error");
  assert.equal((await f.read()).episode, 1);
  const closing = f.publisher.close();
  assert.equal(closing, f.publisher.close());
  await closing;
  assert.equal(f.cancelled(), 1);
  await assert.rejects(fs.stat(f.publisher.path), { code: "ENOENT" });
  await assert.rejects(f.publisher.publish(), /publisher-closed/);
  await assert.rejects(f.publisher.beginRun(), /publisher-closed/);
  f.tick();
  assert.equal((await fs.readdir(join(f.root, "status"))).length, 0);
});

// The timer alone must retry a failed initial write, retaining its fixed diagnostic.
test("initial publication failure still installs a recoverable heartbeat", async (t) => {
  let fail = true;
  const renamed = deferred();
  const f = await fixture(t, {
    io: {
      ...fs,
      async writeFile(...args) {
        if (fail) throw new Error("SYNTHETIC_PRIVATE_STARTUP");
        return fs.writeFile(...args);
      },
      async rename(...args) {
        await fs.rename(...args);
        renamed.resolve();
      },
    },
  });
  await assert.rejects(f.publisher.start(), { message: "TrayGoblin: write-failed" });
  assert.deepEqual(f.reports, ["write-failed"]);
  fail = false;
  f.setClock(12000);
  f.tick();
  await renamed.promise;
  assert.equal((await f.read()).updatedAt, 12000);
  assert.equal((await f.read()).state, "idle");
  await assert.rejects(f.publisher.start(), /publisher-already-started/);
  await f.publisher.close();
  assert.equal(f.cancelled(), 1);
  assert.deepEqual(await fs.readdir(join(f.root, "status")), []);
});

// A pending write must finish before deletion, and queued snapshots retain ordering.
test("atomic serialized writes and pending-write shutdown barrier", async (t) => {
  const entered = deferred();
  const release = deferred();
  let delayed = false;
  let inWrite = 0;
  let maxInWrite = 0;
  const states = [];
  const operations = [];
  const io = {
    ...fs,
    async writeFile(path, json, options) {
      inWrite++;
      maxInWrite = Math.max(maxInWrite, inWrite);
      states.push(JSON.parse(json).state);
      operations.push("write");
      if (delayed) {
        entered.resolve();
        await release.promise;
      }
      await fs.writeFile(path, json, options);
      inWrite--;
    },
    async rename(from, to) {
      operations.push("rename");
      await fs.rename(from, to);
    },
    async unlink(path) {
      operations.push("delete");
      await fs.unlink(path);
    },
  };
  const f = await fixture(t, { io });
  await f.publisher.start();
  delayed = true;
  const working = f.publisher.beginRun();
  await entered.promise;
  assert.equal((await f.read()).state, "idle");
  const waiting = f.publisher.promptStart();
  for (let i = 0; i < 100; i++) f.tick();
  let closed = false;
  const closing = f.publisher.close().then(() => { closed = true; });
  await new Promise((done) => setImmediate(done));
  assert.equal(closed, false);
  await assert.rejects(f.publisher.promptEnd(), /publisher-closed/);
  release.resolve();
  await Promise.all([working, waiting, closing]);
  assert.equal(maxInWrite, 1);
  assert.deepEqual(states, ["idle", "working", "input_required"]);
  assert.deepEqual(operations, ["write", "rename", "write", "rename", "write", "rename", "delete"]);
  await assert.rejects(fs.stat(f.publisher.path), { code: "ENOENT" });
});

// Failure must preserve the previous complete file, report no raw exception, and recover.
for (const stage of ["writeFile", "rename"]) {
  test(`injected ${stage} failure is explicit, atomic and recoverable`, async (t) => {
    let fail = false;
    const f = await fixture(t, {
      io: {
        ...fs,
        async [stage](...args) {
          if (fail) throw new Error("SYNTHETIC_PRIVATE_ERROR /private/path");
          return fs[stage](...args);
        },
      },
    });
    await f.publisher.start();
    fail = true;
    const code = stage === "writeFile" ? "write-failed" : "rename-failed";
    await assert.rejects(f.publisher.beginRun(), { message: `TrayGoblin: ${code}` });
    assert.equal((await f.read()).state, "idle");
    assert.deepEqual(f.reports, [code]);
    assert.equal((await fs.readdir(join(f.root, "status"))).length, 1);
    fail = false;
    await f.publisher.publish();
    assert.equal((await f.read()).state, "working");
  });
}

// Failed cleanup leaves a stale owned file instead of claiming deletion succeeded.
test("delete rejection is reported and idempotent", async (t) => {
  let deletions = 0;
  const f = await fixture(t, {
    expectedCloseFailure: "delete-failed",
    io: {
      ...fs,
      async unlink() {
        deletions++;
        throw new Error("SYNTHETIC_PRIVATE_DELETE");
      },
    },
  });
  await f.publisher.start();
  const closing = f.publisher.close();
  await assert.rejects(closing, { message: "TrayGoblin: delete-failed" });
  assert.equal(closing, f.publisher.close());
  assert.equal(deletions, 1);
  assert.equal((await f.read()).instanceId, f.publisher.instanceId);
  assert.deepEqual(f.reports, ["delete-failed"]);
});

// A late old publisher may neither overwrite nor delete another instance's record.
test("replacement ownership protects both publish and cleanup", async (t) => {
  const f = await fixture(t);
  await f.publisher.start();
  const replacement = { ...(await f.read()), instanceId: "replacement-instance" };
  await fs.writeFile(f.publisher.path, JSON.stringify(replacement));
  await assert.rejects(f.publisher.beginRun(), /ownership-lost/);
  await f.publisher.close();
  assert.deepEqual(await f.read(), replacement);
  assert.deepEqual(f.reports, ["ownership-lost"]);
});

// Corrupt/unreadable ownership is not permission to remove an unknown file.
test("malformed and oversized replacement files survive close", async (t) => {
  for (const data of ["not-json", "x".repeat(MAX_STATUS_BYTES + 1)]) {
    const f = await fixture(t, { expectedCloseFailure: "ownership-check-failed" });
    await f.publisher.start();
    await fs.writeFile(f.publisher.path, data);
    await assert.rejects(f.publisher.close(), /ownership-check-failed/);
    assert.equal(await fs.readFile(f.publisher.path, "utf8"), data);
    assert.deepEqual(f.reports, ["ownership-check-failed"]);
  }
});

// Heartbeat failures are observed even though no event handler awaits the timer.
test("heartbeat rejection reports a fixed diagnostic and subsequent writes recover", async (t) => {
  let fail = false;
  const reported = deferred();
  const reports = [];
  const f = await fixture(t, {
    report(code) { reports.push(code); reported.resolve(); },
    io: {
      ...fs,
      async rename(...args) {
        if (fail) throw new Error("SYNTHETIC_SECRET");
        return fs.rename(...args);
      },
    },
  });
  await f.publisher.start();
  fail = true;
  f.tick();
  await reported.promise;
  fail = false;
  await f.publisher.publish();
  assert.deepEqual(reports, ["rename-failed"]);
});

// Ownership is rechecked after a delayed write so a replacement cannot be overwritten.
test("replacement during an in-flight write survives pending shutdown", async (t) => {
  const entered = deferred();
  const release = deferred();
  let delayed = false;
  const f = await fixture(t, {
    io: {
      ...fs,
      async writeFile(...args) {
        if (delayed) {
          entered.resolve();
          await release.promise;
        }
        return fs.writeFile(...args);
      },
    },
  });
  await f.publisher.start();
  const replacement = { ...(await f.read()), instanceId: "replacement-instance" };
  delayed = true;
  const writing = f.publisher.beginRun();
  await entered.promise;
  await fs.writeFile(f.publisher.path, JSON.stringify(replacement));
  const closing = f.publisher.close();
  release.resolve();
  await assert.rejects(writing, /ownership-lost/);
  await closing;
  assert.deepEqual(await f.read(), replacement);
  assert.deepEqual(f.reports, ["ownership-lost"]);
  assert.equal((await fs.readdir(join(f.root, "status"))).length, 1);
});

// Failed pending writes do not break the close barrier or strand the earlier owned file.
test("shutdown drains a rejected pending write before deleting the owned status", async (t) => {
  const entered = deferred();
  const release = deferred();
  let fail = false;
  const f = await fixture(t, {
    io: {
      ...fs,
      async writeFile(...args) {
        if (fail) {
          entered.resolve();
          await release.promise;
          throw new Error("synthetic private write failure");
        }
        return fs.writeFile(...args);
      },
    },
  });
  await f.publisher.start();
  fail = true;
  const writing = f.publisher.beginRun();
  await entered.promise;
  const closing = f.publisher.close();
  release.resolve();
  await assert.rejects(writing, /write-failed/);
  await closing;
  assert.deepEqual(await fs.readdir(join(f.root, "status")), []);
  assert.deepEqual(f.reports, ["write-failed"]);
});

// Closing during initialization must not install a late timer after the close request.
test("startup and close race cannot resurrect heartbeat resources", async (t) => {
  const entered = deferred();
  const release = deferred();
  let scheduled = 0;
  const f = await fixture(t, {
    setInterval() { scheduled++; return 0; },
    io: {
      ...fs,
      async writeFile(...args) {
        entered.resolve();
        await release.promise;
        return fs.writeFile(...args);
      },
    },
  });
  const starting = f.publisher.start();
  await entered.promise;
  const closing = f.publisher.close();
  release.resolve();
  await Promise.all([starting, closing]);
  assert.equal(scheduled, 0);
  assert.deepEqual(await fs.readdir(join(f.root, "status")), []);
});

// Short filesystem reads are valid; complete ownership records must still parse safely.
test("ownership checks support bounded short reads", async (t) => {
  const f = await fixture(t, {
    io: {
      ...fs,
      async open(...args) {
        const handle = await fs.open(...args);
        return {
          read: (buffer, offset, length, position) =>
            handle.read(buffer, offset, Math.min(length, 7), position),
          close: () => handle.close(),
        };
      },
    },
  });
  await f.publisher.start();
  await f.publisher.beginRun();
  assert.equal((await f.read()).state, "working");
  await f.publisher.close();
  assert.deepEqual(f.reports, []);
});

// Raw filesystem exception text is never trusted even when it mimics our diagnostic prefix.
test("directory and ownership failures remain content-free", async (t) => {
  for (const stage of ["mkdir", "open", "read", "close"]) {
    let fail = false;
    const io = { ...fs };
    if (stage === "mkdir" || stage === "open") {
      io[stage] = async (...args) => {
        if (fail) throw new Error("TrayGoblin: SYNTHETIC_SECRET /private/path");
        return fs[stage](...args);
      };
    } else {
      io.open = async (...args) => {
        const handle = await fs.open(...args);
        return {
          read: async (...readArgs) => {
            if (fail && stage === "read") throw new Error("SYNTHETIC_SECRET");
            return handle.read(...readArgs);
          },
          close: async () => {
            await handle.close();
            if (fail && stage === "close") throw new Error("SYNTHETIC_SECRET");
          },
        };
      };
    }
    const f = await fixture(t, { io });
    await f.publisher.start();
    fail = true;
    const code = stage === "mkdir" ? "publish-failed" : "ownership-check-failed";
    await assert.rejects(f.publisher.publish(), { message: `TrayGoblin: ${code}` });
    assert.deepEqual(f.reports, [code]);
    fail = false;
    await f.publisher.close();
  }
});

// Same-directory/same-session publishers own independent files, counters and cleanup.
test("simultaneous instances never share publication ownership", async (t) => {
  const first = await fixture(t);
  const second = await fixture(t, { directory: join(first.root, "status") });
  await Promise.all([first.publisher.start(), second.publisher.start()]);
  assert.notEqual(first.publisher.instanceId, second.publisher.instanceId);
  assert.equal((await fs.readdir(join(first.root, "status"))).length, 2);
  await first.publisher.beginRun();
  await first.publisher.promptStart();
  assert.equal((await second.read()).state, "idle");
  assert.equal((await second.read()).episode, 0);
  await first.publisher.close();
  assert.equal((await second.read()).instanceId, second.publisher.instanceId);
  await second.publisher.close();
  assert.deepEqual(await fs.readdir(join(first.root, "status")), []);
});
