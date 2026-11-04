# Status reference

**Audience:** Operators

## One aggregate Windows tray

Color is always paired with text and a distinct silhouette.

| Tray state | Meaning |
|---|---|
| **Idle** (green) | No observed session is working or needs attention; can include no sessions |
| **Working** (amber) | At least one session is working, with no attention/unavailable diagnostics |
| **Attention needed** (red) | Explicit input, a session error, unavailable status or a source/monitor diagnostic |

A diagnostic changes the icon/list but does **not** invent an input request or
send an error balloon. The tooltip summarizes counts as **A** attention, **W** working, **I** idle,
**U** unavailable and **D** diagnostics (also explained in the menu). A bounded identity hint identifies the first
attention row; all rows and separate identifier fields remain in the menu. Text is bounded
by UTF-16 units (127 for tooltip, 63 for balloon title, 255 for body), always
NUL-terminated at the Win32 boundary and never cut inside a surrogate pair.
Menu ampersands are literal; controls cannot inject accelerators or columns.

## Persistent sessions and capabilities

All configured sources are observed independently: up to 16 sources, 64 live
rows per source. Capacity/scan limits degrade visibly rather than silently
claiming completeness. Sessions are sorted attention first, then stable source,
session and instance IDs. Same-directory sessions remain separate.

Each row identifies provider, configured source label, optional **manual** pi
label, abbreviated session/instance IDs and state. Its submenu retains separate IDs,
label/model/directory and available run-start (UNIX milliseconds) and last-run
milliseconds. Long metadata is deliberately bounded for display; directory
length never displaces the row's identity. No prompt-derived name is used.

| Session state | Meaning/recovery |
|---|---|
| **Idle** | Ordinary idle, completion or abort; quiet |
| **Working** | Explicit run/turn lifecycle; not a timeout guess |
| **Input required** | Any explicit blocking pi dialog, including manual/settings/compaction; closing restores the underlying state |
| **Session error** | Final pi failure or persisted Copilot session error; new work clears it, not heartbeat or notification acknowledgement |
| **UI tracking fault (reload pi)** | Ambiguous pi dialog hooks; stable integration-error episode until publisher replacement/reload, not a model error |
| **Unavailable** | Expired publisher or unreadable source/session; never asserted to be waiting for input |

Source submenus explicitly report **pi input: explicit dialogs** versus
**Copilot input: unavailable**, session-error capability, unavailable pi files,
WSL attribution when configured and safe diagnostics. Diagnostics cover source
access, malformed/oversized/stale/future/regressed status, unsafe entries,
capacity, event/workspace parsing and identity exhaustion. They contain no raw
provider text. Missing implicit pi roots are normal when the extension is absent.

## Notification policy

- Pi alerts for explicit blocking dialogs and final errors (including the
  distinctly named UI tracking fault). Copilot alerts only for **new persisted
  `session.error`** events. Historical Copilot errors initialize quietly.
- Successful completion, ordinary Idle, cancellation and recovered tool failures
  create **zero** completion alerts. Passive Copilot cannot observe input dialogs.
- One coalesced native balloon per UI tick, with the first attributed session and
  total affected count. Right-click for every outstanding session.
- At most three submissions per episode, failures separated by at least five
  seconds from the **actual adapter completion**. Failed submission never
  acknowledges an episode. **Refresh now** may rearm only undelivered exhausted
  episodes, retaining that spacing. Pending attention stays visible.
- A successful shell return acknowledges **submission only**, not OS delivery.
  Windows notification settings/Focus Assist may suppress the display.
- Explorer icon re-registration retains the same workers/controller and delivery
  history. It neither replays acknowledged errors nor resets exhausted budgets.
- A dialog over an existing pi error restores the SAME error ID on close; lower
  restored IDs are not stale timestamps and never cause a duplicate alert.

## Timing and clocks

Default polling is 1,000 ms (valid range 500–10,000 ms). Fresh attention/clearing
is expected within two polling intervals plus 500 ms after the relevant lifecycle
hook is delivered. Pi publishes a coalesced heartbeat every two seconds; after
10 seconds without a current heartbeat, the publisher becomes unavailable (plus
up to two polling intervals to observe it). Source stalls cannot refresh cached
publisher liveness by completing an old read late.

UI cadence uses elapsed/uptime milliseconds. Workers, parser freshness, controller
and submission completion use **UNIX epoch milliseconds**. They are separate
clock domains. Clock skew is visible, not interpreted as an input request.
**Refresh now** requests/drains snapshots only; it performs no provider-file I/O
and does not join source workers. Startup settings load and explicit settings/
launch actions remain separate operations.

## Actions and privacy

See [First run](first-run.md#right-click-menu). Session selection only chooses the
existing editor action's target, never controls/focuses a terminal. Unmapped
POSIX/WSL directories deliberately fail safely; no automatic path translation.
**View Copilot logs** is always a Copilot source path, never a pi conversation.

Prompt/response content, dialog titles/arguments/results, tool arguments/results,
tokens, credentials and repository file contents are never modeled or displayed.
`TRAY_GOBLIN_LABEL` is explicitly supplied by the operator, at most 64 UTF-8 bytes;
unset/blank preserves unlabeled protocol v1. See the
[pi integration](../../integrations/pi/README.md).

## Legacy diagnostic probe

The existing six-field single-session probe is unchanged and intentionally retains
the MVP selected-session/tool-error view. It is **not** the aggregate GUI policy:

```bash
cargo run --bin tray-goblin-probe -- --session-root tests/fixtures/live-session
```

Use `--all-sessions` with explicitly configured synthetic/authorized sources for
the aggregate view (see [Configuration](configuration.md)). Windows compilation
and host policy tests are not real Windows notification/WSL or resource evidence.
The real field/resource gates remain required before release readiness.
