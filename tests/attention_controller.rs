//! Synthetic policy, bounded-source concurrency and real filesystem integration.

mod copilot_provenance {
    use super::*;
    use tray_goblin::{attention::NotificationRequest, monitor::AttentionMonitor};

    fn file_identity(path: &Path) -> String {
        let metadata = fs::metadata(path).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            format!("{}:{}", metadata.dev(), metadata.ino())
        }
        #[cfg(not(unix))]
        {
            format!("{:?}", metadata.created().unwrap())
        }
    }

    struct ReadEvidence {
        snapshot: SourceSnapshot,
        now: u64,
        lines: u64,
        monitor: usize,
        thread: thread::ThreadId,
    }

    // Exactly one real parser and owned worker; the channel only witnesses completed
    // reads. No hand-counted snapshots or local episode counters drive these tests.
    struct Harness {
        scratch: Scratch,
        source: Source,
        workers: AttentionWorkers,
        controller: AttentionController,
        reports: mpsc::Receiver<ReadEvidence>,
        owner: Option<(usize, thread::ThreadId)>,
        lines: u64,
        lock: (String, std::time::SystemTime, std::time::SystemTime),
    }

    impl Harness {
        fn new(history: &str) -> Self {
            let scratch = Scratch::new();
            let source = sources(&["copilot"], &scratch.0).remove(0);
            let session = source.path.join("copilot-0");
            fs::create_dir_all(&session).unwrap();
            fs::write(session.join("inuse.1.lock"), "").unwrap();
            fs::write(session.join("events.jsonl"), history).unwrap();
            let metadata = fs::metadata(session.join("inuse.1.lock")).unwrap();
            let lock = (
                file_identity(&session.join("inuse.1.lock")),
                metadata.created().unwrap(),
                metadata.modified().unwrap(),
            );
            let (send, reports) = mpsc::sync_channel(1);
            let workers = AttentionWorkers::with_readers(vec![source.clone()], NOW, |source| {
                let mut monitor = Box::new(AttentionMonitor::new(vec![source.clone()]).unwrap());
                let address = (&*monitor as *const AttentionMonitor) as usize;
                let send = send.clone();
                Box::new(move |now| {
                    let snapshot = monitor.poll_at(now).sources.remove(0);
                    send.send(ReadEvidence {
                        snapshot: snapshot.clone(),
                        now,
                        lines: monitor.lines_processed(),
                        monitor: address,
                        thread: thread::current().id(),
                    })
                    .unwrap();
                    snapshot
                })
            })
            .unwrap();
            Self {
                scratch,
                source,
                workers,
                controller: AttentionController::default(),
                reports,
                owner: None,
                lines: 0,
                lock,
            }
        }

        fn log(&self) -> PathBuf {
            self.source.path.join("copilot-0/events.jsonl")
        }
        fn parked(&self) -> PathBuf {
            self.scratch.0.join("offline")
        }

        fn append_error(&self, now: u64) {
            assert_eq!(now % 1_000, 0);
            append(
                &self.source,
                0,
                &format!(
                    "{{\"type\":\"session.error\",\"timestamp\":\"1970-01-01T00:{:02}:{:02}Z\"}}\n",
                    now / 60_000,
                    (now / 1_000) % 60,
                ),
            );
        }

        fn poll(&mut self, now: u64) -> AttentionSnapshot {
            self.workers.request_poll(now);
            let report = self.reports.recv_timeout(Duration::from_secs(1)).unwrap();
            assert_eq!(report.now, now);
            assert_eq!(self.workers.worker_count(), 1);
            let owner = (report.monitor, report.thread);
            if let Some(previous) = self.owner {
                assert_eq!(previous, owner);
            }
            self.owner = Some(owner);
            self.lines = report.lines;
            let expected = serde_json::to_value(&report.snapshot).unwrap();
            let started = Instant::now();
            loop {
                let value = self.workers.drain_at(now);
                if serde_json::to_value(&value.sources[0]).unwrap() == expected
                    && value.sources[0].membership.complete == report.snapshot.membership.complete
                    && value.sources[0].membership.instances == report.snapshot.membership.instances
                {
                    return value;
                }
                assert!(
                    started.elapsed() < Duration::from_secs(1),
                    "parser result not published"
                );
                thread::yield_now();
            }
        }

        fn tick(&mut self, now: u64) -> Option<NotificationRequest> {
            let value = self.poll(now);
            self.controller.tick(value, now)
        }

        fn baseline(&mut self) {
            assert!(self.tick(NOW).is_none());
            assert_eq!(
                self.controller.snapshot().sources[0].sessions[0].state,
                SessionState::Idle
            );
            assert_eq!(self.lines, 0);
        }

        fn first_error(&mut self) -> NotificationRequest {
            self.baseline();
            self.append_error(21_000);
            self.tick(21_000).expect("first post-baseline error alerts")
        }

        fn unavailable(&mut self, now: u64) {
            fs::rename(&self.source.path, self.parked()).unwrap();
            let value = self.poll(now);
            assert!(value.sources[0].sessions.is_empty());
            assert!(!value.sources[0].membership.complete);
            assert_eq!(
                value.sources[0].diagnostics,
                [Diagnostic::SourceUnavailable]
            );
            assert!(self.controller.tick(value, now).is_none());
        }

        // Rejecting a nonregular lock is intentional. It makes membership unknown,
        // not absent, while the actual lock, log, parser and worker stay unchanged.
        fn partial_scan(&mut self, now: u64) {
            let identity = file_identity(&self.log());
            let bytes = fs::read(self.log()).unwrap();
            let lines = self.lines;
            let marker = self.source.path.join("copilot-0/inuse.transient.lock");
            fs::create_dir(&marker).unwrap();
            let value = self.poll(now);
            assert!(value.sources[0].sessions.is_empty());
            assert!(!value.sources[0].membership.complete);
            assert!(value.sources[0].membership.instances.is_empty());
            assert_eq!(
                value.sources[0].diagnostics,
                [Diagnostic::SessionUnavailable]
            );
            assert!(self.controller.tick(value, now).is_none());
            fs::remove_dir(marker).unwrap();
            self.assert_lock_unchanged();
            assert_eq!(file_identity(&self.log()), identity);
            assert_eq!(fs::read(self.log()).unwrap(), bytes);
            assert_eq!(self.lines, lines, "partial scan cannot consume events");
        }

        fn restore(&mut self) {
            fs::rename(self.parked(), &self.source.path).unwrap();
            self.assert_lock_unchanged();
        }

        fn assert_lock_unchanged(&self) {
            let path = self.source.path.join("copilot-0/inuse.1.lock");
            let metadata = fs::metadata(&path).unwrap();
            assert_eq!(
                self.lock,
                (
                    file_identity(&path),
                    metadata.created().unwrap(),
                    metadata.modified().unwrap()
                )
            );
        }
    }

    impl Drop for Harness {
        fn drop(&mut self) {
            self.workers.stop();
        }
    }

    // Acknowledged error -> unavailable root -> same-inode truncation -> observed
    // empty recovery baseline -> NEW persisted error must not inherit acknowledgement.
    #[test]
    fn copilot_new_error_after_source_cache_loss() {
        let mut h = Harness::new("");
        let first = h.first_error();
        h.controller.submitted(first.id, true, 21_000);
        assert!(h.tick(27_000).is_none());
        let event_identity = file_identity(&h.log());
        h.unavailable(28_000);
        fs::write(h.parked().join("copilot-0/events.jsonl"), "").unwrap();
        h.restore();
        let empty = h.poll(29_000);
        assert_eq!(file_identity(&h.log()), event_identity);
        assert_eq!(fs::metadata(h.log()).unwrap().len(), 0);
        assert_eq!(
            empty.sources[0].sessions[0].instance_id,
            first.episodes[0].session.instance_id
        );
        assert_eq!(empty.sources[0].sessions[0].state, SessionState::Idle);
        assert!(empty.sources[0].sessions[0].attention.is_none());
        assert_eq!(h.lines, 1);
        assert!(h.controller.tick(empty, 29_000).is_none());
        h.append_error(30_000);
        let new = h.poll(30_000);
        let session = &new.sources[0].sessions[0];
        assert_eq!(session.state, SessionState::Error);
        let episode = session.attention.as_ref().unwrap();
        assert!(episode.notify);
        assert_eq!(episode.reason, EpisodeReason::SessionError);
        assert_eq!(h.lines, 2);
        let new_id = episode.id;
        let request = h.controller.tick(new.clone(), 30_000);
        h.controller.refresh();
        let after_refresh = h.controller.tick(new, 36_000);
        eprintln!(
            "same lock/log identity; old_id={}, new_id={new_id}, eligible={}, after_refresh={}",
            first.episodes[0].episode.id,
            request.is_some(),
            after_refresh.is_some()
        );
        assert!(
            request.is_some(),
            "NEW persisted error after empty recovery baseline inherited old acknowledgement"
        );
        let request = request.unwrap();
        assert_eq!(request.episodes[0].session, first.episodes[0].session);
        assert_ne!(new_id, first.episodes[0].episode.id);
        h.controller.submitted(request.id, true, 36_000);
        assert!(h.tick(37_000).is_none());
    }

    // Source loss without a log boundary preserves the SAME acknowledged episode;
    // neither recovery nor Refresh replays it, but a genuinely new append alerts.
    #[test]
    fn copilot_source_recovery_preserves_acknowledged_error() {
        let mut h = Harness::new("");
        let first = h.first_error();
        h.controller.submitted(first.id, true, 21_000);
        h.unavailable(22_000);
        h.restore();
        let recovered = h.poll(23_000);
        assert_eq!(
            recovered.sources[0].sessions[0]
                .attention
                .as_ref()
                .unwrap()
                .id,
            first.episodes[0].episode.id
        );
        assert!(h.controller.tick(recovered, 23_000).is_none());
        h.controller.refresh();
        assert!(h.tick(29_000).is_none());
        h.append_error(30_000);
        let new = h.tick(30_000).unwrap();
        assert_eq!(new.episodes[0].session, first.episodes[0].session);
        assert_ne!(new.episodes[0].episode.id, first.episodes[0].episode.id);
    }

    // An unsubmitted request is cancelled while unavailable; recovery keeps its
    // consumed attempt and spacing instead of silently treating it as historical.
    #[test]
    fn copilot_source_recovery_preserves_pending_retry_budget() {
        let mut h = Harness::new("");
        let obsolete = h.first_error();
        h.unavailable(22_000);
        h.controller.submitted(obsolete.id, true, 22_000);
        h.restore();
        assert!(h.tick(23_000).is_none());
        assert!(h.tick(25_999).is_none());
        for now in [26_000, 31_000] {
            let retry = h
                .tick(now)
                .expect("known undelivered error keeps retry eligibility");
            assert_eq!(retry.episodes, obsolete.episodes);
            h.controller.submitted(retry.id, false, now);
        }
        assert!(
            h.tick(36_000).is_none(),
            "initial pending attempt still counts toward three"
        );
        h.controller.refresh();
        let rearmed = h
            .tick(37_000)
            .expect("Refresh rearms exhausted undelivered error");
        h.controller.submitted(rearmed.id, true, 37_000);
        assert!(h.tick(43_000).is_none());
    }

    // Exhaustion survives source loss; explicit Refresh alone can rearm that SAME
    // error. A new persisted event has its OWN three attempts and ignores old callbacks.
    #[test]
    fn copilot_source_recovery_preserves_exhaustion_and_new_error_budget() {
        let mut h = Harness::new("");
        let first = h.first_error();
        h.controller.submitted(first.id, false, 21_000);
        for now in [26_000, 31_000] {
            let retry = h.tick(now).unwrap();
            assert_eq!(retry.episodes, first.episodes);
            h.controller.submitted(retry.id, false, now);
        }
        h.unavailable(32_000);
        h.restore();
        assert!(h.tick(33_000).is_none());
        assert!(
            h.tick(36_000).is_none(),
            "recovery cannot reset exhausted budget"
        );
        h.controller.refresh();
        let old = h
            .tick(37_000)
            .expect("same exhausted error remains explicitly rearmable");
        assert_eq!(old.episodes, first.episodes);
        h.append_error(38_000);
        let fresh = h
            .tick(38_000)
            .expect("new error is independent of old retry budget");
        assert_ne!(fresh.episodes[0].episode.id, first.episodes[0].episode.id);
        assert_eq!(fresh.episodes[0].session, first.episodes[0].session);
        h.controller.submitted(old.id, true, 38_000);
        h.controller.submitted(fresh.id, false, 38_000);
        for now in [43_000, 48_000] {
            assert!(h.tick(now - 1).is_none());
            let retry = h.tick(now).expect("new error retains all three attempts");
            assert_eq!(retry.episodes, fresh.episodes);
            h.controller.submitted(retry.id, false, now);
        }
        assert!(h.tick(53_000).is_none());
    }

    // First submission actually fails; a partial lock scan must not relabel the
    // unchanged error historical/acknowledged or discard either remaining retry.
    #[test]
    fn copilot_partial_scan_preserves_pending_retry_budget() {
        let mut h = Harness::new("");
        let first = h.first_error();
        assert!(first.episodes[0].episode.id > 0);
        h.controller.submitted(first.id, false, 21_250);
        h.partial_scan(22_000);
        assert!(h.tick(23_000).is_none());
        h.controller.refresh();
        assert!(h.tick(26_249).is_none());
        let retry = h
            .tick(26_250)
            .expect("partial scan must not acknowledge failed submission");
        assert_eq!(retry.episodes, first.episodes);
        assert_eq!(
            h.lines, 1,
            "unchanged error must not be reparsed as history"
        );
        h.controller.submitted(retry.id, false, 26_500);
        assert!(h.tick(31_499).is_none());
        let last = h.tick(31_500).expect("second retry survives partial scan");
        assert_eq!(last.episodes, first.episodes);
        h.controller.submitted(last.id, false, 31_500);
        assert!(
            h.tick(36_500).is_none(),
            "exactly three attempts, including before scan"
        );
        h.controller.refresh();
        let rearmed = h
            .tick(36_501)
            .expect("undelivered error remains Refresh-rearmable");
        assert_eq!(rearmed.episodes, first.episodes);
        h.controller.submitted(rearmed.id, true, 36_501);
        h.controller.refresh();
        assert!(h.tick(42_000).is_none());
    }

    // Exhausted is NOT acknowledged: recovery cannot reset its three-attempt
    // budget, and explicit Refresh still rearms the same error with 5s spacing.
    #[test]
    fn copilot_partial_scan_preserves_exhausted_refresh() {
        let mut h = Harness::new("");
        let first = h.first_error();
        h.controller.submitted(first.id, false, 21_000);
        for now in [26_000, 31_000] {
            assert!(h.tick(now - 1).is_none());
            let retry = h.tick(now).unwrap();
            assert_eq!(retry.episodes, first.episodes);
            h.controller.submitted(retry.id, false, now);
        }
        h.partial_scan(32_000);
        assert!(
            h.tick(33_000).is_none(),
            "recovery cannot renew exhausted budget"
        );
        h.controller.refresh();
        assert!(
            h.tick(35_999).is_none(),
            "Refresh cannot bypass submission spacing"
        );
        let rearmed = h
            .tick(36_000)
            .expect("partial scan must not acknowledge exhausted error");
        assert_eq!(rearmed.episodes, first.episodes);
        assert_eq!(
            h.lines, 1,
            "unchanged error must not be reparsed as history"
        );
        h.controller.submitted(rearmed.id, true, 36_000);
        h.controller.refresh();
        assert!(h.tick(42_000).is_none());
    }

    // Successful acknowledgement stays quiet across the SAME incomplete scan;
    // retaining parser provenance must not replay old errors or hide new ones.
    #[test]
    fn copilot_partial_scan_preserves_acknowledged_error() {
        let mut h = Harness::new("");
        let first = h.first_error();
        h.controller.submitted(first.id, true, 21_000);
        h.partial_scan(22_000);
        assert!(h.tick(23_000).is_none());
        let recovered = &h.controller.snapshot().sources[0].sessions[0];
        assert_eq!(recovered.instance_id, first.episodes[0].session.instance_id);
        assert_eq!(
            recovered.attention.as_ref(),
            Some(&first.episodes[0].episode)
        );
        assert_eq!(h.lines, 1);
        h.controller.refresh();
        assert!(h.tick(29_000).is_none());
        h.append_error(30_000);
        let fresh = h.tick(30_000).expect("new append remains eligible");
        assert_eq!(fresh.episodes[0].session, first.episodes[0].session);
        assert_ne!(fresh.episodes[0].episode.id, first.episodes[0].episode.id);
    }

    // Same-lock, same-length log replacement is a REAL file identity boundary,
    // even during source loss: replacement history stays quiet, later append alerts.
    #[test]
    fn copilot_historical_log_replacement_stays_quiet() {
        let mut h =
            Harness::new("{\"type\":\"session.error\",\"timestamp\":\"1970-01-01T00:00:19Z\"}\n");
        assert!(h.tick(NOW).is_none());
        let first = h.controller.snapshot().sources[0].sessions[0].clone();
        assert!(!first.attention.as_ref().unwrap().notify);
        let identity = file_identity(&h.log());
        let length = fs::metadata(h.log()).unwrap().len();
        h.unavailable(22_000);
        let old = h.parked().join("copilot-0/events.jsonl");
        fs::rename(&old, h.scratch.0.join("previous-events")).unwrap();
        fs::write(
            &old,
            "{\"type\":\"session.error\",\"timestamp\":\"1970-01-01T00:00:21Z\"}\n",
        )
        .unwrap();
        assert_ne!(file_identity(&old), identity);
        assert_eq!(fs::metadata(&old).unwrap().len(), length);
        h.restore();
        assert!(h.tick(23_000).is_none());
        let recovered = &h.controller.snapshot().sources[0].sessions[0];
        assert_eq!(recovered.instance_id, first.instance_id);
        assert_eq!(recovered.state, SessionState::Error);
        assert!(!recovered.attention.as_ref().unwrap().notify);
        h.controller.refresh();
        assert!(h.tick(29_000).is_none());
        h.append_error(30_000);
        let fresh = h.tick(30_000).unwrap();
        h.controller.submitted(fresh.id, true, 30_000);
        assert!(h.tick(36_000).is_none());
    }

    // Ordinary observed truncation without source loss must keep historical
    // suppression and allocate a distinct eligible ID for a later new error.
    #[test]
    fn copilot_observed_truncation_renews_error() {
        let mut h = Harness::new("");
        let first = h.first_error();
        h.controller.submitted(first.id, true, 21_000);
        fs::write(h.log(), "").unwrap();
        assert!(h.tick(22_000).is_none());
        h.append_error(23_000);
        let new = h.tick(23_000).unwrap();
        assert_eq!(new.episodes[0].session, first.episodes[0].session);
        assert_ne!(new.episodes[0].episode.id, first.episodes[0].episode.id);
    }

    // Reconstruct ALL parser/worker state while a controller retains delivery
    // history. The same lock is not a new instance and cannot restart error IDs.
    #[test]
    fn copilot_monitor_reconstruction_never_reuses_error_id() {
        let mut h = Harness::new("");
        let first = h.first_error();
        h.controller.submitted(first.id, true, 21_000);
        h.unavailable(22_000);
        h.restore();
        fs::write(h.log(), "").unwrap();
        let clock = Arc::new(AtomicU64::new(23_000));
        let observation_clock = clock.clone();
        h.workers.stop();
        h.workers = AttentionWorkers::with_clock(vec![h.source.clone()], 23_000, move || {
            observation_clock.load(Ordering::SeqCst)
        })
        .unwrap();
        let empty = poll_once(&mut h.workers, 23_000, |v| {
            !v.sources[0].sessions.is_empty()
        });
        assert_eq!(empty.sources[0].sessions[0].state, SessionState::Idle);
        assert_eq!(
            empty.sources[0].sessions[0].instance_id,
            first.episodes[0].session.instance_id
        );
        assert!(h.controller.tick(empty, 23_000).is_none());
        h.append_error(24_000);
        clock.store(24_000, Ordering::SeqCst);
        let new = poll_once(&mut h.workers, 24_000, |v| {
            v.sources[0].sessions[0].attention.is_some()
        });
        let request = h
            .controller
            .tick(new, 24_000)
            .expect("reconstructed monitor cannot reuse acknowledged ID");
        assert_eq!(request.episodes[0].session, first.episodes[0].session);
        assert_ne!(request.episodes[0].episode.id, first.episodes[0].episode.id);
    }
}

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use serde_json::json;
use tray_goblin::{
    attention::{
        AttentionController, AttentionEpisode, AttentionSnapshot, Capabilities, Diagnostic,
        EpisodeReason, SessionSnapshot, SessionState, SourceMembership, SourceSnapshot,
    },
    attention_worker::{AttentionWorkers, SourceReader},
    config::{MonitorConfig, Source},
};

