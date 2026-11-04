//! Offline driver ONLY: exercises the shipped worker/parser/controller/dispatcher.
//! stdin expectations are observations, never publisher input or a parallel policy.
use std::{
    io::{self, BufRead, Write},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime},
};

use serde::Deserialize;
use serde_json::json;
use tray_goblin::{
    app::{AttentionRuntime, Balloon},
    attention::SessionState,
    attention_worker::AttentionWorkers,
    config::MonitorConfig,
    monitor::AttentionMonitor,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Observe {
    phase: String,
    sessions: Vec<Expected>,
    submissions: usize,
    quiet_ms: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    session_id: String,
    instance_id: String,
    label: String,
    directory: String,
    state: SessionState,
    episode: Option<u64>,
}
fn epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("epoch clock")
        .as_millis() as u64
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("config path required")?;
    let config = MonitorConfig::parse(&std::fs::read_to_string(path)?)?;
    let sources = config.resolved_sources()?;
    assert_eq!(sources.len(), 1, "two publishers share one real source");
    let reads = Arc::new(AtomicU64::new(0));
    // Count COMPLETED real monitor reads (not invented notifications). One poll
    // per observed step prevents a prior step's in-flight scan from reading half
    // of the next deliberately simultaneous update. All parsing, membership,
    // worker mailboxes, controller and dispatch remain the shipped code.
    let workers = AttentionWorkers::with_readers(sources, epoch_ms(), |source| {
        let mut monitor = AttentionMonitor::new(vec![source.clone()]).expect("validated source");
        let reads = reads.clone();
        Box::new(move |_| {
            let snapshot = monitor.poll().expect("epoch clock").sources.remove(0);
            reads.fetch_add(1, Ordering::Release);
            snapshot
        })
    })?;
    let mut runtime = AttentionRuntime::new(workers);
    for line in io::stdin().lock().lines() {
        let expected: Observe = serde_json::from_str(&line?)?;
        assert!(expected.quiet_ms <= 5_000);
        assert!(expected.sessions.len() <= 2);
        let started = Instant::now();
        let mut matched_at = None;
        let mut requests = Vec::<Balloon>::new();
        let previous_reads = reads.load(Ordering::Acquire);
        let mut request_poll = true;
        loop {
            // This is the SAME dispatcher used by Win32, with an observed offline
            // adapter instead of Shell_NotifyIconW. Never claim OS delivery here.
            runtime.cycle(request_poll, false, epoch_ms, |balloon| {
                assert!(!balloon.title.is_empty() && !balloon.body.is_empty());
                requests.push(balloon.clone());
                true
            });
            request_poll = false;
            let snapshot = runtime.snapshot();
            let rows: Vec<_> = snapshot.sources.iter().flat_map(|s| &s.sessions).collect();
            let matches = reads.load(Ordering::Acquire) > previous_reads
                && snapshot.sources.len() == 1
                && snapshot.sources[0].diagnostics.is_empty()
                && snapshot.sources[0].capabilities.input_detection
                && rows.len() == expected.sessions.len()
                && expected.sessions.iter().all(|e| {
                    rows.iter().any(|s| {
                        s.session_id == e.session_id
                            && s.instance_id == e.instance_id
                            && s.label.as_deref() == Some(e.label.as_str())
                            && s.directory.as_deref() == Some(e.directory.as_str())
                            && s.model.as_deref() == Some("synthetic-model")
                            && s.state == e.state
                            && s.attention.as_ref().map(|a| a.id) == e.episode
                    })
                });
            if matches {
                let matched = *matched_at.get_or_insert_with(Instant::now);
                if matched.elapsed() >= Duration::from_millis(expected.quiet_ms) {
                    break;
                }
            } else {
                matched_at = None;
            }
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "{}: actual snapshot {}",
                expected.phase,
                serde_json::to_string(snapshot)?
            );
            thread::sleep(Duration::from_millis(5));
        }
        // Nonempty positive phases kill an omitted-dispatch implementation;
        // zero is accepted only when the real dispatcher remained quiet.
        assert_eq!(
            requests.len(),
            expected.submissions,
            "{}: actual adapter submissions",
            expected.phase
        );
        let requests: Vec<_> = requests
            .iter()
            .map(|b| json!({"title": b.title, "body": b.body, "accepted": true}))
            .collect();
        println!(
            "{}",
            json!({
                "phase": expected.phase, "elapsedMs": started.elapsed().as_millis(),
                "quietMs": expected.quiet_ms, "sourceReads": reads.load(Ordering::Acquire), "snapshot": runtime.snapshot(),
                "adapterRequests": requests, "submissionFailed": runtime.submission_failed(),
                "evidenceTier": "offline-shipped-dispatch-not-windows-delivery"
            })
        );
        io::stdout().flush()?;
    }
    runtime.stop();
    Ok(())
}
