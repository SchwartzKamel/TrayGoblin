# Spec: Multibox session attention

> Conforms to `../../specs/CONSTITUTION.md`. Additive follow-up to the completed
> Copilot MVP. The historical root plan/spec describe that MVP; this delta governs
> new work. No prompt-content or resource-budget deviation is authorized.

## JTBD
An operator multiboxing pi and Copilot CLI on Windows and WSL wants to know which
session has an error or explicitly needs input, without watching every terminal.
Simplicity and effectiveness take priority over dashboards and remote control.

## Confirmed decisions
- One native Windows tray is the notification hub for Windows and local WSL.
- Pi uses an approved, opt-in extension inside the existing pi process. It writes
  tiny content-free local status files; no daemon, server, or telemetry collector.
- Pi: alert for explicit blocking extension UI during agent work and final run
  errors. Do not treat successful completion, ordinary Idle, manual cancellation,
  or recoverable tool errors as attention alerts.
- Copilot: alert for new persisted `session.error` events only. Input detection is
  explicitly unavailable: its permission/input/elicitation events are ephemeral
  and not present in the passive event-log boundary. Never guess from a timeout.
- One alert per attention episode; Windows notifications plus a persistent tray
  session list, attention first. No automatic terminal focus or repeated nagging.
- Identity/status only: provider, configured source label, session/instance ID,
  model, directory, state, elapsed/last-run duration where available. Optional
  manual label is user-supplied; never infer or summarize a prompt.

## User-visible success criteria
- A blocked pi session remains visible while other sessions keep working.
- Successful completion and recoverable tool failures produce zero alerts.
- Each alert identifies its source and session, including same-directory sessions.
- WSL/custom roots are explicitly configured; there is no distro discovery or
  explicit launch command. UNC access may itself start/keep a distro running:
  document that Windows behavior rather than promising atomic no-wake monitoring.
  Unavailable roots degrade visibly without freezing native monitoring.
- A slow/unavailable WSL path cannot freeze the tray or stop Windows monitoring.
- Existing single-session probe output/configuration and all regression tests keep
  their documented single-session API/probe behavior. The GUI intentionally gains
  the new aggregate view; only WSL/custom source paths require explicit opt-in.

## Magic moment
- **The whoa:** Continue working while many agents run; receive a single useful
  alert identifying the pi session waiting on approval, not every finished job.
- **Demo path:** Configure Windows and WSL sources; start concurrent pi/Copilot
  sessions; make pi show a blocking confirmation during work; identify its session
  from the notification/list; answer it; observe attention clear. Trigger a final
  pi error and a new Copilot session error. Normal completion remains quiet.
- **Smallest end-to-end slice:** The shipped pi extension, loaded by the real pi
  runtime under a synthetic offline driver, publishes a waiting transition that
  the Rust monitor/attention controller consumes with correct identity and dedupe.
- **Merely functional vs magical:** An aggregate busy/idle light or noisy
  notifications for every completion does not solve the operator's problem.

## Architecture and boundaries
- Use the existing Rust parser/config/shell modules and stdlib primitives; retain
  the original `SessionMonitor::poll`/probe path for backward compatibility.
- Add bounded configured sources (`id`, `label`, `kind`, `path`), with kinds
  `copilot` and `pi`. Preserve legacy `session_root` and polling aliases/ranges.
  Missing/legacy config uses native Copilot plus `~/.traygoblin/pi`; legacy
  `session_root`/`sessionRoot` replaces only the implicit Copilot root. Missing
  implicit pi root is normal (extension not installed), not a monitoring error.
  An explicit `sources` array replaces both defaults; an empty array disables
  monitoring. Reject duplicate IDs, unknown kinds, and invalid bounds. WSL paths use explicit
  `\\wsl.localhost\Distro\home\user\...` roots with `wsl_distro`; never traverse
  arbitrary homes. `wsl_distro` is an attribution label, not a launch mechanism.
  Poll only these explicit roots. Do not add a per-tick WSL subprocess/cache or
  promise that querying liveness makes filesystem access atomic with shutdown.
  Configured UNC reads may wake WSL; removing that source and restarting TrayGoblin
  disables those reads. M6 records the target host's actual shutdown behavior.
- Pi status protocol v1: explicit allow-listed scalars, stable session/instance
  identity, monotonic attention-episode number, heartbeat timestamp, and state.
  Max file size 16 KiB, strings bounded, filenames independent of content.
  Write atomically, serialize writes, heartbeat every 2 seconds, mark stale after
  10 seconds, clean up idempotently at shutdown/reload/session replacement.
  A stopped/crashed publisher is unavailable, not falsely Working or input-waiting.
- Observe `ui_prompt_start/end` without reading titles. Track only dialogs during
  an active run. Observe boundary `outcome` without touching messages, results,
  context entries, history, credentials, or tokens; only final settlement produces
  a terminal-error episode. Nested/dialog rejection paths must clear waiting.
- Multi-session Copilot monitoring keeps independent offsets/turn state per active
  lock-directory. Existing historical events initialize state without notification
  replay. New `session.error` records create episodes; tool failures alone do not.
