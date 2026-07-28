//! Integration tests for the polling monitor. Each test names the exact
//! state transition or acceptance criterion it protects, per
//! `specs/CONSTITUTION.md`'s testing principle.

use std::{
    fs::{self, FileTimes},
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

use tray_goblin::{
    config::{MAX_POLL_INTERVAL_MS, MIN_POLL_INTERVAL_MS, MonitorConfig},
    monitor::SessionMonitor,
    status::{AttentionReason, StatusState},
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// A private scratch directory under `target/`, never `/tmp`, so tests stay
/// self-contained inside the repository's own (gitignored) build output.
fn scratch_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-scratch")
        .join("monitor")
        .join(
            format!(
                "{name}-{}-{:?}",
                std::process::id(),
                std::time::Instant::now()
            )
            .replace([':', '.', ' '], "_"),
        );
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_session(
    root: &Path,
    session: &str,
    lock_name: &str,
    lock_contents: &str,
    workspace: &str,
    events: &str,
) {
    let dir = root.join(session);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(lock_name), lock_contents).unwrap();
    fs::write(dir.join("workspace.yaml"), workspace).unwrap();
    fs::write(dir.join("events.jsonl"), events).unwrap();
}

fn set_modified(path: &Path, seconds_since_epoch: u64) {
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(
            FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(seconds_since_epoch)),
        )
        .unwrap();
}

// This protects session selection: with two active sessions, the one whose
// lock marker is newest must be polled, not the one modified first on disk.
#[test]
fn selects_the_newest_active_session() {
    let root = scratch_dir("newest-session");
    write_session(
        &root,
        "session-old",
        "inuse.1.lock",
        "2026-07-28T07:00:00Z",
        "repository: old-demo\ncwd: C:/fixture/old\n",
        "{\"type\":\"assistant.turn_start\",\"timestamp\":\"2026-07-28T06:59:00Z\"}\n{\"type\":\"assistant.turn_end\",\"timestamp\":\"2026-07-28T06:59:05Z\"}\n",
    );
    write_session(
        &root,
        "session-new",
        "inuse.2.lock",
        "2026-07-28T07:20:00Z",
        "repository: new-demo\ncwd: C:/fixture/new\n",
        "{\"type\":\"assistant.turn_start\",\"timestamp\":\"2026-07-28T07:20:00Z\"}\n",
    );
    set_modified(&root.join("session-old/inuse.1.lock"), 1_000);
    set_modified(&root.join("session-new/inuse.2.lock"), 2_000);

    let mut monitor = SessionMonitor::new(&root);
    let snapshot = monitor.poll();

    assert_eq!(snapshot.repository.as_deref(), Some("new-demo"));
    assert_eq!(snapshot.state, StatusState::Generating);
    assert_eq!(
        monitor.active_session_path(),
        Some(root.join("session-new").as_path())
    );
}

// This protects the Idle -> Working transition driven by `assistant.turn_start`.
#[test]
fn turn_start_sets_generating() {
    let root = scratch_dir("turn-start");
    write_session(
        &root,
        "session",
        "inuse.1.lock",
        "2026-07-28T07:20:00Z",
        "repository: demo\ncwd: C:/fixture/demo\n",
        "{\"type\":\"assistant.turn_start\",\"timestamp\":\"2026-07-28T07:20:00Z\"}\n",
    );

    let mut monitor = SessionMonitor::new(&root);
    let snapshot = monitor.poll();

    assert_eq!(snapshot.state, StatusState::Generating);
    assert_eq!(snapshot.attention_reason, None);
}

// This protects the Working -> Idle transition and completed-turn duration
// recorded once `assistant.turn_end` arrives after a prior `turn_start`.
#[test]
fn turn_end_sets_idle_and_duration() {
    let root = scratch_dir("turn-end");
    write_session(
        &root,
        "session",
        "inuse.1.lock",
        "2026-07-28T07:20:00Z",
        "repository: demo\ncwd: C:/fixture/demo\n",
        "{\"type\":\"assistant.turn_start\",\"timestamp\":\"2026-07-28T07:20:00Z\"}\n{\"type\":\"assistant.turn_end\",\"timestamp\":\"2026-07-28T07:20:04Z\"}\n",
    );

    let mut monitor = SessionMonitor::new(&root);
    let snapshot = monitor.poll();

    assert_eq!(snapshot.state, StatusState::Idle);
    assert_eq!(snapshot.last_turn_duration, Some(Duration::from_secs(4)));
}

