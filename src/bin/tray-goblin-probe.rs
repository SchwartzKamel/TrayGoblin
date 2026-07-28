//! Content-free diagnostic probe: polls a Copilot CLI session-state root
//! once and prints the resulting status snapshot as JSON. Never reads or
//! emits prompt, response, tool argument/result, token, credential, or
//! repository file content — only the fields modeled by
//! `tray_goblin::status::StatusSnapshot`.

use std::{path::PathBuf, process::ExitCode};

use serde::Serialize;
use tray_goblin::{
    config::{ConfigParseError, MonitorConfig, default_session_root},
    monitor::SessionMonitor,
    status::{AttentionReason, StatusSnapshot, StatusState},
};

struct Args {
    session_root: Option<PathBuf>,
    config_path: Option<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut session_root = None;
    let mut config_path = None;
    let mut raw_args = std::env::args().skip(1);

    while let Some(arg) = raw_args.next() {
        match arg.as_str() {
            "--session-root" => {
                let value = raw_args
                    .next()
                    .ok_or_else(|| "--session-root requires a path".to_owned())?;
                session_root = Some(PathBuf::from(value));
            }
            "--config" => {
                let value = raw_args
                    .next()
                    .ok_or_else(|| "--config requires a path".to_owned())?;
                config_path = Some(PathBuf::from(value));
            }
            other => return Err(format!("unrecognized argument: {other}")),
        }
    }

    Ok(Args {
        session_root,
        config_path,
    })
}

/// Explicit, content-free JSON wire shape. Kept separate from the internal
/// `StatusSnapshot` type so the emitted field set is an intentional
/// allow-list rather than whatever the core struct happens to derive.
#[derive(Serialize)]
struct ProbeSnapshot {
    state: &'static str,
    model: Option<String>,
    repository: Option<String>,
    active_directory: Option<String>,
    last_turn_duration_ms: Option<u64>,
    attention_reason: Option<&'static str>,
}

impl From<StatusSnapshot> for ProbeSnapshot {
    fn from(snapshot: StatusSnapshot) -> Self {
        Self {
            state: match snapshot.state {
                StatusState::Idle => "idle",
                StatusState::Generating => "working",
                StatusState::Error => "attention_needed",
            },
            model: snapshot.model,
            repository: snapshot.repository,
            active_directory: snapshot
                .active_directory
                .map(|path| path.to_string_lossy().into_owned()),
            last_turn_duration_ms: snapshot.last_turn_duration.map(|d| d.as_millis() as u64),
            attention_reason: snapshot.attention_reason.map(|reason| match reason {
                AttentionReason::StateUnavailable => "state_unavailable",
                AttentionReason::ToolFailed => "tool_failed",
            }),
        }
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;

    let config = match &args.config_path {
        Some(path) => {
            let contents = std::fs::read_to_string(path)
                .map_err(|error| format!("cannot read config: {error}"))?;
            MonitorConfig::parse(&contents).map_err(|error: ConfigParseError| error.to_string())?
        }
        None => MonitorConfig::default(),
    };

    let session_root = args
        .session_root
        .or_else(|| config.session_root().cloned())
        .or_else(default_session_root)
        .ok_or_else(|| "no session root provided and none could be determined".to_owned())?;

    let mut monitor = SessionMonitor::new(session_root);
    let snapshot = ProbeSnapshot::from(monitor.poll());

    let json = serde_json::to_string_pretty(&snapshot)
        .map_err(|error| format!("failed to encode snapshot: {error}"))?;
    println!("{json}");

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
