import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import * as fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { promisify } from "node:util";
import test from "node:test";

// This crosses the installed loader/runner boundary under an isolated, credential-free HOME.
test("pi 1.4.0 runtime-loads the shipped publisher with real blocking UI wrapping", async (t) => {
  const home = await fs.mkdtemp(join(tmpdir(), "traygoblin-pi-host-"));
  t.after(() => fs.rm(home, { recursive: true, force: true }));
  const workspace = join(home, "workspace");
  await fs.mkdir(workspace);
  const packageRoot = process.env.PI_CODING_AGENT_PACKAGE ??
    resolve(dirname(process.execPath), "../lib/node_modules/@earendil-works/pi-coding-agent");
  const { stdout, stderr } = await promisify(execFile)(
    process.execPath,
    [resolve("scripts/test-pi-host.mjs"), packageRoot, resolve("integrations/pi/tray-goblin.js")],
    {
      cwd: workspace,
      env: {
        PATH: dirname(process.execPath),
        HOME: home,
        USERPROFILE: home,
        XDG_CONFIG_HOME: join(home, "config"),
        XDG_CACHE_HOME: join(home, "cache"),
        PI_CODING_AGENT_DIR: join(home, "agent"),
        TRAY_GOBLIN_STATUS_DIR: join(home, "status"),
      },
      timeout: 30000,
      maxBuffer: 128 * 1024,
    },
  );
  assert.equal(stderr, [
    "TrayGoblin: write-failed",
    "TrayGoblin: rename-failed",
    "TrayGoblin: delete-failed",
    "",
  ].join("\n"));
  const evidence = JSON.parse(stdout);
  assert.equal(evidence.hostVersion, "1.4.0");
  assert.equal(evidence.extensionsLoaded, 1);
  assert.equal(evidence.uiStarts, 7);
  assert.equal(evidence.uiEnds, 7);
  assert.equal(evidence.sensitiveReads, 0);
  assert.equal(evidence.networkCalls, 0);
  assert.equal(evidence.heartbeatObserved, true);
  assert.equal(evidence.replacementObserved, true);
  assert.equal(evidence.cleanupObserved, true);
  assert.equal(evidence.finalEpisode, 5);
  assert.deepEqual(evidence.ioFailures, [
    "TrayGoblin: write-failed", "TrayGoblin: rename-failed", "TrayGoblin: delete-failed",
  ]);
  assert.equal(evidence.pendingShutdownObserved, true);
  assert.ok(evidence.sessionIdReads > 0);
  assert.ok(evidence.modelIdReads > 0);
  await assert.rejects(fs.stat(join(home, "agent", "auth.json")), { code: "ENOENT" });
  await assert.rejects(fs.stat(join(home, "agent", "settings.json")), { code: "ENOENT" });
  assert.deepEqual(await fs.readdir(join(home, "status")), []);
});

