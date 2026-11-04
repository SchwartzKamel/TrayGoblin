# Architecture

**Audience:** Operators and Agents

TrayGoblin is one native Rust process. The Copilot session parser, monitor, and tray decision logic
are platform-neutral and host-testable; only the Win32 notification-area shell is Windows-only.
This split is the accepted decision in
[`specs/adr/0001-native-rust-tray.md`](../specs/adr/0001-native-rust-tray.md), taken to protect the
memory, CPU, privacy, and manual-release constraints in
[`specs/CONSTITUTION.md`](../specs/CONSTITUTION.md).

## Legacy MVP process model

- The existing MVP shell is one process, one UI thread. It runs a standard Win32 message loop. Its `WM_TIMER`
  wakes at the configured interval below one second and once per second for slower settings;
  `app::App` enforces the configured poll due time.
- No background service, no sidecar, no embedded browser, no network client.
- Release builds use the Windows GUI subsystem, so there is no console window; startup failures are
  reported through a content-free message box.
- Configuration is read once at startup. Changing it requires restarting TrayGoblin.

## Legacy MVP data flow

```text
Copilot CLI  ──writes──▶  %USERPROFILE%\.copilot\session-state\<session>\
                              inuse.<pid>.lock
                              workspace.yaml
                              events.jsonl
                                     │  read-only, appended bytes only
                                     ▼
                         monitor::SessionMonitor
                            selects newest active session
                            parses an allow-list of fields
                                     │
                                     ▼
                         status::StatusSnapshot   (content-free)
                                     │
                    ┌────────────────┴─────────────────┐
                    ▼                                  ▼
              app::App (decisions)            bin/tray-goblin-probe
         icon variant, tooltip, actions        one JSON snapshot, exits
                    │
                    ▼
              tray (Windows only)
        Shell_NotifyIcon, menu, WM_TIMER
```

## Module map

| Module | Responsibility | Platform |
|---|---|---|
| `src/config.rs` | JSON configuration: legacy `pollIntervalMs` (500–10,000, default 1,000) and `sessionRoot`, plus aggregate `sources`; unknown keys ignored; invalid values are typed errors | Neutral |
| `src/events.rs` | Parses one JSONL event line into an allow-listed `SessionEvent`; unknown types, unknown fields, and malformed lines are non-fatal | Neutral |
| `src/session.rs` | Parses `workspace.yaml` into repository name and active directory only, tolerating nested and aliased shapes | Neutral |
| `src/monitor.rs` | Selects the active session, caches per-file byte offsets, applies events to a turn state machine, produces a `StatusSnapshot` | Neutral |
| `src/status.rs` | The content-free snapshot type and the `Idle` / `Generating` / `Error` state enum | Neutral |
| `src/attention.rs` | Accepted M2 aggregate v1 types; unaccepted M3 notification-policy candidate | Neutral |
| `src/attention_worker.rs` | Unaccepted M3 candidate: fixed source workers, bounded mailboxes, nonblocking UI drain/stop | Neutral |
| `src/icon.rs` | Renders 32×32 tray icons from reviewable text pixel maps in `assets/`, one silhouette and palette per state | Neutral |
| `src/actions.rs` | Menu vocabulary and pure launch-target selection; typed, content-free action errors; settings-file creation | Neutral |
| `src/app.rs` | Decisions: when to poll, which icon variant, what the tooltip says, which effects the shell must perform | Neutral |
| `src/tray.rs` | Win32 window, `Shell_NotifyIcon` registration and retry, timer, context menu, process launching | Windows only |
| `src/main.rs` | Starts the tray on Windows; on other hosts prints how to use the probe instead | Both |
| `src/bin/tray-goblin-probe.rs` | Diagnostic binary that prints one snapshot as JSON using an explicit field allow-list | Neutral |

The legacy MVP dependency direction is one-way: `tray` depends on `app`, `app` depends on `monitor`,
`monitor` depends on `events`, `session`, and `status`. Nothing platform-neutral depends on
Windows.

## Aggregate attention engine

### M3 recovery checkpoint

**M3 remains in progress and unaccepted; mechanisms below await candidate-bound review.**
The [active plan](../.wgm/IMPLEMENTATION_PLAN.md) records accepted M1 `0d79111` and M2 `058ef74`
(93 workspace tests). Rejected M3 candidate `e193b6c` passed 41 targeted / 122 workspace tests
but retained **spec CHANGES-REQUESTED** after three recovery cycles.

