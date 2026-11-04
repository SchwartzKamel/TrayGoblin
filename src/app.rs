//! Platform-neutral tray application state machine.
//!
//! The Windows shell owns windows, timers, and process launching; this module
//! owns *decisions*: when to poll, what the icon and tooltip should say, which
//! local target a menu command selects, and how failures are surfaced. Keeping
//! it free of Win32 types is what makes tray behavior testable on any host.

use std::{path::PathBuf, time::Duration};

use crate::{
    actions::{ActionContext, ActionError, LaunchTarget, TrayAction, resolve_launch_target},
    attention::{
        AttentionController, AttentionSnapshot, Diagnostic, EpisodeReason, NotificationRequest,
        SessionIdentity, SessionSnapshot, SessionState, SourceSnapshot,
    },
    attention_worker::AttentionWorkers,
    config::{MonitorConfig, Provider},
    icon::IconVariant,
    status::{AttentionReason, StatusSnapshot},
};

/// Maximum cadence of the Windows timer that drives
/// [`AppMessage::TimerTick`]. Faster configured polling uses its own shorter
/// timer; slower polling keeps a one-second wake-up so due times are noticed
/// promptly without polling early.
pub const TIMER_INTERVAL_MS: u64 = 1_000;

/// Maximum characters a Win32 `NOTIFYICONDATAW::szTip` can hold, excluding
/// the terminating NUL.
pub const MAX_TOOLTIP_CHARS: usize = 127;

/// What the notification area should currently display. Text always repeats
/// the state name so the tray never relies on color alone.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrayView {
    pub variant: IconVariant,
    pub tooltip: String,
}

/// Everything the shell can tell the application.
#[derive(Clone, Debug, PartialEq)]
pub enum AppMessage {
    /// The tray icon has been created and the first poll should be requested.
    Started { now_ms: u64 },
    /// The one-second Win32 timer fired.
    TimerTick { now_ms: u64 },
    /// The user selected a menu item.
    MenuCommand { action: TrayAction, now_ms: u64 },
    /// A poll completed and produced a new content-free snapshot.
    SnapshotUpdated {
        snapshot: StatusSnapshot,
        active_session: Option<PathBuf>,
    },
    /// The shell failed to start a previously resolved target.
    LaunchFailed { action: TrayAction },
}

/// What the shell should do next.
#[derive(Clone, Debug, PartialEq)]
pub enum AppEffect {
    /// Poll the monitor now and report the result with
    /// [`AppMessage::SnapshotUpdated`].
    RequestPoll,
    /// Update the notification-area icon and tooltip.
    UpdateTray(TrayView),
    /// Open a local target, reporting failure with
    /// [`AppMessage::LaunchFailed`].
    Launch {
        action: TrayAction,
        target: LaunchTarget,
    },
    /// Remove the tray icon and exit.
    Quit,
}

/// Tray application state. Holds only content-free status data plus local
/// paths; prompt, response, tool, token, and credential data never reach it.
pub struct App {
    poll_interval_ms: u64,
    next_poll_due_ms: u64,
    session_root: Option<PathBuf>,
    settings_path: Option<PathBuf>,
    active_session: Option<PathBuf>,
    snapshot: StatusSnapshot,
    last_action_error: Option<ActionError>,
    published_view: Option<TrayView>,
    aggregate: Option<AttentionSnapshot>,
    selected: Option<SessionIdentity>,
    submission_failed: bool,
}

impl App {
    pub fn new(
        config: &MonitorConfig,
        session_root: Option<PathBuf>,
        settings_path: Option<PathBuf>,
    ) -> Self {
        Self {
            poll_interval_ms: config.poll_interval_ms(),
            next_poll_due_ms: 0,
            session_root,
            settings_path,
            active_session: None,
            snapshot: StatusSnapshot::default(),
            last_action_error: None,
            published_view: None,
            aggregate: None,
            selected: None,
            submission_failed: false,
        }
    }

