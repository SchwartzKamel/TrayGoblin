import * as fs from "node:fs/promises";
import { randomUUID } from "node:crypto";
import { homedir } from "node:os";
import { join, resolve } from "node:path";

export const HEARTBEAT_MS = 2000;
export const MAX_STATUS_BYTES = 16 * 1024;

class StatusError extends Error {}

export function statusDirectory(env = process.env, home = homedir()) {
  return env.TRAY_GOBLIN_STATUS_DIR
    ? resolve(env.TRAY_GOBLIN_STATUS_DIR)
    : join(home, ".traygoblin", "pi");
}

function bounded(value, maxBytes) {
  if (typeof value !== "string") {
    throw new Error("TrayGoblin: invalid metadata");
  }
  let result = "";
  let bytes = 0;
  for (const char of value) {
    if (/[\u0000-\u001f\u007f-\u009f]/u.test(char)) continue;
    bytes += Buffer.byteLength(char);
    if (bytes > maxBytes) break;
    result += char;
  }
  return result;
}

// Dependencies are injectable only for deterministic filesystem/clock fault tests.
export function createStatusPublisher(metadata, options = {}) {
  const io = options.io ?? fs;
  const now = options.now ?? Date.now;
  const schedule = options.setInterval ?? setInterval;
  const cancel = options.clearInterval ?? clearInterval;
  const report =
    options.report ??
    ((code) => process.stderr.write(`TrayGoblin: ${code}\n`));
  const directory = options.directory ?? statusDirectory();
  const instanceId = randomUUID();
  const path = join(directory, `${instanceId}.json`);
  const temporary = join(directory, `${instanceId}.tmp`);
  const status = {
    version: 1,
    provider: "pi",
    sessionId: bounded(metadata.sessionId, 128),
    instanceId,
    model: bounded(metadata.model, 128),
    directory: bounded(metadata.directory, 2048),
    state: "idle",
    episode: 0,
    updatedAt: 0,
    runStartedAt: null,
    lastRunDurationMs: null,
  };
  let active = false;
  let prompts = 0;
  let outcome;
  let tail = Promise.resolve();
  let pending = 0;
  let closing = false;
  let closePromise;
  let timer;

  function failure(code) {
    report(code);
    return new StatusError(`TrayGoblin: ${code}`);
  }

  function timestamp() {
    const value = now();
    if (!Number.isSafeInteger(value) || value < 0) throw failure("invalid-clock");
    return value;
  }

  function enqueue(operation) {
    pending++;
    const result = tail.then(operation).finally(() => pending--);
    // Keep the serialization barrier usable after a reported failed operation.
    tail = result.catch(() => {});
    return result;
  }

  async function ownsFile() {
    let handle;
    try {
      handle = await io.open(path, "r");
      const buffer = Buffer.alloc(MAX_STATUS_BYTES + 1);
      let bytesRead = 0;
      while (bytesRead < buffer.length) {
        const chunk = await handle.read(
          buffer, bytesRead, buffer.length - bytesRead, bytesRead,
        );
        if (chunk.bytesRead === 0) break;
        bytesRead += chunk.bytesRead;
      }
      if (bytesRead > MAX_STATUS_BYTES) {
        throw new Error("oversized ownership record");
      }
      const owner = JSON.parse(buffer.toString("utf8", 0, bytesRead));
      return owner.instanceId === instanceId;
    } catch (error) {
      if (error.code === "ENOENT") return null;
      throw failure("ownership-check-failed");
    } finally {
      if (handle) {
        try {
          await handle.close();
        } catch {
          throw failure("ownership-check-failed");
        }
      }
    }
  }

  function publish() {
    if (closing) return Promise.reject(failure("publisher-closed"));
    try {
      status.updatedAt = timestamp();
    } catch (error) {
      return Promise.reject(error);
    }
    const json = JSON.stringify(status) + "\n";
    if (Buffer.byteLength(json) > MAX_STATUS_BYTES) {
      return Promise.reject(failure("status-too-large"));
    }
    return enqueue(async () => {
      let temporaryCreated = false;
      try {
        await io.mkdir(directory, { recursive: true, mode: 0o700 });
        if ((await ownsFile()) === false) {
          throw failure("ownership-lost");
        }
        temporaryCreated = true;
        try {
          await io.writeFile(temporary, json, { mode: 0o600, flag: "wx" });
        } catch {
          throw failure("write-failed");
        }
        if ((await ownsFile()) === false) {
          throw failure("ownership-lost");
        }
        try {
          await io.rename(temporary, path);
          temporaryCreated = false;
        } catch {
          throw failure("rename-failed");
        }
      } catch (error) {
        if (error instanceof StatusError) throw error;
        throw failure("publish-failed");
      } finally {
        if (temporaryCreated) {
          try {
            await io.unlink(temporary);
          } catch (error) {
            if (error.code !== "ENOENT") throw failure("temporary-delete-failed");
          }
        }
      }
    });
  }

  function update(change) {
    if (closing) return Promise.reject(failure("publisher-closed"));
    change();
    return publish();
  }

  function attention(state) {
    if (status.state !== state) {
      if (status.episode === Number.MAX_SAFE_INTEGER) {
        throw failure("episode-exhausted");
      }
      status.episode++;
      status.state = state;
    }
  }

  function beginRunState() {
    if (!active) {
      active = true;
      status.runStartedAt = timestamp();
      status.state = prompts ? "input_required" : "working";
    }
    outcome = undefined;
  }

  return {
    path,
    instanceId,
    publish,
    async start() {
      if (timer !== undefined || closing) {
        throw failure("publisher-already-started");
      }
      try {
        await publish();
      } finally {
        // Initial I/O failure must not disable later heartbeat recovery.
        if (!closing && timer === undefined) {
          timer = schedule(() => {
            // A slow filesystem must not accumulate heartbeat writes.
            if (!pending && !closing) void publish().catch(() => {});
          }, HEARTBEAT_MS);
          timer.unref?.();
        }
      }
    },
    beginRun() {
      return update(beginRunState);
    },
    beforeSettle(value) {
      if (closing) throw failure("publisher-closed");
      if (!["completed", "aborted", "error"].includes(value)) {
        throw failure("invalid-outcome");
      }
      outcome = value;
    },
    settle(aborted) {
      return update(() => {
        if (!active) return;
        active = false;
        prompts = 0;
        status.lastRunDurationMs = Math.max(0, timestamp() - status.runStartedAt);
        status.runStartedAt = null;
        if (!aborted && outcome === "error") attention("error");
        else status.state = "idle";
        outcome = undefined;
      });
    },
    promptStart(hostBusy = false) {
      return update(() => {
        // Earlier startup handlers can open UI before our own run-start handler.
        if (!active && hostBusy) beginRunState();
        if (active && ++prompts === 1) attention("input_required");
      });
    },
    promptEnd() {
      return update(() => {
        if (active && prompts > 0 && --prompts === 0) status.state = "working";
      });
    },
    setModel(model) {
      return update(() => {
        status.model = bounded(model, 128);
      });
    },
    close() {
      if (closePromise) return closePromise;
      closing = true;
      if (timer !== undefined) cancel(timer);
      timer = undefined;
      closePromise = enqueue(async () => {
        if ((await ownsFile()) === true) {
          try {
            await io.unlink(path);
          } catch (error) {
            if (error.code !== "ENOENT") throw failure("delete-failed");
          }
        }
      });
      return closePromise;
    },
  };
}

export default function trayGoblin(pi) {
  let publisher;
  pi.on("session_start", async (_event, ctx) => {
    if (publisher) await publisher.close();
    publisher = createStatusPublisher({
      sessionId: ctx.sessionManager.getSessionId(),
      model: ctx.model?.id ?? "",
      directory: ctx.cwd,
    });
    await publisher.start();
  });
  pi.on("before_agent_start", () => publisher?.beginRun());
  pi.on("agent_start", () => publisher?.beginRun());
  pi.on("agent_before_settle", (event) => {
    publisher?.beforeSettle(event.outcome);
  });
  pi.on("agent_settled", (event) => publisher?.settle(event.aborted));
  pi.on("ui_prompt_start", (_event, ctx) =>
    publisher?.promptStart(!ctx.isIdle()),
  );
  pi.on("ui_prompt_end", () => publisher?.promptEnd());
  pi.on("model_select", (_event, ctx) =>
    publisher?.setModel(ctx.model?.id ?? ""),
  );
  pi.on("session_shutdown", async () => {
    const previous = publisher;
    publisher = undefined;
    await previous?.close();
  });
}