// This protects the Attention-needed transition: a failed tool execution
// during the active turn must override Working immediately.
#[test]
fn failed_tool_sets_attention_needed() {
    let root = scratch_dir("failed-tool");
    write_session(
        &root,
        "session",
        "inuse.1.lock",
        "2026-07-28T07:20:00Z",
        "repository: demo\ncwd: C:/fixture/demo\n",
        "{\"type\":\"assistant.turn_start\",\"timestamp\":\"2026-07-28T07:20:00Z\"}\n{\"type\":\"tool.execution_complete\",\"timestamp\":\"2026-07-28T07:20:01Z\",\"data\":{\"success\":false}}\n",
    );

    let mut monitor = SessionMonitor::new(&root);
    let snapshot = monitor.poll();

    assert_eq!(snapshot.state, StatusState::Error);
    assert_eq!(snapshot.attention_reason, Some(AttentionReason::ToolFailed));
}

// This protects operator visibility: a failed turn remains Attention needed
// after completion and clears only when a new turn starts.
#[test]
fn attention_needed_persists_until_next_turn_start() {
    let root = scratch_dir("sticky-attention");
    write_session(
        &root,
        "session",
        "inuse.1.lock",
        "2026-07-28T07:20:00Z",
        "repository: demo\ncwd: C:/fixture/demo\n",
        "",
    );

    let mut monitor = SessionMonitor::new(&root);
    let events_path = root.join("session").join("events.jsonl");

    fs::write(
        &events_path,
        "{\"type\":\"assistant.turn_start\",\"timestamp\":\"2026-07-28T07:20:00Z\"}\n{\"type\":\"tool.execution_complete\",\"timestamp\":\"2026-07-28T07:20:01Z\",\"data\":{\"success\":false}}\n{\"type\":\"assistant.turn_end\",\"timestamp\":\"2026-07-28T07:20:02Z\"}\n",
    )
    .unwrap();
    let after_turn_end = monitor.poll();
    assert_eq!(after_turn_end.state, StatusState::Error);
    assert_eq!(
        after_turn_end.attention_reason,
        Some(AttentionReason::ToolFailed)
    );
    assert_eq!(
        after_turn_end.last_turn_duration,
        Some(Duration::from_secs(2))
    );

    let mut appended = fs::read_to_string(&events_path).unwrap();
    appended
        .push_str("{\"type\":\"assistant.turn_start\",\"timestamp\":\"2026-07-28T07:21:00Z\"}\n");
    fs::write(&events_path, appended).unwrap();

    let after_next_turn_start = monitor.poll();
    assert_eq!(after_next_turn_start.state, StatusState::Generating);
    assert_eq!(after_next_turn_start.attention_reason, None);
}

// This protects state independently from duration metadata: a turn-start
// event without a timestamp is still an active Working turn.
#[test]
fn turn_start_without_timestamp_sets_generating() {
    let root = scratch_dir("turn-start-no-timestamp");
    write_session(
        &root,
        "session",
        "inuse.1.lock",
        "",
        "repository: demo\n",
        "{\"type\":\"assistant.turn_start\"}\n",
    );

    let mut monitor = SessionMonitor::new(&root);
    let snapshot = monitor.poll();

    assert_eq!(snapshot.state, StatusState::Generating);
}

// This protects tooltip accuracy: if the newest completed turn has no
// measurable start timestamp, it must not inherit an older turn's duration.
#[test]
fn unmeasurable_completed_turn_clears_stale_duration() {
    let root = scratch_dir("unmeasurable-duration");
    write_session(
        &root,
        "session",
        "inuse.1.lock",
        "",
        "repository: demo\n",
        "{\"type\":\"assistant.turn_start\",\"timestamp\":\"2026-07-28T07:20:00Z\"}\n{\"type\":\"assistant.turn_end\",\"timestamp\":\"2026-07-28T07:20:05Z\"}\n",
    );

    let mut monitor = SessionMonitor::new(&root);
    assert_eq!(
        monitor.poll().last_turn_duration,
        Some(Duration::from_secs(5))
    );

    let events_path = root.join("session/events.jsonl");
    let mut appended = fs::read_to_string(&events_path).unwrap();
    appended.push_str(
        "{\"type\":\"assistant.turn_start\"}\n{\"type\":\"assistant.turn_end\",\"timestamp\":\"2026-07-28T07:21:00Z\"}\n",
    );
    fs::write(&events_path, appended).unwrap();

    let snapshot = monitor.poll();
    assert_eq!(snapshot.state, StatusState::Idle);
    assert_eq!(snapshot.last_turn_duration, None);
}