const NOW: u64 = 20_000;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn sources(kinds: &[&str], root: &Path) -> Vec<Source> {
    let values: Vec<_> = kinds
        .iter()
        .enumerate()
        .map(|(i, kind)| {
            json!({"id":format!("source-{i}"),"label":format!("Synthetic {i}"),"kind":kind,
            "path":root.join(format!("source-{i}"))})
        })
        .collect();
    MonitorConfig::parse(&json!({"sources": values}).to_string())
        .unwrap()
        .resolved_sources()
        .unwrap()
}

fn session(id: &str, state: SessionState, episode: u64) -> SessionSnapshot {
    SessionSnapshot {
        session_id: id.into(),
        instance_id: format!("instance-{id}"),
        directory: Some("/synthetic/shared".into()),
        model: Some("synthetic".into()),
        state,
        attention: match state {
            SessionState::InputRequired => Some(EpisodeReason::InputRequired),
            SessionState::Error => Some(EpisodeReason::RunError),
            _ => None,
        }
        .map(|reason| AttentionEpisode {
            id: episode,
            reason,
            notify: true,
        }),
        run_started_at_ms: None,
        last_run_duration_ms: None,
        publisher_expires_at_ms: None,
    }
}

fn snapshot(sessions: Vec<SessionSnapshot>) -> AttentionSnapshot {
    AttentionSnapshot {
        sources: vec![source_snapshot(
            &sources(&["pi"], Path::new("/synthetic"))[0],
            sessions,
        )],
    }
}

