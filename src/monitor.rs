use std::{
    collections::HashMap,
    fs,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
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
            EventKind::SessionError => {}
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

use crate::{
    attention::{
        AttentionEpisode, AttentionSnapshot, Capabilities, Diagnostic, EpisodeReason, MAX_PI_BYTES,
        MAX_SESSIONS, MAX_SOURCE_MEMBERSHIP, SessionSnapshot, SessionState, SourceMembership,
        SourceSnapshot, is_uuid, parse_pi_status,
    },
    config::{ConfigError, Provider, Source, validate_sources},
};

const MAX_DIRECTORY_ENTRIES: usize = MAX_SOURCE_MEMBERSHIP;
const MAX_EVENT_READ: usize = 256 * 1024;
const MAX_WORKSPACE_BYTES: usize = 16 * 1024;

/// Synchronous aggregate core. The UI must use M3's isolated source workers,
/// not call this filesystem polling path from a window procedure.
pub struct AttentionMonitor {
    sources: Vec<SourceMonitor>,
}

impl AttentionMonitor {
    pub fn new(sources: Vec<Source>) -> Result<Self, ConfigError> {
        validate_sources(&sources)?;
        Ok(Self {
            sources: sources.into_iter().map(SourceMonitor::new).collect(),
        })
    }

    pub fn poll(&mut self) -> Result<AttentionSnapshot, std::time::SystemTimeError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u64;
        Ok(self.poll_at(now))
    }

    pub fn poll_at(&mut self, now_ms: u64) -> AttentionSnapshot {
        self.poll_with_clock(&|| now_ms)
    }

    /// Sample observation time after filesystem reads, independently of request age.
    pub fn poll_with_clock(&mut self, clock: &dyn Fn() -> u64) -> AttentionSnapshot {
        AttentionSnapshot {
            sources: self
                .sources
                .iter_mut()
                .map(|s| s.poll_with_clock(clock))
                .collect(),
        }
    }

    pub fn lines_processed(&self) -> u64 {
        self.sources
            .iter()
            .map(SourceMonitor::lines_processed)
            .sum()
    }
}

/// One configured source's bounded, independent parser/offset state.
pub struct SourceMonitor {
    source: Source,
    copilot: HashMap<String, CopilotSession>,
    pi_timestamps: HashMap<String, u64>,
    lines_processed: u64,
}

impl SourceMonitor {
    fn new(source: Source) -> Self {
        Self {
            source,
            copilot: HashMap::new(),
            pi_timestamps: HashMap::new(),
            lines_processed: 0,
        }
    }

    pub fn lines_processed(&self) -> u64 {
        self.lines_processed
    }

    pub fn poll_at(&mut self, now_ms: u64) -> SourceSnapshot {
        self.poll_with_clock(&|| now_ms)
    }

