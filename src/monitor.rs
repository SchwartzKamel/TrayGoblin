use std::{
    collections::HashMap,
    fs,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, FixedOffset};

use crate::{
    events::{EventKind, ParsedEvent, SessionEvent, parse_event_line},
    session::parse_workspace_metadata,
    status::{AttentionReason, StatusSnapshot, StatusState},
};

const EVENTS_FILE_NAME: &str = "events.jsonl";
const WORKSPACE_FILE_NAME: &str = "workspace.yaml";
const LOCK_FILE_PREFIX: &str = "inuse.";
const LOCK_FILE_SUFFIX: &str = ".lock";

/// Tracks the in-progress and most recently completed turn for the currently
/// selected session. Only state, timestamps, model, and success ever land
/// here; never prompt, response, tool, token, or credential content.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct TurnState {
    turn_active: bool,
    turn_start: Option<DateTime<FixedOffset>>,
    turn_failed: bool,
    last_model: Option<String>,
    last_turn_duration_ms: Option<u64>,
}

/// Polls Copilot CLI's local session-state directory and derives a
/// content-free [`StatusSnapshot`]. Caches per-file byte offsets so repeated
/// polls read only appended bytes instead of rescanning whole files.
pub struct SessionMonitor {
    session_root: PathBuf,
    active_session: Option<PathBuf>,
    offsets: HashMap<PathBuf, u64>,
    turn_state: TurnState,
    lines_processed: u64,
}

impl SessionMonitor {
    pub fn new(session_root: impl Into<PathBuf>) -> Self {
        Self {
            session_root: session_root.into(),
            active_session: None,
            offsets: HashMap::new(),
            turn_state: TurnState::default(),
            lines_processed: 0,
        }
    }

    /// Total event lines read across the monitor's lifetime. Exposed so
    /// tests can prove offset caching: appending N lines and polling again
    /// must only advance this counter by N, never by the whole file again.
    pub fn lines_processed(&self) -> u64 {
        self.lines_processed
    }

    pub fn active_session_path(&self) -> Option<&Path> {
        self.active_session.as_deref()
    }

    /// Selects the newest active session (if any) and derives the current
    /// status snapshot from any events appended since the last poll.
    pub fn poll(&mut self) -> StatusSnapshot {
        let Some(session_dir) = find_active_session(&self.session_root) else {
            self.active_session = None;
            self.offsets.clear();
            self.turn_state = TurnState::default();
            return StatusSnapshot::default();
        };

        if self.active_session.as_deref() != Some(session_dir.as_path()) {
            // Rebuild state from the selected session so switching away and
            // back never combines a reset state machine with an EOF offset.
            self.offsets.clear();
            self.turn_state = TurnState::default();
            self.active_session = Some(session_dir.clone());
        }

        self.poll_session(&session_dir)
    }

    fn poll_session(&mut self, session_dir: &Path) -> StatusSnapshot {
        let workspace = read_workspace_metadata(session_dir);
        let repository = workspace.as_ref().and_then(|w| w.repository.clone());
        let active_directory = workspace.as_ref().and_then(|w| w.active_directory.clone());

        let events_path = session_dir.join(EVENTS_FILE_NAME);
        match self.read_new_events(&events_path) {
            Ok(new_events) => {
                for event in new_events {
                    self.apply_event(event);
                }
            }
            Err(_) => {
                // The selected session is active but its state file cannot
                // be read: report Attention needed with a content-free
                // reason instead of guessing at stale turn state.
                return StatusSnapshot {
                    state: StatusState::Error,
                    model: self.turn_state.last_model.clone(),
                    repository,
                    active_directory,
                    last_turn_duration: self
                        .turn_state
                        .last_turn_duration_ms
                        .map(std::time::Duration::from_millis),
                    attention_reason: Some(AttentionReason::StateUnavailable),
                };
            }
        }

        let state = if self.turn_state.turn_failed {
            StatusState::Error
        } else if self.turn_state.turn_active {
            StatusState::Generating
        } else {
            StatusState::Idle
        };

        StatusSnapshot {
            state,
            model: self.turn_state.last_model.clone(),
            repository,
            active_directory,
            last_turn_duration: self
                .turn_state
                .last_turn_duration_ms
                .map(std::time::Duration::from_millis),
            attention_reason: (state == StatusState::Error).then_some(AttentionReason::ToolFailed),
        }
    }

