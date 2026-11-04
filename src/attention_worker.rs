//! Fixed source ownership and bounded mailboxes. No filesystem work or joins on UI.

use std::{
    fmt, io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use crate::{
    attention::{
        AttentionSnapshot, Capabilities, Diagnostic, STALE_AFTER_MS, SessionState,
        SourceMembership, SourceSnapshot,
    },
    config::{ConfigError, Provider, Source, validate_sources},
    monitor::AttentionMonitor,
};

/// Injectable filesystem boundary; production uses the existing aggregate parser.
pub trait SourceReader: Send {
    fn read(&mut self, now_ms: u64) -> SourceSnapshot;
}

impl<F: FnMut(u64) -> SourceSnapshot + Send> SourceReader for F {
    fn read(&mut self, now_ms: u64) -> SourceSnapshot {
        self(now_ms)
    }
}

#[derive(Debug)]
pub enum WorkerError {
    Configuration(ConfigError),
    Spawn(io::Error),
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Configuration(_) => "invalid attention source configuration",
            Self::Spawn(_) => "could not start attention source worker",
        })
    }
}

impl std::error::Error for WorkerError {}

#[derive(Clone, Copy)]
struct Poll {
    generation: u64,
    now_ms: u64,
}

struct Publication {
    poll: Poll,
    snapshot: SourceSnapshot,
}

struct Ownership {
    stopped: AtomicBool,
    latest: Mutex<Option<Publication>>,
    // The running thread also owns this record. Dropping the UI owner while a
    // read is blocked does not discard its handle or create a replacement.
    handle: Mutex<Option<JoinHandle<()>>>,
}

struct Worker {
    source: Source,
    commands: SyncSender<Poll>,
    ownership: Arc<Ownership>,
    generation: u64,
    observed_at: u64,
    snapshot: SourceSnapshot,
}

/// Sources are immutable until restart: one owned thread and two single slots each.
pub struct AttentionWorkers {
    workers: Vec<Worker>,
    generation: u64,
    stopped: bool,
}

impl AttentionWorkers {
    pub fn new(sources: Vec<Source>, now_ms: u64) -> Result<Self, WorkerError> {
        Self::with_clock(sources, now_ms, || {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("system clock before Unix epoch")
                .as_millis() as u64
        })
    }

    /// The clock is sampled by the parser after I/O; poll timestamps only bound
    /// result age. Synthetic callers must inject their own observation clock.
    pub fn with_clock(
        sources: Vec<Source>,
        now_ms: u64,
        clock: impl Fn() -> u64 + Send + Sync + 'static,
    ) -> Result<Self, WorkerError> {
        let clock = Arc::new(clock);
        Self::with_readers(sources, now_ms, |source| {
            let mut monitor =
                AttentionMonitor::new(vec![source.clone()]).expect("validated source");
            let clock = clock.clone();
            Box::new(move |_| monitor.poll_with_clock(&*clock).sources.remove(0))
        })
    }

    pub fn with_readers(
        sources: Vec<Source>,
        now_ms: u64,
        mut reader: impl FnMut(&Source) -> Box<dyn SourceReader>,
    ) -> Result<Self, WorkerError> {
        validate_sources(&sources).map_err(WorkerError::Configuration)?;
        let mut result = Self {
            workers: Vec::new(),
            generation: 0,
            stopped: false,
        };
        for source in sources {
            let mut reader = reader(&source);
            let ownership = Arc::new(Ownership {
                stopped: AtomicBool::new(false),
                latest: Mutex::new(None),
                handle: Mutex::new(None),
            });
            let owned = ownership.clone();
            let (commands, receive) = mpsc::sync_channel::<Poll>(1);
            let handle = thread::Builder::new()
                .name(format!("attention-{}", source.id))
                .spawn(move || {
                    while !owned.stopped.load(Ordering::Acquire) {
                        match receive.recv_timeout(Duration::from_millis(100)) {
                            Ok(poll) => {
                                if owned.stopped.load(Ordering::Acquire) {
                                    break;
                                }
                                let snapshot = reader.read(poll.now_ms);
                                let mut latest = owned.latest.lock().expect("worker mailbox");
                                if !owned.stopped.load(Ordering::Acquire) {
                                    *latest = Some(Publication { poll, snapshot });
                                }
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => {}
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        }
                    }
                })
                .map_err(WorkerError::Spawn)?;
            *ownership.handle.lock().expect("new worker handle") = Some(handle);
            result.workers.push(Worker {
                snapshot: empty_snapshot(&source),
                source,
                commands,
                ownership,
                generation: 0,
                observed_at: now_ms,
            });
        }
        Ok(result)
    }

    /// Never waits for a reader. A full command slot coalesces repeated polls.
    pub fn request_poll(&mut self, now_ms: u64) {
        if self.stopped {
            return;
        }
        self.generation += 1;
        for worker in &mut self.workers {
            match worker.commands.try_send(Poll {
                generation: self.generation,
                now_ms,
            }) {
                Ok(()) | Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Disconnected(_)) => {
                    worker.snapshot = unavailable_snapshot(&worker.source);
                }
            }
        }
    }