    fn poll_with_clock(&mut self, clock: &dyn Fn() -> u64) -> SourceSnapshot {
        let mut result = SourceSnapshot {
            source: self.source.clone(),
            capabilities: Capabilities {
                input_detection: self.source.kind == Provider::Pi,
                session_errors: true,
            },
            sessions: Vec::new(),
            unavailable_pi_statuses: 0,
            diagnostics: Vec::new(),
            membership: SourceMembership::default(),
        };
        let root = match fs::canonicalize(&self.source.path) {
            Ok(root) => root,
            Err(error) => {
                if !(self.source.implicit
                    && self.source.kind == Provider::Pi
                    && error.kind() == io::ErrorKind::NotFound)
                {
                    result.diagnose(Diagnostic::SourceUnavailable);
                }
                // Keep bounded known Copilot offsets/error provenance through
                // temporary source loss; the returned snapshot is still unavailable.
                self.pi_timestamps.clear();
                return result;
            }
        };
        let root = match SourceDirectory::open(&root) {
            Ok(root) => root,
            Err(_) => {
                result.diagnose(Diagnostic::SourceUnavailable);
                self.pi_timestamps.clear();
                return result;
            }
        };
        let entries = match fs::read_dir(&root.path) {
            Ok(entries) => entries,
            Err(_) => {
                result.diagnose(Diagnostic::SourceUnavailable);
                self.pi_timestamps.clear();
                return result;
            }
        };
        let mut candidates = Vec::new();
        let mut unavailable_pi = Vec::new();
        let mut pi_timestamps = HashMap::new();
        result.membership.complete = true;
        for (index, entry) in entries.enumerate() {
            if index >= MAX_DIRECTORY_ENTRIES {
                result.diagnose(Diagnostic::SessionOverflow);
                result.membership.complete = false;
                break;
            }
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    result.diagnose(Diagnostic::SessionUnavailable);
                    result.membership.complete = false;
                    continue;
                }
            };
            let name = match entry.file_name().into_string() {
                Ok(name) => name,
                Err(_) => continue,
            };
            let kind = match entry.file_type() {
                Ok(kind) => kind,
                Err(_) => {
                    result.diagnose(Diagnostic::SessionUnavailable);
                    result.membership.complete = false;
                    continue;
                }
            };
            match self.source.kind {
                Provider::Pi => {
                    // Filter names and regular files BEFORE opening anything.
                    if !name.strip_suffix(".json").is_some_and(is_uuid) {
                        continue;
                    }
                    result.membership.instances.insert(
                        name.strip_suffix(".json")
                            .expect("checked filename")
                            .to_owned(),
                    );
                    if !kind.is_file() {
                        result.diagnose(Diagnostic::UnsafeEntry);
                        continue;
                    }
                    match self.poll_pi(&name, &entry.path(), clock) {
                        Ok((snapshot, timestamp)) => {
                            if result.sessions.len() == MAX_SESSIONS {
                                result.diagnose(Diagnostic::SessionOverflow);
                            } else {
                                pi_timestamps.insert(name, timestamp);
                                result.sessions.push(snapshot);
                            }
                        }
                        Err(diagnostic) => {
                            result.diagnose(diagnostic);
                            result.unavailable_pi_statuses += 1;
                            if unavailable_pi.len() < MAX_SESSIONS {
                                unavailable_pi.push(name);
                            }
                        }
                    }
                }
                Provider::Copilot => {
                    if kind.is_symlink() {
                        result.diagnose(Diagnostic::UnsafeEntry);
                        continue;
                    }
                    if !kind.is_dir() {
                        continue;
                    }
                    if !bounded_identity(&name, 128) {
                        result.diagnose(Diagnostic::UnsafeEntry);
                        continue;
                    }
                    let directory = match SourceDirectory::open(&entry.path()) {
                        Ok(directory) => directory,
                        Err(_) => {
                            result.diagnose(Diagnostic::UnsafeEntry);
                            result.membership.complete = false;
                            continue;
                        }
                    };
                    match active_lock_identity(&directory.path) {
                        Ok(Some(lock)) => {
                            result.membership.instances.insert(format!("{name}:{lock}"));
                            if candidates.len() == MAX_SESSIONS {
                                result.diagnose(Diagnostic::SessionOverflow);
                                result.membership.complete = false;
                                break;
                            }
                            candidates.push((
                                name,
                                directory.path.clone(),
                                Some(lock),
                                Some(directory),
                            ));
                        }
                        Ok(None) => {}
                        Err(_) => {
                            result.diagnose(Diagnostic::SessionUnavailable);
                            result.membership.complete = false;
                        }
                    }
                    continue;
                }
            }
        }
        // Membership, not the selected/validated rows, proves absence. Prune
        // before admission so complete recovery immediately frees parser capacity.
        self.copilot.retain(|name, session| {
            !result.membership.complete
                || result
                    .membership
                    .instances
                    .contains(&format!("{name}:{}", session.lock))
        });
        candidates.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, path, lock, _directory_guard) in candidates {
            let Some(lock) = lock else { continue };
            if !self.copilot.contains_key(&name) && self.copilot.len() == MAX_SESSIONS {
                // Unknown old instances keep their provenance, but cannot grow
                // this cache or let an untracked new session claim healthy state.
                result.diagnose(Diagnostic::SessionOverflow);
                result
                    .sessions
                    .push(unavailable_session(&name, &format!("{name}:{lock}")));
                continue;
            }
            let session = self.copilot.entry(name.clone()).or_default();
            if session.lock != lock {
                *session = CopilotSession {
                    lock,
                    ..CopilotSession::default()
                };
            }
            let mut snapshot = unavailable_session(&name, &format!("{name}:{}", session.lock));
            match session.poll(&path, clock, &mut self.lines_processed, &mut result) {
                Ok(()) => {
                    snapshot.model = session.model.clone();
                    snapshot.directory = session.directory.clone();
                    snapshot.state = session.state;
                    snapshot.attention = session.attention.clone();
                    snapshot.run_started_at_ms = session.run_started_at;
                    snapshot.last_run_duration_ms = session.duration;
                }
                Err(_) => result.diagnose(Diagnostic::SessionUnavailable),
            }
            result.sessions.push(snapshot);
        }
        if self.source.kind == Provider::Pi {
            let remaining = MAX_SESSIONS - result.sessions.len();
            if result.unavailable_pi_statuses > remaining {
                result.diagnose(Diagnostic::SessionOverflow);
            }
            // Unavailable placeholders use only spare capacity; residue never displaces live attention.
            for name in unavailable_pi.into_iter().take(remaining) {
                if let Some(timestamp) = self.pi_timestamps.get(&name) {
                    pi_timestamps.insert(name.clone(), *timestamp);
                }
                let id = name
                    .strip_suffix(".json")
                    .expect("candidate name was checked");
                result.sessions.push(unavailable_session(id, id));
            }
            self.pi_timestamps = pi_timestamps;
            result
                .sessions
                .sort_by(|a, b| a.instance_id.cmp(&b.instance_id));
        }
        result
    }

    fn poll_pi(
        &self,
        name: &str,
        path: &Path,
        clock: &dyn Fn() -> u64,
    ) -> Result<(SessionSnapshot, u64), Diagnostic> {
        let id = name
            .strip_suffix(".json")
            .expect("candidate name was checked");
        let status = read_bounded_regular(path, MAX_PI_BYTES)
            .map_err(|error| {
                if error.kind() == io::ErrorKind::FileTooLarge {
                    Diagnostic::OversizedPiStatus
                } else {
                    Diagnostic::SessionUnavailable
                }
            })
            .and_then(|bytes| parse_pi_status(&bytes, clock()))
            .and_then(|status| {
                if status.instance_id != id {
                    Err(Diagnostic::InvalidPiStatus)
                } else if self
                    .pi_timestamps
                    .get(name)
                    .is_some_and(|t| status.updated_at < *t)
                {
                    Err(Diagnostic::RegressedPiStatus)
                } else {
                    Ok(status)
                }
            });
        status.map(|status| {
            let timestamp = status.updated_at;
            (status.snapshot(), timestamp)
        })
    }
}

