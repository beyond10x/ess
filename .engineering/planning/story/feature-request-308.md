---
format: aep.planning-md/3
id: story:feature-request-308
kind: story
status: implemented
title: 'A constrained newtype identity refuses replay scenarios: complete subject requires a finite exact typed observer'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#308
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/replay.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/subject.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/retained_replay.rs
- confidence: cited
  path: docs/design/retained-command-results.md
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T10:32:09Z", actor: "human:timo", revision: 9, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T10:32:09Z", actor: "human:timo", revision: 10, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "active", to: "implemented", at: "2026-10-02T13:29:03Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

Resolve beyond10x/ess#308: A constrained newtype identity refuses replay scenarios: complete subject requires a finite exact typed observer.

## Origin

beyond10x/ess#308, found during wave w2 (#287 / #272).

## Fit review

1. Need: compare complete actual subject snapshots when the identity is a constrained primitive newtype. Issue #308 reports refusal of otherwise valid replay scenarios; no new authored syntax is proposed.
2. Class: capability gap, correcting the earlier one-line classification. docs/design/retained-command-results.md:221-222 explicitly excludes invariant and reading observers recursively; replay.rs:177 enforces that policy. This is not merely an accidental implementation refusal.
3. Existing expression: unconstrained wrappers work; weakening the declared identity or reverting to partial snapshots loses the consumer's guarantee. SubjectShape::of (subject.rs:40-53) delegates to the restrictive shared declaration extractor.
4. Fit: complete-subject preservation proves exact represented-value equality, separately from invariant satisfaction. Preserve nominal wrapper representation and structural row admission (subject.rs:89-107). Give subject observation its own explicit declaration profile, retaining unsupported reading/recursive/numeric/resource boundaries. Retained-result responses keep their existing stricter profile; do not globally remove replay::declarations_for's constraint refusal.
5. Second adopter: preserved deployment configuration keyed by a constrained deployment-name type. Existing SubjectShape identity selection and comparison have the same need.
6. Cost: existing Declaration::Newtype { of } represents the structural comparison without inventing an invariant evaluator. Generated descriptors must make no claim to validate erased invariants. No serialized field is planned; verify old-reader admission and show that existing descriptor meaning remains structural equality before concluding no format bump. Update the binding design's explicit limitation. A change to descriptor meaning instead requires format review.
7. Alternatives: keep the named refusal, leaving the need unmet; remove all shared constraints globally, silently broadening retained responses; selected bounded subject-only structural observation with explicit limits and tests for the unchanged sibling contract.

## Decisions

Accept, redesigned: distinguish structural complete-subject comparison from invariant evaluation, and restrict the new admission to the subject profile. Preserve exact identity and field comparison, malformed-value refusals, and retained-response refusal behavior. The previous 2026-10-01 one-line acceptance lacked this distinction; the source-backed review supersedes that rationale without claiming implementation. No user decision is missing for this routine design choice within the authorized backlog work.

## Acceptance

- issue_308_constrained_identity_has_an_exact_subject_snapshot: valid constrained String identity produces a real replay/preservation scenario with complete subject observations, not a downgrade to legacy steps.
- issue_308_constrained_identity_snapshot_detects_changed_row: honest target passes and a target mutating a preserved field fails exact comparison.
- issue_308_nested_newtype_representation_is_preserved: required/Optional nested wrappers retain wire representation and type admission.
- Decimal/Binary64, reading semantics, recursion/resource bounds, missing fields and malformed values preserve their existing refusals. Separate retained-response test proves that profile was not broadened.
- Old-reader/descriptor test and binding design establish the exact format consequence; do not assert invariant checking from structural admission.

unit: story:feature-request-308 — constrained newtype complete-subject observation
verdict: green (focused; independent review and final grouped package checks pending)
cases: executed 46→46, red 3→0; baseline 43 passed / 3 failed, treatment 46 passed / 0 failed
origin: n/a
wrote-outside-worktree: <task-owned temporary directory outside Git> (authorized test temporary directory)
needs-coordinator: independent review and deferred full package verification

The unchanged execution count is deliberate: baseline and treatment use identical final test bytes, including five added cases. Three expose the source-admission gap and turn red→green; the existing-descriptor reader and strict retained-response/reading controls already pass on the base. No original test was removed, weakened, filtered or newly ignored.

Base: 63e64f6cd38990f7bdeabf58ce4e24dc65abd359. Frozen diff: 308-frozen.patch. Exact four-file hashes: 308-source-sha256.txt. No staging, commit, push, AEP mutation or remote write.

1. Unit and acceptance

A constrained String newtype, nested through a second nominal wrapper, now produces the actual retained-replay scenario with SnapshotCompleteSubject and ExpectCompleteSubjectUnchanged. It does not fall back to the legacy snapshot pair. The honest Rust target performs the original invocation and real retry with two actual view queries; a target that changes the preserved stamp fails. Missing or malformed values in either original or post-command rows fail, with malformed original observations stopping before retry.