// This protects the file-unreadable path: an active session with no events
// file must report Attention needed instead of a stale or default state.
#[test]
fn missing_events_file_sets_state_unavailable() {
    let root = scratch_dir("missing-events");
    let session_dir = root.join("session");
    fs::create_dir_all(&session_dir).unwrap();
    fs::write(session_dir.join("inuse.1.lock"), "2026-07-28T07:20:00Z").unwrap();
    fs::write(session_dir.join("workspace.yaml"), "repository: demo\n").unwrap();

    let mut monitor = SessionMonitor::new(&root);
    let snapshot = monitor.poll();

    assert_eq!(snapshot.state, StatusState::Error);
    assert_eq!(
        snapshot.attention_reason,
        Some(AttentionReason::StateUnavailable)
    );
}

// This protects Idle as the no-session default: an empty session root (no
// Copilot process active) must never be reported as an error.
#[test]
fn no_active_session_is_idle() {
    let root = scratch_dir("no-session");

    let mut monitor = SessionMonitor::new(&root);
    let snapshot = monitor.poll();

    assert_eq!(snapshot.state, StatusState::Idle);
    assert_eq!(snapshot.attention_reason, None);
}

// This is the performance backpressure for event offsets: appending one line
// and polling again must advance the processed-line counter by exactly one,
// proving the monitor never rescans the whole file after initialization.
#[test]
fn caches_event_offsets_across_polls() {
    let root = scratch_dir("offset-cache");
    write_session(
        &root,
        "session",
        "inuse.1.lock",
        "2026-07-28T07:20:00Z",
        "repository: demo\ncwd: C:/fixture/demo\n",
        "{\"type\":\"assistant.turn_start\",\"timestamp\":\"2026-07-28T07:20:00Z\"}\n",
    );

    let mut monitor = SessionMonitor::new(&root);
    monitor.poll();
    assert_eq!(monitor.lines_processed(), 1);

    let events_path = root.join("session").join("events.jsonl");
    let mut appended = fs::read_to_string(&events_path).unwrap();
    appended.push_str("{\"type\":\"assistant.turn_end\",\"timestamp\":\"2026-07-28T07:20:04Z\"}\n");
    fs::write(&events_path, appended).unwrap();

    let snapshot = monitor.poll();
    assert_eq!(snapshot.state, StatusState::Idle);
    assert_eq!(
        monitor.lines_processed(),
        2,
        "second poll should only read the newly appended line, not rescan the file"
    );
}

// This protects append races: an incomplete JSONL record must remain unread
// until its terminating newline arrives, then be processed exactly once.
#[test]
fn defers_partial_event_lines_until_complete() {
    let root = scratch_dir("partial-line");
    write_session(
        &root,
        "session",
        "inuse.1.lock",
        "",
        "repository: demo\n",
        "{\"type\":\"assistant.turn_start\"",
    );

    let mut monitor = SessionMonitor::new(&root);
    assert_eq!(monitor.poll().state, StatusState::Idle);
    assert_eq!(monitor.lines_processed(), 0);

    let events_path = root.join("session/events.jsonl");
    fs::write(&events_path, "{\"type\":\"assistant.turn_start\"}\n").unwrap();

    assert_eq!(monitor.poll().state, StatusState::Generating);
    assert_eq!(monitor.lines_processed(), 1);
}