    pub fn snapshot(&self) -> &StatusSnapshot {
        &self.snapshot
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.poll_interval_ms)
    }

    pub fn last_action_error(&self) -> Option<ActionError> {
        self.last_action_error
    }

    /// Single entry point for shell events. Returns the effects the shell
    /// must perform, in order.
    pub fn handle(&mut self, message: AppMessage) -> Vec<AppEffect> {
        match message {
            AppMessage::Started { now_ms } => {
                self.schedule_next_poll(now_ms);
                let mut effects = self.publish_view();
                effects.push(AppEffect::RequestPoll);
                effects
            }
            AppMessage::TimerTick { now_ms } => {
                if now_ms < self.next_poll_due_ms {
                    return Vec::new();
                }
                self.advance_next_poll(now_ms);
                vec![AppEffect::RequestPoll]
            }
            AppMessage::MenuCommand { action, now_ms } => self.handle_menu_command(action, now_ms),
            AppMessage::SnapshotUpdated {
                snapshot,
                active_session,
            } => {
                self.snapshot = snapshot;
                self.active_session = active_session;
                self.publish_view()
            }
            AppMessage::LaunchFailed { action } => {
                self.last_action_error = Some(ActionError::LaunchFailed(action));
                self.publish_view()
            }
        }
    }

    fn handle_menu_command(&mut self, action: TrayAction, now_ms: u64) -> Vec<AppEffect> {
        match action {
            // Refresh now polls immediately and restarts the interval so the
            // timer cannot fire a redundant poll right afterwards.
            TrayAction::RefreshNow => {
                self.last_action_error = None;
                self.schedule_next_poll(now_ms);
                let mut effects = self.publish_view();
                effects.push(AppEffect::RequestPoll);
                effects
            }
            TrayAction::Quit => vec![AppEffect::Quit],
            _ => match self.resolve_action(action) {
                Ok(target) => {
                    self.last_action_error = None;
                    let mut effects = self.publish_view();
                    effects.push(AppEffect::Launch { action, target });
                    effects
                }
                Err(error) => {
                    self.last_action_error = Some(error);
                    self.publish_view()
                }
            },
        }
    }

    fn resolve_action(&self, action: TrayAction) -> Result<LaunchTarget, ActionError> {
        if action == TrayAction::OpenInVsCode {
            if let Some((_, session)) = self.selected_session() {
                if session
                    .directory
                    .as_deref()
                    .is_some_and(|path| !safe_windows_path(path))
                {
                    return Err(ActionError::UnsafeWindowsPath);
                }
            }
        }
        resolve_launch_target(action, &self.action_context())
    }

    /// Worker snapshots contain metadata only; this method never opens provider files.
    pub fn update_attention(
        &mut self,
        snapshot: AttentionSnapshot,
        submission_failed: bool,
    ) -> Vec<AppEffect> {
        self.aggregate = Some(snapshot);
        self.submission_failed = submission_failed;
        self.publish_view()
    }

    pub fn attention_snapshot(&self) -> Option<&AttentionSnapshot> {
        self.aggregate.as_ref()
    }

    /// Selection only chooses the existing editor action's target; no terminal control.
    pub fn select_session(&mut self, identity: SessionIdentity) {
        self.selected = Some(identity);
    }

    fn selected_session(&self) -> Option<(&SourceSnapshot, &SessionSnapshot)> {
        let rows = session_rows(self.aggregate.as_ref()?);
        rows.iter()
            .copied()
            .find(|(source, session)| {
                self.selected.as_ref() == Some(&session_identity(source, session))
            })
            .or_else(|| rows.first().copied())
    }

    pub fn action_context(&self) -> ActionContext {
        if let Some(snapshot) = &self.aggregate {
            let active_directory = self.selected_session().and_then(|(_, session)| {
                (session.state != SessionState::Unavailable)
                    .then_some(session)
                    .and_then(|session| session.directory.as_deref())
                    .filter(|path| safe_windows_path(path))
                    .map(PathBuf::from)
            });
            // Logs ALWAYS select a Copilot source, never a pi status/conversation path.
            let copilot = session_rows(snapshot)
                .into_iter()
                .find(|(source, session)| {
                    source.source.kind == Provider::Copilot
                        && session.state != SessionState::Unavailable
                });
            let active_session = copilot.and_then(|(source, session)| {
                (safe_session_component(&session.session_id)
                    && safe_windows_path(&source.source.path.to_string_lossy()))
                .then(|| source.source.path.join(&session.session_id))
            });
            let session_root = snapshot
                .sources
                .iter()
                .find(|source| source.source.kind == Provider::Copilot)
                .filter(|source| safe_windows_path(&source.source.path.to_string_lossy()))
                .map(|source| source.source.path.clone());
            return ActionContext {
                session_root,
                active_session,
                active_directory,
                settings_path: self.settings_path.clone(),
            };
        }
        ActionContext {
            session_root: self.session_root.clone(),
            active_session: self.active_session.clone(),
            active_directory: self.snapshot.active_directory.clone(),
            settings_path: self.settings_path.clone(),
        }
    }

    fn schedule_next_poll(&mut self, now_ms: u64) {
        self.next_poll_due_ms = now_ms.saturating_add(self.poll_interval_ms);
    }

    fn advance_next_poll(&mut self, now_ms: u64) {
        let elapsed_intervals = now_ms
            .saturating_sub(self.next_poll_due_ms)
            .checked_div(self.poll_interval_ms)
            .unwrap_or_default()
            .saturating_add(1);
        self.next_poll_due_ms = self
            .next_poll_due_ms
            .saturating_add(self.poll_interval_ms.saturating_mul(elapsed_intervals));
    }

    /// Emits a tray update only when the rendered view actually changed,
    /// keeping idle CPU near zero as required by the performance principle.
    fn publish_view(&mut self) -> Vec<AppEffect> {
        let view = self.render_view();
        if self.published_view.as_ref() == Some(&view) {
            return Vec::new();
        }

        self.published_view = Some(view.clone());
        vec![AppEffect::UpdateTray(view)]
    }

    pub fn render_view(&self) -> TrayView {
        if let Some(snapshot) = &self.aggregate {
            return render_attention_view(snapshot, self.last_action_error, self.submission_failed);
        }
        let variant = IconVariant::for_state(self.snapshot.state);

        TrayView {
            variant,
            tooltip: render_tooltip(variant, &self.snapshot, self.last_action_error),
        }
    }
}