fn source_snapshot(source: &Source, sessions: Vec<SessionSnapshot>) -> SourceSnapshot {
    SourceSnapshot {
        source: source.clone(),
        capabilities: Capabilities {
            input_detection: source.kind == tray_goblin::config::Provider::Pi,
            session_errors: true,
        },
        membership: SourceMembership {
            instances: sessions.iter().map(|s| s.instance_id.clone()).collect(),
            complete: true,
        },
        sessions,
        unavailable_pi_statuses: 0,
        diagnostics: Vec::new(),
    }
}

fn one(state: SessionState, episode: u64) -> AttentionSnapshot {
    snapshot(vec![session("one", state, episode)])
}

// Proves coalescing, same-directory/source/session identity and success-only dedupe.
#[test]
fn coalesces_distinct_identities_once_per_tick_and_acknowledges_only_success() {
    let mut controller = AttentionController::default();
    let mut value = snapshot(vec![
        session("one", SessionState::InputRequired, 1),
        session("two", SessionState::Error, 1),
    ]);
    let mut second = value.sources[0].clone();
    second.source.id = "other-source".into();
    value.sources.push(second);
    let request = controller.tick(value.clone(), NOW).unwrap();
    assert_eq!(request.episodes.len(), 4);
    let identities: std::collections::HashSet<_> =
        request.episodes.iter().map(|e| &e.session).collect();
    assert_eq!(identities.len(), 4);
    assert!(controller.tick(value.clone(), NOW).is_none());
    controller.submitted(request.id, false, NOW);
    assert!(controller.tick(value.clone(), NOW + 4_999).is_none());
    let retry = controller.tick(value.clone(), NOW + 5_000).unwrap();
    assert_eq!(retry.episodes.len(), 4);
    controller.submitted(retry.id, true, NOW + 5_000);
    for now in [NOW + 5_001, NOW + 10_000, NOW + 60_000] {
        assert!(controller.tick(value.clone(), now).is_none());
    }
    assert_eq!(controller.snapshot().sources[0].sessions.len(), 2);
}

// Proves exactly three attempts, inclusive 5s spacing, persistence and explicit rearm.
#[test]
fn retry_exhaustion_stays_visible_and_refresh_respects_spacing_and_success() {
    let mut controller = AttentionController::default();
    let value = one(SessionState::Error, 1);
    for attempt in 0..3 {
        let now = NOW + attempt * 5_000;
        if attempt != 0 {
            assert!(controller.tick(value.clone(), now - 1).is_none());
        }
        let request = controller.tick(value.clone(), now).unwrap();
        controller.submitted(request.id, false, now);
    }
    assert!(controller.tick(value.clone(), NOW + 14_999).is_none());
    controller.refresh();
    assert!(controller.tick(value.clone(), NOW + 14_999).is_none());
    let request = controller.tick(value.clone(), NOW + 15_000).unwrap();
    controller.submitted(request.id, true, NOW + 15_000);
    controller.refresh();
    assert!(controller.tick(value.clone(), NOW + 20_000).is_none());
    assert!(
        controller.snapshot().sources[0].sessions[0]
            .attention
            .is_some()
    );

    let mut exhausted = AttentionController::default();
    for i in 0..3 {
        let now = NOW + 5_000 * i;
        let request = exhausted.tick(value.clone(), now).unwrap();
        exhausted.submitted(request.id, false, now);
    }
    assert!(exhausted.tick(value, NOW + 100_000).is_none());
    assert!(
        exhausted.snapshot().sources[0].sessions[0]
            .attention
            .is_some()
    );
}