fn unavailable_session(session: &str, instance: &str) -> SessionSnapshot {
    SessionSnapshot {
        session_id: session.into(),
        instance_id: instance.into(),
        model: None,
        directory: None,
        state: SessionState::Unavailable,
        attention: None,
        run_started_at_ms: None,
        last_run_duration_ms: None,
        publisher_expires_at_ms: None,
    }
}

fn bounded_identity(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

struct SourceDirectory {
    _file: fs::File,
    path: PathBuf,
}

impl SourceDirectory {
    fn open(path: &Path) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.file_type().is_dir() || is_reparse_point(&metadata) {
            return Err(io::Error::other("not a regular source directory"));
        }
        let mut options = fs::OpenOptions::new();
        options.read(true);
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(0x10000 | 0x20000 | 0x800); // O_DIRECTORY | O_NOFOLLOW | O_NONBLOCK
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // Hold the directory against rename/replacement throughout this poll.
            options
                .custom_flags(0x02000000 | 0x00200000)
                .share_mode(0x1 | 0x2);
        }
        let file = options.open(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_dir() || is_reparse_point(&metadata) {
            return Err(io::Error::other("not a regular source directory"));
        }
        #[cfg(target_os = "linux")]
        let path = {
            use std::os::fd::AsRawFd;
            // Descendant opens stay anchored to this directory even if its name is replaced.
            PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()))
        };
        #[cfg(not(target_os = "linux"))]
        let path = path.to_path_buf();
        Ok(Self { _file: file, path })
    }
}

fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

