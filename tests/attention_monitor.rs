//! Synthetic, content-free aggregate monitor contracts; no runtime HOME access.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use serde_json::{Value, json};
use tray_goblin::{
    attention::{Diagnostic, EpisodeReason, MAX_PI_BYTES, SessionState, parse_pi_status},
    config::{MonitorConfig, Provider},
    monitor::AttentionMonitor,
};

const ID: &str = "11111111-1111-4111-8111-111111111111";
const NOW: u64 = 20_000;
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-scratch/attention")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn config(&self, kinds: &[&str]) -> MonitorConfig {
        let sources: Vec<_> = kinds.iter().enumerate().map(|(i, kind)| {
            let path = self.0.join(format!("source-{i}"));
            fs::create_dir_all(&path).unwrap();
            json!({"id":format!("source-{i}"),"label":format!("Synthetic {i}"),"kind":kind,"path":path})
        }).collect();
        MonitorConfig::parse(&json!({"sources":sources}).to_string()).unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn monitor(config: &MonitorConfig) -> AttentionMonitor {
    AttentionMonitor::new(config.resolved_sources().unwrap()).unwrap()
}

fn pi() -> Value {
    serde_json::from_str(include_str!("fixtures/attention/pi-v1.json")).unwrap()
}

fn write_pi(root: &Path, value: &Value) {
    fs::write(
        root.join(format!("{}.json", value["instanceId"].as_str().unwrap())),
        value.to_string(),
    )
    .unwrap();
}

fn copilot(root: &Path, id: &str, events: &str) -> PathBuf {
    let path = root.join(id);
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("inuse.1.lock"), "").unwrap();
    fs::write(path.join("workspace.yaml"), "cwd: /synthetic/shared\n").unwrap();
    fs::write(path.join("events.jsonl"), events).unwrap();
    path
}

fn append(path: &Path, events: &str) {
    fs::OpenOptions::new()
        .append(true)
        .open(path.join("events.jsonl"))
        .unwrap()
        .write_all(events.as_bytes())
        .unwrap();
}