// Proves retry spacing starts at actual adapter completion, not request creation.
#[test]
fn delayed_adapter_failure_cannot_compress_retries() {
    let mut controller = AttentionController::default();
    let value = one(SessionState::InputRequired, 1);
    let request = controller.tick(value.clone(), NOW).unwrap();
    assert!(controller.tick(value.clone(), NOW + 10_000).is_none());
    controller.submitted(request.id, false, NOW + 10_000);
    assert!(controller.tick(value.clone(), NOW + 14_999).is_none());
    assert!(controller.tick(value, NOW + 15_000).is_some());
}

// Proves an older error restored after a dialog is not a new attention episode.
#[test]
fn restored_error_keeps_acknowledgement_while_new_errors_and_dialogs_alert() {
    let mut controller = AttentionController::default();
    for (state, episode, now) in [
        (SessionState::Error, 1, NOW),
        (SessionState::InputRequired, 2, NOW + 1),
    ] {
        let request = controller.tick(one(state, episode), now).unwrap();
        controller.submitted(request.id, true, now);
    }
    assert!(
        controller
            .tick(one(SessionState::Error, 1), NOW + 2)
            .is_none()
    );
    let request = controller
        .tick(one(SessionState::InputRequired, 3), NOW + 3)
        .unwrap();
    controller.submitted(request.id, true, NOW + 3);
    assert!(
        controller
            .tick(one(SessionState::Error, 1), NOW + 4)
            .is_none()
    );
    assert!(
        controller
            .tick(one(SessionState::Error, 4), NOW + 5)
            .is_some()
    );
}

// Proves an error first observed after its dialog still alerts despite a lower ID.
#[test]
fn first_observed_restored_error_is_not_suppressed_by_episode_order() {
    let mut controller = AttentionController::default();
    let request = controller
        .tick(one(SessionState::InputRequired, 2), NOW)
        .unwrap();
    controller.submitted(request.id, true, NOW);
    let error = controller
        .tick(one(SessionState::Error, 1), NOW + 1)
        .unwrap();
    assert_eq!(error.episodes[0].episode.id, 1);
}

// Proves transient source loss hides stale attention without re-alerting an
// acknowledged same-instance episode; confirmed removal still forgets the instance.
#[test]
fn unavailable_source_preserves_bounded_dedupe_but_not_pending_attention() {
    let mut controller = AttentionController::default();
    let value = one(SessionState::Error, 1);
    let request = controller.tick(value.clone(), NOW).unwrap();
    controller.submitted(request.id, true, NOW);
    let mut unavailable = snapshot(vec![]);
    unavailable.sources[0]
        .diagnostics
        .push(Diagnostic::SourceUnavailable);
    assert!(controller.tick(unavailable, NOW + 11_000).is_none());
    assert!(controller.snapshot().sources[0].sessions.is_empty());
    assert!(controller.tick(value.clone(), NOW + 12_000).is_none());
    assert!(controller.tick(snapshot(vec![]), NOW + 13_000).is_none());
    assert!(controller.tick(value, NOW + 14_000).is_some());
}

// Proves cleared/removed/replaced episodes cannot be acknowledged by stale callbacks.
#[test]
fn drops_obsolete_pending_and_never_inherits_delivery_across_instances() {
    let mut controller = AttentionController::default();
    let obsolete = controller.tick(one(SessionState::Error, 1), NOW).unwrap();
    assert!(
        controller
            .tick(one(SessionState::Idle, 0), NOW + 1)
            .is_none()
    );
    let renewed = controller
        .tick(one(SessionState::Error, 2), NOW + 2)
        .unwrap();
    controller.submitted(obsolete.id, true, NOW + 2);
    controller.submitted(renewed.id, false, NOW + 2);
    assert!(
        controller
            .tick(one(SessionState::Error, 2), NOW + 5_002)
            .is_some()
    );
    assert!(controller.tick(snapshot(vec![]), NOW + 5_003).is_none());
    let mut replacement = one(SessionState::Error, 2);
    replacement.sources[0].sessions[0].instance_id = "replacement".into();
    assert!(controller.tick(replacement, NOW + 5_004).is_some());
}

// Proves historical Copilot errors, completion, abort and ordinary work remain quiet.
#[test]
fn quiet_states_and_historical_errors_do_not_request_notifications() {
    let mut controller = AttentionController::default();
    for (i, state) in [
        SessionState::Idle,
        SessionState::Working,
        SessionState::Idle,
        SessionState::Unavailable,
    ]
    .into_iter()
    .enumerate()
    {
        assert!(controller.tick(one(state, 0), NOW + i as u64).is_none());
    }
    let mut historical = one(SessionState::Error, 1);
    historical.sources[0].sessions[0]
        .attention
        .as_mut()
        .unwrap()
        .notify = false;
    assert!(controller.tick(historical, NOW + 10).is_none());
    assert!(
        controller
            .tick(one(SessionState::Idle, 0), NOW + 11)
            .is_none()
    );
    assert!(
        controller
            .tick(one(SessionState::Error, 2), NOW + 12)
            .is_some()
    );
}

// Proves a mixed retry/new burst coalesces only eligible, current episodes.
#[test]
fn mixed_retries_prune_cleared_episodes_without_rearming_delivered_ones() {
    let mut controller = AttentionController::default();
    let initial = snapshot(vec![
        session("one", SessionState::Error, 1),
        session("two", SessionState::InputRequired, 2),
    ]);
    let request = controller.tick(initial, NOW).unwrap();
    let changed = snapshot(vec![
        session("one", SessionState::Idle, 0),
        session("two", SessionState::InputRequired, 2),
    ]);
    assert!(controller.tick(changed, NOW + 1).is_none());
    controller.submitted(request.id, false, NOW + 1);
    let changed = snapshot(vec![
        session("one", SessionState::Error, 3),
        session("two", SessionState::InputRequired, 2),
        session("three", SessionState::InputRequired, 1),
    ]);
    let request = controller.tick(changed.clone(), NOW + 5_001).unwrap();
    assert_eq!(request.episodes.len(), 3);
    assert_eq!(request.episodes[0].episode.id, 3);
    controller.submitted(request.id, true, NOW + 5_001);
    assert!(controller.tick(changed, NOW + 10_001).is_none());
}

#[track_caller]
fn await_snapshot(
    workers: &mut AttentionWorkers,
    now_ms: u64,
    predicate: impl Fn(&AttentionSnapshot) -> bool,
) -> AttentionSnapshot {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        workers.request_poll(now_ms);
        let value = workers.drain_at(now_ms);
        if predicate(&value) {
            return value;
        }
        assert!(
            Instant::now() < deadline,
            "source publication exceeded 1 second at {now_ms}: {:?}",
            value
                .sources
                .iter()
                .map(|s| (
                    s.sessions.len(),
                    &s.diagnostics,
                    s.sessions.first().map(|s| s.state)
                ))
                .collect::<Vec<_>>()
        );
        thread::sleep(Duration::from_millis(1));
    }
}