    /// Injected wall clock matches the publisher's timestamps. Old reads cannot
    /// refresh liveness merely by completing late.
    pub fn drain_at(&mut self, now_ms: u64) -> AttentionSnapshot {
        if self.stopped {
            return AttentionSnapshot::default();
        }
        for worker in &mut self.workers {
            if let Ok(mut latest) = worker.ownership.latest.try_lock() {
                if let Some(publication) = latest.take() {
                    if publication.poll.generation > worker.generation
                        && publication.poll.now_ms <= now_ms
                        && now_ms - publication.poll.now_ms <= STALE_AFTER_MS
                    {
                        worker.generation = publication.poll.generation;
                        worker.observed_at = publication.poll.now_ms;
                        worker.snapshot = publication.snapshot;
                    }
                }
            }
            let finished = worker
                .ownership
                .handle
                .try_lock()
                .map(|handle| handle.as_ref().is_some_and(JoinHandle::is_finished))
                .unwrap_or(false);
            if finished || now_ms.saturating_sub(worker.observed_at) > STALE_AFTER_MS {
                worker.snapshot = unavailable_snapshot(&worker.source);
            }
            let mut expired = 0;
            for session in &mut worker.snapshot.sessions {
                if session.state != SessionState::Unavailable
                    && session
                        .publisher_expires_at_ms
                        .is_some_and(|deadline| now_ms > deadline)
                {
                    session.state = SessionState::Unavailable;
                    session.attention = None;
                    expired += 1;
                }
            }
            if expired > 0 {
                worker.snapshot.unavailable_pi_statuses += expired;
                worker.snapshot.diagnose(Diagnostic::StalePiStatus);
            }
        }
        AttentionSnapshot {
            sources: self.workers.iter().map(|w| w.snapshot.clone()).collect(),
        }
    }