/// Opening reparse points themselves on Windows and refusing final symlinks on
/// Linux prevents a replaced status/metadata file from escaping the source.
fn open_regular(path: &Path) -> io::Result<fs::File> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(io::Error::other("not a regular source file"));
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(0x20000 | 0x800); // O_NOFOLLOW | O_NONBLOCK
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || is_reparse_point(&metadata) {
        return Err(io::Error::other("not a regular source file"));
    }
    Ok(file)
}

fn read_bounded_regular(path: &Path, max: usize) -> io::Result<Vec<u8>> {
    read_bounded(open_regular(path)?, max)
}

fn read_bounded(reader: impl Read, max: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(max as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > max {
        return Err(io::Error::new(
            io::ErrorKind::FileTooLarge,
            "source file exceeds limit",
        ));
    }
    Ok(bytes)
}

fn active_lock_identity(directory: &Path) -> io::Result<Option<String>> {
    let mut lock = None;
    for (index, entry) in fs::read_dir(directory)?.enumerate() {
        if index >= MAX_DIRECTORY_ENTRIES {
            return Err(io::Error::other("session directory exceeds limit"));
        }
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !is_lock_file_name(name) {
            continue;
        }
        if !bounded_identity(name, 128) || !entry.file_type()?.is_file() {
            return Err(io::Error::other("invalid lock marker"));
        }
        let metadata = fs::symlink_metadata(entry.path())?;
        if !metadata.is_file() || is_reparse_point(&metadata) {
            return Err(io::Error::other("invalid lock marker"));
        }
        let created = metadata
            .created()
            .or_else(|_| metadata.modified())?
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let identity = format!("{name}:{created}");
        if lock.as_ref().is_none_or(|current| &identity < current) {
            lock = Some(identity);
        }
    }
    Ok(lock)
}

#[cfg(test)]
mod attention_io_tests {
    use super::*;
    use std::io::Write;

    // The final available ID is issued exactly once. Exhaustion stays a typed,
    // visible failure forever, not a wrap to a previously acknowledged ID.
    #[test]
    fn copilot_episode_allocator_never_recycles_at_exhaustion() {
        let sequence = AtomicU64::new(u64::MAX - 1);
        assert_eq!(allocate_copilot_episode(&sequence), Ok(u64::MAX));
        for _ in 0..3 {
            assert_eq!(
                allocate_copilot_episode(&sequence),
                Err(Diagnostic::EpisodeIdentityExhausted)
            );
            assert_eq!(sequence.load(Ordering::Relaxed), u64::MAX);
        }
    }

    // Real lock/log boundaries prove retained source-loss caches stay bounded,
    // resume unchanged provenance, and still prune confirmed removal/replacement.
    #[test]
    fn copilot_source_loss_retains_bounded_cache_until_confirmed_removal() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-scratch/attention-copilot-cache")
            .join(std::process::id().to_string());
        let source = root.join("source");
        fs::create_dir_all(&source).unwrap();
        for i in 0..MAX_SESSIONS {
            let session = source.join(format!("session-{i:02}"));
            fs::create_dir(&session).unwrap();
            fs::write(session.join("inuse.1.lock"), "").unwrap();
            fs::write(session.join(EVENTS_FILE_NAME), "").unwrap();
        }
        let config = crate::config::MonitorConfig::parse(
            &serde_json::json!({"sources":[{"id":"copilot","label":"Synthetic","kind":"copilot","path":source}]}).to_string()
        ).unwrap();
        let mut monitor = AttentionMonitor::new(config.resolved_sources().unwrap()).unwrap();
        monitor.poll_at(20_000);
        fs::write(
            source.join("session-00/events.jsonl"),
            "{\"type\":\"session.error\"}\n",
        )
        .unwrap();
        let first = monitor.poll_at(21_000).sources[0].sessions[0].clone();
        assert!(first.attention.as_ref().unwrap().notify);
        let lines = monitor.lines_processed();
        for cycle in 0..10 {
            fs::rename(&source, root.join("offline")).unwrap();
            let unavailable = monitor.poll_at(22_000 + cycle * 2);
            assert_eq!(
                unavailable.sources[0].diagnostics,
                [Diagnostic::SourceUnavailable]
            );
            assert!(unavailable.sources[0].sessions.is_empty());
            assert_eq!(monitor.sources[0].copilot.len(), MAX_SESSIONS);
            fs::rename(root.join("offline"), &source).unwrap();
            let restored = monitor.poll_at(22_001 + cycle * 2);
            assert_eq!(restored.sources[0].sessions[0], first);
            assert_eq!(
                monitor.lines_processed(),
                lines,
                "unchanged log is not replayed"
            );
            assert_eq!(monitor.sources[0].copilot.len(), MAX_SESSIONS);
        }
        fs::remove_file(source.join("session-01/inuse.1.lock")).unwrap();
        monitor.poll_at(23_000);
        assert_eq!(monitor.sources[0].copilot.len(), MAX_SESSIONS - 1);
        assert!(!monitor.sources[0].copilot.contains_key("session-01"));
        fs::rename(
            source.join("session-00/inuse.1.lock"),
            source.join("session-00/inuse.2.lock"),
        )
        .unwrap();
        let replaced = monitor.poll_at(24_000).sources[0].sessions[0].clone();
        assert_ne!(replaced.instance_id, first.instance_id);
        assert!(
            !replaced.attention.as_ref().unwrap().notify,
            "new lock initializes quiet history"
        );
        assert_ne!(
            replaced.attention.as_ref().unwrap().id,
            first.attention.as_ref().unwrap().id
        );
        fs::remove_dir_all(&root).unwrap();
        monitor.poll_at(25_000); // Source loss is still unknown membership.
        assert_eq!(monitor.sources[0].copilot.len(), MAX_SESSIONS - 1);
        fs::create_dir_all(&source).unwrap();
        assert!(monitor.poll_at(26_000).sources[0].membership.complete);
        assert!(monitor.sources[0].copilot.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    // Partial-scan churn cannot evict known provenance or grow the 64-parser
    // cache. Refused admission is visibly unavailable; positive replacement and
    // a complete recovery still reclaim state and admit previously blocked rows.
    #[test]
    fn copilot_partial_membership_bounds_admission_and_reclaims_absence() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-scratch/attention-copilot-partial-cache")
            .join(std::process::id().to_string());
        fs::create_dir_all(&root).unwrap();
        let source = root.join("source");
        fs::create_dir(&source).unwrap();
        let create = |name: &str, events: &str| {
            let session = source.join(name);
            fs::create_dir(&session).unwrap();
            fs::write(session.join("inuse.1.lock"), "").unwrap();
            fs::write(session.join(EVENTS_FILE_NAME), events).unwrap();
        };
        for i in 0..MAX_SESSIONS {
            create(&format!("known-{i:02}"), "");
        }
        let config = crate::config::MonitorConfig::parse(
            &serde_json::json!({"sources":[{"id":"copilot","label":"Synthetic","kind":"copilot","path":source}]}).to_string()
        ).unwrap();
        let mut monitor = AttentionMonitor::new(config.resolved_sources().unwrap()).unwrap();
        assert!(monitor.poll_at(20_000).sources[0].membership.complete);
        for i in 0..2 {
            fs::write(
                source.join(format!("known-{i:02}/{EVENTS_FILE_NAME}")),
                "{\"type\":\"session.error\"}\n",
            )
            .unwrap();
        }
        let first = monitor.poll_at(21_000);
        let known = first.sources[0].sessions[0].clone();
        let replaced = first.sources[0].sessions[1].clone();
        assert!(known.attention.as_ref().unwrap().notify);
        assert!(replaced.attention.as_ref().unwrap().notify);
        assert_eq!(monitor.lines_processed(), 2);
        for i in 0..MAX_SESSIONS {
            fs::create_dir(source.join(format!("known-{i:02}/inuse.transient.lock"))).unwrap();
        }
        for generation in 0..10 {
            for i in 0..8 {
                create(
                    &format!("new-{generation:02}-{i}"),
                    "{\"type\":\"session.error\"}\n",
                );
            }
            let partial = monitor.poll_at(22_000 + generation);
            let result = &partial.sources[0];
            assert!(!result.membership.complete);
            assert_eq!(result.membership.instances.len(), 8);
            assert!(result.diagnostics.contains(&Diagnostic::SessionUnavailable));
            assert!(result.diagnostics.contains(&Diagnostic::SessionOverflow));
            assert_eq!(result.sessions.len(), 8);
            assert!(
                result
                    .sessions
                    .iter()
                    .all(|s| s.state == SessionState::Unavailable && s.attention.is_none())
            );
            assert_eq!(monitor.sources[0].copilot.len(), MAX_SESSIONS);
            assert_eq!(
                monitor.lines_processed(),
                2,
                "refused parsers must not consume logs"
            );
            assert_eq!(
                monitor.sources[0].copilot["known-00"].attention,
                known.attention
            );
            if generation < 9 {
                for i in 0..8 {
                    fs::remove_dir_all(source.join(format!("new-{generation:02}-{i}"))).unwrap();
                }
            }
        }
        fs::remove_dir(source.join("known-00/inuse.transient.lock")).unwrap();
        fs::remove_dir(source.join("known-01/inuse.transient.lock")).unwrap();
        fs::rename(
            source.join("known-01/inuse.1.lock"),
            source.join("known-01/inuse.2.lock"),
        )
        .unwrap();
        let partial = monitor.poll_at(23_000);
        let result = &partial.sources[0];
        assert!(!result.membership.complete);
        assert_eq!(
            result.sessions[0], known,
            "known parser resumes at capacity"
        );
        assert_ne!(result.sessions[1].instance_id, replaced.instance_id);
        let replacement = result.sessions[1].attention.as_ref().unwrap();
        assert!(
            !replacement.notify,
            "positive lock replacement starts quiet history"
        );
        assert_ne!(replacement.id, replaced.attention.as_ref().unwrap().id);
        assert_eq!(monitor.sources[0].copilot.len(), MAX_SESSIONS);
        assert_eq!(monitor.lines_processed(), 3);

        // Only a complete membership scan confirms that these old instances are
        // genuinely absent. Reclaim them BEFORE admitting the eight new parsers.
        for i in 2..MAX_SESSIONS {
            fs::remove_dir_all(source.join(format!("known-{i:02}"))).unwrap();
        }
        let recovered = monitor.poll_at(24_000);
        let result = &recovered.sources[0];
        assert!(result.membership.complete);
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.sessions.len(), 10);
        assert_eq!(result.sessions[0], known);
        assert_eq!(result.sessions[1], partial.sources[0].sessions[1]);
        assert_eq!(monitor.sources[0].copilot.len(), 10);
        assert!(!monitor.sources[0].copilot.contains_key("known-02"));
        assert!(
            result.sessions[2..]
                .iter()
                .all(|s| s.state == SessionState::Error && !s.attention.as_ref().unwrap().notify)
        );
        assert_eq!(monitor.lines_processed(), 11);
        fs::OpenOptions::new()
            .append(true)
            .open(source.join("new-09-0/events.jsonl"))
            .unwrap()
            .write_all(b"{\"type\":\"session.error\"}\n")
            .unwrap();
        let fresh = monitor.poll_at(25_000);
        let new = fresh.sources[0].sessions[2].attention.as_ref().unwrap();
        assert!(new.notify);
        assert_ne!(new.id, result.sessions[2].attention.as_ref().unwrap().id);
        assert_eq!(monitor.lines_processed(), 12);
        fs::remove_dir_all(root).unwrap();
    }

    // Proves stale-saturation recovery never grows retained timestamp storage beyond 64 identities.
    #[test]
    fn pi_timestamp_storage_stays_bounded_across_stale_saturation() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-scratch/attention-timestamps")
            .join(std::process::id().to_string());
        fs::create_dir_all(&root).unwrap();
        let config = crate::config::MonitorConfig::parse(
            &serde_json::json!({"sources":[{"id":"pi","label":"Synthetic","kind":"pi","path":root}]})
                .to_string(),
        )
        .unwrap();
        let mut monitor = AttentionMonitor::new(config.resolved_sources().unwrap()).unwrap();
        for generation in 0..3 {
            for i in 0..128 {
                let mut value: serde_json::Value =
                    serde_json::from_str(include_str!("../tests/fixtures/attention/pi-v1.json"))
                        .unwrap();
                let id = format!("{i:08x}-1111-4111-8111-111111111111");
                value["instanceId"] = serde_json::json!(id);
                value["runStartedAt"] = serde_json::Value::Null;
                value["updatedAt"] = serde_json::json!(if (i < 64) == (generation % 2 == 0) {
                    20_000 + generation
                } else {
                    0
                });
                fs::write(root.join(format!("{id}.json")), value.to_string()).unwrap();
            }
            let snapshot = monitor.poll_at(20_000 + generation);
            assert_eq!(snapshot.sources[0].sessions.len(), MAX_SESSIONS);
            assert!(
                snapshot.sources[0]
                    .sessions
                    .iter()
                    .all(|session| session.state == SessionState::InputRequired)
            );
            assert_eq!(snapshot.sources[0].unavailable_pi_statuses, 64);
            assert_eq!(monitor.sources[0].pi_timestamps.len(), MAX_SESSIONS);
        }
        fs::remove_dir_all(&root).unwrap();
        assert!(monitor.poll_at(20_003).sources[0].sessions.is_empty());
        assert!(monitor.sources[0].pi_timestamps.is_empty());
    }

    // Measures actual bytes consumed, not just rejection, to pin the 16 KiB + 1 read cap.
    #[test]
    fn pi_reader_consumes_only_one_byte_beyond_the_limit() {
        let mut reader = io::Cursor::new(vec![b'x'; MAX_PI_BYTES + 2]);
        let error = read_bounded(&mut reader, MAX_PI_BYTES).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::FileTooLarge);
        assert_eq!(reader.position(), (MAX_PI_BYTES + 1) as u64);
        let exact = read_bounded(io::Cursor::new(vec![b'x'; MAX_PI_BYTES]), MAX_PI_BYTES).unwrap();
        assert_eq!(exact.len(), MAX_PI_BYTES);
    }

    // Replaces an opened directory with an escaping link; descendant reads must retain the original inode.
    #[cfg(target_os = "linux")]
    #[test]
    fn opened_source_directory_remains_pinned_during_name_replacement() {
        use std::os::unix::fs::symlink;
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-scratch/attention-pinned")
            .join(std::process::id().to_string());
        fs::create_dir_all(root.join("source")).unwrap();
        fs::create_dir(root.join("outside")).unwrap();
        fs::write(root.join("source/status"), "original").unwrap();
        fs::write(root.join("outside/status"), "escaped").unwrap();
        let directory = SourceDirectory::open(&root.join("source")).unwrap();
        fs::rename(root.join("source"), root.join("renamed")).unwrap();
        symlink(root.join("outside"), root.join("source")).unwrap();
        assert_eq!(
            read_bounded_regular(&directory.path.join("status"), MAX_PI_BYTES).unwrap(),
            b"original"
        );
        drop(directory);
        fs::remove_dir_all(root).unwrap();
    }
}