// Proves fixed readers survive ten stalls, bounded requests, fast UI, independent progress,
// stop during blocked I/O and rejection of a late result without joining the reader.
#[test]
fn delayed_reader_never_blocks_ui_or_grows_workers_and_stop_rejects_late_data() {
    let configured = sources(&["pi", "copilot"], Path::new("/synthetic"));
    let starts = Arc::new(AtomicUsize::new(0));
    let reads = Arc::new(AtomicUsize::new(0));
    let threads = Arc::new(Mutex::new(std::collections::HashSet::new()));
    let (release, permit) = mpsc::sync_channel::<()>(0);
    let (entered, entered_rx) = mpsc::sync_channel::<()>(1);
    let (exited, exited_rx) = mpsc::sync_channel::<()>(1);
    let mut permit = Some(permit);
    let count = starts.clone();
    let calls = reads.clone();
    let worker_threads = threads.clone();
    let mut workers = AttentionWorkers::with_readers(configured, NOW, move |source| {
        count.fetch_add(1, Ordering::SeqCst);
        let source = source.clone();
        let worker_threads = worker_threads.clone();
        if source.id == "source-0" {
            let permit = permit.take().unwrap();
            let calls = calls.clone();
            let entered = entered.clone();
            let exited = exited.clone();
            Box::new(move |_| {
                worker_threads
                    .lock()
                    .unwrap()
                    .insert(thread::current().id());
                calls.fetch_add(1, Ordering::SeqCst);
                entered.send(()).unwrap();
                permit.recv().unwrap();
                exited.send(()).unwrap();
                source_snapshot(&source, vec![session("blocked", SessionState::Error, 1)])
            }) as Box<dyn SourceReader>
        } else {
            Box::new(move |now: u64| {
                worker_threads
                    .lock()
                    .unwrap()
                    .insert(thread::current().id());
                source_snapshot(
                    &source,
                    vec![session(&now.to_string(), SessionState::Idle, 0)],
                )
            })
        }
    })
    .unwrap();

    for cycle in 0..10 {
        let now = NOW + cycle * 1_000;
        workers.request_poll(now);
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let start = Instant::now();
        for _ in 0..100 {
            workers.request_poll(now);
            workers.drain_at(now);
        }
        assert!(start.elapsed() < Duration::from_millis(250));
        let progress = Instant::now();
        await_snapshot(&mut workers, now, |v| {
            v.sources[1]
                .sessions
                .first()
                .is_some_and(|s| s.session_id == now.to_string())
        });
        assert!(progress.elapsed() < Duration::from_secs(1));
        assert_eq!(workers.worker_count(), 2);
        assert_eq!(starts.load(Ordering::SeqCst), 2);
        assert_eq!(threads.lock().unwrap().len(), 2);
        // A single queued poll is allowed; release both reads before the next cycle.
        release.send(()).unwrap();
        exited_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        release.send(()).unwrap();
        exited_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    }
    workers.request_poll(NOW + 10_000);
    entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let start = Instant::now();
    workers.stop();
    assert!(start.elapsed() < Duration::from_millis(250));
    assert_eq!(workers.worker_count(), 2);
    release.send(()).unwrap();
    exited_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(workers.drain_at(NOW + 10_001).sources.is_empty());
    workers.request_poll(NOW + 10_001);
    assert_eq!(reads.load(Ordering::SeqCst), 21);
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-scratch/controller")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn pi_file(source: &Source, i: usize, state: &str, episode: u64, now: u64) {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/attention/pi-v1.json")).unwrap();
    value["instanceId"] = json!(format!("{i:08x}-1111-4111-8111-111111111111"));
    value["sessionId"] = json!(format!("pi-{i}"));
    value["directory"] = json!("/synthetic/shared");
    value["state"] = json!(state);
    value["episode"] = json!(episode);
    value["attentionReason"] = if state == "error" {
        json!("run_error")
    } else {
        json!(null)
    };
    value["updatedAt"] = json!(now);
    fs::write(
        source
            .path
            .join(format!("{}.json", value["instanceId"].as_str().unwrap())),
        value.to_string(),
    )
    .unwrap();
}

fn append(source: &Source, i: usize, event: &str) {
    fs::OpenOptions::new()
        .append(true)
        .open(source.path.join(format!("copilot-{i}/events.jsonl")))
        .unwrap()
        .write_all(event.as_bytes())
        .unwrap();
}

fn poll_once(
    workers: &mut AttentionWorkers,
    now: u64,
    ready: impl Fn(&AttentionSnapshot) -> bool,
) -> AttentionSnapshot {
    workers.request_poll(now);
    let start = Instant::now();
    loop {
        let value = workers.drain_at(now);
        if ready(&value) {
            return value;
        }
        assert!(
            start.elapsed() < Duration::from_secs(1),
            "poll did not publish"
        );
        thread::yield_now();
    }
}

// Proves omitted stale placeholders cannot replay an acknowledged episode.
#[test]
fn saturated_stale_placeholders_preserve_acknowledgement() {
    saturated_membership_preserves_history(false, true);
}

// Proves omitted stale placeholders cannot reset an exhausted attempt budget.
#[test]
fn saturated_stale_placeholders_preserve_exhausted_attempts() {
    saturated_membership_preserves_history(false, false);
}

// Proves 64 live rows cannot erase acknowledgement of an omitted malformed file.
#[test]
fn saturated_live_sessions_preserve_malformed_acknowledgement() {
    saturated_membership_preserves_history(true, true);
}

// Proves 64 live rows cannot reset attempts for an omitted malformed file.
#[test]
fn saturated_live_sessions_preserve_malformed_exhausted_attempts() {
    saturated_membership_preserves_history(true, false);
}

fn saturated_membership_preserves_history(malformed: bool, acknowledged: bool) {
    let scratch = Scratch::new();
    let configured = sources(&["pi"], &scratch.0);
    let source = &configured[0];
    fs::create_dir_all(&source.path).unwrap();
    for i in 0..65 {
        pi_file(source, i, "idle", 0, 0);
    }
    let order = || {
        fs::read_dir(&source.path)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>()
    };
    let filenames = order();
    let affected_file = source.path.join(&filenames[64]);
    let affected = usize::from_str_radix(&filenames[64].to_str().unwrap()[..8], 16).unwrap();
    let mut workers = AttentionWorkers::with_readers(configured.clone(), NOW, |source| {
        let mut monitor =
            tray_goblin::monitor::AttentionMonitor::new(vec![source.clone()]).unwrap();
        Box::new(move |now| monitor.poll_at(now).sources.remove(0))
    })
    .unwrap();
    let mut controller = AttentionController::default();
    let mut now = NOW;
    for attempt in 0..if acknowledged { 1 } else { 3 } {
        now = NOW + attempt * 5_000;
        pi_file(source, affected, "error", 1, now);
        assert_eq!(order(), filenames, "fixture enumeration changed");
        let value = poll_once(&mut workers, now, |v| {
            v.sources[0].sessions.iter().any(|s| {
                s.session_id == format!("pi-{affected}")
                    && s.publisher_expires_at_ms == Some(now + 10_000)
            })
        });
        let request = controller.tick(value, now).unwrap();
        assert_eq!(request.episodes.len(), 1);
        controller.submitted(request.id, acknowledged, now);
    }
    now += 10_001;
    let diagnostic = if malformed {
        for i in 0..65 {
            if i != affected {
                pi_file(source, i, "idle", 0, now);
            }
        }
        fs::write(&affected_file, "{").unwrap();
        Diagnostic::InvalidPiStatus
    } else {
        Diagnostic::StalePiStatus
    };
    assert_eq!(order(), filenames, "fixture enumeration changed");
    let omitted = poll_once(&mut workers, now, |v| {
        v.sources[0].diagnostics.contains(&diagnostic)
            && v.sources[0]
                .diagnostics
                .contains(&Diagnostic::SessionOverflow)
    });
    assert_eq!(omitted.sources[0].sessions.len(), 64);
    assert_eq!(
        omitted.sources[0].unavailable_pi_statuses,
        if malformed { 1 } else { 65 }
    );
    assert!(
        !omitted.sources[0]
            .sessions
            .iter()
            .any(|s| { affected_file.file_stem().unwrap().to_str().unwrap() == s.instance_id })
    );
    eprintln!(
        "saturation measured: malformed={malformed}, ack={acknowledged}, \
         files={}, visible={}, unavailable={}, affected_visible=false",
        filenames.len(),
        omitted.sources[0].sessions.len(),
        omitted.sources[0].unavailable_pi_statuses,
    );
    assert!(controller.tick(omitted, now).is_none());
    now += 10_001;
    pi_file(source, affected, "error", 1, now);
    let recovered = poll_once(&mut workers, now, |v| {
        v.sources[0]
            .sessions
            .iter()
            .any(|s| s.session_id == format!("pi-{affected}") && s.state == SessionState::Error)
    });
    assert!(
        controller.tick(recovered, now).is_none(),
        "capacity omission reset history: malformed={malformed}, ack={acknowledged}"
    );
    if !acknowledged {
        controller.refresh();
        let request = controller.tick(workers.drain_at(now + 1), now + 1).unwrap();
        controller.submitted(request.id, true, now + 1);
    }
    pi_file(source, 65, "idle", 0, 0);
    fs::remove_file(&affected_file).unwrap();
    now += 1_000;
    let removed = poll_once(&mut workers, now, |v| {
        v.sources[0]
            .sessions
            .iter()
            .all(|s| s.state == SessionState::Unavailable)
            && !v.sources[0]
                .sessions
                .iter()
                .any(|s| affected_file.file_stem().unwrap().to_str().unwrap() == s.instance_id)
    });
    assert!(
        removed.sources[0]
            .diagnostics
            .contains(&Diagnostic::SessionOverflow)
    );
    assert!(controller.tick(removed, now).is_none());
    now += 1_000;
    pi_file(source, affected, "error", 1, now);
    let returned = poll_once(&mut workers, now, |v| {
        v.sources[0]
            .sessions
            .iter()
            .any(|s| s.state == SessionState::Error)
    });
    assert!(
        controller.tick(returned, now).is_some(),
        "complete membership must still confirm deletion under visible overflow"
    );
    workers.stop();
}