// Transient startup failure must not make real wrapped input status go stale after recovery.
test("pi 1.4.0 heartbeat recovers after initial publication failure without restart", async (t) => {
  const home = await fs.mkdtemp(join(tmpdir(), "traygoblin-pi-startup-"));
  t.after(() => fs.rm(home, { recursive: true, force: true }));
  const workspace = join(home, "workspace");
  await fs.mkdir(workspace);
  const packageRoot = process.env.PI_CODING_AGENT_PACKAGE ??
    resolve(dirname(process.execPath), "../lib/node_modules/@earendil-works/pi-coding-agent");
  const { stdout, stderr } = await promisify(execFile)(
    process.execPath,
    [resolve("scripts/test-pi-host.mjs"), packageRoot, resolve("integrations/pi/tray-goblin.js"), "startup-recovery"],
    {
      cwd: workspace,
      env: {
        PATH: dirname(process.execPath),
        HOME: home,
        USERPROFILE: home,
        XDG_CONFIG_HOME: join(home, "config"),
        XDG_CACHE_HOME: join(home, "cache"),
        PI_CODING_AGENT_DIR: join(home, "agent"),
        TRAY_GOBLIN_STATUS_DIR: join(home, "status"),
      },
      timeout: 15000,
      maxBuffer: 128 * 1024,
    },
  );
  assert.equal(stderr, "TrayGoblin: write-failed\n");
  const evidence = JSON.parse(stdout);
  assert.equal(evidence.hostVersion, "1.4.0");
  assert.equal(evidence.extensionsLoaded, 1);
  assert.equal(evidence.startupRecovery.initialError, "TrayGoblin: write-failed");
  assert.ok(evidence.startupRecovery.inputAt >= evidence.startupRecovery.recoveredAt);
  assert.ok(evidence.startupRecovery.heartbeatAt > evidence.startupRecovery.inputAt);
  assert.equal(evidence.startupRecovery.episode, 1);
  assert.equal(evidence.uiStarts, 1);
  assert.equal(evidence.uiEnds, 1);
  assert.equal(evidence.heartbeatObserved, true);
  assert.equal(evidence.cleanupObserved, true);
  assert.equal(evidence.sensitiveReads, 0);
  assert.equal(evidence.networkCalls, 0);
  await assert.rejects(fs.stat(join(home, "agent", "auth.json")), { code: "ENOENT" });
  await assert.rejects(fs.stat(join(home, "agent", "settings.json")), { code: "ENOENT" });
  assert.deepEqual(await fs.readdir(join(home, "status")), []);
});

// Real sequential startup dispatch must observe an earlier extension's UI, including fast-end races.
test("pi 1.4.0 earlier startup handler input is load-order independent and clears synchronously", async (t) => {
  const home = await fs.mkdtemp(join(tmpdir(), "traygoblin-pi-order-"));
  t.after(() => fs.rm(home, { recursive: true, force: true }));
  const workspace = join(home, "workspace");
  await fs.mkdir(workspace);
  const packageRoot = process.env.PI_CODING_AGENT_PACKAGE ??
    resolve(dirname(process.execPath), "../lib/node_modules/@earendil-works/pi-coding-agent");
  const { stdout, stderr } = await promisify(execFile)(
    process.execPath,
    [resolve("scripts/test-pi-host.mjs"), packageRoot, resolve("integrations/pi/tray-goblin.js"), "startup-order"],
    {
      cwd: workspace,
      env: {
        PATH: dirname(process.execPath),
        HOME: home,
        USERPROFILE: home,
        XDG_CONFIG_HOME: join(home, "config"),
        XDG_CACHE_HOME: join(home, "cache"),
        PI_CODING_AGENT_DIR: join(home, "agent"),
        TRAY_GOBLIN_STATUS_DIR: join(home, "status"),
      },
      timeout: 15000,
      maxBuffer: 128 * 1024,
    },
  );
  assert.equal(stderr, "");
  const evidence = JSON.parse(stdout);
  assert.equal(evidence.hostVersion, "1.4.0");
  assert.equal(evidence.extensionsLoaded, 2);
  assert.deepEqual(evidence.startupOrder.waitingEpisodes, [1, 2]);
  assert.deepEqual(evidence.startupOrder.clearedEpisodes, [1, 2]);
  assert.equal(evidence.startupOrder.fastEndCleared, true);
  assert.equal(evidence.uiStarts, 7);
  assert.equal(evidence.uiEnds, 7);
  assert.ok(evidence.busyMetadataReads > 2);
  assert.equal(evidence.cleanupObserved, true);
  assert.equal(evidence.sensitiveReads, 0);
  assert.equal(evidence.networkCalls, 0);
  await assert.rejects(fs.stat(join(home, "agent", "auth.json")), { code: "ENOENT" });
  await assert.rejects(fs.stat(join(home, "agent", "settings.json")), { code: "ENOENT" });
  assert.deepEqual(await fs.readdir(join(home, "status")), []);
});