- Bound sources to 16 and visible live sessions to 64 per source; overflow is a
  visible diagnostic rather than silent truncation. Test a mixed 64-session load;
  this is a stress baseline, not a measured Windows capacity/performance claim.
- Isolate filesystem polling from the UI; a blocked source cannot stall other
  sources. Use bounded channels/latest snapshots, not unbounded queues. Stale
  source results cannot overwrite newer generations or resurrect removed sessions.
- Coalesce simultaneous attention changes into at most one native notification per
  poll tick. Retain attention in the tray even if Windows suppresses notifications.
  Shell notification failure must not silently acknowledge an undelivered episode.
- Use existing Win32 notification-area APIs (balloon notifications are sufficient);
  do not add a webview, toast service, database, or Node sidecar.

## Failure/recovery contract (validation-signal scope of M3/M6)
- Source configuration is immutable for one application lifetime (settings changes
  require restart). Allocate exactly one worker per source, never replacement
  threads on stall/recovery. Repeated polls must not increase worker/queue counts.
- UI-facing snapshot drain and stop request each return within 250 ms in a
  controlled delayed-reader test. Other source publication is observed within
  1 second in that test. Retain the blocked worker's handle/ownership record until
  it exits or the whole app process terminates; never synchronously join it on UI.
  Test stop while blocked, ten stall/recovery cycles without worker growth, late
  publication after stop, and stale generations. Process exit is the cancellation
  boundary for uninterruptible filesystem reads; no in-process source replacement.
- Error clearing is explicit: a new pi run clears its previous final error; a
  new Copilot `assistant.turn_start` clears its previous session error. Normal
  Idle/heartbeat and notification submission/acknowledgement never clear errors.
  UI prompt end clears only that input episode and resumes Working if the run is
  active. A fresh successful/aborted run is Idle without attention. Session
  shutdown/removal ends visibility and drops pending alerts for that instance;
  replacement gets a distinct identity and never inherits an old error/episode.
  These conventional recovery defaults require no new operator setting. Test
  idle persistence, new-work clear, abort/success, replacement and episode renewal.
- Fresh input/error state and clearing are observed within two polling intervals
  plus 500 ms; heartbeat/source staleness clears input visibility after 10 seconds
  plus two intervals. Test with injected clock, not long wall-clock sleeps.
- Publisher close is an ordered barrier: pending writes complete before its own
  file is removed; later writes are rejected. Test injected write/rename/delete
  rejection, shutdown during a pending write, and a replacement file owned by a
  different instance. Failed deletion leaves the stale ownership-marked file and
  reports a content-free failure, never deletes another instance or claims success.
- At most three notification submissions per episode (initial plus two retries),
  separated by at least 5 seconds on failure. After exhaustion retain outstanding
  attention in the tray; explicit Refresh may rearm undelivered episodes. Drop
  obsolete/cleared pending episodes; successful submission acknowledges once and
  Explorer re-registration never rearms it. Test repeated failures then success,
  exhaustion, obsolete pending state, and coalesced mixed retry episodes.
- Windows field script parameters: `-ExecutablePath` and `-EvidencePath`. Record
  UTC time, git/build SHA, executable SHA-256, PID/path, Windows/WSL versions,
  configured source kinds and synthetic load, and per-case observed pass/fail.
  Each attention/clear transition must meet the two-interval-plus-500ms bound;
  successful completion must stay quiet for three intervals. Check suppressed
  notifications via persistent menu attention, Explorer restart without replay,
  graceful pi reload/forced exit, WSL shutdown immediately after a successful
  source read, continued native monitoring during three poll ticks, and explicit
  observation of whether UNC access wakes WSL on that host. No no-wake guarantee
  is asserted. Unavailable source diagnostics must appear after10s+2P; intentional
  WSL/source recovery is observed within2P+500ms of successful file access. Remove
  a WSL source/restart TrayGoblin and confirm no further access, then test clean quit.
  Do not auto-fill operator observations or manufacture field evidence on Linux.
- The performance script must bind an explicit `-Path` to the measured PID/path
  and SHA-256 (reject mismatched existing processes) and include that identity in
  its JSON report. Require the exact candidate digest from the field report.
- OCI check is CLI-only: `bash scripts/demo-attention.sh --container podman` must
  build the host release probe, run it non-root/network-disabled in the existing
  Debian bookworm-slim image with only read-only executable/fixture mounts, and
  assert the real JSON result. It cannot count as Windows or pi-host evidence.

## Requirements delta: acceptance → exact backpressure