// Proves an actual 4096-entry scan omission cannot replay successful delivery.
#[test]
fn incomplete_filesystem_scan_preserves_acknowledgement() {
    incomplete_filesystem_scan_preserves_history(true);
}

// Proves an actual 4096-entry scan omission cannot reset exhausted attempts.
#[test]
fn incomplete_filesystem_scan_preserves_exhausted_attempts() {
    incomplete_filesystem_scan_preserves_history(false);
}

fn incomplete_filesystem_scan_preserves_history(acknowledged: bool) {
    let scratch = Scratch::new();
    let configured = sources(&["pi"], &scratch.0);
    let source = &configured[0];
    let mut saturated = source.clone();
    saturated.path = scratch.0.join("saturated");
    fs::create_dir_all(&saturated.path).unwrap();
    for i in 0..4097 {
        pi_file(&saturated, i, "idle", 0, 0);
    }
    let filenames: Vec<_> = fs::read_dir(&saturated.path)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    let affected = usize::from_str_radix(&filenames[4096].to_str().unwrap()[..8], 16).unwrap();
    fs::create_dir_all(&source.path).unwrap();
    let mut workers = AttentionWorkers::with_readers(configured.clone(), NOW, |source| {
        let mut monitor =
            tray_goblin::monitor::AttentionMonitor::new(vec![source.clone()]).unwrap();
        Box::new(move |now| monitor.poll_at(now).sources.remove(0))
    })
    .unwrap();
    let mut controller = AttentionController::default();
    let mut now = NOW;
    for attempt in 0..if acknowledged { 1 } else { 3 } {
        now = NOW + attempt * 5_000;
        pi_file(source, affected, "error", 1, now);
        let value = poll_once(&mut workers, now, |v| {
            v.sources[0]
                .sessions
                .first()
                .is_some_and(|s| s.publisher_expires_at_ms == Some(now + 10_000))
        });
        let request = controller.tick(value, now).unwrap();
        controller.submitted(request.id, acknowledged, now);
    }
    // Swap only the synthetic tree layout so the same filename is beyond the
    // actual scan boundary; source configuration and worker ownership stay fixed.
    let parked = scratch.0.join("parked");
    fs::rename(&source.path, &parked).unwrap();
    fs::rename(&saturated.path, &source.path).unwrap();
    now += 1_000;
    let omitted = poll_once(&mut workers, now, |v| {
        v.sources[0].unavailable_pi_statuses == 4096
    });
    let membership = &omitted.sources[0].membership;
    assert!(!membership.complete);
    assert_eq!(membership.instances.len(), 4096);
    let affected_instance = filenames[4096].to_str().unwrap().trim_end_matches(".json");
    assert!(!membership.instances.contains(affected_instance));
    assert!(controller.tick(omitted, now).is_none());
    fs::rename(&source.path, &saturated.path).unwrap();
    fs::rename(&parked, &source.path).unwrap();
    now += 1_000;
    pi_file(source, affected, "error", 1, now);
    let recovered = poll_once(&mut workers, now, |v| {
        v.sources[0]
            .sessions
            .first()
            .is_some_and(|s| s.state == SessionState::Error)
    });
    assert!(
        controller.tick(recovered, now).is_none(),
        "incomplete scan must not confirm removal: ack={acknowledged}"
    );
    workers.stop();
}

// Proves stale same-instance status cannot rearm exhausted attempts without Refresh.
#[test]
fn pi_stale_status_preserves_exhausted_attempts() {
    pi_publication_unavailability_preserves_delivery_history_until_removal(false, false);
}

// Proves stale same-instance status cannot replay a successfully submitted episode.
#[test]
fn pi_stale_status_preserves_acknowledgement() {
    pi_publication_unavailability_preserves_delivery_history_until_removal(false, true);
}

// Proves malformed same-instance status cannot reset the three-attempt retry budget.
#[test]
fn pi_malformed_status_preserves_exhausted_attempts() {
    pi_publication_unavailability_preserves_delivery_history_until_removal(true, false);
}

// Proves malformed same-instance status cannot replay an acknowledged episode.
#[test]
fn pi_malformed_status_preserves_acknowledgement() {
    pi_publication_unavailability_preserves_delivery_history_until_removal(true, true);
}

fn pi_publication_unavailability_preserves_delivery_history_until_removal(
    malformed: bool,
    acknowledged: bool,
) {
    let scratch = Scratch::new();
    let configured = sources(&["pi"], &scratch.0);
    let source = &configured[0];
    fs::create_dir_all(&source.path).unwrap();
    let mut workers = AttentionWorkers::with_readers(configured.clone(), NOW, |source| {
        let mut monitor =
            tray_goblin::monitor::AttentionMonitor::new(vec![source.clone()]).unwrap();
        Box::new(move |now| monitor.poll_at(now).sources.remove(0))
    })
    .unwrap();
    let mut controller = AttentionController::default();
    let mut now = NOW;
    for attempt in 0..if acknowledged { 1 } else { 3 } {
        now = NOW + attempt * 5_000;
        pi_file(source, 0, "error", 1, now);
        let value = poll_once(&mut workers, now, |v| {
            v.sources[0]
                .sessions
                .first()
                .is_some_and(|s| s.publisher_expires_at_ms == Some(now + 10_000))
        });
        let request = controller.tick(value, now).unwrap();
        assert_eq!(request.episodes[0].session.session_id, "pi-0");
        assert_ne!(
            request.episodes[0].session.session_id,
            request.episodes[0].session.instance_id
        );
        controller.submitted(request.id, acknowledged, now);
    }
    let file = source
        .path
        .join("00000000-1111-4111-8111-111111111111.json");
    let diagnostic = if malformed {
        fs::write(&file, "{").unwrap();
        now += 1;
        Diagnostic::InvalidPiStatus
    } else {
        now += 10_001;
        Diagnostic::StalePiStatus
    };
    let unavailable = poll_once(&mut workers, now, |v| {
        v.sources[0].diagnostics.contains(&diagnostic)
    });
    assert_eq!(
        unavailable.sources[0].sessions[0].state,
        SessionState::Unavailable
    );
    assert!(controller.tick(unavailable, now).is_none());
    now += 5_000;
    pi_file(source, 0, "error", 1, now);
    let recovered = poll_once(&mut workers, now, |v| {
        v.sources[0].sessions[0].state == SessionState::Error
    });
    assert!(
        controller.tick(recovered, now).is_none(),
        "same episode replayed: malformed={malformed}, acknowledged={acknowledged}"
    );
    assert!(
        controller.snapshot().sources[0].sessions[0]
            .attention
            .is_some()
    );
    if !acknowledged {
        controller.refresh();
        let request = controller.tick(workers.drain_at(now + 1), now + 1).unwrap();
        controller.submitted(request.id, true, now + 1);
    }

    fs::remove_file(&file).unwrap();
    now += 1_000;
    let removed = poll_once(&mut workers, now, |v| v.sources[0].sessions.is_empty());
    assert!(controller.tick(removed, now).is_none());
    now += 1_000;
    pi_file(source, 0, "error", 1, now);
    let returned = poll_once(&mut workers, now, |v| !v.sources[0].sessions.is_empty());
    let request = controller
        .tick(returned, now)
        .expect("confirmed removal prunes history");
    controller.submitted(request.id, true, now);
    fs::remove_file(&file).unwrap();
    now += 1_000;
    pi_file(source, 1, "error", 1, now);
    let replaced = poll_once(&mut workers, now, |v| {
        v.sources[0].sessions[0].session_id == "pi-1"
    });
    let request = controller
        .tick(replaced, now)
        .expect("replacement has a fresh identity");
    assert_eq!(request.episodes.len(), 1);
    assert_eq!(request.episodes[0].session.session_id, "pi-1");
    controller.submitted(request.id, true, now);
    fs::remove_file(
        source
            .path
            .join("00000001-1111-4111-8111-111111111111.json"),
    )
    .unwrap();
    now += 1_000;
    pi_file(source, 0, "error", 1, now);
    let replaced_back = poll_once(&mut workers, now, |v| {
        v.sources[0].sessions[0].session_id == "pi-0"
    });
    assert!(
        controller.tick(replaced_back, now).is_some(),
        "replacement must prune the old instance history"
    );
    workers.stop();
}