    fn apply_event(&mut self, event: SessionEvent) {
        match event.kind {
            EventKind::TurnStart => {
                self.turn_state.turn_active = true;
                self.turn_state.turn_start = event.timestamp;
                self.turn_state.turn_failed = false;
            }
            EventKind::TurnEnd => {
                self.turn_state.last_turn_duration_ms =
                    match (self.turn_state.turn_start, event.timestamp) {
                        (Some(start), Some(end)) => (end - start)
                            .to_std()
                            .ok()
                            .map(|duration| duration.as_millis() as u64),
                        _ => None,
                    };
                self.turn_state.turn_active = false;
                self.turn_state.turn_start = None;
            }
            EventKind::ModelChange | EventKind::AssistantMessage => {
                if let Some(model) = event.model {
                    self.turn_state.last_model = Some(model);
                }
            }
            EventKind::ToolExecutionComplete => {
                if self.turn_state.turn_active && event.success == Some(false) {
                    self.turn_state.turn_failed = true;
                }
            }
        }
    }

    /// Reads only the bytes appended to `path` since the last successful
    /// read, using the cached offset for that path. Falls back to reading
    /// from the start if the file shrank (e.g. rotated) since last time.
    fn read_new_events(&mut self, path: &Path) -> io::Result<Vec<SessionEvent>> {
        let mut file = fs::File::open(path)?;
        let file_len = file.metadata()?.len();

        let cached_offset = self.offsets.get(path).copied().unwrap_or(0);
        let file_was_truncated = cached_offset > file_len;
        let start_offset = if file_was_truncated {
            self.turn_state = TurnState::default();
            0
        } else {
            cached_offset
        };

        file.seek(SeekFrom::Start(start_offset))?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        // A writer may be in the middle of appending a JSONL record. Advance
        // only through the last newline so the incomplete record is retried
        // on the next poll instead of being skipped permanently.
        let complete_len = buffer.rfind('\n').map_or(0, |index| index + 1);
        self.offsets
            .insert(path.to_path_buf(), start_offset + complete_len as u64);

        let events = buffer[..complete_len]
            .lines()
            .inspect(|_| self.lines_processed += 1)
            .filter_map(|line| match parse_event_line(line) {
                ParsedEvent::Known(event) => Some(event),
                ParsedEvent::Unsupported | ParsedEvent::Malformed(_) => None,
            })
            .collect();

        Ok(events)
    }
}

/// Finds the newest active session directory below `session_root`. A
/// session is active when it contains an `inuse.*.lock` marker; if several
/// sessions are active, the newest lock wins.
fn find_active_session(session_root: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(session_root).ok()?;

    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| newest_lock_time(&path).map(|time| (time, path)))
        .max_by(|(left_time, left_path), (right_time, right_path)| {
            left_time
                .cmp(right_time)
                .then_with(|| left_path.cmp(right_path))
        })
        .map(|(_, path)| path)
}

/// Returns the newest modification time among a session's active lock
/// markers. Lock contents are intentionally never opened or modeled.
fn newest_lock_time(session_dir: &Path) -> Option<SystemTime> {
    let entries = fs::read_dir(session_dir).ok()?;

    entries
        .flatten()
        .filter(|entry| is_lock_file_name(&entry.file_name().to_string_lossy()))
        .filter_map(|entry| entry.metadata().ok()?.modified().ok())
        .max()
}

fn is_lock_file_name(name: &str) -> bool {
    name.starts_with(LOCK_FILE_PREFIX) && name.ends_with(LOCK_FILE_SUFFIX)
}

fn read_workspace_metadata(session_dir: &Path) -> Option<crate::session::WorkspaceMetadata> {
    let contents = fs::read_to_string(session_dir.join(WORKSPACE_FILE_NAME)).ok()?;
    parse_workspace_metadata(&contents).ok()
}