M3.R4 reproduces that defect through synthetic files → real monitor → owned worker → the
same controller: root loss, same-lock log truncation, an observed empty recovery baseline,
then a NEW persisted error reuses an acknowledged ID and receives no request (Refresh
also stays quiet). The regression compiles and assertion-fails against exact `e193b6c`
sources; passing prior gates did not disprove it. The correction below is core evidence,
not acceptance. The subsequent R4 candidate `edec9fa` passed 49 targeted / 132
workspace tests and fresh rugged/spec checks, but quality review traced another
boundary: an incomplete session lock scan still pruned the known parser. R4.1
confirms this through the same real file → monitor → owned worker → controller path:
failed submissions lose remaining retries, exhausted errors lose Refresh, and an
unchanged acknowledged error is reclassified as new historical state. All three
regressions compile and assertion-fail on exact `edec9fa` before production edits.
The membership correction below preserves the R4 allocator and root-loss repair.
M3 is still unaccepted. M4/M5 remain pending with no attention GUI/demo; M6 actual
Windows/WSL interaction and performance remain blocked.

### Accepted aggregate path and candidate controller design

The additive attention engine is host-testable; connecting it to the Win32 shell
is a separate integration task. The legacy `SessionMonitor::poll` and default
six-field probe remain unchanged. The aggregate probe uses the synchronous
`AttentionMonitor`; the planned GUI would instead use candidate `AttentionWorkers`, subject to
M3 acceptance. That planned delta adds isolated polling, notifications and a session list; it
does not describe the existing MVP GUI.

`AttentionWorkers::new` validates immutable configured sources and allocates exactly
one owned worker per source (maximum 16). Each worker reuses an independent M2
`AttentionMonitor`, including its bounded filesystem scan, 64-live-session limit,
allow-listed pi v1 schema and independent Copilot offsets. UI `request_poll` uses
one bounded command slot per worker; a full slot coalesces polls. Completed reads
replace one latest-result slot, never queue an unbounded history. `drain_at` only
tries mailbox locks, does no filesystem I/O and never joins a thread. Results carry
monotonic request generations and the request's injected wall-clock timestamp.
Regressed generations, future timestamps and reads older than 10 seconds are
rejected; late completion cannot renew source liveness. After 10 seconds without
a fresh source result, the UI snapshot reports `SourceUnavailable` and removes
stale attention. A newer successful read clears that diagnostic independently.

Request time only bounds worker result age; it is not the event observation time.
The production monitor samples the wall clock after each bounded file read, so a
valid Copilot error appended during delayed I/O is not rejected as future and
permanently consumed. `AttentionWorkers::with_clock` injects this observation clock;
`AttentionMonitor::poll_with_clock` shares the same parsing path. Deterministic
`poll_at` callers retain their fixed-clock semantics. Tests advance synthetic
observation clocks explicitly, never substitute real wall time for synthetic time.

Pi session snapshots also carry an internal, non-serialized freshness deadline
derived from the validated publisher timestamp plus 10 seconds. Each UI drain
expires cached sessions past that inclusive deadline to `Unavailable`, removes
their attention and reports `StalePiStatus`, even while the next source read is
blocked. A recent source read cannot renew publisher liveness. This scalar costs
one optional timestamp per bounded session; it does not change the pi v1 protocol,
aggregate JSON, Copilot freshness or worker/queue counts. Fresh sibling sessions
remain visible, and the controller retains episode acknowledgement across expiry.

`stop` is a nonblocking flag, not filesystem cancellation. The worker keeps its
own ownership record and handle while a read is blocked, even if the UI owner is
dropped. No replacement thread is ever started. After the read returns, publication
is rejected if stopped and the thread exits; process termination is the only
cancellation boundary for permanently blocked OS I/O. Idle stopped workers notice
the stop flag within their 100 ms receive timeout. Configuration changes require
application restart.

`AttentionController::tick` consumes the bounded aggregate snapshot once per poll
tick. Identity is source ID + session ID + instance ID, never directory. It returns
at most one `NotificationRequest`, coalescing eligible current episodes. The adapter
must call `submitted(request_id, succeeded, now_ms)` after native submission;
request creation alone does not acknowledge delivery. Old callbacks cannot
acknowledge replacements. Failures permit at most three attempts, spaced at least
5 seconds from reported submission; exhausted attention stays in `snapshot()`.
Explicit `refresh` rearms only exhausted undelivered current episodes and does not
bypass spacing. Explorer re-registration is not a refresh.

Delivery bookkeeping retains one episode per reason per tracked session rather than
unbounded history. This preserves an acknowledged underlying error when a dialog
temporarily overlays it, including restoration of a lower episode ID. A lower
first-observed error is still eligible. Clearing/removal drops obsolete pending
requests; removal discards instance bookkeeping. Historical Copilot errors and
ordinary Idle/Working/quiet completion never produce requests. Heartbeats and
successful notification submission do not clear visible errors.
Transient `SourceUnavailable` hides attention and drops pending requests but retains
the bounded delivery slots until that source can confirm which instances remain;
recovery of an already acknowledged episode does not replay it.