// Controllers can outlive ANY parser, source cache, worker or monitor. IDs must
// therefore be opaque and non-recycling for the whole process, not per session.
static COPILOT_EPISODES: AtomicU64 = AtomicU64::new(0);

fn allocate_copilot_episode(sequence: &AtomicU64) -> Result<u64, Diagnostic> {
    sequence
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |last| {
            last.checked_add(1)
        })
        .map(|previous| previous + 1)
        .map_err(|_| Diagnostic::EpisodeIdentityExhausted)
}

struct CopilotSession {
    lock: String,
    offset: u64,
    baseline: Option<u64>,
    discarding: bool,
    state: SessionState,
    model: Option<String>,
    directory: Option<String>,
    attention: Option<AttentionEpisode>,
    run_started_at: Option<u64>,
    duration: Option<u64>,
    event_file_identity: Option<String>,
}

impl Default for CopilotSession {
    fn default() -> Self {
        Self {
            lock: String::new(),
            offset: 0,
            baseline: None,
            discarding: false,
            state: SessionState::Idle,
            model: None,
            directory: None,
            attention: None,
            run_started_at: None,
            duration: None,
            event_file_identity: None,
        }
    }
}

fn file_identity(metadata: &fs::Metadata) -> io::Result<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(format!("{}:{}", metadata.dev(), metadata.ino()))
    }
    #[cfg(not(unix))]
    {
        let created = metadata
            .created()?
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        Ok(created.to_string())
    }
}