// This protects session reactivation: returning to a previously selected
// session must rebuild its turn state instead of reusing its old EOF offset.
#[test]
fn reactivated_session_rebuilds_state() {
    let root = scratch_dir("reactivated-session");
    write_session(
        &root,
        "session-a",
        "inuse.1.lock",
        "",
        "repository: session-a\n",
        "{\"type\":\"assistant.turn_start\"}\n",
    );
    set_modified(&root.join("session-a/inuse.1.lock"), 1_000);

    let mut monitor = SessionMonitor::new(&root);
    assert_eq!(monitor.poll().state, StatusState::Generating);

    write_session(
        &root,
        "session-b",
        "inuse.2.lock",
        "",
        "repository: session-b\n",
        "",
    );
    set_modified(&root.join("session-b/inuse.2.lock"), 2_000);
    assert_eq!(monitor.poll().repository.as_deref(), Some("session-b"));

    fs::remove_file(root.join("session-b/inuse.2.lock")).unwrap();
    let reactivated = monitor.poll();
    assert_eq!(reactivated.repository.as_deref(), Some("session-a"));
    assert_eq!(reactivated.state, StatusState::Generating);
}

// This proves future/unknown event types, extra JSON fields, and a
// non-timestamp lock marker remain non-fatal while known events still drive
// a correct completed turn, and workspace metadata
// (including its nested future shape) keeps resolving. This is the
// acceptance backpressure for future-format compatibility.
#[test]
fn degraded_fixture_is_forward_compatible() {
    let root = fixture("degraded-session");

    let mut monitor = SessionMonitor::new(&root);
    let snapshot = monitor.poll();

    assert_eq!(snapshot.state, StatusState::Error);
    assert_eq!(snapshot.attention_reason, Some(AttentionReason::ToolFailed));
    assert_eq!(snapshot.model.as_deref(), Some("future-model-x"));
    assert_eq!(snapshot.repository.as_deref(), Some("octo-org/nested-demo"));
    assert_eq!(snapshot.last_turn_duration, Some(Duration::from_secs(3)));

    let modeled = format!("{snapshot:?}");
    assert!(!modeled.contains("SENSITIVE_SENTINEL"));
}

// This proves the committed live-session fixture (used by the probe demo
// command documented in AGENTS.md) selects the newer session and reports
// Working, exercising the same monitor the tray shell will poll.
#[test]
fn live_session_fixture_selects_newest_and_reports_working() {
    let root = fixture("live-session");

    let mut monitor = SessionMonitor::new(&root);
    let snapshot = monitor.poll();

    assert_eq!(snapshot.state, StatusState::Generating);
    assert_eq!(snapshot.model.as_deref(), Some("gpt-5.6-sol"));
    assert_eq!(
        snapshot.repository.as_deref(),
        Some("octo-org/content-free-demo")
    );
}

// This protects the documented 500-10,000 ms configuration boundary at
// exactly the values the monitor's caller (the future tray shell) may pass.
#[test]
fn poll_interval_accepts_the_documented_boundaries() {
    assert!(MonitorConfig::new(None, MIN_POLL_INTERVAL_MS).is_ok());
    assert!(MonitorConfig::new(None, MAX_POLL_INTERVAL_MS).is_ok());
    assert!(MonitorConfig::new(None, MIN_POLL_INTERVAL_MS - 1).is_err());
    assert!(MonitorConfig::new(None, MAX_POLL_INTERVAL_MS + 1).is_err());
}

// This protects the probe's privacy contract end-to-end: run the actual
// compiled `tray-goblin-probe` binary against the live-session fixture and
// prove its stdout is well-formed, content-free JSON containing only the
// documented allow-listed fields.
#[test]
fn probe_emits_content_free_json() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tray-goblin-probe"))
        .arg("--session-root")
        .arg(fixture("live-session"))
        .output()
        .expect("probe binary should run");

    assert!(
        output.status.success(),
        "probe exited non-zero: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("probe stdout should be valid UTF-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("probe stdout should be valid JSON");

    assert_eq!(parsed["state"], "working");
    assert_eq!(parsed["model"], "gpt-5.6-sol");
    assert_eq!(parsed["repository"], "octo-org/content-free-demo");

    let mut allowed_keys: Vec<_> = parsed
        .as_object()
        .expect("snapshot should be a JSON object")
        .keys()
        .cloned()
        .collect();
    allowed_keys.sort();
    assert_eq!(
        allowed_keys,
        vec![
            "active_directory",
            "attention_reason",
            "last_turn_duration_ms",
            "model",
            "repository",
            "state",
        ]
    );

    for forbidden in [
        "SENSITIVE_SENTINEL",
        "prompt",
        "assistantContent",
        "toolArguments",
        "toolResult",
        "token",
        "credential",
        "repositoryFileContents",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "probe output modeled forbidden field: {forbidden}"
        );
    }
}