    /// Does not join, even on Drop. Process termination cancels stuck OS reads.
    pub fn stop(&mut self) {
        self.stopped = true;
        for worker in &self.workers {
            worker.ownership.stopped.store(true, Ordering::Release);
        }
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for AttentionWorkers {
    fn drop(&mut self) {
        self.stop();
    }
}

fn empty_snapshot(source: &Source) -> SourceSnapshot {
    SourceSnapshot {
        source: source.clone(),
        capabilities: Capabilities {
            input_detection: source.kind == Provider::Pi,
            session_errors: true,
        },
        sessions: Vec::new(),
        unavailable_pi_statuses: 0,
        diagnostics: Vec::new(),
        membership: SourceMembership::default(),
    }
}

fn unavailable_snapshot(source: &Source) -> SourceSnapshot {
    let mut snapshot = empty_snapshot(source);
    snapshot.diagnostics.push(Diagnostic::SourceUnavailable);
    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MonitorConfig;
    use std::time::Instant;

    fn source() -> Source {
        MonitorConfig::parse(
            r#"{"sources":[{"id":"test","label":"Synthetic","kind":"pi","path":"/synthetic"}]}"#,
        )
        .unwrap()
        .resolved_sources()
        .unwrap()
        .remove(0)
    }

    fn publish(workers: &AttentionWorkers, generation: u64, now_ms: u64) {
        let worker = &workers.workers[0];
        *worker.ownership.latest.lock().unwrap() = Some(Publication {
            poll: Poll { generation, now_ms },
            snapshot: empty_snapshot(&worker.source),
        });
    }

    // Proves a regressed generation, expired read or future clock cannot resurrect
    // removed sessions or mask unavailable-source diagnostics; newer work recovers.
    #[test]
    fn rejects_stale_generations_and_expired_results_with_exact_staleness_boundary() {
        let mut workers = AttentionWorkers::with_readers(vec![source()], 20_000, |source| {
            let source = source.clone();
            Box::new(move |_| empty_snapshot(&source))
        })
        .unwrap();
        publish(&workers, 2, 20_000);
        assert!(workers.drain_at(20_000).sources[0].diagnostics.is_empty());
        assert!(workers.drain_at(30_000).sources[0].diagnostics.is_empty());
        assert_eq!(
            workers.drain_at(30_001).sources[0].diagnostics,
            vec![Diagnostic::SourceUnavailable]
        );
        for (generation, time) in [(1, 30_001), (2, 30_001), (3, 20_000), (4, 40_000)] {
            publish(&workers, generation, time);
            assert_eq!(
                workers.drain_at(30_001).sources[0].diagnostics,
                vec![Diagnostic::SourceUnavailable]
            );
        }
        publish(&workers, 5, 30_001);
        assert!(workers.drain_at(30_001).sources[0].diagnostics.is_empty());
    }

    // Proves both mailbox bounds and handle retention even after the UI owner
    // drops during uninterruptible I/O; no late result is published after stop.
    #[test]
    fn blocked_drop_keeps_ownership_until_exit_with_one_queued_poll() {
        let (entered, entered_rx) = mpsc::sync_channel(0);
        let (release, permit) = mpsc::sync_channel(0);
        let mut permit = Some(permit);
        let mut workers = AttentionWorkers::with_readers(vec![source()], 20_000, |source| {
            let source = source.clone();
            let permit = permit.take().unwrap();
            let entered = entered.clone();
            Box::new(move |_| {
                entered.send(()).unwrap();
                permit.recv().unwrap();
                empty_snapshot(&source)
            })
        })
        .unwrap();
        drop(entered);
        let ownership = Arc::downgrade(&workers.workers[0].ownership);
        workers.request_poll(20_000);
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        workers.request_poll(21_000);
        assert!(matches!(
            workers.workers[0].commands.try_send(Poll {
                generation: 3,
                now_ms: 22_000
            }),
            Err(TrySendError::Full(_))
        ));
        let start = Instant::now();
        drop(workers);
        assert!(start.elapsed() < Duration::from_millis(250));
        {
            let owned = ownership
                .upgrade()
                .expect("blocked thread still owns handle");
            assert!(!owned.handle.lock().unwrap().as_ref().unwrap().is_finished());
            assert!(owned.latest.lock().unwrap().is_none());
            assert!(owned.stopped.load(Ordering::Acquire));
        }
        release.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while let Some(owned) = ownership.upgrade() {
            assert!(owned.latest.lock().unwrap().is_none());
            assert!(
                Instant::now() < deadline,
                "worker did not exit after read released"
            );
            thread::yield_now();
        }
        assert!(matches!(
            entered_rx.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
    }

    // Proves UI drain never waits on the publication lock and queued generations
    // collapse to the single newest completed snapshot when UI does not drain.
    #[test]
    fn latest_result_overwrites_and_contended_ui_drain_does_not_wait() {
        let mut workers = AttentionWorkers::with_readers(vec![source()], 20_000, |source| {
            let source = source.clone();
            Box::new(move |_| empty_snapshot(&source))
        })
        .unwrap();
        let owned = workers.workers[0].ownership.clone();
        let deadline = Instant::now() + Duration::from_secs(1);
        for generation in 1..=100 {
            workers.request_poll(20_000);
            while owned
                .latest
                .lock()
                .unwrap()
                .as_ref()
                .is_none_or(|p| p.poll.generation != generation)
            {
                assert!(
                    Instant::now() < deadline,
                    "latest publication did not advance"
                );
                thread::yield_now();
            }
        }
        let guard = owned.latest.lock().unwrap();
        let start = Instant::now();
        workers.drain_at(20_000);
        assert!(start.elapsed() < Duration::from_millis(250));
        drop(guard);
        workers.drain_at(20_000);
        assert_eq!(workers.workers[0].generation, 100);
        assert!(owned.latest.lock().unwrap().is_none());
        assert_eq!(workers.workers[0].snapshot.sessions.len(), 0);
    }
}
