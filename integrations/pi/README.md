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
The Windows tray consumes these sources through independent workers; release
packaging and real Windows/WSL delivery validation remain separate plan tasks.

## Same-repository terminal labels

Optional `TRAY_GOBLIN_LABEL` is **explicit user metadata**, not pi's automatic
session name, history or a generated prompt summary. Set it before launching pi:

```powershell
# Native Windows PowerShell; adjust the checkout path:
$env:TRAY_GOBLIN_LABEL = 'review'
pi --extension C:\src\TrayGoblin\integrations\pi\tray-goblin.js
```

```sh
# A separate terminal inside WSL, in the same repository if desired:
TRAY_GOBLIN_LABEL=build pi --extension /home/operator/TrayGoblin/integrations/pi/tray-goblin.js
```

The tray shows provider/source, this label and distinct session/instance IDs.
Labels are capped at 64 UTF-8 bytes after control removal/truncation, then trimmed;
blank/unset omits the field and retains unlabeled v1 compatibility. There is no
naming subsystem or terminal UI customization. Restart pi to change its inherited
label. WSL source roots must still be explicitly configured in the Windows tray.
Unmapped POSIX working directories cannot be opened through the Windows editor
action; use that terminal instead. No terminal focus/control or implicit path mapping.

## Status protocol v1

These are **all** the published keys. Do not infer identity from the directory.
All timestamps/durations are integer milliseconds, all numbers are nonnegative
safe JSON integers, and optional measurements use JSON `null`, not omitted keys.
The sole optional metadata field `label` is omitted when unset/blank.
String limits are UTF-8 bytes after control-character removal and truncation.

| Key | Type / bound | Meaning |
|---|---|---|
| `version` | integer `1` | Protocol version |
| `provider` | string `pi` | CLI family |
| `sessionId` | string, 128 bytes | Only `sessionManager.getSessionId()` |
| `label` | OPTIONAL string, 64 bytes | Only manually supplied `TRAY_GOBLIN_LABEL`; never generated naming/history |
| `instanceId` | UUID string, 36 ASCII bytes | Fresh per publisher lifetime |
| `model` | string, 128 bytes | Only `ctx.model.id`; empty if unknown |
| `directory` | string, 2048 bytes | Only `ctx.cwd` |
| `state` | `idle`, `working`, `input_required`, `error` | Explicit lifecycle state |
| `attentionReason` | null, `run_error`, `ui_tracking_unavailable` | Non-error states use null; final run failure or distinct integration fault |
| `episode` | integer, initially 0 | Unique allocated input/error ID; may restore an older error ID |
| `updatedAt` | integer | Unix heartbeat/publication time |
| `runStartedAt` | integer or null | Unix start of active work |
| `lastRunDurationMs` | integer or null | Most recent settled work duration |

Consumers deduplicate attention by `(instanceId, episode)`, accept at most 16 KiB
per file, and treat a heartbeat older than 10 seconds as unavailable. Heartbeats
run every **2 seconds** (coalesced while writes are pending), including Idle/error.
An initial publication failure reports a fixed diagnostic but keeps the heartbeat
active to retry after filesystem recovery, without restarting pi.
Episode allocation uses a private monotonic counter, but the published ID is
**not** a publication sequence. Closing a dialog over an error restores that
error's original, potentially lower ID. Rust consumers must accept that older ID
and deduplicate it without issuing another error notification; use instance and
`updatedAt` for freshness, never an episode high-water mark. Non-attention states
retain the last visible ID; only input/error states alert.
No completion alerts, timeout guesses, tool-error alerts, or inferred input.

Work starts at `before_agent_start` (or `agent_start`), remains active across
low-level `agent_end`/continuations, and clears the previous terminal error.
Every explicit blocking dialog creates input attention, including idle/manual,
settings, and compaction dialogs. It overlays the independently observed
Idle/Working/Error state; resolve **or reject** restores that state, including the
same underlying error ID. UI hooks never infer a run from `ctx.isIdle()` or clear
an error. A dialog opened before our run-start hook arrives overlays Idle/error
until that lifecycle hook is delivered; UI alone cannot establish work timing.
Prompt start/end update state synchronously before queued filesystem I/O.
Only the latest `agent_before_settle.outcome`, consumed at `agent_settled`, can
produce a terminal error; an aborted settlement stays quiet. Settlement updates
the underlying state without hiding an outstanding dialog.

Pi 1.4 supplies no dialog IDs or UI timestamps and emits only the outermost waiting
interval, including nested/overlapping UI. Independent hook dispatches await
earlier extension observers, which can delay a start until after its end.
An end without an observed start, or another start before the observed end,
makes tracking ambiguous. Instead of reconstructing per-dialog truth or silently
hiding a newer dialog with old event debt, the publisher latches `state=error`,
`attentionReason=ui_tracking_unavailable`, and one stable fault episode. This is
an integration monitoring error, **not** a model failure or an inferred input
request. Later UI events, new work, settlement and heartbeats do not clear or
renew it. The independently observed lifecycle state/timing is retained privately;
reload/replacement creates a fresh healthy publisher. Reload pi to reset tracking;
if the fault recurs, another extension's delayed hooks may be involved.
Coherent dialogs still restore the SAME previous run error (`run_error`) identity.
Observation latency begins at delivered hooks, not UI creation. No private host
API patch or extension load-order requirement is used. No titles, dialog arguments/results,
messages, histories, context entries, tokens, or credentials are accessed.
Unknown/unavailable model metadata is an empty string, never history-derived.

Writes are serialized: private temporary file, then atomic rename in the same
directory. At most **32** lifecycle snapshots are retained, including the executing
write. Overflow rejects publication with the fixed `write-queue-full` diagnostic;
the latest state remains in memory for recovery on a later unblocked heartbeat.
Heartbeats never add snapshots while writes are pending. One additional ordered
close barrier is reserved even at capacity. Close stops the timer and rejects later
writes, waits for pending writes, then removes only an ownership-matching file.
Reload/session replacement closes
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
`emitBeforeAgentStart` dispatch, including fast completion during delayed
publication and manual-dialog restoration controls. Another earlier extension
delays start observers until after real wrapped dialogs end, including two
outstanding starts released in reverse order. The old-debt/NEW-open-dialog
regression checks all three underlying states: the distinct fault is already
visible while the new confirmation waits, remains stable through delayed callbacks,
closes, heartbeats and lifecycle changes, and resets only on publisher replacement.
Manual-compaction event dispatch,
all five UI kinds, rejected dialogs, overlapping completion orders, Idle restore,
and exact error-ID restoration cross the real loader/runner/UI wrapper.
Compaction busy-state controls exercise installed `AgentSession` getters on a
synthetic object; they do not execute model-backed compaction.
Write/rename/delete faults and pending-write shutdown also cross that loaded
extension's real handler/runner boundary, with filesystem injection confined to
the isolated child process.
Blocked-I/O tests send more than 32 lifecycle updates, assert the exact retained
bound/overflow diagnostic, recover latest state by heartbeat, and close at capacity.
This is not model-service, interactive terminal, Windows notification, or WSL
field evidence. No user configuration, conversation, or auth data is needed.