/// Win32 timer cadence for a validated polling interval. The default remains
/// one second, while the accepted 500-999 ms range is honored exactly.
pub fn timer_interval_ms(poll_interval_ms: u64) -> u32 {
    poll_interval_ms.min(TIMER_INTERVAL_MS) as u32
}

/// Builds the hover text. Every line is derived from the content-free
/// snapshot: state, model, repository or active directory, last turn
/// duration, and an optional action failure.
pub fn render_tooltip(
    variant: IconVariant,
    snapshot: &StatusSnapshot,
    action_error: Option<ActionError>,
) -> String {
    let mut lines = vec![format!("{} — {}", crate::APPLICATION_NAME, variant.label())];

    if let Some(model) = snapshot.model.as_deref() {
        lines.push(format!("Model: {model}"));
    }

    if let Some(repository) = snapshot.repository.as_deref() {
        lines.push(format!("Repo: {repository}"));
    } else if let Some(directory) = snapshot.active_directory.as_deref() {
        lines.push(format!("Dir: {}", directory.display()));
    }

    if let Some(duration) = snapshot.last_turn_duration {
        lines.push(format!("Last turn: {}", format_duration(duration)));
    }

    if let Some(reason) = snapshot.attention_reason {
        lines.push(format!("Reason: {}", attention_text(reason)));
    }

    if let Some(error) = action_error {
        lines.push(error.to_string());
    }

    truncate_tooltip(&lines.join("\n"))
}

fn attention_text(reason: AttentionReason) -> &'static str {
    match reason {
        AttentionReason::StateUnavailable => "Copilot session state is unreadable",
        AttentionReason::ToolFailed => "a tool execution failed in the last turn",
    }
}

/// Human-readable turn duration. Sub-minute turns keep one decimal so the
/// magic-moment demo shows a precise completion time.
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs_f64();
    if total_seconds < 60.0 {
        return format!("{total_seconds:.1} s");
    }

    let whole_seconds = duration.as_secs();
    format!("{} m {:02} s", whole_seconds / 60, whole_seconds % 60)
}

/// Win32 truncates tooltips silently at 128 wide characters; truncating here
/// keeps the state line intact and marks the cut explicitly.
fn truncate_tooltip(tooltip: &str) -> String {
    bounded_text(tooltip, MAX_TOOLTIP_CHARS)
}

/// The actual UI dispatch seam: owns the accepted fixed workers/controller for
/// the entire process lifetime, independently of Explorer's icon registration.
/// Cadence/uptime must NEVER enter this API; every clock sample is UNIX epoch ms.
pub struct AttentionRuntime {
    workers: AttentionWorkers,
    controller: AttentionController,
    submission_failed: bool,
}

impl AttentionRuntime {
    pub fn new(workers: AttentionWorkers) -> Self {
        Self {
            workers,
            controller: AttentionController::default(),
            submission_failed: false,
        }
    }

    pub fn snapshot(&self) -> &AttentionSnapshot {
        self.controller.snapshot()
    }

    pub fn submission_failed(&self) -> bool {
        self.submission_failed
    }