| ID | EARS requirement | Command / evidence tier |
|---|---|---|
| R1 ADDED | When pi waits on a blocking extension dialog during a run, the extension shall publish a content-free input episode and clear it after resolve/reject. | `node --test tests/pi-status.test.mjs tests/pi-host.test.mjs` — real installed pi 1.4.0 loader/runner, offline |
| R2 ADDED | When pi settles after an error, the extension shall publish one error episode; successful settlement, abort, and recovered tool errors shall remain quiet. | Same node command; delayed/rejected UI and continuation cases through pinned host |
| R3 ADDED | When publishing, the extension shall write bounded atomic status, heartbeat while alive, and remove only its own file on shutdown/reload. | Same node command; sensitive-field getters throw if accessed, cleanup and concurrent-write assertions |
| R4 ADDED | While several sources/sessions are active, the monitor shall preserve independent identities/state/offsets and explicitly state provider input-detection capability. | `cargo test --workspace` — `tests/attention_monitor.rs` and config/parser tests |
| R5 MODIFIED | When a new Copilot session error is appended, the attention monitor shall create an attributed episode without replaying historical errors or alerting on failed tools alone. | `cargo test --test attention_monitor` |
| R6 ADDED | While polling repeats or simultaneous sessions require input, the controller shall deduplicate/coalesce alerts without hiding outstanding attention. | `cargo test --test attention_controller` |
| R7 ADDED | If a source stalls, disappears, publishes stale/oversized/malformed state, or recovers, the system shall remain bounded, visibly degrade, and recover independently. | `cargo test --test attention_monitor --test attention_controller` — 64-session, partial-write, delayed-source tests |
| R8 ADDED | When an eligible episode reaches the Windows adapter, the app shall construct a bounded content-free notification and attention-first session menu without blocking the window procedure. | `cargo test --workspace && cargo build --release --target x86_64-pc-windows-gnu --bin tray-goblin` — policy/runtime seam plus Windows compilation only |
| R9 MODIFIED | When legacy config/probe is used, the existing six-field content-free snapshot and MVP behavior shall remain compatible. | `bash scripts/demo.sh 0.1.0 && bash scripts/check-docs.sh` |
| R10 ADDED | When an operator follows setup, the package shall contain the pi integration and exact native/WSL setup, provider limitations, alert behavior, and troubleshooting instructions. | `bash scripts/demo-attention.sh --container podman && bash scripts/check-docs.sh` — offline package/demo checks |
| R11 ADDED | On a real Windows+WSL host, the tray shall deliver attributed attention requests, retain attention when notifications are suppressed, recover from Explorer/source restarts, and quit cleanly. | `pwsh -NoProfile -File scripts/test-attention-windows.ps1 -ExecutablePath target/x86_64-pc-windows-gnu/release/tray-goblin.exe -EvidencePath .wgm/windows-attention.json` — Windows-origin interactive field evidence; Linux is not a substitute |
| R12 UNCHANGED | Before stable promotion, the Windows process shall meet the existing 50 MB / 5% idle CPU resource gate. | `pwsh -NoProfile -File scripts/measure-performance.ps1 -Path target/x86_64-pc-windows-gnu/release/tray-goblin.exe -DurationSeconds 30 -JsonPath .wgm/windows-performance.json` — real Windows only, digest must match field report |

## Holdout scenarios
- `.wgm/scenarios/pi-attention.yaml` (tier 1), `mixed-sessions.yaml` (tier 2),
  `source-recovery.yaml` (tier 3). Implementers must not open these files.
- Preserve the original root holdouts as MVP regression journeys.

## Ruggedness
- **Actual operator/environment:** This operator runs many pi/Copilot sessions on
  native Windows and WSL and wants low-noise attention alerts. No remote collector
  or always-attentive operator is assumed.
- **Intrinsic constraint:** Passive Copilot cannot observe transient input prompts;
  declare the capability gap rather than silently pretending to support it.
- **User capacity:** Installation is an explicit pi extension plus source paths;
  provide exact examples and safe missing-source diagnostics, not hidden discovery.
- **Dominant risk — operational stress:** Concurrent errors/dialogs, stale status,
  source stalls and alert storms require deterministic lifecycle/recovery tests.
- **Simplification:** Local files + native notification/menu; accept manual source
  configuration and no automatic terminal focus in exchange for no extra service.
- **Exact checks:** R1–R10 cover deterministic core/adapter policy at their named
  origins; R11–R12 explicitly owe Windows field/performance evidence. Stopping below
  the target origin must remain recorded as unverified, never called delivery proof.
- Plan-exit verdict is recorded in `../IMPLEMENTATION_PLAN.md` and `../rugged/`.

## Defaults and exclusions
- Default polling remains 1,000 ms; valid configuration remains 500–10,000 ms.
- Notification content may be suppressed by Windows notification settings/Focus
  Assist. Outstanding attention must remain discoverable in the tray list.
- No successful-completion alerts, idle/stall heuristics, prompt/response display,
  tool argument/result capture, token/cost accounting, credentials, repository file
  contents, terminal scraping/focus/control, remote SSH, auto-update, or Linux GUI.

## Blocking implementation discovery (not an approved scope change)
M1's24-test suite passes, but independent actual-host review disproves its inferred
active-run predicate and UI dispatch-order assumptions. `ctx.isIdle()` is also
false for standalone compaction/summarization; delayed earlier UI observers can
invert start/end delivery to this extension. M1 is blocked; no acceptance test is
waived or weakened. The current stricter agent-run-only requirement remains in
force until the user decides whether all explicit blocking pi dialogs may alert,
including manual dialogs, with prior state restored after closure. This simpler
policy is recommended, but event-order recovery still needs its own real-host
counterexample/regression. See plan M1.R3 and `.wgm/rugged/M1-current.md`.