// Proves a timestamped Copilot error appended during blocked source I/O reaches
// the actual controller exactly once, while a genuinely future event stays rejected.
#[test]
fn timestamped_copilot_error_during_blocked_read_is_not_offset_consumed() {
    let scratch = Scratch::new();
    let configured = sources(&["copilot"], &scratch.0);
    let root = configured[0].path.join("copilot-0");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("inuse.1.lock"), "").unwrap();
    fs::write(root.join("workspace.yaml"), "cwd: /synthetic/shared\n").unwrap();
    fs::write(root.join("events.jsonl"), "").unwrap();
    let clock = Arc::new(AtomicU64::new(NOW));
    let (release, permit) = mpsc::sync_channel::<()>(0);
    let (entered, entered_rx) = mpsc::sync_channel::<()>(1);
    let mut permit = Some(permit);
    let mut workers = AttentionWorkers::with_readers(configured.clone(), NOW, |source| {
        let mut monitor =
            tray_goblin::monitor::AttentionMonitor::new(vec![source.clone()]).unwrap();
        let permit = permit.take().unwrap();
        let entered = entered.clone();
        let clock = clock.clone();
        Box::new(move |now| {
            if now == 21_000 {
                entered.send(()).unwrap();
                permit.recv().unwrap();
            }
            monitor
                .poll_with_clock(&|| clock.load(Ordering::SeqCst))
                .sources
                .remove(0)
        })
    })
    .unwrap();
    let mut controller = AttentionController::default();
    let initial = poll_once(&mut workers, NOW, |v| !v.sources[0].sessions.is_empty());
    assert!(controller.tick(initial, NOW).is_none());
    clock.store(21_000, Ordering::SeqCst);
    workers.request_poll(21_000);
    entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    append(
        &configured[0],
        0,
        "{\"type\":\"session.error\",\"timestamp\":\"1970-01-01T00:00:22Z\"}\n",
    );
    clock.store(23_000, Ordering::SeqCst);
    release.send(()).unwrap();
    let start = Instant::now();
    let observed = loop {
        let value = workers.drain_at(23_000);
        if value.sources[0].sessions[0].attention.is_some()
            || value.sources[0]
                .diagnostics
                .contains(&Diagnostic::MalformedEvent)
        {
            break value;
        }
        assert!(start.elapsed() < Duration::from_secs(1));
        thread::yield_now();
    };
    let request = controller.tick(observed, 23_000);
    append(
        &configured[0],
        0,
        "{\"type\":\"session.model_change\",\"data\":{\"model\":\"synthetic-read-again\"}}\n",
    );
    let read_again = poll_once(&mut workers, 23_000, |v| {
        v.sources[0].sessions[0].model.as_deref() == Some("synthetic-read-again")
    });
    assert!(
        read_again.sources[0].sessions[0].attention.is_some(),
        "valid mid-read error was permanently offset-consumed, even after another read"
    );
    assert!(
        request.is_some(),
        "valid mid-read error was rejected and consumed"
    );
    let request = request.unwrap();
    assert_eq!(request.episodes.len(), 1);
    assert_eq!(
        request.episodes[0].episode.reason,
        EpisodeReason::SessionError
    );
    let error_id = request.episodes[0].episode.id;
    assert!(error_id > 0);
    controller.submitted(request.id, true, 23_000);
    assert!(controller.tick(read_again, 23_001).is_none());
    append(
        &configured[0],
        0,
        "{\"type\":\"session.error\",\"timestamp\":\"1970-01-01T00:00:24Z\"}\n",
    );
    let future = poll_once(&mut workers, 23_000, |v| {
        v.sources[0]
            .diagnostics
            .contains(&Diagnostic::MalformedEvent)
    });
    assert!(controller.tick(future, 23_001).is_none());
    assert_eq!(
        controller.snapshot().sources[0].sessions[0]
            .attention
            .as_ref()
            .unwrap()
            .id,
        error_id
    );
    workers.stop();
}

// Proves an aging real pi publication expires during a blocked next read, without
// extending retry eligibility or expiring a newer session or independent Copilot source.
#[test]
fn cached_publisher_expiry_survives_blocked_source_reads() {
    const P: u64 = 1_000;
    let scratch = Scratch::new();
    let configured = sources(&["pi", "copilot"], &scratch.0);
    for source in &configured {
        fs::create_dir_all(&source.path).unwrap();
    }
    pi_file(&configured[0], 0, "input_required", 1, NOW);
    pi_file(&configured[0], 1, "error", 1, NOW + 9 * P);
    let copilot = configured[1].path.join("copilot-0");
    fs::create_dir_all(&copilot).unwrap();
    fs::write(copilot.join("inuse.1.lock"), "").unwrap();
    fs::write(copilot.join("workspace.yaml"), "cwd: /synthetic/shared\n").unwrap();
    fs::write(copilot.join("events.jsonl"), "").unwrap();

    let (release, permit) = mpsc::sync_channel::<()>(0);
    let (entered, entered_rx) = mpsc::sync_channel::<()>(1);
    let mut permit = Some(permit);
    let mut workers = AttentionWorkers::with_readers(configured.clone(), NOW + 9 * P, |source| {
        let mut monitor =
            tray_goblin::monitor::AttentionMonitor::new(vec![source.clone()]).unwrap();
        if source.id == "source-0" {
            let permit = permit.take().unwrap();
            let entered = entered.clone();
            Box::new(move |now| {
                if now == NOW + 10 * P {
                    entered.send(()).unwrap();
                    permit.recv().unwrap();
                }
                monitor.poll_at(now).sources.remove(0)
            }) as Box<dyn SourceReader>
        } else {
            Box::new(move |now| monitor.poll_at(now).sources.remove(0))
        }
    })
    .unwrap();
    // Send only one initial poll: repeated requests may leave an older queued
    // command that legitimately coalesces away the subsequent blocked-read poll.
    workers.request_poll(NOW + 9 * P);
    let wait = Instant::now();
    let initial = loop {
        let value = workers.drain_at(NOW + 9 * P);
        if value.sources[0].sessions.len() == 2 && value.sources[1].sessions.len() == 1 {
            break value;
        }
        assert!(wait.elapsed() < Duration::from_secs(1));
        thread::sleep(Duration::from_millis(1));
    };
    let mut controller = AttentionController::default();
    let first = controller.tick(initial, NOW + 9 * P).unwrap();
    assert_eq!(first.episodes.len(), 2);
    controller.submitted(first.id, false, NOW + 9 * P);
    append(&configured[1], 0, "{\"type\":\"session.error\"}\n");
    workers.request_poll(NOW + 10 * P);
    entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();

    let boundary = workers.drain_at(NOW + 10 * P);
    assert_eq!(
        boundary.sources[0].sessions[0].state,
        SessionState::InputRequired
    );
    let expired = workers.drain_at(NOW + 10 * P + 1);
    let deadline = workers.drain_at(NOW + 12 * P + 1);
    let progress = await_snapshot(&mut workers, NOW + 12 * P + 1, |v| {
        v.sources[1].sessions[0].state == SessionState::Error
    });
    let copilot_request = controller.tick(progress, NOW + 12 * P + 1).unwrap();
    controller.submitted(copilot_request.id, true, NOW + 12 * P + 1);
    let retry = controller.tick(workers.drain_at(NOW + 15 * P), NOW + 15 * P);

    // Release even before checking assertions so a failed counterfactual cannot strand a worker.
    workers.stop();
    release.send(()).unwrap();
    for value in [&expired, &deadline] {
        let source = &value.sources[0];
        assert_eq!(source.sessions[0].state, SessionState::Unavailable);
        assert!(source.sessions[0].attention.is_none());
        assert_eq!(source.sessions[1].state, SessionState::Error);
        assert!(source.sessions[1].attention.is_some());
        assert_eq!(source.unavailable_pi_statuses, 1);
        assert!(source.diagnostics.contains(&Diagnostic::StalePiStatus));
        assert!(!source.diagnostics.contains(&Diagnostic::SourceUnavailable));
    }
    assert_eq!(copilot_request.episodes.len(), 1);
    assert_eq!(copilot_request.episodes[0].session.source_id, "source-1");
    assert!(retry.is_some(), "unexpired pi error remains retry eligible");
    let retry = retry.unwrap();
    assert_eq!(retry.episodes.len(), 1);
    assert_eq!(retry.episodes[0].session.session_id, "pi-1");
    assert_eq!(workers.worker_count(), 2);
}

