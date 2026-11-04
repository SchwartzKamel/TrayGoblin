//! Content-free aggregate types, pi v1 schema and notification delivery policy.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Deserializer, Serialize};

use crate::config::Source;

pub const MAX_PI_BYTES: usize = 16 * 1024;
pub const MAX_LABEL_BYTES: usize = 64;
pub const MAX_SESSIONS: usize = 64;
pub const MAX_SOURCE_MEMBERSHIP: usize = 4096;
pub const STALE_AFTER_MS: u64 = 10_000;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    Working,
    InputRequired,
    Error,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeReason {
    InputRequired,
    RunError,
    UiTrackingUnavailable,
    SessionError,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct AttentionEpisode {
    pub id: u64,
    pub reason: EpisodeReason,
    /// False for Copilot errors encountered in the initial historical log.
    pub notify: bool,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct SessionSnapshot {
    pub session_id: String,
    /// Explicit operator metadata only; never a provider-generated session name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub instance_id: String,
    pub model: Option<String>,
    pub directory: Option<String>,
    pub state: SessionState,
    pub attention: Option<AttentionEpisode>,
    pub run_started_at_ms: Option<u64>,
    pub last_run_duration_ms: Option<u64>,
    /// Inclusive publisher freshness deadline; None for non-publisher snapshots.
    #[serde(skip)]
    pub publisher_expires_at_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Diagnostic {
    SourceUnavailable,
    SessionUnavailable,
    SessionOverflow,
    DeliveryHistoryOverflow,
    EpisodeIdentityExhausted,
    InvalidPiStatus,
    OversizedPiStatus,
    StalePiStatus,
    FuturePiStatus,
    RegressedPiStatus,
    UnsafeEntry,
    MalformedEvent,
    OversizedEvent,
    InvalidWorkspace,
}

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
pub struct Capabilities {
    pub input_detection: bool,
    pub session_errors: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct SourceSnapshot {
    pub source: Source,
    pub capabilities: Capabilities,
    pub sessions: Vec<SessionSnapshot>,
    pub unavailable_pi_statuses: usize,
    pub diagnostics: Vec<Diagnostic>,
    /// Internal lifecycle evidence, independent of the visible session projection.
    #[serde(skip)]
    pub membership: SourceMembership,
}

/// Absence proves removal only after a complete scan. Presence does not imply
/// healthy status, nor reveal a pi session ID from an unreadable publication.
#[derive(Clone, Debug, Default)]
pub struct SourceMembership {
    pub instances: HashSet<String>,
    pub complete: bool,
}

impl SourceSnapshot {
    pub(crate) fn diagnose(&mut self, diagnostic: Diagnostic) {
        if !self.diagnostics.contains(&diagnostic) {
            self.diagnostics.push(diagnostic);
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct AttentionSnapshot {
    pub sources: Vec<SourceSnapshot>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PiStatus {
    version: u64,
    provider: String,
    pub session_id: String,
    #[serde(default)]
    pub label: Option<String>,
    pub instance_id: String,
    pub model: String,
    pub directory: String,
    pub state: SessionState,
    #[serde(deserialize_with = "nullable")]
    attention_reason: Option<EpisodeReason>,
    episode: u64,
    pub updated_at: u64,
    #[serde(deserialize_with = "nullable")]
    run_started_at: Option<u64>,
    #[serde(deserialize_with = "nullable")]
    last_run_duration_ms: Option<u64>,
}

fn nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}

pub fn is_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}

pub fn parse_pi_status(bytes: &[u8], now_ms: u64) -> Result<PiStatus, Diagnostic> {
    if bytes.len() > MAX_PI_BYTES {
        return Err(Diagnostic::OversizedPiStatus);
    }
    let status: PiStatus =
        serde_json::from_slice(bytes).map_err(|_| Diagnostic::InvalidPiStatus)?;
    let bounded = |s: &str, max| s.len() <= max && !s.chars().any(char::is_control);
    let reason_valid = match status.state {
        SessionState::Error => matches!(
            status.attention_reason,
            Some(EpisodeReason::RunError | EpisodeReason::UiTrackingUnavailable)
        ),
        SessionState::Idle | SessionState::Working | SessionState::InputRequired => {
            status.attention_reason.is_none()
        }
        SessionState::Unavailable => false,
    };
    if status.version != 1
        || status.provider != "pi"
        || !is_uuid(&status.instance_id)
        || !bounded(&status.session_id, 128)
        || status
            .label
            .as_ref()
            .is_some_and(|label| label.trim().is_empty() || !bounded(label, MAX_LABEL_BYTES))
        || !bounded(&status.model, 128)
        || !bounded(&status.directory, 2048)
        || status.episode > MAX_SAFE_INTEGER
        || status.updated_at > MAX_SAFE_INTEGER
        || status.run_started_at.is_some_and(|t| t > status.updated_at)
        || status
            .last_run_duration_ms
            .is_some_and(|t| t > MAX_SAFE_INTEGER)
        || (matches!(
            status.state,
            SessionState::InputRequired | SessionState::Error
        ) && status.episode == 0)
        || !reason_valid
    {
        return Err(Diagnostic::InvalidPiStatus);
    }
    if status.updated_at > now_ms {
        return Err(Diagnostic::FuturePiStatus);
    }
    if now_ms - status.updated_at > STALE_AFTER_MS {
        return Err(Diagnostic::StalePiStatus);
    }
    Ok(status)
}

impl PiStatus {
    pub(crate) fn snapshot(self) -> SessionSnapshot {
        let reason = match self.state {
            SessionState::InputRequired => Some(EpisodeReason::InputRequired),
            SessionState::Error => self.attention_reason,
            _ => None,
        };
        SessionSnapshot {
            session_id: self.session_id,
            label: self.label,
            instance_id: self.instance_id,
            model: (!self.model.is_empty()).then_some(self.model),
            directory: (!self.directory.is_empty()).then_some(self.directory),
            state: self.state,
            attention: reason.map(|reason| AttentionEpisode {
                id: self.episode,
                reason,
                notify: true,
            }),
            run_started_at_ms: self.run_started_at,
            last_run_duration_ms: self.last_run_duration_ms,
            publisher_expires_at_ms: Some(self.updated_at + STALE_AFTER_MS),
        }
    }
}

/// Directory is deliberately not identity: concurrent sessions may share it.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SessionIdentity {
    pub source_id: String,
    pub session_id: String,
    pub instance_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationEpisode {
    pub session: SessionIdentity,
    pub episode: AttentionEpisode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationRequest {
    pub id: u64,
    pub episodes: Vec<NotificationEpisode>,
}

#[derive(Default)]
struct Delivery {
    episode: u64,
    attempts: u8,
    last_attempt_ms: Option<u64>,
    acknowledged: bool,
}

#[derive(Default)]
struct SessionDelivery {
    current: Option<AttentionEpisode>,
    // One slot per reason preserves a hidden error across a dialog without
    // retaining an unbounded history of completed attention episodes.
    reasons: HashMap<EpisodeReason, Delivery>,
}

/// Platform-neutral notification policy; the native adapter must report success.
#[derive(Default)]
pub struct AttentionController {
    sessions: HashMap<SessionIdentity, SessionDelivery>,
    snapshot: AttentionSnapshot,
    pending: Option<NotificationRequest>,
    next_request: u64,
    last_tick_ms: Option<u64>,
}

impl AttentionController {
    pub fn snapshot(&self) -> &AttentionSnapshot {
        &self.snapshot
    }

    /// Call once per UI poll tick, including ticks without fresh worker results.
    pub fn tick(
        &mut self,
        mut snapshot: AttentionSnapshot,
        now_ms: u64,
    ) -> Option<NotificationRequest> {
        let sources: HashSet<_> = snapshot
            .sources
            .iter()
            .map(|s| s.source.id.clone())
            .collect();
        self.sessions.retain(|identity, delivery| {
            delivery.current = None;
            sources.contains(&identity.source_id)
        });
        for source in &mut snapshot.sources {
            let visible: HashMap<_, _> = source
                .sessions
                .iter()
                .filter(|s| s.state != SessionState::Unavailable)
                .map(|s| (s.instance_id.as_str(), s.session_id.as_str()))
                .collect();
            let unavailable = source.diagnostics.contains(&Diagnostic::SourceUnavailable);
            self.sessions.retain(|identity, _| {
                if identity.source_id != source.source.id {
                    return true;
                }
                if let Some(session_id) = visible.get(identity.instance_id.as_str()) {
                    return *session_id == identity.session_id;
                }
                unavailable
                    || !source.membership.complete
                    || source.membership.instances.contains(&identity.instance_id)
            });
            let mut retained = self
                .sessions
                .keys()
                .filter(|identity| identity.source_id == source.source.id)
                .count();
            let mut overflow = false;
            for session in &source.sessions {
                if session.state == SessionState::Unavailable {
                    continue;
                }
                let identity = SessionIdentity {
                    source_id: source.source.id.clone(),
                    session_id: session.session_id.clone(),
                    instance_id: session.instance_id.clone(),
                };
                if !self.sessions.contains_key(&identity) {
                    if retained == MAX_SOURCE_MEMBERSHIP {
                        overflow = true;
                        continue;
                    }
                    retained += 1;
                }
                let delivery = self.sessions.entry(identity).or_default();
                delivery.current = session.attention.clone();
                if let Some(episode) = &session.attention {
                    let slot = delivery.reasons.entry(episode.reason).or_default();
                    if slot.episode != episode.id {
                        *slot = Delivery {
                            episode: episode.id,
                            acknowledged: !episode.notify,
                            ..Delivery::default()
                        };
                    }
                }
            }
            if overflow {
                source.diagnose(Diagnostic::DeliveryHistoryOverflow);
            }
        }
        self.snapshot = snapshot;
        if let Some(pending) = &mut self.pending {
            pending.episodes.retain(|item| {
                self.sessions
                    .get(&item.session)
                    .is_some_and(|session| session.current.as_ref() == Some(&item.episode))
            });
            if pending.episodes.is_empty() {
                self.pending = None;
            }
        }
        if self.last_tick_ms == Some(now_ms) {
            return None;
        }
        self.last_tick_ms = Some(now_ms);
        if self.pending.is_some() {
            return None;
        }
        let mut episodes = Vec::new();
        // Snapshot order gives stable source/session ordering, not hash order.
        for source in &self.snapshot.sources {
            for session in &source.sessions {
                if session.state == SessionState::Unavailable {
                    continue;
                }
                let identity = SessionIdentity {
                    source_id: source.source.id.clone(),
                    session_id: session.session_id.clone(),
                    instance_id: session.instance_id.clone(),
                };
                let Some(delivery) = self.sessions.get_mut(&identity) else {
                    // Admission failure is reported on the source, not a reset
                    // of an older instance's acknowledged/exhausted delivery.
                    continue;
                };
                let Some(episode) = &delivery.current else {
                    continue;
                };
                let slot = delivery
                    .reasons
                    .get_mut(&episode.reason)
                    .expect("episode slot");
                if episode.notify
                    && !slot.acknowledged
                    && slot.attempts < 3
                    && slot
                        .last_attempt_ms
                        .is_none_or(|last| now_ms.saturating_sub(last) >= 5_000)
                {
                    slot.attempts += 1;
                    slot.last_attempt_ms = Some(now_ms);
                    episodes.push(NotificationEpisode {
                        session: identity,
                        episode: episode.clone(),
                    });
                }
            }
        }
        if episodes.is_empty() {
            return None;
        }
        self.next_request += 1;
        let request = NotificationRequest {
            id: self.next_request,
            episodes,
        };
        self.pending = Some(request.clone());
        Some(request)
    }

    /// A failed submission consumes its attempt, but never acknowledges attention.
    pub fn submitted(&mut self, request_id: u64, succeeded: bool, now_ms: u64) {
        if self.pending.as_ref().is_none_or(|p| p.id != request_id) {
            return;
        }
        let pending = self.pending.take().expect("matching pending request");
        for item in pending.episodes {
            if let Some(session) = self.sessions.get_mut(&item.session) {
                if session.current.as_ref() == Some(&item.episode) {
                    if let Some(slot) = session.reasons.get_mut(&item.episode.reason) {
                        slot.last_attempt_ms = Some(now_ms);
                        slot.acknowledged = succeeded;
                    }
                }
            }
        }
    }

    /// Only explicit Refresh rearms exhausted, undelivered current episodes.
    /// Preserve last-attempt time so Refresh cannot bypass the retry spacing.
    pub fn refresh(&mut self) {
        for session in self.sessions.values_mut() {
            if let Some(episode) = &session.current {
                if let Some(slot) = session.reasons.get_mut(&episode.reason) {
                    if !slot.acknowledged && slot.attempts >= 3 {
                        slot.attempts = 0;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MonitorConfig;

    fn snapshot(first: usize, complete: bool) -> AttentionSnapshot {
        let source = MonitorConfig::parse(
            r#"{"sources":[{"id":"test","label":"Synthetic","kind":"pi","path":"/synthetic"}]}"#,
        )
        .unwrap()
        .resolved_sources()
        .unwrap()
        .remove(0);
        let sessions: Vec<_> = (first..first + MAX_SESSIONS)
            .map(|id| SessionSnapshot {
                session_id: format!("session-{id}"),
                label: None,
                instance_id: format!("instance-{id}"),
                model: None,
                directory: None,
                state: SessionState::Error,
                attention: Some(AttentionEpisode {
                    id: 1,
                    reason: EpisodeReason::RunError,
                    notify: true,
                }),
                run_started_at_ms: None,
                last_run_duration_ms: None,
                publisher_expires_at_ms: None,
            })
            .collect();
        AttentionSnapshot {
            sources: vec![SourceSnapshot {
                source,
                capabilities: Capabilities {
                    input_detection: true,
                    session_errors: true,
                },
                membership: SourceMembership {
                    instances: sessions.iter().map(|s| s.instance_id.clone()).collect(),
                    complete,
                },
                sessions,
                unavailable_pi_statuses: 0,
                diagnostics: Vec::new(),
            }],
        }
    }

    // Proves partial-scan churn cannot grow history/work past 4096 slots/source
    // or evict unknown acknowledged identities; complete recovery frees capacity.
    #[test]
    fn incomplete_membership_history_is_bounded_and_recovers_visibly() {
        let mut controller = AttentionController::default();
        for generation in 0..80 {
            let now = generation as u64;
            let request = controller.tick(snapshot(generation * MAX_SESSIONS, false), now);
            assert_eq!(request.is_some(), generation < 64);
            if let Some(request) = request {
                controller.submitted(request.id, true, now);
            }
            assert_eq!(
                controller.sessions.len(),
                ((generation + 1) * MAX_SESSIONS).min(MAX_SOURCE_MEMBERSHIP)
            );
            assert_eq!(
                controller.snapshot.sources[0]
                    .diagnostics
                    .contains(&Diagnostic::DeliveryHistoryOverflow),
                generation >= 64
            );
            assert!(controller.sessions.values().all(|s| s.reasons.len() <= 4));
            assert_eq!(controller.snapshot.sources[0].sessions.len(), MAX_SESSIONS);
            assert_eq!(
                controller.snapshot.sources[0].membership.instances.len(),
                MAX_SESSIONS
            );
        }
        assert!(controller.tick(snapshot(0, false), 81).is_none());
        assert_eq!(controller.sessions.len(), MAX_SOURCE_MEMBERSHIP);
        let recovered = controller
            .tick(snapshot(80 * MAX_SESSIONS, true), 82)
            .unwrap();
        assert_eq!(recovered.episodes.len(), MAX_SESSIONS);
        assert_eq!(controller.sessions.len(), MAX_SESSIONS);
        assert!(
            !controller.snapshot.sources[0]
                .diagnostics
                .contains(&Diagnostic::DeliveryHistoryOverflow)
        );
        controller.submitted(recovered.id, true, 82);
        controller.tick(AttentionSnapshot::default(), 83);
        assert!(controller.sessions.is_empty());
    }

    // Proves positive fresh identity replaces old session history even when the
    // surrounding scan is incomplete, without waiting for a complete rescan.
    #[test]
    fn visible_replacement_prunes_history_during_incomplete_scan() {
        let mut controller = AttentionController::default();
        let initial = snapshot(0, false);
        let request = controller.tick(initial.clone(), 0).unwrap();
        controller.submitted(request.id, true, 0);
        let mut replaced = initial.clone();
        replaced.sources[0].sessions[0].session_id = "replacement".into();
        let request = controller.tick(replaced, 1).unwrap();
        assert_eq!(request.episodes.len(), 1);
        assert_eq!(request.episodes[0].session.session_id, "replacement");
        assert_eq!(controller.sessions.len(), MAX_SESSIONS);
        controller.submitted(request.id, true, 1);
        let restored = controller.tick(initial, 2).unwrap();
        assert_eq!(restored.episodes.len(), 1);
        assert_eq!(restored.episodes[0].session.session_id, "session-0");
        assert_eq!(controller.sessions.len(), MAX_SESSIONS);
    }
}
