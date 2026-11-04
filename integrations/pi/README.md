# Pi status publisher (opt-in)

This extension runs inside pi; it adds no process, daemon, network call, or model
request. Tested host: `@earendil-works/pi-coding-agent` **1.4.0**, Node 25.2.1.
Try it without modifying configuration:

```sh
pi --extension /absolute/path/to/integrations/pi/tray-goblin.js
```

For persistent opt-in, use the INSTALLED package, not the extracted ZIP or a
throwaway checkout. The TrayGoblin installer persists this folder and its own
README below `%LOCALAPPDATA%\Programs\TrayGoblin\integrations\pi`:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pi install "$tg\integrations\pi"
# Before uninstalling TrayGoblin:
pi remove "$tg\integrations\pi"
```

These are explicit user commands: pi itself saves/removes the local declaration.
TrayGoblin never silently edits pi settings. Local packages are referenced, not
copied by pi. Reload/restart pi after installation/removal, BEFORE deleting the
package path. Removing the extracted ZIP folder after TrayGoblin installation is
safe. Inside WSL copy this shipped folder to `$HOME/.local/share/traygoblin/pi`,
then `pi install "$HOME/.local/share/traygoblin/pi"`; remove with the SAME path.
Keep that Linux copy updated manually; do not point WSL pi at a transient Windows
Downloads folder. The installed `resources/SETUP.md` gives complete Windows/WSL setup.

On native Windows and WSL the default status directory is
`~/.traygoblin/pi` in that pi process's home. Set `TRAY_GOBLIN_STATUS_DIR` before
starting pi to override it (relative values resolve against the process working
directory). Each process/session lifetime publishes a randomly named
`<instanceId>.json`; consumers must ignore `.tmp` files. Configure a WSL consumer
source explicitly, e.g. `\\wsl.localhost\Ubuntu\home\operator\.traygoblin\pi`.
UNC reads may wake/keep WSL running; this publisher makes no no-wake guarantee.
The Windows tray consumes these sources through independent workers. Packaging
includes this integration/resources; compilation and the offline journey do NOT
prove real Windows/WSL delivery or resource budgets (M6 remains blocked).

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

## Offline gate prerequisites

For Agent validation on Linux/WSL, Node >=22.19 and the actual npm package
`@earendil-works/pi-coding-agent@1.4.0` are required, not just a `pi` executable.
**Online provisioning is a SEPARATELY authorized operator prerequisite**, never a
test or offline-gate step. Only if separately authorized and these public
prerequisites are missing:

```bash
npm install --global --ignore-scripts @earendil-works/pi-coding-agent@1.4.0
podman pull docker.io/library/debian:bookworm-slim
```

Do NOT run those commands as validation. With the already installed package and
cached image, check offline BEFORE isolating HOME (no model calls):

```bash
(
  set -euo pipefail
  export PI_CODING_AGENT_PACKAGE="$(npm root -g)/@earendil-works/pi-coding-agent"
  node -e 'const p=require(process.env.PI_CODING_AGENT_PACKAGE+"/package.json"); require("node:assert/strict").equal(p.name,"@earendil-works/pi-coding-agent"); require("node:assert/strict").equal(p.version,"1.4.0"); console.log(p.name,p.version)'
  check_home=$(mktemp -d)
  trap 'rm -rf "$check_home"' EXIT
  ( cd "$check_home"
    HOME="$check_home" USERPROFILE="$check_home" PI_CODING_AGENT_DIR="$check_home/agent" \
      PI_OFFLINE=1 PI_SKIP_VERSION_CHECK=1 PI_TELEMETRY=0 \
      node "$PI_CODING_AGENT_PACKAGE/dist/cli.js" --help
  )
  podman image exists docker.io/library/debian:bookworm-slim
  podman image inspect docker.io/library/debian:bookworm-slim --format '{{.Id}}'
  podman info --format '{{.Store.GraphRoot}} {{.Store.RunRoot}}'
)
```

Tests default to `../lib/node_modules/@earendil-works/pi-coding-agent` relative to
`process.execPath`'s directory. If npm uses another prefix, export the checked
absolute `PI_CODING_AGENT_PACKAGE` directory in the gate's calling shell; it is a
TrayGoblin test override, not a pi CLI option. Verify the manifest AND real CLI
help rather than network-installing as a test.

Record the actual public GraphRoot/RunRoot above before isolation; export them as
`TRAY_GOBLIN_PODMAN_ROOT` / `TRAY_GOBLIN_PODMAN_RUNROOT` in the gate's calling shell.
An isolated HOME otherwise selects an empty engine store, even when the image is
cached. These explicit paths select public image storage only: no credential or
user-config mounts. The gate uses image-exists and `--pull=never`, network-none,
read-only CLI execution; it fails if the cache/engine is unavailable. It never
runs online provisioning.

## Offline validation

```sh
node --test tests/pi-status.test.mjs tests/pi-host.test.mjs
```

This development command requires a source checkout, not just the portable ZIP.
Run from the repository root with the actual npm
`@earendil-works/pi-coding-agent@1.4.0` package beside Node, or set
`PI_CODING_AGENT_PACKAGE` to its absolute package directory (not the executable).
Use the [prerequisite recipe above](#offline-gate-prerequisites) for provisioning
and offline checks, separate from validation. The host test creates an
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
The continuous `bash scripts/demo-attention.sh --container podman` installs the
actual archive into an isolated root, removes extraction, then keeps two actual
pi runtimes alive in the SAME synthetic directory with different manual labels.
Their real atomic files traverse the shipped Rust worker/parser/controller and
AttentionRuntime dispatcher, with nonempty adapter requests and restoration/
quiet/dedupe assertions. Separate OCI evidence runs only the release CLI probe
nonroot/network-none/read-only in cached Debian. Neither is Windows evidence.