Copilot observation IDs are opaque, allocated by a checked atomic process-lifetime
sequence shared by all sources and monitor instances. They are not event ordinals,
timestamps, log lengths or volatile parser counters. A controller can outlive a parser,
worker or monitor, so none of those reconstructions may recycle an ID. Only process
exit discards both the allocator and delivery history. The sequence never wraps;
theoretical exhaustion returns an unavailable session with the content-free diagnostic
`episode_identity_exhausted`, rather than risking a collision. No persisted ledger,
provider body/ID field or pi protocol change is needed.

Temporary source failure retains the existing bounded Copilot parser cache (at most
64 sessions/source), while still publishing empty, explicitly unavailable snapshots.
The same lifetime contract applies to incomplete scans: omitted/invalid candidates
are not confirmed removals. Parser pruning uses typed `SourceMembership`, independently
of the selected rows, and runs before admission so complete recovery immediately
reclaims genuinely absent entries. A positively observed replacement lock still
rebuilds that session's parser, including during an otherwise incomplete scan.

Uncertainty never expands the 64-parser bound or silently evicts known provenance.
When retained unknown entries fill it, a new candidate receives an `Unavailable` row
and the existing fixed `session_overflow` diagnostic; its log is not consumed and
it cannot claim healthy state. Complete membership recovery frees absent entries
and permits admission again. Tests exercise ten generations of blocked admissions,
same-instance recovery at capacity, positive lock replacement, and complete pruning.

An unchanged log resumes the same known episode, acknowledgement, pending-attempt
count and exhausted budget. Observed log shrink or file-identity replacement resets
state/offsets to a quiet historical baseline; a subsequent new append gets a fresh
process ID and its own delivery budget. Controller delivery policy, Refresh, membership
retention, worker ownership, publisher expiry and observation-clock sampling are unchanged.

Per-publication pi unavailability also retains slots by source and instance:
stale/malformed placeholders know the filename instance ID, not the publisher's
session ID. Such placeholders hide attention and cancel pending requests without
resetting acknowledgement or exhausted attempts. They allocate no new delivery
slots. A successful snapshot confirming removal or a different instance prunes
the old identity; recovery of the same instance and episode does not rearm it.

Visible rows are not authoritative membership. Each source result carries internal,
non-serialized `SourceMembership`: up to 4096 positive instance IDs and a scan
completeness bit. Pi membership comes from allow-listed filenames before status
validation or 64-row admission; an unreadable/stale or omitted publication therefore
still preserves its instance's delivery history. Scan overflow or enumeration/access
failure makes absent membership unknown, not deleted. Workers carry this evidence
unchanged through their single latest-result slot; cached expiry only hides attention.
The controller clears current/pending attention for hidden instances but prunes their
history only on complete absence, fresh positive replacement, or source removal from
configuration. Ordinary row overflow does not make an otherwise complete scan partial.

Unknown membership cannot cause unbounded retention: delivery history admits at most
4096 identities per source, with at most four reason slots each. At that cap it preserves
existing acknowledgement/attempt budgets, reports `delivery_history_overflow`, and
leaves new untracked attention visible without submitting it. A complete scan or positive
replacement can free slots; restarting also discards process-local history. This
conservative failure mode sacrifices new notifications under prolonged incomplete-scan
churn rather than silently replaying old ones. Visible rows remain capped at 64; pi
timestamp caches remain capped at 64, directory scanning at 4096 entries, and all
membership/history lookups are hash-based. No publisher protocol/aggregate JSON field,
dependency, worker or queue is added.

`cargo test --test attention_monitor --test attention_controller && cargo test --workspace`
exercises policy, injected-clock staleness/latency, bounded mailboxes, delayed I/O,
nonblocking stop/drop and 64 mixed synthetic sessions through the real worker and
controller entry points. This is Linux/core evidence, not Windows notifications,
UNC/WSL behavior, Windows resource measurements or new pi-host runtime evidence.

## Legacy MVP session selection

A Copilot session folder is *active* when it contains an `inuse.*.lock` marker. The monitor picks
the session whose newest lock marker has the most recent modification time, breaking ties by path
so selection is deterministic. When the selected session changes, cached offsets and turn state are
cleared and rebuilt from the new session, so re-activating an earlier session can never combine a
reset state machine with an end-of-file offset.

## Legacy MVP incremental reading

Each poll opens `events.jsonl`, seeks to the cached offset for that path, and reads only the
appended bytes.

