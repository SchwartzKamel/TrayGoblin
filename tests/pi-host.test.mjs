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
  assert.equal(evidence.manualLabel, null);
  assert.equal(evidence.extensionsLoaded, 1);
  assert.equal(evidence.uiStarts, 7);
  assert.equal(evidence.uiEnds, 7);
  assert.equal(evidence.sensitiveReads, 0);
  assert.equal(evidence.networkCalls, 0);
  assert.equal(evidence.heartbeatObserved, true);
  assert.equal(evidence.replacementObserved, true);
  assert.equal(evidence.cleanupObserved, true);
  assert.equal(evidence.finalEpisode, 7);
  assert.equal(evidence.busyMetadataReads, 0);
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

// Explicit per-terminal labels traverse the actual installed loader/runner and
// all lifecycle/privacy/fault controls; blank labels preserve unlabeled v1.
for (const [label, expected] of [["  review & 😀\n\u001b " + "α".repeat(40), "review & 😀 " + "α".repeat(24)], [" \t\n", null]]) {
  test(`pi 1.4.0 publishes only bounded manual label (${expected === null ? "blank" : "named"})`, async (t) => {
    const home = await fs.mkdtemp(join(tmpdir(), "traygoblin-pi-label-"));
    t.after(() => fs.rm(home, { recursive: true, force: true }));
    const workspace = join(home, "workspace");
    await fs.mkdir(workspace);
    const packageRoot = process.env.PI_CODING_AGENT_PACKAGE ??
      resolve(dirname(process.execPath), "../lib/node_modules/@earendil-works/pi-coding-agent");
    const { stdout, stderr } = await promisify(execFile)(process.execPath, [
      resolve("scripts/test-pi-host.mjs"), packageRoot, resolve("integrations/pi/tray-goblin.js"),
    ], {
      cwd: workspace,
      env: {
        PATH: dirname(process.execPath), HOME: home, USERPROFILE: home,
        XDG_CONFIG_HOME: join(home, "config"), XDG_CACHE_HOME: join(home, "cache"),
        PI_CODING_AGENT_DIR: join(home, "agent"), TRAY_GOBLIN_STATUS_DIR: join(home, "status"),
        TRAY_GOBLIN_LABEL: label,
      }, timeout: 30000, maxBuffer: 128 * 1024,
    });
    assert.equal(stderr, "TrayGoblin: write-failed\nTrayGoblin: rename-failed\nTrayGoblin: delete-failed\n");
    const evidence = JSON.parse(stdout);
    assert.equal(evidence.hostVersion, "1.4.0");
    assert.equal(evidence.manualLabel, expected);
    assert.equal(evidence.sensitiveReads, 0);
    assert.equal(evidence.networkCalls, 0);
    assert.equal(evidence.cleanupObserved, true);
    assert.equal(evidence.replacementObserved, true);
    assert.equal(evidence.uiStarts, 7);
    assert.equal(evidence.uiEnds, 7);
    assert.deepEqual(await fs.readdir(join(home, "status")), []);
    await assert.rejects(fs.stat(join(home, "agent", "auth.json")), { code: "ENOENT" });
    await assert.rejects(fs.stat(join(home, "agent", "settings.json")), { code: "ENOENT" });
  });
}

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
  assert.deepEqual(evidence.startupOrder.waitingEpisodes, [2, 4]);
  assert.deepEqual(evidence.startupOrder.clearedEpisodes, [2, 4]);
  assert.equal(evidence.startupOrder.fastEndCleared, true);
  assert.equal(evidence.uiStarts, 7);
  assert.equal(evidence.uiEnds, 7);
  assert.equal(evidence.busyMetadataReads, 0);
  assert.equal(evidence.cleanupObserved, true);
  assert.equal(evidence.sensitiveReads, 0);
  assert.equal(evidence.networkCalls, 0);
  await assert.rejects(fs.stat(join(home, "agent", "auth.json")), { code: "ENOENT" });
  await assert.rejects(fs.stat(join(home, "agent", "settings.json")), { code: "ENOENT" });
  assert.deepEqual(await fs.readdir(join(home, "status")), []);
});