    /// Nonblocking request/drain only. `submit` is the production adapter call,
    /// not a declaration of OS delivery. Sample AGAIN after it actually returns:
    /// retries must be spaced from completion, even if the adapter took time.
    pub fn cycle(
        &mut self,
        request_poll: bool,
        refresh: bool,
        mut epoch_ms: impl FnMut() -> u64,
        mut submit: impl FnMut(&Balloon) -> bool,
    ) {
        let now = epoch_ms();
        if refresh {
            self.controller.refresh();
        }
        if request_poll {
            self.workers.request_poll(now);
        }
        let snapshot = self.workers.drain_at(now);
        if let Some(request) = self.controller.tick(snapshot, now) {
            let balloon = render_notification(self.controller.snapshot(), &request);
            let succeeded = submit(&balloon);
            self.controller.submitted(request.id, succeeded, epoch_ms());
            self.submission_failed = !succeeded;
        }
        if !self
            .snapshot()
            .sources
            .iter()
            .any(|source| source.sessions.iter().any(needs_attention))
        {
            self.submission_failed = false;
        }
    }

    pub fn stop(&mut self) {
        self.workers.stop();
    }
}

/// Shared by the host-tested constructor and NIF_INFO adapter (including NUL).
pub const BALLOON_TITLE_UNITS: usize = 64;
pub const BALLOON_BODY_UNITS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Balloon {
    pub title: String,
    pub body: String,
}

/// Clean, bounded UTF-16 text. Truncate whole Unicode scalars, never half a
/// surrogate pair. Strip layout/control injection; only our own layout is used.
pub fn bounded_text(value: &str, max_units: usize) -> String {
    let clean: String = value.chars().map(|ch| {
        if ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') { ' ' } else { ch }
    }).collect();
    if clean.encode_utf16().count() <= max_units {
        return clean;
    }
    let mut result = String::new();
    let mut units = 0;
    for ch in clean.chars() {
        if units + ch.len_utf16() + 1 > max_units {
            break;
        }
        result.push(ch);
        units += ch.len_utf16();
    }
    if max_units > 0 {
        result.push('…');
    }
    result
}

/// Fixed Win32 arrays always get a terminator, and cannot split surrogates.
pub fn write_wide_buffer(buffer: &mut [u16], value: &str) {
    buffer.fill(0);
    let text = bounded_text(value, buffer.len().saturating_sub(1));
    for (out, unit) in buffer.iter_mut().zip(text.encode_utf16()) {
        *out = unit;
    }
}

/// Menu labels are literal data, never accelerator/column syntax.
pub fn menu_text(value: &str) -> String {
    bounded_text(value, 200).replace('&', "&&")
}

pub fn needs_attention(session: &SessionSnapshot) -> bool {
    matches!(
        session.state,
        SessionState::InputRequired | SessionState::Error
    )
}

pub fn session_identity(source: &SourceSnapshot, session: &SessionSnapshot) -> SessionIdentity {
    SessionIdentity {
        source_id: source.source.id.clone(),
        session_id: session.session_id.clone(),
        instance_id: session.instance_id.clone(),
    }
}

/// Attention-first, deterministic ordering, independent of hash/directory order.
pub fn session_rows(snapshot: &AttentionSnapshot) -> Vec<(&SourceSnapshot, &SessionSnapshot)> {
    let mut rows: Vec<_> = snapshot
        .sources
        .iter()
        .flat_map(|source| source.sessions.iter().map(move |session| (source, session)))
        .collect();
    rows.sort_by(|(sa, a), (sb, b)| {
        (
            !needs_attention(a),
            &sa.source.id,
            &a.session_id,
            &a.instance_id,
        )
            .cmp(&(
                !needs_attention(b),
                &sb.source.id,
                &b.session_id,
                &b.instance_id,
            ))
    });
    rows
}

pub fn provider_text(provider: Provider) -> &'static str {
    match provider {
        Provider::Pi => "pi",
        Provider::Copilot => "Copilot",
    }
}

pub fn session_state_text(session: &SessionSnapshot) -> &'static str {
    match session.attention.as_ref().map(|episode| episode.reason) {
        Some(EpisodeReason::UiTrackingUnavailable) if session.state == SessionState::Error => {
            "UI tracking fault (reload pi)"
        }
        _ => match session.state {
            SessionState::Idle => "Idle",
            SessionState::Working => "Working",
            SessionState::InputRequired => "Input required",
            SessionState::Error => "Session error",
            SessionState::Unavailable => "Unavailable",
        },
    }
}

/// Put identifying fields before expandable metadata so even long directories
/// cannot crowd out source, manual label and distinct session/instance IDs.
pub fn session_text(source: &SourceSnapshot, session: &SessionSnapshot) -> String {
    format!(
        "{} | {} | {} | {}/{} | {}",
        provider_text(source.source.kind),
        bounded_text(&source.source.label, 24),
        bounded_text(session.label.as_deref().unwrap_or("unlabeled"), 64),
        bounded_text(&session.session_id, 16),
        bounded_text(&session.instance_id, 12),
        session_state_text(session)
    )
}