// Proves replacement/default semantics and every source-config boundary without changing process HOME.
#[test]
fn source_configuration_preserves_defaults_aliases_and_explicit_replacement() {
    let default = MonitorConfig::default().resolved_sources().unwrap();
    assert_eq!(default.len(), 2);
    assert_eq!(default[0].kind, Provider::Copilot);
    assert_eq!(default[1].kind, Provider::Pi);
    assert!(default[1].path.ends_with(".traygoblin/pi"));
    let legacy =
        MonitorConfig::parse(r#"{"sessionRoot":"/synthetic/legacy","poll_interval_ms":750}"#)
            .unwrap();
    let sources = legacy.resolved_sources().unwrap();
    assert_eq!(sources[0].path, Path::new("/synthetic/legacy"));
    assert_eq!(sources[1], default[1]);
    assert_eq!(legacy.poll_interval_ms(), 750);
    assert!(
        MonitorConfig::parse(r#"{"sources":[]}"#)
            .unwrap()
            .resolved_sources()
            .unwrap()
            .is_empty()
    );
    let entry = json!({"id":"one","label":"Synthetic","kind":"pi","path":"/synthetic","wsl_distro":"Ubuntu"});
    let single = MonitorConfig::parse(
        &json!({"sessionRoot":"/ignored","sources":[entry.clone()]}).to_string(),
    )
    .unwrap();
    assert_eq!(single.resolved_sources().unwrap().len(), 1);
    assert_eq!(
        single.sources().unwrap()[0].wsl_distro.as_deref(),
        Some("Ubuntu")
    );
    let sixteen: Vec<_> = (0..16)
        .map(|i| {
            let mut value = entry.clone();
            value["id"] = json!(format!("source-{i}"));
            value
        })
        .collect();
    assert!(MonitorConfig::parse(&json!({"sources":sixteen}).to_string()).is_ok());
    for bad in [
        json!({"sources":vec![entry.clone();17]}),
        json!({"sources":[entry.clone(),entry.clone()]}),
        json!({"sources":[{"id":"one","label":"x","kind":"unknown","path":"/x"}]}),
        json!({"sources":[{"id":"","label":"x","kind":"pi","path":"/x"}]}),
        json!({"sources":[{"id":"x","label":"x","kind":"pi","path":""}]}),
        json!({"sources":[{"id":"x","label":"\n","kind":"pi","path":"/x"}]}),
        json!({"sources":[{"id":"x".repeat(129),"label":"x","kind":"pi","path":"/x"}]}),
        json!({"sources":[{"id":"x","label":"x","kind":"pi","path":"/x","wsl_distro":""}]}),
    ] {
        assert!(MonitorConfig::parse(&bad.to_string()).is_err(), "{bad}");
    }
}

// Proves missing implicit pi is normal, while a missing explicitly configured source is visible.
#[test]
fn absent_sources_have_explicit_default_vs_configured_semantics() {
    let scratch = Scratch::new();
    let mut defaults = MonitorConfig::default().resolved_sources().unwrap();
    for (i, source) in defaults.iter_mut().enumerate() {
        source.path = scratch.0.join(format!("missing-{i}"));
    }
    let snapshot = AttentionMonitor::new(defaults).unwrap().poll_at(NOW);
    assert_eq!(
        snapshot.sources[0].diagnostics,
        [Diagnostic::SourceUnavailable]
    );
    assert!(snapshot.sources[1].diagnostics.is_empty());
    let config = MonitorConfig::parse(
        &json!({"sources":[{"id":"x","label":"x","kind":"pi","path":scratch.0.join("missing")}]})
            .to_string(),
    )
    .unwrap();
    assert_eq!(
        monitor(&config).poll_at(NOW).sources[0].diagnostics,
        [Diagnostic::SourceUnavailable]
    );
}

// Proves the actual v1 key/nullable schema, error reasons, UTF-8 bounds and privacy allow-list.
#[test]
fn pi_parser_matches_m1_schema_and_rejects_inconsistent_scalars() {
    let mut value = pi();
    value["prompt"] = json!("SENSITIVE_SENTINEL");
    value["toolArguments"] = json!({"credential":"SENSITIVE_SENTINEL"});
    let status = parse_pi_status(value.to_string().as_bytes(), NOW).unwrap();
    assert!(!format!("{status:?}").contains("SENSITIVE_SENTINEL"));
    for reason in ["run_error", "ui_tracking_unavailable"] {
        value["state"] = json!("error");
        value["attentionReason"] = json!(reason);
        assert!(parse_pi_status(value.to_string().as_bytes(), NOW).is_ok());
    }
    for (key, bad) in [
        ("version", json!(2)),
        ("provider", json!("copilot")),
        ("instanceId", json!("not-a-uuid")),
        ("state", json!("unavailable")),
        ("state", json!("unknown")),
        ("model", json!("a".repeat(129))),
        ("model", json!("é".repeat(65))),
        ("sessionId", json!("x\n")),
        ("directory", json!("x".repeat(2049))),
        ("episode", json!(0)),
        ("episode", json!(9_007_199_254_740_992_u64)),
        ("episode", json!(-1)),
        ("updatedAt", json!(1.5)),
        ("runStartedAt", json!(NOW + 1)),
        ("lastRunDurationMs", json!(9_007_199_254_740_992_u64)),
        ("attentionReason", Value::Null),
    ] {
        let mut bad_status = value.clone();
        bad_status[key] = bad;
        assert!(
            parse_pi_status(bad_status.to_string().as_bytes(), NOW).is_err(),
            "{key}"
        );
    }
    for key in [
        "attentionReason",
        "runStartedAt",
        "lastRunDurationMs",
        "model",
    ] {
        let mut missing = pi();
        missing.as_object_mut().unwrap().remove(key);
        assert!(
            parse_pi_status(missing.to_string().as_bytes(), NOW).is_err(),
            "{key}"
        );
    }
    value = pi();
    value["attentionReason"] = json!("run_error");
    assert_eq!(
        parse_pi_status(value.to_string().as_bytes(), NOW).unwrap_err(),
        Diagnostic::InvalidPiStatus
    );
}

// Proves exact heartbeat/size boundaries, including a valid JSON file one byte beyond 16 KiB.
#[test]
fn pi_size_freshness_and_future_boundaries_are_exact() {
    let mut value = pi();
    let bytes = value.to_string().into_bytes();
    assert!(parse_pi_status(&bytes, NOW + 10_000).is_ok());
    assert_eq!(
        parse_pi_status(&bytes, NOW + 10_001).unwrap_err(),
        Diagnostic::StalePiStatus
    );
    assert_eq!(
        parse_pi_status(&bytes, NOW - 1).unwrap_err(),
        Diagnostic::FuturePiStatus
    );
    value["padding"] = json!("");
    let base = value.to_string().len();
    value["padding"] = json!("x".repeat(MAX_PI_BYTES - base));
    let exact = value.to_string();
    assert_eq!(exact.len(), MAX_PI_BYTES);
    assert!(parse_pi_status(exact.as_bytes(), NOW).is_ok());
    value["padding"] = json!("x".repeat(MAX_PI_BYTES - base + 1));
    let over = value.to_string();
    assert_eq!(over.len(), MAX_PI_BYTES + 1);
    assert_eq!(
        parse_pi_status(over.as_bytes(), NOW).unwrap_err(),
        Diagnostic::OversizedPiStatus
    );
    let scratch = Scratch::new();
    let config = scratch.config(&["pi"]);
    let root = &config.sources().unwrap()[0].path;
    fs::write(root.join(format!("{ID}.json")), exact).unwrap();
    let mut monitor = monitor(&config);
    assert_eq!(
        monitor.poll_at(NOW).sources[0].sessions[0].state,
        SessionState::InputRequired
    );
    fs::write(root.join(format!("{ID}.json")), over).unwrap();
    let snapshot = monitor.poll_at(NOW);
    assert_eq!(
        snapshot.sources[0].diagnostics,
        [Diagnostic::OversizedPiStatus]
    );
    assert_eq!(
        snapshot.sources[0].sessions[0].state,
        SessionState::Unavailable
    );
}

// Proves timestamp freshness is independent of episode ordering and faults retain their distinct reason.
#[test]
fn pi_restores_older_error_identity_without_rejecting_it_as_stale() {
    let scratch = Scratch::new();
    let config = scratch.config(&["pi"]);
    let root = &config.sources().unwrap()[0].path;
    let mut value = pi();
    value["state"] = json!("error");
    value["attentionReason"] = json!("run_error");
    write_pi(root, &value);
    let mut monitor = monitor(&config);
    let first = monitor.poll_at(NOW).sources[0].sessions[0]
        .attention
        .clone()
        .unwrap();
    value["state"] = json!("input_required");
    value["attentionReason"] = Value::Null;
    value["episode"] = json!(2);
    value["updatedAt"] = json!(NOW + 1);
    write_pi(root, &value);
    assert_eq!(
        monitor.poll_at(NOW + 1).sources[0].sessions[0]
            .attention
            .as_ref()
            .unwrap()
            .id,
        2
    );
    value["state"] = json!("error");
    value["attentionReason"] = json!("run_error");
    value["episode"] = json!(1);
    value["updatedAt"] = json!(NOW + 2);
    write_pi(root, &value);
    assert_eq!(
        monitor.poll_at(NOW + 2).sources[0].sessions[0]
            .attention
            .as_ref(),
        Some(&first)
    );
    value["attentionReason"] = json!("ui_tracking_unavailable");
    value["episode"] = json!(3);
    value["updatedAt"] = json!(NOW + 3);
    write_pi(root, &value);
    assert_eq!(
        monitor.poll_at(NOW + 3).sources[0].sessions[0]
            .attention
            .as_ref()
            .unwrap()
            .reason,
        EpisodeReason::UiTrackingUnavailable
    );
    value["updatedAt"] = json!(NOW + 2);
    write_pi(root, &value);
    let regressed = monitor.poll_at(NOW + 3);
    assert_eq!(
        regressed.sources[0].sessions[0].state,
        SessionState::Unavailable
    );
    assert_eq!(
        regressed.sources[0].diagnostics,
        [Diagnostic::RegressedPiStatus]
    );
}

// Proves partial/mismatched/stale/future publications degrade visibly, recover, and disappear on removal.
#[test]
fn pi_invalid_publications_never_expose_input_and_recover_independently() {
    let scratch = Scratch::new();
    let config = scratch.config(&["pi", "pi"]);
    let root = &config.sources().unwrap()[0].path;
    let other = &config.sources().unwrap()[1].path;
    write_pi(root, &pi());
    write_pi(other, &pi());
    let mut monitor = monitor(&config);
    assert_eq!(monitor.poll_at(NOW).sources[0].sessions.len(), 1);
    for (data, diagnostic) in [
        ("{\"version\":".to_owned(), Diagnostic::InvalidPiStatus),
        (
            {
                let mut v = pi();
                v["instanceId"] = json!("22222222-2222-4222-8222-222222222222");
                v.to_string()
            },
            Diagnostic::InvalidPiStatus,
        ),
        (
            {
                let mut v = pi();
                v["updatedAt"] = json!(NOW + 1);
                v.to_string()
            },
            Diagnostic::FuturePiStatus,
        ),
        (
            {
                let mut v = pi();
                v["updatedAt"] = json!(0);
                v["runStartedAt"] = Value::Null;
                v.to_string()
            },
            Diagnostic::StalePiStatus,
        ),
    ] {
        fs::write(root.join(format!("{ID}.json")), data).unwrap();
        let snapshot = monitor.poll_at(NOW);
        assert_eq!(
            snapshot.sources[0].sessions[0].state,
            SessionState::Unavailable
        );
        assert!(snapshot.sources[0].sessions[0].attention.is_none());
        assert_eq!(snapshot.sources[0].diagnostics, [diagnostic]);
        assert_eq!(
            snapshot.sources[1].sessions[0].state,
            SessionState::InputRequired
        );
        write_pi(root, &pi());
        assert_eq!(
            monitor.poll_at(NOW).sources[0].sessions[0].state,
            SessionState::InputRequired
        );
    }
    fs::remove_file(root.join(format!("{ID}.json"))).unwrap();
    assert!(monitor.poll_at(NOW).sources[0].sessions.is_empty());
}

// Proves 64 stale publications cannot hide the sole live waiting session at enumeration position 65.
#[test]
fn pi_stale_saturation_does_not_hide_fresh_attention() {
    let scratch = Scratch::new();
    let config = scratch.config(&["pi"]);
    let root = &config.sources().unwrap()[0].path;
    for i in 0..65 {
        let mut value = pi();
        value["instanceId"] = json!(format!("{i:08x}-1111-4111-8111-111111111111"));
        value["updatedAt"] = json!(0);
        value["runStartedAt"] = Value::Null;
        write_pi(root, &value);
    }
    let names: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    let fresh_path = root.join(&names[64]);
    let mut fresh = pi();
    fresh["instanceId"] = json!(fresh_path.file_stem().unwrap().to_str().unwrap());
    fs::write(&fresh_path, fresh.to_string()).unwrap();
    assert_eq!(
        fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>(),
        names,
        "rewriting a publication must not change this regression's enumeration order"
    );
    let mut monitor = monitor(&config);
    for now in [NOW, NOW + 1] {
        let snapshot = monitor.poll_at(now);
        let source = &snapshot.sources[0];
        let waiting = source
            .sessions
            .iter()
            .find(|session| session.instance_id == fresh["instanceId"].as_str().unwrap())
            .expect("stale residue must not displace live attention");
        assert_eq!(waiting.state, SessionState::InputRequired);
        assert_eq!(
            waiting.attention.as_ref().unwrap().reason,
            EpisodeReason::InputRequired
        );
        assert!(source.sessions.len() <= 64);
        assert!(source.diagnostics.contains(&Diagnostic::StalePiStatus));
        assert!(source.diagnostics.contains(&Diagnostic::SessionOverflow));
        assert_eq!(fs::read_dir(root).unwrap().count(), 65);
        assert_eq!(fs::read(&fresh_path).unwrap(), fresh.to_string().as_bytes());
    }
}

// Proves residue is counted separately even with 64 live sessions, then stale placeholders remain bounded.
#[test]
fn pi_live_capacity_and_unavailable_summary_are_independent() {
    let scratch = Scratch::new();
    let config = scratch.config(&["pi"]);
    let root = &config.sources().unwrap()[0].path;
    for i in 0..128 {
        let mut value = pi();
        value["instanceId"] = json!(format!("{i:08x}-1111-4111-8111-111111111111"));
        if i < 64 {
            value["updatedAt"] = json!(0);
            value["runStartedAt"] = Value::Null;
        }
        write_pi(root, &value);
    }
    let mut monitor = monitor(&config);
    for (now, live, unavailable) in [(NOW, 64, 64), (NOW + 10_001, 0, 128)] {
        let snapshot = monitor.poll_at(now);
        let source = &snapshot.sources[0];
        assert_eq!(source.sessions.len(), 64);
        assert_eq!(
            source
                .sessions
                .iter()
                .filter(|s| s.attention.is_some())
                .count(),
            live
        );
        assert_eq!(source.unavailable_pi_statuses, unavailable);
        assert_eq!(source.diagnostics.len(), 2);
        assert!(source.diagnostics.contains(&Diagnostic::StalePiStatus));
        assert!(source.diagnostics.contains(&Diagnostic::SessionOverflow));
        assert_eq!(fs::read_dir(root).unwrap().count(), 128);
    }
}

// Proves freshness filtering cannot extend the 4096-entry scan, even under entirely stale saturation.
#[test]
fn pi_stale_scan_stops_at_the_directory_bound() {
    let scratch = Scratch::new();
    let config = scratch.config(&["pi"]);
    let root = &config.sources().unwrap()[0].path;
    for i in 0..4097 {
        let mut value = pi();
        value["instanceId"] = json!(format!("{i:08x}-1111-4111-8111-111111111111"));
        value["updatedAt"] = json!(0);
        value["runStartedAt"] = Value::Null;
        write_pi(root, &value);
    }
    let snapshot = monitor(&config).poll_at(NOW);
    let source = &snapshot.sources[0];
    assert_eq!(source.unavailable_pi_statuses, 4096);
    assert_eq!(source.sessions.len(), 64);
    assert!(
        source
            .sessions
            .iter()
            .all(|s| s.state == SessionState::Unavailable)
    );
    assert_eq!(source.diagnostics.len(), 2);
    assert!(source.diagnostics.contains(&Diagnostic::StalePiStatus));
    assert!(source.diagnostics.contains(&Diagnostic::SessionOverflow));
    assert_eq!(fs::read_dir(root).unwrap().count(), 4097);
    // The scan cap limits lifecycle evidence too; omitted filenames are not deletions.
    assert!(!source.membership.complete);
    assert_eq!(source.membership.instances.len(), 4096);
}

// Proves only bounded regular UUID.json candidates are read, never temporary/arbitrary filenames or directories.
#[test]
fn pi_filters_names_and_nonregular_entries_before_opening() {
    let scratch = Scratch::new();
    let config = scratch.config(&["pi"]);
    let root = &config.sources().unwrap()[0].path;
    for name in [
        "auth.json",
        "anything.json",
        "11111111-1111-4111-8111-111111111111.tmp",
    ] {
        fs::write(root.join(name), vec![b'x'; MAX_PI_BYTES + 1]).unwrap();
    }
    fs::create_dir(root.join(format!("{ID}.json"))).unwrap();
    let result = monitor(&config).poll_at(NOW);
    assert!(result.sources[0].sessions.is_empty());
    assert_eq!(result.sources[0].diagnostics, [Diagnostic::UnsafeEntry]);
}

// Proves symlinked pi files, Copilot directories/locks/events/workspaces cannot expose an escaped target.
#[cfg(unix)]
#[test]
fn symlinks_never_escape_configured_source_children() {
    use std::os::unix::fs::symlink;
    let scratch = Scratch::new();
    let config = scratch.config(&["pi", "copilot"]);
    let pi_root = &config.sources().unwrap()[0].path;
    let copilot_root = &config.sources().unwrap()[1].path;
    let outside = scratch.0.join("outside.json");
    fs::write(&outside, pi().to_string()).unwrap();
    symlink(&outside, pi_root.join(format!("{ID}.json"))).unwrap();
    let outside_session = copilot(
        &scratch.0,
        "outside-session",
        "{\"type\":\"session.error\"}\n",
    );
    symlink(&outside_session, copilot_root.join("escaped")).unwrap();
    let session = copilot(copilot_root, "session", "");
    fs::remove_file(session.join("events.jsonl")).unwrap();
    symlink(
        outside_session.join("events.jsonl"),
        session.join("events.jsonl"),
    )
    .unwrap();
    fs::remove_file(session.join("workspace.yaml")).unwrap();
    symlink(
        outside_session.join("workspace.yaml"),
        session.join("workspace.yaml"),
    )
    .unwrap();
    let mut monitor = monitor(&config);
    let snapshot = monitor.poll_at(NOW);
    assert!(snapshot.sources[0].sessions.is_empty());
    assert_eq!(snapshot.sources[0].diagnostics, [Diagnostic::UnsafeEntry]);
    assert_eq!(snapshot.sources[1].sessions.len(), 1);
    assert_eq!(
        snapshot.sources[1].sessions[0].state,
        SessionState::Unavailable
    );
    assert!(snapshot.sources[1].sessions[0].directory.is_none());
    assert_eq!(monitor.lines_processed(), 0);
    fs::remove_file(session.join("inuse.1.lock")).unwrap();
    symlink(
        outside_session.join("inuse.1.lock"),
        session.join("inuse.1.lock"),
    )
    .unwrap();
    assert!(monitor.poll_at(NOW).sources[1].sessions.is_empty());
}

// Proves ALL active directories have independent offsets/error clearing, not newest-session selection.
#[test]
fn copilot_sessions_do_not_replay_history_or_alert_for_tools() {
    let scratch = Scratch::new();
    let config = scratch.config(&["copilot"]);
    let root = &config.sources().unwrap()[0].path;
    let one = copilot(
        root,
        "one",
        "{\"type\":\"assistant.turn_start\"}\n{\"type\":\"session.error\",\"data\":{\"message\":\"SENSITIVE_SENTINEL\"}}\n",
    );
    let two = copilot(root, "two", "{\"type\":\"assistant.turn_start\"}\n");
    let mut monitor = monitor(&config);
    let initial = monitor.poll_at(NOW);
    let sessions = &initial.sources[0].sessions;
    assert_eq!(sessions.len(), 2);
    assert!(!sessions[0].attention.as_ref().unwrap().notify);
    assert_eq!(sessions[1].state, SessionState::Working);
    assert!(!initial.sources[0].capabilities.input_detection);
    assert!(
        !serde_json::to_string(&initial)
            .unwrap()
            .contains("SENSITIVE_SENTINEL")
    );
    assert_eq!(monitor.lines_processed(), 3);
    monitor.poll_at(NOW);
    assert_eq!(monitor.lines_processed(), 3);
    append(
        &two,
        "{\"type\":\"tool.execution_complete\",\"success\":false}\n{\"type\":\"assistant.turn_end\"}\n",
    );
    append(&one, "{\"type\":\"session.error\"}\n");
    let updated = monitor.poll_at(NOW);
    let sessions = &updated.sources[0].sessions;
    assert_eq!(sessions[0].attention.as_ref().unwrap().id, 2);
    assert!(sessions[0].attention.as_ref().unwrap().notify);
    assert_eq!(sessions[1].state, SessionState::Idle);
    assert!(sessions[1].attention.is_none());
    assert_eq!(monitor.lines_processed(), 6);
    append(&one, "{\"type\":\"assistant.turn_end\"}\n");
    assert_eq!(
        monitor.poll_at(NOW).sources[0].sessions[0].state,
        SessionState::Error
    );
    append(&one, "{\"type\":\"assistant.turn_start\"}\n");
    let cleared = monitor.poll_at(NOW);
    assert_eq!(cleared.sources[0].sessions[0].state, SessionState::Working);
    assert!(cleared.sources[0].sessions[0].attention.is_none());
    fs::remove_file(two.join("inuse.1.lock")).unwrap();
    assert_eq!(monitor.poll_at(NOW).sources[0].sessions.len(), 1);
}

// Proves newline-only offsets retain partial appends and malformed/unknown/future input remains content-free.
#[test]
fn copilot_partial_appends_metadata_duration_and_recovery() {
    let scratch = Scratch::new();
    let config = scratch.config(&["copilot"]);
    let root = &config.sources().unwrap()[0].path;
    let session = copilot(root, "one", "");
    let mut monitor = monitor(&config);
    monitor.poll_at(NOW);
    append(
        &session,
        "{\"type\":\"assistant.turn_start\",\"timestamp\":\"1970-01-01T00:00:10Z\"}\n{\"type\":\"session.model_change\",\"data\":{\"model\":\"synthetic-model\"}}\n{\"type\":\"session.error\"",
    );
    let first = monitor.poll_at(NOW);
    assert_eq!(first.sources[0].sessions[0].state, SessionState::Working);
    assert_eq!(monitor.lines_processed(), 2);
    append(
        &session,
        "}\ninvalid\n{\"type\":\"future.type\"}\n{\"type\":\"assistant.turn_start\",\"timestamp\":\"2099-01-01T00:00:00Z\"}\n{\"type\":\"assistant.turn_end\",\"timestamp\":\"1970-01-01T00:00:15Z\"}\n",
    );
    let error = monitor.poll_at(NOW);
    assert_eq!(error.sources[0].sessions[0].state, SessionState::Error);
    assert_eq!(
        error.sources[0].sessions[0].last_run_duration_ms,
        Some(5000)
    );
    assert_eq!(
        error.sources[0].sessions[0].model.as_deref(),
        Some("synthetic-model")
    );
    assert_eq!(
        error.sources[0].sessions[0].directory.as_deref(),
        Some("/synthetic/shared")
    );
    assert_eq!(error.sources[0].diagnostics, [Diagnostic::MalformedEvent]);
    assert_eq!(monitor.lines_processed(), 7);
    fs::remove_file(session.join("events.jsonl")).unwrap();
    let unavailable = monitor.poll_at(NOW);
    assert_eq!(
        unavailable.sources[0].sessions[0].state,
        SessionState::Unavailable
    );
    fs::write(
        session.join("events.jsonl"),
        "{\"type\":\"session.error\"}\n",
    )
    .unwrap();
    let recovered = monitor.poll_at(NOW);
    assert_eq!(recovered.sources[0].sessions[0].state, SessionState::Error);
    assert_eq!(
        recovered.sources[0].sessions[0].directory.as_deref(),
        Some("/synthetic/shared")
    );
    assert!(
        !recovered.sources[0].sessions[0]
            .attention
            .as_ref()
            .unwrap()
            .notify
    );
    append(&session, "{\"type\":\"session.error\"}\n");
    let renewed = monitor.poll_at(NOW);
    assert!(
        renewed.sources[0].sessions[0]
            .attention
            .as_ref()
            .unwrap()
            .notify
    );
    assert!(
        renewed.sources[0].sessions[0]
            .attention
            .as_ref()
            .unwrap()
            .id
            > error.sources[0].sessions[0].attention.as_ref().unwrap().id
    );
}

// Proves backlog/overlong records are bounded, historical errors remain quiet across multiple polls.
#[test]
fn copilot_large_history_and_oversized_records_are_bounded() {
    let scratch = Scratch::new();
    let config = scratch.config(&["copilot"]);
    let root = &config.sources().unwrap()[0].path;
    let history = "{\"type\":\"future.type\"}\n".repeat(15_000);
    let session = copilot(root, "one", &(history + "{\"type\":\"session.error\"}\n"));
    let mut monitor = monitor(&config);
    monitor.poll_at(NOW);
    assert!(monitor.lines_processed() < 15_001);
    let initialized = monitor.poll_at(NOW);
    assert!(
        !initialized.sources[0].sessions[0]
            .attention
            .as_ref()
            .unwrap()
            .notify
    );
    append(
        &session,
        &("x".repeat(256 * 1024 + 1) + "\n{\"type\":\"assistant.turn_start\"}\n"),
    );
    assert!(
        monitor.poll_at(NOW).sources[0]
            .diagnostics
            .contains(&Diagnostic::OversizedEvent)
    );
    let recovered = monitor.poll_at(NOW);
    assert_eq!(
        recovered.sources[0].sessions[0].state,
        SessionState::Working
    );
    assert!(recovered.sources[0].sessions[0].attention.is_none());
}

// Proves same-directory identities survive 64-session mixed load and source overflow is visible, not silent.
#[test]
fn sixty_four_mixed_sessions_remain_independent_with_visible_overflow() {
    let scratch = Scratch::new();
    let config = scratch.config(&["pi", "copilot"]);
    let pi_root = &config.sources().unwrap()[0].path;
    let copilot_root = &config.sources().unwrap()[1].path;
    for i in 0..32 {
        let mut value = pi();
        value["instanceId"] = json!(format!("{i:08x}-1111-4111-8111-111111111111"));
        write_pi(pi_root, &value);
        copilot(
            copilot_root,
            &format!("session-{i:02}"),
            "{\"type\":\"assistant.turn_start\"}\n",
        );
    }
    let mut monitor = monitor(&config);
    let snapshot = monitor.poll_at(NOW);
    assert_eq!(
        snapshot
            .sources
            .iter()
            .map(|s| s.sessions.len())
            .sum::<usize>(),
        64
    );
    for source in &snapshot.sources {
        assert!(source.diagnostics.is_empty());
        let identities: std::collections::HashSet<_> = source
            .sessions
            .iter()
            .map(|s| (&s.session_id, &s.instance_id))
            .collect();
        assert_eq!(identities.len(), 32);
    }
    for i in 32..64 {
        let mut value = pi();
        value["instanceId"] = json!(format!("{i:08x}-1111-4111-8111-111111111111"));
        write_pi(pi_root, &value);
        copilot(copilot_root, &format!("session-{i:02}"), "");
    }
    let exact = monitor.poll_at(NOW);
    for source in &exact.sources {
        assert_eq!(source.sessions.len(), 64);
        assert!(source.diagnostics.is_empty());
    }
    let mut extra = pi();
    extra["instanceId"] = json!("00000040-1111-4111-8111-111111111111");
    write_pi(pi_root, &extra);
    copilot(copilot_root, "session-64", "");
    let full = monitor.poll_at(NOW);
    for source in &full.sources {
        assert_eq!(source.sessions.len(), 64);
        assert_eq!(source.diagnostics, [Diagnostic::SessionOverflow]);
    }
}

// Proves per-source same-name sessions do not share state, and removed publisher instances cannot linger.
#[test]
fn source_identity_and_session_replacement_drop_previous_attention() {
    let scratch = Scratch::new();
    let config = scratch.config(&["copilot", "copilot", "pi"]);
    let sources = config.sources().unwrap();
    let first = copilot(&sources[0].path, "same", "");
    copilot(&sources[1].path, "same", "");
    write_pi(&sources[2].path, &pi());
    let mut monitor = monitor(&config);
    monitor.poll_at(NOW);
    append(&first, "{\"type\":\"session.error\"}\n");
    let initial = monitor.poll_at(NOW);
    assert_eq!(initial.sources[0].sessions[0].state, SessionState::Error);
    assert_eq!(initial.sources[1].sessions[0].state, SessionState::Idle);
    let identity = initial.sources[0].sessions[0].instance_id.clone();
    fs::remove_file(first.join("inuse.1.lock")).unwrap();
    fs::write(first.join("inuse.2.lock"), "").unwrap();
    let replaced = monitor.poll_at(NOW);
    assert_ne!(replaced.sources[0].sessions[0].instance_id, identity);
    assert!(
        !replaced.sources[0].sessions[0]
            .attention
            .as_ref()
            .unwrap()
            .notify
    );
    fs::remove_file(sources[2].path.join(format!("{ID}.json"))).unwrap();
    let mut replacement = pi();
    replacement["instanceId"] = json!("22222222-2222-4222-8222-222222222222");
    replacement["state"] = json!("idle");
    write_pi(&sources[2].path, &replacement);
    let next = monitor.poll_at(NOW);
    assert_eq!(next.sources[2].sessions.len(), 1);
    assert_eq!(next.sources[2].sessions[0].state, SessionState::Idle);
    assert!(next.sources[2].sessions[0].attention.is_none());
}

// Proves the actual binary preserves its six-field API and exposes configured aggregate capabilities.
#[test]
fn probe_default_wire_shape_is_unchanged_and_aggregate_is_explicit() {
    let scratch = Scratch::new();
    let config = scratch.config(&["copilot", "pi"]);
    let sources = config.sources().unwrap();
    copilot(
        &sources[0].path,
        "one",
        "{\"type\":\"assistant.turn_start\"}\n",
    );
    let config_path = scratch.0.join("config.json");
    fs::write(
        &config_path,
        json!({"sources": sources, "sessionRoot":sources[0].path}).to_string(),
    )
    .unwrap();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_tray-goblin-probe"))
            .env_clear()
            .env("HOME", &scratch.0)
            .env("USERPROFILE", &scratch.0)
            .args(["--config", config_path.to_str().unwrap()])
            .args(extra)
            .output()
            .unwrap()
    };
    let legacy = run(&[]);
    assert!(
        legacy.status.success(),
        "{}",
        String::from_utf8_lossy(&legacy.stderr)
    );
    let value: Value = serde_json::from_slice(&legacy.stdout).unwrap();
    let keys: Vec<_> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "active_directory",
            "attention_reason",
            "last_turn_duration_ms",
            "model",
            "repository",
            "state"
        ]
    );
    assert_eq!(value["state"], "working");
    let aggregate = run(&["--all-sessions"]);
    assert!(aggregate.status.success());
    let value: Value = serde_json::from_slice(&aggregate.stdout).unwrap();
    assert_eq!(value["sources"].as_array().unwrap().len(), 2);
    assert_eq!(
        value["sources"][0]["capabilities"]["input_detection"],
        false
    );
    assert_eq!(value["sources"][1]["capabilities"]["input_detection"], true);
    assert_eq!(value["sources"][0]["sessions"][0]["state"], "working");
    assert!(
        !run(&["--all-sessions", "--session-root", "/synthetic"])
            .status
            .success()
    );
    fs::write(&config_path, r#"{"sources":[]}"#).unwrap();
    let disabled = run(&["--all-sessions"]);
    assert!(disabled.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&disabled.stdout).unwrap(),
        json!({"sources":[]})
    );
}