// Proves the production worker's injected observation clock is sampled after the
// pi bytes are read, and is not replaced with the earlier request time or real time.
#[test]
fn worker_observation_clock_is_sampled_after_reading_publisher_bytes() {
    let scratch = Scratch::new();
    let configured = sources(&["pi"], &scratch.0);
    fs::create_dir_all(&configured[0].path).unwrap();
    pi_file(&configured[0], 0, "error", 1, 22_000);
    let source = configured[0].clone();
    let clock = Arc::new(AtomicU64::new(23_000));
    let observation_clock = clock.clone();
    let swapped = AtomicUsize::new(0);
    let mut workers = AttentionWorkers::with_clock(configured, NOW, move || {
        let now = observation_clock.load(Ordering::SeqCst);
        if swapped.fetch_add(1, Ordering::SeqCst) == 0 {
            pi_file(&source, 0, "idle", 1, now);
        }
        now
    })
    .unwrap();
    workers.request_poll(21_000);
    let start = Instant::now();
    let observed = loop {
        let value = workers.drain_at(23_000);
        if !value.sources[0].sessions.is_empty() {
            break value;
        }
        assert!(start.elapsed() < Duration::from_secs(1));
        thread::yield_now();
    };
    assert_eq!(observed.sources[0].sessions[0].state, SessionState::Error);
    assert_eq!(
        observed.sources[0].sessions[0].publisher_expires_at_ms,
        Some(32_000)
    );
    let mut controller = AttentionController::default();
    let request = controller.tick(observed, 23_000).unwrap();
    controller.submitted(request.id, true, 23_000);
    clock.store(23_001, Ordering::SeqCst);
    let cleared = poll_once(&mut workers, 23_001, |v| {
        v.sources[0].sessions[0].state == SessionState::Idle
    });
    assert!(controller.tick(cleared, 23_001).is_none());
    workers.stop();
}

// Proves the real M1 schema -> M2 parser -> fixed workers -> controller path at 64 sessions.
// Injected clock asserts fresh/clear <=2P+500ms, idle error persistence, quiet recovered
// tools/success/abort, restored IDs and stale publishers <=10s+2P without wall-clock sleeps.
#[test]
fn real_workers_controller_sixty_four_mixed_sessions_recover_and_clear_on_time() {
    const P: u64 = 1_000;
    let scratch = Scratch::new();
    let configured = sources(&["pi", "copilot"], &scratch.0);
    for source in &configured {
        fs::create_dir_all(&source.path).unwrap();
    }
    for i in 0..32 {
        pi_file(&configured[0], i, "idle", 0, NOW);
        let root = configured[1].path.join(format!("copilot-{i}"));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("inuse.1.lock"), "").unwrap();
        fs::write(root.join("workspace.yaml"), "cwd: /synthetic/shared\n").unwrap();
        fs::write(root.join("events.jsonl"), "").unwrap();
    }
    let clock = Arc::new(AtomicU64::new(NOW));
    let observation_clock = clock.clone();
    let mut workers = AttentionWorkers::with_clock(configured.clone(), NOW, move || {
        observation_clock.load(Ordering::SeqCst)
    })
    .unwrap();
    let mut controller = AttentionController::default();
    let initial = await_snapshot(&mut workers, NOW, |v| {
        v.sources.iter().map(|s| s.sessions.len()).sum::<usize>() == 64
    });
    assert!(controller.tick(initial, NOW).is_none());

    for i in 0..32 {
        pi_file(&configured[0], i, "error", 1, NOW + 1);
        append(&configured[1], i, "{\"type\":\"session.error\"}\n");
    }
    let observed = NOW + P;
    clock.store(observed, Ordering::SeqCst);
    let errors = await_snapshot(&mut workers, observed, |v| {
        v.sources
            .iter()
            .flat_map(|s| &s.sessions)
            .filter(|s| s.attention.is_some())
            .count()
            == 64
    });
    assert!(observed - (NOW + 1) <= 2 * P + 500);
    let request = controller.tick(errors.clone(), observed).unwrap();
    assert_eq!(request.episodes.len(), 64);
    controller.submitted(request.id, true, observed);
    assert!(controller.tick(errors, NOW + P + 1).is_none());

    for i in 0..32 {
        pi_file(&configured[0], i, "input_required", 2, NOW + P + 2);
    }
    clock.store(NOW + 2 * P, Ordering::SeqCst);
    let dialogs = await_snapshot(&mut workers, NOW + 2 * P, |v| {
        v.sources[0].sessions.len() == 32
            && v.sources[0]
                .sessions
                .iter()
                .all(|s| s.state == SessionState::InputRequired)
    });
    let request = controller.tick(dialogs, NOW + 2 * P).unwrap();
    assert_eq!(request.episodes.len(), 32);
    controller.submitted(request.id, true, NOW + 2 * P);
    for i in 0..32 {
        pi_file(&configured[0], i, "error", 1, NOW + 2 * P + 1);
    }
    clock.store(NOW + 3 * P, Ordering::SeqCst);
    let restored = await_snapshot(&mut workers, NOW + 3 * P, |v| {
        v.sources[0].sessions.len() == 32
            && v.sources[0]
                .sessions
                .iter()
                .all(|s| s.state == SessionState::Error)
    });
    assert!(controller.tick(restored, NOW + 3 * P).is_none());

    for i in 0..32 {
        pi_file(&configured[0], i, "working", 2, NOW + 3 * P + 1);
        append(&configured[1], i, "{\"type\":\"assistant.turn_start\"}\n");
    }
    let cleared_at = NOW + 4 * P;
    clock.store(cleared_at, Ordering::SeqCst);
    let cleared = await_snapshot(&mut workers, cleared_at, |v| {
        v.sources.iter().map(|s| s.sessions.len()).sum::<usize>() == 64
            && v.sources
                .iter()
                .flat_map(|s| &s.sessions)
                .all(|s| s.state == SessionState::Working && s.attention.is_none())
    });
    assert!(cleared_at - (NOW + 3 * P + 1) <= 2 * P + 500);
    assert!(controller.tick(cleared, cleared_at).is_none());
    for i in 0..32 {
        pi_file(&configured[0], i, "idle", 2, NOW + 4 * P + 1);
        append(
            &configured[1],
            i,
            "{\"type\":\"tool.execution_complete\",\"data\":{\"success\":false}}\n{\"type\":\"assistant.turn_end\"}\n",
        );
    }
    clock.store(NOW + 5 * P, Ordering::SeqCst);
    let quiet = await_snapshot(&mut workers, NOW + 5 * P, |v| {
        v.sources.iter().map(|s| s.sessions.len()).sum::<usize>() == 64
            && v.sources
                .iter()
                .flat_map(|s| &s.sessions)
                .all(|s| s.state == SessionState::Idle && s.attention.is_none())
    });
    assert!(controller.tick(quiet, NOW + 5 * P).is_none());
    for i in 0..32 {
        pi_file(&configured[0], i, "input_required", 3, NOW + 5 * P);
    }
    clock.store(NOW + 6 * P, Ordering::SeqCst);
    let fresh = await_snapshot(&mut workers, NOW + 6 * P, |v| {
        v.sources[0].sessions.len() == 32
            && v.sources[0]
                .sessions
                .iter()
                .all(|s| s.state == SessionState::InputRequired)
    });
    let request = controller.tick(fresh, NOW + 6 * P).unwrap();
    controller.submitted(request.id, false, NOW + 6 * P);
    // At exactly 10s the publisher is fresh; one millisecond beyond is unavailable.
    clock.store(NOW + 15 * P, Ordering::SeqCst);
    let at_boundary = await_snapshot(&mut workers, NOW + 15 * P, |v| {
        v.sources[0].sessions.len() == 32
    });
    assert_eq!(
        at_boundary.sources[0].sessions[0].state,
        SessionState::InputRequired
    );
    let stale_at = NOW + 15 * P + 1;
    clock.store(stale_at, Ordering::SeqCst);
    let stale = await_snapshot(&mut workers, stale_at, |v| {
        v.sources[0].sessions.len() == 32
            && v.sources[0]
                .sessions
                .iter()
                .all(|s| s.state == SessionState::Unavailable && s.attention.is_none())
            && v.sources[0]
                .diagnostics
                .contains(&Diagnostic::StalePiStatus)
    });
    assert!(stale_at - (NOW + 5 * P) <= 10_000 + 2 * P);
    assert!(controller.tick(stale, stale_at).is_none());
    for i in 0..32 {
        pi_file(&configured[0], i, "idle", 3, stale_at + 1);
    }
    clock.store(stale_at + P, Ordering::SeqCst);
    let recovered = await_snapshot(&mut workers, stale_at + P, |v| {
        v.sources[0].sessions.len() == 32
            && v.sources[0]
                .sessions
                .iter()
                .all(|s| s.state == SessionState::Idle)
    });
    assert!(controller.tick(recovered, stale_at + P).is_none());
    workers.stop();
}