pub fn source_text(source: &SourceSnapshot) -> String {
    format!(
        "{} | {} | {} sessions | input: {} | errors: {} | {} diagnostics",
        bounded_text(&source.source.label, 40),
        provider_text(source.source.kind),
        source.sessions.len(),
        if source.capabilities.input_detection {
            "explicit dialogs"
        } else {
            "unavailable"
        },
        if source.capabilities.session_errors {
            "supported"
        } else {
            "unavailable"
        },
        source.diagnostics.len()
    )
}

pub fn diagnostic_text(diagnostic: Diagnostic) -> &'static str {
    match diagnostic {
        Diagnostic::SourceUnavailable => "Source unavailable: check configured root",
        Diagnostic::SessionUnavailable => "Session unavailable: check file access",
        Diagnostic::SessionOverflow => "Session/scan capacity reached",
        Diagnostic::DeliveryHistoryOverflow => "Delivery history capacity reached",
        Diagnostic::EpisodeIdentityExhausted => "Episode identity exhausted: restart tray",
        Diagnostic::InvalidPiStatus => "Invalid pi status",
        Diagnostic::OversizedPiStatus => "Oversized pi status",
        Diagnostic::StalePiStatus => "Stale pi heartbeat",
        Diagnostic::FuturePiStatus => "Future pi clock: check clocks",
        Diagnostic::RegressedPiStatus => "Regressed pi clock: check clocks",
        Diagnostic::UnsafeEntry => "Unsafe source entry ignored",
        Diagnostic::MalformedEvent => "Malformed Copilot event",
        Diagnostic::OversizedEvent => "Oversized Copilot event",
        Diagnostic::InvalidWorkspace => "Invalid workspace metadata",
    }
}

pub fn render_attention_view(
    snapshot: &AttentionSnapshot,
    error: Option<ActionError>,
    submission_failed: bool,
) -> TrayView {
    let rows = session_rows(snapshot);
    let attention = rows.iter().filter(|(_, s)| needs_attention(s)).count();
    let working = rows
        .iter()
        .filter(|(_, s)| s.state == SessionState::Working)
        .count();
    let idle = rows
        .iter()
        .filter(|(_, s)| s.state == SessionState::Idle)
        .count();
    let unavailable: usize = snapshot
        .sources
        .iter()
        .map(|source| {
            source.unavailable_pi_statuses.max(
                source
                    .sessions
                    .iter()
                    .filter(|s| s.state == SessionState::Unavailable)
                    .count(),
            )
        })
        .sum();
    let diagnostics: usize = snapshot.sources.iter().map(|s| s.diagnostics.len()).sum();
    let variant = if attention > 0 || unavailable > 0 || diagnostics > 0 || submission_failed {
        IconVariant::AttentionNeeded
    } else if working > 0 {
        IconVariant::Working
    } else {
        IconVariant::Idle
    };
    // Action failures take precedence over details so their safe explanation is
    // never silently truncated behind a long label/path. Counts remain in menu.
    let tooltip = if let Some(error) = error {
        format!("{} — {error}", crate::APPLICATION_NAME)
    } else {
        let summary = format!(
            "{} — {} | A{attention} W{working} I{idle} U{unavailable} D{diagnostics}",
            crate::APPLICATION_NAME,
            if submission_failed {
                "Submission failed"
            } else {
                variant.label()
            }
        );
        if let Some((source, session)) = rows.first().filter(|(_, s)| needs_attention(s)) {
            format!(
                "{summary} | {}/{}/{}/{}/{}",
                provider_text(source.source.kind),
                bounded_text(&source.source.label, 12),
                bounded_text(session.label.as_deref().unwrap_or("unlabeled"), 16),
                bounded_text(&session.session_id, 8),
                bounded_text(&session.instance_id, 8)
            )
        } else {
            summary
        }
    };
    TrayView {
        variant,
        tooltip: bounded_text(&tooltip, MAX_TOOLTIP_CHARS),
    }
}

pub fn render_notification(snapshot: &AttentionSnapshot, request: &NotificationRequest) -> Balloon {
    let first = request
        .episodes
        .first()
        .and_then(|item| {
            snapshot
                .sources
                .iter()
                .find(|source| source.source.id == item.session.source_id)
                .and_then(|source| {
                    source
                        .sessions
                        .iter()
                        .find(|session| session_identity(source, session) == item.session)
                        .map(|session| session_text(source, session))
                })
        })
        .unwrap_or_else(|| "Attention retained in tray session list".into());
    Balloon {
        title: bounded_text(
            &format!("TrayGoblin — {} need attention", request.episodes.len()),
            BALLOON_TITLE_UNITS - 1,
        ),
        body: bounded_text(
            &format!(
                "{first}. {} session(s); right-click tray for all.",
                request.episodes.len()
            ),
            BALLOON_BODY_UNITS - 1,
        ),
    }
}