The production change is an explicit DeclarationProfile in replay.rs:163. Retained-result response extraction selects RetainedResult at :117 and keeps the existing invariant/reading refusal. SubjectShape::of selects CompleteSubject at subject.rs:52. Only that profile admits constrained Newtype bodies (:186); reading attachments and other constrained bodies still refuse. Both profiles retain the existing ExactShape finite-type, numeric, recursion, depth, declaration, field and byte limits. No invariant evaluator or new persisted authority is introduced.

Tests at retained_replay.rs:1968, :1998, :2049, :2080 and :2125 cover:
- Required complete observation pair, actual replay presence, no legacy downgrade, and suite/12 retained.
- Honest replay versus changed-row mutant, missing original value, invalid original stamp, missing post-command state and invalid post-command state, with real callback counts.
- Nested required String wrappers and Optional nested wrappers as scalar wire values; absent, present and null Optional values remain admitted, object-wrapped/Boolean malformed representations and missing required fields fail.
- Existing descriptor JSON accepted by the base reader. The descriptor deliberately carries no invariant; a structurally valid string outside the source invariant remains structurally admissible. Decimal/Binary64 through the wrapper, recursion, field/depth limits and oversized complete rows still fail.
- A direct replay::Observation::of call on a constrained response must return the invariant/reading refusal; this specifically catches accidentally broadening the shared retained-result profile. A source reading attachment still produces a named refusal and no replay scenario.

Format consequence: the newly synthesized constrained-source shape equals the existing structural descriptor in DESCRIPTOR, which the exact base implementation already admits. JSON suite round-trip still admits suite/12. No serialized field, step, variant or descriptor semantics changed. The binding design now explicitly distinguishes structural exact comparison from invariant satisfaction and preserves the retained-response limitation. Other constrained declarations remain refused.

2. git diff --stat

 crates/verify/ess-conformance/src/replay.rs        |  15 +-
 crates/verify/ess-conformance/src/subject.rs       |  12 +-
 .../ess-conformance/tests/retained_replay.rs       | 292 +++++++++++++++++++++
 docs/design/retained-command-results.md            |  14 +-
 4 files changed, 326 insertions(+), 7 deletions(-)

3. Red evidence

Environment for cargo commands:
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=2 TMPDIR=<task-owned temporary directory outside Git>

Initial test-first command, before production edits:
cargo test -p ess-conformance --locked --test retained_replay issue_308

308-red.log:
path: "retained.core.Records: replay response invariant/reading observer is unsupported"
type_ref: "complete subject observation"
reason: "complete subject requires a finite exact typed observer"
test result: FAILED. 2 passed; 3 failed; 0 ignored; 0 measured; 31 filtered out; finished in 0.06s

The whole retained_replay binary was then run before production changes (308-baseline.log; actual cargo exit 101):
test result: FAILED. 33 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.40s

After final formatting, replay.rs and subject.rs were temporarily restored from exact committed HEAD and the five-binary command below was run against the final tests. 308-baseline-final.log retains the same three failures. Both production files were immediately restored from target-local frozen copies before the treatment run. No other source file was temporarily changed.

4. Final focused verification

Identical baseline and treatment command:
cargo test -p ess-conformance --locked --no-fail-fast --test retained_replay --test response_admission --test response_payload --test response_union --test adversary_response_union

Runner summaries in 308-baseline-final.log → 308-focused.log:
- adversary_response_union: executed 1→1; 1 passed / 0 failed on both.
- response_admission: executed 1→1; 1 passed / 0 failed on both.
- response_payload: executed 7→7; 7 passed / 0 failed on both.
- response_union: executed 1→1; 1 passed / 0 failed on both.
- retained_replay: executed 36→36; 33 passed / 3 failed → 36 passed / 0 failed.
- Combined baseline exit 101; treatment exit 0, 46 passed / 0 failed / 0 ignored.

The existing retained_replay binary includes the generated Go retained-result and complete-row runtime checks. These are results produced by the implementor's focused command, not a claim of independent verification or whole-package coverage.

cargo clippy -p ess-conformance --all-targets --locked -- -D warnings
308-clippy.log, exit 0:
    Checking ess-conformance v0.51.0
    Finished `dev` profile [unoptimized] target(s) in 16.72s

task fmt-check
308-fmt.log, exit 0.

git diff --check
Exit 0, no output.

5. Deliberate bounds

No new invariant satisfaction guarantee: subject observation checks represented-value types and exact equality only. No broad admission of constrained structs or reading types. No retained-response capability expansion. No primitive float comparison change, general schema engine, new wire fields, suite format bump or runtime protocol change. Existing readers are checked against the exact base implementation, not claimed to have been executed at every historical release.

Full affected-package and grouped source validation remain deferred by the coordinator until #308/#298 batch content is final. No monolithic package command or site build was run by this worker. The changed design file is repository engineering documentation; final delivery checks remain with the coordinator.

6. Outside writes and handoff

Authorized test temporary directory: <task-owned temporary directory outside Git>. Source, design, logs, reports, frozen copies and patches remain within the managed worktree. Initial free disk 44 GiB; final 40 GiB; the 8 GiB floor was maintained. Target-local cache and no CARGO_TARGET_DIR.

Own lease scope-boolean-308 is released at handoff. Coordinator owns independent review, full group verification, commit and cleanup.