impl CopilotSession {
    fn poll(
        &mut self,
        directory: &Path,
        clock: &dyn Fn() -> u64,
        lines: &mut u64,
        result: &mut SourceSnapshot,
    ) -> io::Result<()> {
        self.directory = None;
        match read_bounded_regular(&directory.join(WORKSPACE_FILE_NAME), MAX_WORKSPACE_BYTES) {
            Ok(bytes) => match std::str::from_utf8(&bytes)
                .ok()
                .and_then(|s| parse_workspace_metadata(s).ok())
            {
                Some(metadata) => {
                    self.directory = metadata
                        .active_directory
                        .map(|p| p.to_string_lossy().into_owned())
                        .filter(|s| bounded_identity(s, 2048));
                }
                None => result.diagnose(Diagnostic::InvalidWorkspace),
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => result.diagnose(Diagnostic::InvalidWorkspace),
        }
        let mut file = open_regular(&directory.join(EVENTS_FILE_NAME))?;
        let metadata = file.metadata()?;
        let length = metadata.len();
        let identity = file_identity(&metadata)?;
        if length < self.offset
            || self
                .event_file_identity
                .as_ref()
                .is_some_and(|id| id != &identity)
        {
            let lock = self.lock.clone();
            let directory = self.directory.take();
            *self = Self {
                lock,
                directory,
                ..Self::default()
            };
        }
        self.event_file_identity = Some(identity);
        let baseline = *self.baseline.get_or_insert(length);
        file.seek(SeekFrom::Start(self.offset))?;
        let mut bytes = Vec::new();
        file.take(MAX_EVENT_READ as u64).read_to_end(&mut bytes)?;
        let now = clock();
        let mut start = 0;
        for (index, byte) in bytes.iter().enumerate() {
            if *byte != b'\n' {
                continue;
            }

            let historical = self.offset + (start as u64) < baseline;
            if self.discarding {
                self.discarding = false;
                result.diagnose(Diagnostic::OversizedEvent);
            } else {
                *lines += 1;
                match std::str::from_utf8(&bytes[start..index])
                    .ok()
                    .map(parse_event_line)
                {
                    Some(ParsedEvent::Known(event)) => {
                        let future = event.timestamp.is_some_and(|t| {
                            u64::try_from(t.timestamp_millis()).map_or(true, |time| time > now)
                        });
                        if future {
                            result.diagnose(Diagnostic::MalformedEvent);
                        } else {
                            self.apply(event, !historical).map_err(|diagnostic| {
                                result.diagnose(diagnostic);
                                io::Error::other("Copilot error identity exhausted")
                            })?;
                        }
                    }
                    Some(ParsedEvent::Unsupported) => {}
                    _ => result.diagnose(Diagnostic::MalformedEvent),
                }
            }
            start = index + 1;
        }
        if start == 0 && bytes.len() == MAX_EVENT_READ {
            self.discarding = true;
            self.offset += bytes.len() as u64;
            result.diagnose(Diagnostic::OversizedEvent);
        } else {
            self.offset += start as u64;
        }
        Ok(())
    }

    fn apply(&mut self, event: SessionEvent, notify: bool) -> Result<(), Diagnostic> {
        let timestamp = event
            .timestamp
            .and_then(|t| u64::try_from(t.timestamp_millis()).ok());
        match event.kind {
            EventKind::TurnStart => {
                self.state = SessionState::Working;
                self.run_started_at = timestamp;
                self.attention = None;
            }
            EventKind::TurnEnd => {
                self.duration = self
                    .run_started_at
                    .zip(timestamp)
                    .and_then(|(start, end)| end.checked_sub(start));
                self.run_started_at = None;
                if self.attention.is_none() {
                    self.state = SessionState::Idle;
                }
            }
            EventKind::SessionError => {
                let id = allocate_copilot_episode(&COPILOT_EPISODES)?;
                self.state = SessionState::Error;
                self.attention = Some(AttentionEpisode {
                    id,
                    reason: EpisodeReason::SessionError,
                    notify,
                });
            }
            EventKind::ModelChange | EventKind::AssistantMessage => {
                if let Some(model) = event.model {
                    if bounded_identity(&model, 128) {
                        self.model = Some(model);
                    }
                }
            }
            EventKind::ToolExecutionComplete => {}
        }
        Ok(())
    }
}
