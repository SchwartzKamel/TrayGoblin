//! Content-free aggregate wire types and the shipped pi publisher's v1 schema.

use serde::{Deserialize, Deserializer, Serialize};

use crate::config::Source;

pub const MAX_PI_BYTES: usize = 16 * 1024;
pub const MAX_SESSIONS: usize = 64;
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
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
    pub instance_id: String,
    pub model: Option<String>,
    pub directory: Option<String>,
    pub state: SessionState,
    pub attention: Option<AttentionEpisode>,
    pub run_started_at_ms: Option<u64>,
    pub last_run_duration_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Diagnostic {
    SourceUnavailable,
    SessionUnavailable,
    SessionOverflow,
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
        }
    }
}
