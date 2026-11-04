# TrayGoblin local attention setup

Windows 10/11 x86-64, standard user. This archive is UNPUBLISHED implementation
validation material, not proof of Windows delivery or release readiness. It is
unsigned and has no updater. No daemon, server, dashboard or Node sidecar.

## Install and keep resources

Verify the ZIP with `Get-FileHash -Algorithm SHA256` against the sibling `.sha256`
file BEFORE extraction. In the extracted folder:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
```

The installer writes its ownership marker before copying files. It copies the
executable, installer/uninstaller, LICENSE, README.txt, `integrations/pi/` and
`resources/` to `%LOCALAPPDATA%\Programs\TrayGoblin`, creates a per-user Startup
shortcut and launches the tray. Reinstall repairs/replaces that installation,
keeping `%APPDATA%\TrayGoblin\config.json`. It only stops its installed executable
path, not another portable copy. `-NoLaunch` / `-NoStartup` suppress those actions.
All instructions below use the INSTALLED paths: the extracted folder may now be
removed. See [pi protocol and limitations](../integrations/pi/README.md).

## Opt in to pi explicitly

Install pi 1.4.0 independently in each environment where you run it. TrayGoblin
never edits pi settings. Local pi packages reference a path; they are not copied
by pi. Use the persistent installation path, not Downloads or a checkout:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
# Explicit personal opt-in; pi itself saves this declaration:
pi install "$tg\integrations\pi"
# In each terminal, choose a DIFFERENT manual label before launching pi:
$env:TRAY_GOBLIN_LABEL = 'review'
pi
```

For one invocation without saving settings:
`pi --extension "$tg\integrations\pi\tray-goblin.js"`.
Restart pi after install; `/reload` replaces a running extension runtime.
Labels are optional, user-supplied (64 UTF-8 bytes), never generated from content.
Session/instance IDs distinguish two terminals in the SAME directory.

Inside WSL, copy the shipped `integrations/pi` folder to a durable Linux path,
then explicitly install it there (replace the Windows mount path for your user):

```sh
mkdir -p "$HOME/.local/share/traygoblin"
cp -R /mnt/c/Users/operator/AppData/Local/Programs/TrayGoblin/integrations/pi "$HOME/.local/share/traygoblin/pi"
pi install "$HOME/.local/share/traygoblin/pi"
TRAY_GOBLIN_LABEL=build pi
```

Upgrades: replace that WSL copy from the new installed payload and reload pi.
Native pi writes `~/.traygoblin/pi` in its Windows home; WSL pi writes in its
Linux home. `TRAY_GOBLIN_STATUS_DIR` overrides the producer path. Configure the
Windows consumer to the SAME folder; there is no implicit WSL discovery.

## Configure Windows and WSL

Right-click **Open settings**, edit `%APPDATA%\TrayGoblin\config.json`, then
**Quit** and restart the installed executable. **Refresh now** does not reload
settings. The [example](config.example.json) is a template: replace `operator`
and `Ubuntu` with your own paths/distro. Do not copy non-existent WSL roots blindly.
Omitting `sources` supplies native Copilot and pi defaults. Explicit `sources`
REPLACES both defaults, so include all desired roots; `[]` disables monitoring.
Legacy `sessionRoot` overrides only implicit Copilot; `pollIntervalMs` defaults to
1000, range 500–10000. Maximum 16 sources, 64 live rows/source; overflow is visible.

UNC reads such as `\\wsl.localhost\Ubuntu\home\operator\.traygoblin\pi` may
wake/keep WSL running. No liveness subprocess or no-wake guarantee is used. A
blocked source does not block the tray/other sources. To stop WSL reads, remove
that source, quit and restart TrayGoblin. POSIX working directories do not map
automatically to Windows editor actions; use the corresponding terminal.

## Attention and recovery

- Pi: ALL explicit blocking dialogs, including idle/manual/settings/compaction,
  plus FINAL settled run errors. Resolve/reject restores the underlying state,
  including the same earlier error identity. New work clears that run error.