- The offset advances only through the last complete newline, so a record that Copilot is still
  writing is retried on the next poll rather than skipped.
- If the file has shrunk since the last poll — rotation or truncation — the monitor resets the turn
  state and re-reads from the start.
- Full-file rescans after initialization are therefore avoided, which is what keeps the CPU budget
  reachable at a one-second cadence.

## Legacy MVP turn state machine

| Event | Effect |
|---|---|
| `assistant.turn_start` | Turn becomes active, start timestamp recorded, any previous failure cleared |
| `assistant.turn_end` | Turn closes; last-turn duration is the wall-clock difference when both timestamps are present |
| `session.model_change`, `assistant.message` | Newest non-empty model name is retained |
| `tool.execution_complete` with `success: false` during an active turn | Turn is marked failed |
| Anything else | Ignored |

The snapshot state is then derived: failed turn → `Error` (**Attention needed**), active turn →
`Generating` (**Working**), otherwise `Idle`. An unreadable events file for an otherwise active
session also yields `Error`, with the `StateUnavailable` reason instead of `ToolFailed`. A failure
stays visible until the next turn starts.

Internal names come from [`specs/CONTEXT.md`](../specs/CONTEXT.md); the user-visible labels are
**Idle**, **Working**, and **Attention needed**.

## Privacy boundary

The boundary is the parser, not the display layer:

- File readers temporarily buffer/scan raw bytes, including skipped unknown fields; the
  prohibition is on sensitive-field deserialization/modeling, persistence, logging and display,
  not a claim that those bytes never enter memory.
- `events.rs` and `session.rs` deserialize an allow-list of fields — event type, timestamp, model,
  success, repository, active directory. Sensitive fields are never named in any struct, so they
  cannot be held even transiently in the program's model.
- The legacy `tray-goblin-probe` serializes through its own six-field `ProbeSnapshot`;
  `--all-sessions` uses the separate [aggregate allow-list](operator/privacy.md#aggregate-probecore-metadata).
- GUI action and startup errors are typed enums whose `Display` text names the action and the missing
  prerequisite only — never a path, a session identifier, or an operating-system message.
  Probe config I/O errors can include OS text; see [Privacy](operator/privacy.md).
- `events::tests::does_not_model_sensitive_fields` is the deterministic contract that fails the
  build if this ever regresses.

## Legacy MVP failure handling

| Failure | Behaviour |
|---|---|
| No session folder exists | `Idle`, no error dialog |
| No active session | `Idle` |
| `events.jsonl` unreadable | **Attention needed**, reason `state_unavailable` |
| Malformed or unknown event line | Ignored; polling continues |
| Unknown configuration key | Ignored |
| Out-of-range or invalid configuration | Explicit startup error; TrayGoblin does not start with a surprising cadence |
| Notification-area registration fails | Retried on every timer tick and on Explorer's `TaskbarCreated` broadcast |
| A menu target cannot be launched | Content-free message added to the tooltip; the tray keeps running |

## Legacy MVP performance design

| Budget | How the design meets it |
|---|---|
| Under 50 MB working set | One native process, no webview or scripting runtime, small dependency set |
| Under 5% idle CPU | One timer tick per interval; incremental reads; the tray is redrawn only when the rendered view actually changes |
| State visible within two seconds | 1,000 ms default interval, immediate poll at startup and on **Refresh now** |

Budgets are asserted on Windows with `scripts/measure-performance.ps1`; that measurement must pass
before promoting a preview build to stable.

## Legacy MVP validation boundary

| Deterministic on Linux or any host | Manual on Windows |
|---|---|
| Parser, monitor, configuration, icon, action, and app-decision tests | Tray icon appearance and legibility |
| Fixture-driven probe journeys | Tooltip rendering and truncation in the real shell |
| Windows x86-64 cross-build producing a PE executable | Menu commands launching VS Code, the session folder, and settings |
| Reproducible packaging and checksum generation | Startup shortcut behaviour after sign-in |
| Installer and uninstaller static and sandbox checks | Working-set and CPU measurement |

See [Testing](agent/testing.md) for the exact commands on each side of that line.

## Legacy MVP deliberate non-goals

Restarting or controlling Copilot CLI, reading repository files, a settings window, desktop
notifications, themes, token or cost visualization, auto-update, code signing, MSI/MSIX, and ARM64
artifacts are all out of scope for this pass. See
[`specs/tray-status.md`](../specs/tray-status.md#out-of-scope-this-pass).

## Related

- [MVP](MVP.md) — product scope
- [Manual release](manual-release.md) — how the executable becomes a published artifact
- [Development](agent/development.md) — how to work on the code
- [Privacy](operator/privacy.md) — the operator-facing summary of the same boundary