/// Do not pass POSIX paths, URI schemes, quotes or control characters to a
/// Windows launcher. WSL directories deliberately fail safely (no implicit map).
pub fn safe_windows_path(path: &str) -> bool {
    if path
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '"' | '<' | '>' | '|' | '?' | '*'))
    {
        return false;
    }
    let bytes = path.as_bytes();
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
    {
        return !path[2..].contains(':');
    }
    if let Some(unc) = path.strip_prefix("\\\\") {
        let mut parts = unc.split('\\');
        return parts
            .next()
            .is_some_and(|s| !s.is_empty() && s != "." && s != "..")
            && parts
                .next()
                .is_some_and(|s| !s.is_empty() && s != "." && s != "..")
            && !unc.contains(':');
    }
    false
}

fn safe_session_component(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && !value.chars().any(|ch| {
            ch.is_control() || matches!(ch, '/' | '\\' | ':' | '"' | '<' | '>' | '|' | '?' | '*')
        })
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use super::{
        App, AppEffect, AppMessage, MAX_TOOLTIP_CHARS, TIMER_INTERVAL_MS, format_duration,
        timer_interval_ms,
    };
    use crate::{
        actions::{ActionError, LaunchTarget, TrayAction},
        config::MonitorConfig,
        icon::IconVariant,
        status::{AttentionReason, StatusSnapshot, StatusState},
    };

    fn app() -> App {
        App::new(
            &MonitorConfig::default(),
            Some(PathBuf::from("C:/Users/dev/.copilot/session-state")),
            Some(PathBuf::from(
                "C:/Users/dev/AppData/Roaming/TrayGoblin/config.json",
            )),
        )
    }

    fn working_snapshot() -> StatusSnapshot {
        StatusSnapshot {
            state: StatusState::Generating,
            model: Some("claude-sonnet-4".to_owned()),
            repository: Some("octo/demo".to_owned()),
            active_directory: Some(PathBuf::from("C:/src/demo")),
            last_turn_duration: Some(Duration::from_millis(2_400)),
            attention_reason: None,
        }
    }

    fn tray_view(effects: &[AppEffect]) -> super::TrayView {
        effects
            .iter()
            .find_map(|effect| match effect {
                AppEffect::UpdateTray(view) => Some(view.clone()),
                _ => None,
            })
            .expect("expected a tray update")
    }

    // Acceptance: "When the user selects Refresh now, the tray shall poll
    // immediately rather than waiting for the timer."
    #[test]
    fn manual_refresh_requests_poll() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });
        // A tick well before the interval elapses must stay silent...
        assert!(app.handle(AppMessage::TimerTick { now_ms: 200 }).is_empty());

        let effects = app.handle(AppMessage::MenuCommand {
            action: TrayAction::RefreshNow,
            now_ms: 200,
        });

        // ...while Refresh now polls at once.
        assert!(effects.contains(&AppEffect::RequestPoll));
    }

    // The default one-second cadence is an acceptance criterion: state must
    // be visible within two seconds of a turn starting or ending.
    #[test]
    fn timer_polls_once_per_configured_interval() {
        let mut app = app();
        assert_eq!(
            app.poll_interval(),
            Duration::from_millis(TIMER_INTERVAL_MS)
        );

        app.handle(AppMessage::Started { now_ms: 10_000 });

        assert!(
            app.handle(AppMessage::TimerTick { now_ms: 10_999 })
                .is_empty(),
            "polling must not run faster than the configured interval"
        );
        assert_eq!(
            app.handle(AppMessage::TimerTick { now_ms: 11_000 }),
            vec![AppEffect::RequestPoll]
        );
        assert!(
            app.handle(AppMessage::TimerTick { now_ms: 11_500 })
                .is_empty()
        );
    }

    // A delayed timer message must not move the schedule forward from its
    // delivery time and accidentally suppress the next on-time timer tick.
    #[test]
    fn delayed_timer_tick_preserves_the_polling_schedule() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 10_000 });

        assert_eq!(
            app.handle(AppMessage::TimerTick { now_ms: 11_010 }),
            vec![AppEffect::RequestPoll]
        );
        assert_eq!(
            app.handle(AppMessage::TimerTick { now_ms: 12_000 }),
            vec![AppEffect::RequestPoll]
        );
    }

    // A manual refresh restarts the interval so it cannot be followed by an
    // immediate redundant timer poll.
    #[test]
    fn manual_refresh_restarts_the_polling_interval() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });
        app.handle(AppMessage::MenuCommand {
            action: TrayAction::RefreshNow,
            now_ms: 400,
        });

        assert!(
            app.handle(AppMessage::TimerTick { now_ms: 1_000 })
                .is_empty()
        );
        assert_eq!(
            app.handle(AppMessage::TimerTick { now_ms: 1_400 }),
            vec![AppEffect::RequestPoll]
        );
    }

    // A slower configured cadence must be honored by the same one-second timer.
    #[test]
    fn configured_interval_overrides_the_timer_cadence() {
        let config = MonitorConfig::new(None, 3_000).unwrap();
        let mut app = App::new(&config, None, None);
        app.handle(AppMessage::Started { now_ms: 0 });

        assert!(
            app.handle(AppMessage::TimerTick { now_ms: 2_000 })
                .is_empty()
        );
        assert_eq!(
            app.handle(AppMessage::TimerTick { now_ms: 3_000 }),
            vec![AppEffect::RequestPoll]
        );
    }

    // The shell wakes at the configured cadence below one second and at
    // one-second intervals for slower configurations.
    #[test]
    fn shell_timer_honors_subsecond_configuration() {
        assert_eq!(timer_interval_ms(500), 500);
        assert_eq!(timer_interval_ms(750), 750);
        assert_eq!(timer_interval_ms(1_000), 1_000);
        assert_eq!(timer_interval_ms(10_000), 1_000);
    }

    // Startup must show something immediately and ask for the first poll.
    #[test]
    fn startup_publishes_idle_and_requests_the_first_poll() {
        let mut app = app();

        let effects = app.handle(AppMessage::Started { now_ms: 0 });

        assert_eq!(tray_view(&effects).variant, IconVariant::Idle);
        assert!(effects.contains(&AppEffect::RequestPoll));
    }

    // The magic moment: a Working snapshot must reach the tray as both a
    // distinct icon and text that names the state, model, repo, and duration.
    #[test]
    fn snapshot_updates_icon_and_tooltip_together() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });

        let effects = app.handle(AppMessage::SnapshotUpdated {
            snapshot: working_snapshot(),
            active_session: Some(PathBuf::from("C:/Users/dev/.copilot/session-state/abc")),
        });
        let view = tray_view(&effects);

        assert_eq!(view.variant, IconVariant::Working);
        assert!(view.tooltip.contains("Working"));
        assert!(view.tooltip.contains("claude-sonnet-4"));
        assert!(view.tooltip.contains("octo/demo"));
        assert!(view.tooltip.contains("2.4 s"));
    }

    // Attention needed must explain itself without exposing session content.
    #[test]
    fn attention_states_explain_the_reason_in_text() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });

        for (reason, expected) in [
            (AttentionReason::ToolFailed, "a tool execution failed"),
            (
                AttentionReason::StateUnavailable,
                "Copilot session state is unreadable",
            ),
        ] {
            let effects = app.handle(AppMessage::SnapshotUpdated {
                snapshot: StatusSnapshot {
                    state: StatusState::Error,
                    attention_reason: Some(reason),
                    ..StatusSnapshot::default()
                },
                active_session: None,
            });
            let view = tray_view(&effects);

            assert_eq!(view.variant, IconVariant::AttentionNeeded);
            assert!(view.tooltip.contains("Attention needed"));
            assert!(view.tooltip.contains(expected), "tooltip: {}", view.tooltip);

            // Reset so the next reason is a real change.
            app.handle(AppMessage::SnapshotUpdated {
                snapshot: StatusSnapshot::default(),
                active_session: None,
            });
        }
    }

    // Repeating an identical snapshot must not repaint the tray.
    #[test]
    fn unchanged_snapshot_does_not_republish_the_view() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });
        app.handle(AppMessage::SnapshotUpdated {
            snapshot: working_snapshot(),
            active_session: None,
        });

        let effects = app.handle(AppMessage::SnapshotUpdated {
            snapshot: working_snapshot(),
            active_session: None,
        });

        assert!(effects.is_empty());
    }

    // Open in VS Code opens the observed active directory.
    #[test]
    fn open_in_vs_code_launches_the_active_directory() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });
        app.handle(AppMessage::SnapshotUpdated {
            snapshot: working_snapshot(),
            active_session: None,
        });

        let effects = app.handle(AppMessage::MenuCommand {
            action: TrayAction::OpenInVsCode,
            now_ms: 500,
        });

        assert!(effects.contains(&AppEffect::Launch {
            action: TrayAction::OpenInVsCode,
            target: LaunchTarget::Editor {
                directory: PathBuf::from("C:/src/demo")
            }
        }));
    }

    // View Copilot logs opens the active session folder once one is known.
    #[test]
    fn view_copilot_logs_launches_the_active_session_folder() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });
        app.handle(AppMessage::SnapshotUpdated {
            snapshot: working_snapshot(),
            active_session: Some(PathBuf::from("C:/Users/dev/.copilot/session-state/abc")),
        });

        let effects = app.handle(AppMessage::MenuCommand {
            action: TrayAction::ViewCopilotLogs,
            now_ms: 500,
        });

        assert!(effects.contains(&AppEffect::Launch {
            action: TrayAction::ViewCopilotLogs,
            target: LaunchTarget::Folder {
                directory: PathBuf::from("C:/Users/dev/.copilot/session-state/abc")
            }
        }));
    }

    #[test]
    fn quit_asks_the_shell_to_exit() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });

        assert_eq!(
            app.handle(AppMessage::MenuCommand {
                action: TrayAction::Quit,
                now_ms: 900,
            }),
            vec![AppEffect::Quit]
        );
    }

    // An unresolvable command must never launch anything and must explain
    // itself in the tooltip without leaking a path.
    #[test]
    fn unresolvable_command_surfaces_a_content_free_tooltip() {
        let mut app = App::new(&MonitorConfig::default(), None, None);
        app.handle(AppMessage::Started { now_ms: 0 });

        let effects = app.handle(AppMessage::MenuCommand {
            action: TrayAction::OpenInVsCode,
            now_ms: 100,
        });
        let view = tray_view(&effects);

        assert!(
            !effects
                .iter()
                .any(|effect| matches!(effect, AppEffect::Launch { .. }))
        );
        assert_eq!(
            app.last_action_error(),
            Some(ActionError::NoActiveDirectory)
        );
        assert!(view.tooltip.contains("Open in VS Code is unavailable"));
        assert!(!view.tooltip.contains(":\\") && !view.tooltip.contains("C:/"));
    }

    // A shell-reported launch failure is surfaced the same safe way.
    #[test]
    fn launch_failure_is_reported_without_session_content() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });

        let effects = app.handle(AppMessage::LaunchFailed {
            action: TrayAction::OpenSettings,
        });
        let view = tray_view(&effects);

        assert_eq!(
            app.last_action_error(),
            Some(ActionError::LaunchFailed(TrayAction::OpenSettings))
        );
        assert!(view.tooltip.contains("Open settings could not be started"));
        assert!(!view.tooltip.contains("config.json"));
    }

    // A successful command clears the previous failure notice.
    #[test]
    fn successful_command_clears_the_previous_failure() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });
        app.handle(AppMessage::LaunchFailed {
            action: TrayAction::OpenSettings,
        });

        app.handle(AppMessage::MenuCommand {
            action: TrayAction::RefreshNow,
            now_ms: 100,
        });

        assert_eq!(app.last_action_error(), None);
        assert!(!app.render_view().tooltip.contains("could not be started"));
    }

    // Win32 silently truncates long tooltips; we cut deliberately instead.
    #[test]
    fn tooltip_stays_within_the_win32_limit() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });

        let effects = app.handle(AppMessage::SnapshotUpdated {
            snapshot: StatusSnapshot {
                state: StatusState::Generating,
                model: Some("m".repeat(120)),
                repository: Some("r".repeat(120)),
                active_directory: Some(PathBuf::from("d".repeat(200))),
                last_turn_duration: Some(Duration::from_secs(1)),
                attention_reason: None,
            },
            active_session: None,
        });
        let view = tray_view(&effects);

        assert!(view.tooltip.chars().count() <= MAX_TOOLTIP_CHARS);
        assert!(view.tooltip.starts_with("TrayGoblin — Working"));
    }

    // Duration text is part of the magic moment, so its formatting is pinned.
    #[test]
    fn durations_are_formatted_for_humans() {
        assert_eq!(format_duration(Duration::from_millis(450)), "0.5 s");
        assert_eq!(format_duration(Duration::from_millis(2_400)), "2.4 s");
        assert_eq!(format_duration(Duration::from_secs(75)), "1 m 15 s");
    }

    // Without a repository, the active directory is the fallback context.
    #[test]
    fn tooltip_falls_back_to_the_active_directory() {
        let mut app = app();
        app.handle(AppMessage::Started { now_ms: 0 });

        let effects = app.handle(AppMessage::SnapshotUpdated {
            snapshot: StatusSnapshot {
                repository: None,
                ..working_snapshot()
            },
            active_session: None,
        });

        assert!(tray_view(&effects).tooltip.contains("Dir: C:/src/demo"));
    }
}