- Copilot: NEW persisted `session.error` only. Input capability is unavailable;
  historical errors and tool failures do not alert. A new turn clears its error.
- Ordinary Idle, completion, cancellation and recovered tool failures are quiet.
- One coalesced request per tick, success-only acknowledgement. Failed submissions
  retry at most three times, at least five seconds apart. **Refresh now** rearms
  exhausted undelivered attention, never acknowledged episodes. Explorer icon
  re-registration does not replay acknowledged errors.
- Windows/Focus Assist may suppress a notification even after successful shell
  submission. The attention-first persistent session list is the fallback.
- Pi heartbeats every 2 seconds; older than 10 seconds is Unavailable, NOT input.
  Crash residue is not deleted by the tray. Reload/shutdown cleans only owned files.
- Stale/future/regressed clocks, unreadable/malformed files and unavailable roots
  show fixed diagnostics. Fix clocks/access and reload/recover the publisher.
- `ui_tracking_unavailable` is an integration fault from ambiguous pi UI hook
  ordering, not a model error or inferred input. It latches once until pi reload;
  if recurring, investigate extensions delaying hooks. Do not clear it by waiting.
- **Quit** removes the icon and exits; no terminal focus/control is performed.

Only allow-listed identity/model/directory/state/timing metadata is modeled.
Temporary local Copilot byte scanning is distinct from modeling content. No
prompt/result/argument/token/credential data is collected, logged or displayed.

## Remove opt-in before uninstalling

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pi remove "$tg\integrations\pi"
# Restart/reload all pi processes to close the publisher, then:
powershell -NoProfile -ExecutionPolicy Bypass -File "$tg\uninstall.ps1"
```

In WSL first run `pi remove "$HOME/.local/share/traygoblin/pi"`, reload/quit pi,
then remove only that copied integration folder if no longer needed. Uninstall
never edits pi settings or removes producer status directories. It removes its
marked install root/resources and Startup shortcut; keeps TrayGoblin config by
default. `-RemoveConfiguration` removes only config.json (other files remain).

## Real Windows evidence (not performed by the offline demo)

The collectors are shipped here, not dependent on an extracted source checkout:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pwsh -NoProfile -File "$tg\resources\test-attention-windows.ps1" -ExecutablePath "$tg\tray-goblin.exe" -EvidencePath .\windows-attention.json -PollIntervalMs 1000
if ($LASTEXITCODE -ne 0) { throw 'Actual attention field gate failed' }
pwsh -NoProfile -File "$tg\resources\measure-performance.ps1" -Path "$tg\tray-goblin.exe" -DurationSeconds 30 -JsonPath .\windows-performance.json
if ($LASTEXITCODE -ne 0) { throw 'Actual performance gate failed' }
```

`-PollIntervalMs` declares the EXPECTED cadence: match the actual running config's
`pollIntervalMs` or `poll_interval_ms`; omitted/null means 1000ms. For a nondefault
config location, select that running file with `-ConfigurationPath`, for example
ONLY if the tray already uses this file and 2000ms:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pwsh -NoProfile -File "$tg\resources\test-attention-windows.ps1" -ExecutablePath "$tg\tray-goblin.exe" -EvidencePath .\windows-attention.json -PollIntervalMs 2000 -ConfigurationPath 'D:\TrayGoblinProfile\TrayGoblin\config.json'
if ($LASTEXITCODE -ne 0) { throw 'Actual attention field gate failed' }
```

Neither parameter changes tray cadence or selects a config for the GUI. Edit the
actual tray config and restart first; no GUI config flag is implied. P determines
attention/clear (2P+500ms) and quiet checks (3P). Run each whole fence and stop on
native failure; do not collect performance after a failed attention gate.

Use synthetic activity only. The first is INTERACTIVE: prepare actual native+WSL
sources, perform every case and record observations/timings; no automatic pass.
Both reject non-Windows. `build.json` binds build commit/dirty flag and executable
SHA256; compare the SAME digest in both reports. Native Windows/WSL delivery and
50 MB/5% evidence are still required (M6). No publication is authorized by this ZIP.
