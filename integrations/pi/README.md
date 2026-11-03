# Pi status publisher (opt-in)

This extension runs inside pi; it adds no process, daemon, network call, or model
request. Tested host: `@earendil-works/pi-coding-agent` **1.4.0**, Node 25.2.1.
Try it without modifying configuration:

```sh
pi --extension /absolute/path/to/integrations/pi/tray-goblin.js
```

For a persistent opt-in installation, `pi install /absolute/path/to/integrations/pi`
adds this local package to pi's personal settings. Remove it with
`pi remove /absolute/path/to/integrations/pi`. Reload/restart pi after changing
extensions. The integration does not edit settings itself.

On native Windows and WSL the default status directory is
`~/.traygoblin/pi` in that pi process's home. Set `TRAY_GOBLIN_STATUS_DIR` before
starting pi to override it (relative values resolve against the process working
directory). Each process/session lifetime publishes a randomly named
`<instanceId>.json`; consumers must ignore `.tmp` files. Configure a WSL consumer
source explicitly, e.g. `\\wsl.localhost\Ubuntu\home\operator\.traygoblin\pi`.
UNC reads may wake/keep WSL running; this publisher makes no no-wake guarantee.
Consumer integration and release packaging are subsequent plan tasks.

## Status protocol v1

These are **all** the published keys. Do not infer identity from the directory.
All timestamps/durations are integer milliseconds, all numbers are nonnegative
safe JSON integers, and optional measurements use JSON `null`, not omitted keys.
String limits are UTF-8 bytes after control-character removal and truncation.

| Key | Type / bound | Meaning |
|---|---|---|
| `version` | integer `1` | Protocol version |
| `provider` | string `pi` | CLI family |
| `sessionId` | string, 128 bytes | Only `sessionManager.getSessionId()` |
| `instanceId` | UUID string, 36 ASCII bytes | Fresh per publisher lifetime |
| `model` | string, 128 bytes | Only `ctx.model.id`; empty if unknown |
| `directory` | string, 2048 bytes | Only `ctx.cwd` |
| `state` | `idle`, `working`, `input_required`, `error` | Explicit lifecycle state |
| `episode` | integer, initially 0 | Increment on each new input/error episode |
| `updatedAt` | integer | Unix heartbeat/publication time |
| `runStartedAt` | integer or null | Unix start of active work |
| `lastRunDurationMs` | integer or null | Most recent settled work duration |

Consumers deduplicate attention by `(instanceId, episode)`, accept at most 16 KiB
per file, and treat a heartbeat older than 10 seconds as unavailable. Heartbeats
run every **2 seconds** (coalesced while writes are pending), including Idle/error.
An initial publication failure reports a fixed diagnostic but keeps the heartbeat
active to retry after filesystem recovery, without restarting pi.
`episode` persists through non-attention states; only input/error states alert.
No completion alerts, timeout guesses, tool-error alerts, or inferred input.

Work starts at `before_agent_start` (or `agent_start`), remains active across
low-level `agent_end`/continuations, and clears the previous terminal error.
For dialogs opened by earlier startup handlers, content-free `ctx.isIdle()`
metadata recognizes the already-busy host without depending on extension load
order. Prompt start/end update state synchronously before queued filesystem I/O,
so even an immediately finished dialog cannot leave input attention stuck.
Only the latest `agent_before_settle.outcome`, consumed at `agent_settled`, can
produce a terminal error; an aborted settlement stays quiet. Dialog events during
work create input episodes; the outer dialog ending (resolve **or reject**)
clears waiting. Idle dialogs are ignored. No titles, dialog arguments/results,
messages, histories, context entries, tokens, or credentials are accessed.
Unknown/unavailable model metadata is an empty string, never history-derived.

Writes are serialized: private temporary file, then atomic rename in the same
directory. Close stops the timer and rejects later writes, waits for pending writes,
then removes only an ownership-matching file. Reload/session replacement closes
the old publisher before starting a new random instance. A failed deletion leaves
the ownership-marked file to go stale and emits a fixed content-free diagnostic;
errors never include exception text, paths, or payloads. A file already replaced
by another instance is preserved. Random per-instance paths avoid legitimate
cross-process write/delete races; this is not a defense against a hostile process
modifying the same file between ownership check and filesystem operation.
Forced process termination can leave stale `.json`/`.tmp` files; no global sweep
or deletion of other instances is attempted.

## Offline validation

```sh
node --test tests/pi-status.test.mjs tests/pi-host.test.mjs
```

Run from the repository root with pi 1.4.0 installed beside the Node installation,
or set `PI_CODING_AGENT_PACKAGE` to its package directory. The host test creates an
isolated HOME/agent directory and runtime-loads this exact extension using pi's
loader and `ExtensionRunner`. Synthetic delayed/resolved/rejected/nested UI goes
through the runner's real UI wrapper. Lifecycle/outcome events are driven offline.
An earlier runtime-loaded extension awaits confirmation through actual
`emitBeforeAgentStart` dispatch with busy context, including fast completion
during delayed publication and idle controls.
Write/rename/delete faults and pending-write shutdown also cross that loaded
extension's real handler/runner boundary, with filesystem injection confined to
the isolated child process.
This is not model-service, interactive terminal, Windows notification, or WSL
field evidence. No user configuration, conversation, or auth data is needed.