// These cross the installed loader/runner/UI wrapper, not hand-written prompt-event stubs.
for (const mode of ["dialog-policy", "prompt-order"]) {
  test(`pi 1.4.0 ${mode} preserves independently observed state and balanced attention`, async (t) => {
    const home = await fs.mkdtemp(join(tmpdir(), `traygoblin-pi-${mode}-`));
    t.after(() => fs.rm(home, { recursive: true, force: true }));
    const workspace = join(home, "workspace");
    await fs.mkdir(workspace);
    const packageRoot = process.env.PI_CODING_AGENT_PACKAGE ??
      resolve(dirname(process.execPath), "../lib/node_modules/@earendil-works/pi-coding-agent");
    const { stdout, stderr } = await promisify(execFile)(
      process.execPath,
      [resolve("scripts/test-pi-host.mjs"), packageRoot, resolve("integrations/pi/tray-goblin.js"), mode],
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
    assert.equal(evidence.uiStarts, evidence.uiEnds);
    assert.equal(evidence.busyMetadataReads, 0);
    assert.equal(evidence.sensitiveReads, 0);
    assert.equal(evidence.networkCalls, 0);
    assert.equal(evidence.cleanupObserved, true);
    if (mode === "dialog-policy") {
      assert.equal(evidence.uiStarts, 14);
      assert.equal(evidence.dialogPolicy.idleRestored, true);
      assert.deepEqual(evidence.dialogPolicy.restoredErrorEpisodes, [7, 7, 7, 7, 7, 7]);
      assert.equal(evidence.dialogPolicy.overlapControls, 2);
    } else {
      assert.equal(evidence.uiStarts, 12);
      assert.equal(evidence.promptOrder.invertedDialogs, 6);
      assert.deepEqual(evidence.promptOrder.faultReasons, Array(3).fill("ui_tracking_unavailable"));
      assert.equal(evidence.promptOrder.freshControls, 6);
    }
    await assert.rejects(fs.stat(join(home, "agent", "auth.json")), { code: "ENOENT" });
    await assert.rejects(fs.stat(join(home, "agent", "settings.json")), { code: "ENOENT" });
    assert.deepEqual(await fs.readdir(join(home, "status")), []);
  });
}

// Each base state must fail visibly before old callbacks resolve, using the installed outermost UI wrapper.
for (const underlying of ["idle", "working", "error"]) {
  test(`pi 1.4.0 old-debt NEW dialog latches a stable tracking fault over ${underlying}`, async (t) => {
    const home = await fs.mkdtemp(join(tmpdir(), "traygoblin-pi-new-dialog-"));
    t.after(() => fs.rm(home, { recursive: true, force: true }));
    const workspace = join(home, "workspace");
    await fs.mkdir(workspace);
    const packageRoot = process.env.PI_CODING_AGENT_PACKAGE ??
      resolve(dirname(process.execPath), "../lib/node_modules/@earendil-works/pi-coding-agent");
    const { stdout, stderr } = await promisify(execFile)(
      process.execPath,
      [
        resolve("scripts/test-pi-host.mjs"), packageRoot,
        resolve("integrations/pi/tray-goblin.js"), `old-debt-new-dialog-${underlying}`,
      ],
      {
        cwd: workspace,
        env: {
          PATH: dirname(process.execPath), HOME: home, USERPROFILE: home,
          XDG_CONFIG_HOME: join(home, "config"), XDG_CACHE_HOME: join(home, "cache"),
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
    assert.equal(evidence.uiStarts, 6);
    assert.equal(evidence.uiEnds, 6);
    assert.equal(evidence.sensitiveReads, 0);
    assert.equal(evidence.busyMetadataReads, 0);
    assert.equal(evidence.networkCalls, 0);
    assert.equal(evidence.cleanupObserved, true);
    assert.deepEqual(evidence.oldDebtNewDialog, {
      underlying, faultEpisode: underlying === "error" ? 2 : 1,
      newDialogWaiting: true, stableFault: true,
      heartbeatObserved: true, reloadRecovered: true,
      state: "error", attentionReason: "ui_tracking_unavailable",
    });
    await assert.rejects(fs.stat(join(home, "agent", "auth.json")), { code: "ENOENT" });
    await assert.rejects(fs.stat(join(home, "agent", "settings.json")), { code: "ENOENT" });
    assert.deepEqual(await fs.readdir(join(home, "status")), []);
  });
}
