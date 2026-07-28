use std::{path::PathBuf, time::Duration};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StatusState {
    #[default]
    Idle,
    Generating,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttentionReason {
    StateUnavailable,
    ToolFailed,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StatusSnapshot {
    pub state: StatusState,
    pub model: Option<String>,
    pub repository: Option<String>,
    pub active_directory: Option<PathBuf>,
    pub last_turn_duration: Option<Duration>,
    pub attention_reason: Option<AttentionReason>,
}

#[cfg(test)]
mod tests {
    use super::{StatusSnapshot, StatusState};

    // This protects the no-session startup state consumed by the future tray shell.
    #[test]
    fn snapshot_defaults_to_idle() {
        let snapshot = StatusSnapshot::default();

        assert_eq!(snapshot.state, StatusState::Idle);
        assert_eq!(snapshot.attention_reason, None);
    }
}
