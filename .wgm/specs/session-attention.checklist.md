# Session attention spec quality

- [x] Job and actual operator are explicit. [Spec §JTBD]
- [x] Native Windows and local WSL are the actual target. [Spec §Confirmed decisions]
- [x] Privacy remains content-free across publisher, parser, output, and alerts. [Spec §Architecture and boundaries]
- [x] Pi extension installation has user approval. [Spec §Confirmed decisions]
- [x] Successful completion, abort, and recoverable tool errors stay quiet. [Spec §Confirmed decisions]
- [x] Passive Copilot input detection is explicitly excluded with user approval. [Spec §Confirmed decisions]
- [x] Session identities and attention episodes are distinct concepts. [Spec §Architecture and boundaries]
- [x] Poll, heartbeat, stale, input size, source, and session limits are explicit. [Spec §Architecture and boundaries]
- [x] WSL stalling, concurrency, notification suppression, cleanup, and retry are covered. [Spec §Architecture and boundaries]
- [x] Every requirement maps to exact backpressure and an evidence origin. [Spec §Requirements delta]
- [x] Legacy behavior remains protected and no existing tests are waived. [Spec §User-visible success criteria]
- [x] Smallest end-to-end demo and tiered holdout locations are specified. [Spec §Magic moment, §Holdout scenarios]
- [x] Real Windows field evidence is separately owed, not implied by cross-build. [Spec §Ruggedness]
- [x] All tasks have exact files/areas, validation, acceptance, and status. [Plan §Tasks]

14/14 requirements-writing checks; not an implementation or field-validation verdict.
