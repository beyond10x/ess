---
format: aep.planning-md/1
id: review-result:subject-state-implementation-adversary-1
kind: review-result
status: active
title: Subject-state implementation adversary, round 1
relations:
- reviews: story:subject-state-outcome-guards
revision: 1
---
unit: subject-state-outcome-guards; integration working tree based on ff5ec1ac895ea88b0b452d0213c06f02382270af
verdict: CONFIRMED
cases: executed 7→9 in the affected domain selection, red 2
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: assigned evidence directory and existing coordinator build cache; private inventory retained separately
needs-coordinator: record this finding before correcting subject authority selection
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 c077222597c5be6b397fdff34d4c188b33a99922b010bcf8cbb5e158830bc64a, retained as local-evidence:runtime-gaps/publication-replay/snapshots/c077222597c5be6b397fdff34d4c188b33a99922b010bcf8cbb5e158830bc64a.md. Source creation recorded at 2026-09-11T03:45:20Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 1b9805bbc03ff4b6dcf85ce6857903f5b29336ca55fecd4196bc34be2c656435, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/1b9805bbc03ff4b6dcf85ce6857903f5b29336ca55fecd4196bc34be2c656435-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

1. Test-only review delta (`git diff --no-index --stat /dev/null crates/specify/ess-domain/tests/subject_state_adversary.rs`):

```text
 .../ess-domain/tests/subject_state_adversary.rs    | 37 ++++++++++++++++++++++
 1 file changed, 37 insertions(+)
```

The integration checkout was handed over with coordinator-owned dirty source and planning files; ordinary `git diff --stat` omits the new untracked test and also contains that existing implementation. This reviewer wrote only the new test file above in the checkout. The retained full status is an observation of the candidate, not a claim that those other changes are reviewer edits. No source edits, Git writes, AEP writes or cleanup were performed.

2. New cases, written before execution:

- `crates/specify/ess-domain/tests/subject_state_adversary.rs:25`: `external_subject_cannot_hide_uncovered_held_state` first assembles a complete two-entity specification, then proves the incomplete ordinary-order declaration is refused, and finally requires the reordered declaration to remain refused. The final assertion is red at line 30: external-first ordering admits uncovered `Call.Bridged`.
- `crates/specify/ess-domain/tests/subject_state_adversary.rs:34`: `external_subject_cannot_reject_valid_held_state` first assembles that same complete specification and then moves the external branch first. The final assertion is red at line 36: the declared `Call.Bridged` state is wrongly checked against `Observer`.

The first attempt at the first case had an invalid control: a lifecycle transition had no driving command. Its `missing_causation` failure is retained in `uncovered-red.log` and is **not** evidence of this finding. The test fixture was corrected by adding a legitimate Bridge command and wrong-state refusal. The complete specification then assembled successfully before either intended assertion ran. The two corrected isolated runs each exited 101, selected exactly one test, and failed at their final assertions. No implementation changed between them.

Corrected first isolated run, `cargo test --offline --locked -p ess-domain --test subject_state_adversary external_subject_cannot_hide_uncovered_held_state -- --exact`:

```text
running 1 test
test external_subject_cannot_hide_uncovered_held_state ... FAILED

thread 'external_subject_cannot_hide_uncovered_held_state' (3190764) panicked at crates/specify/ess-domain/tests/subject_state_adversary.rs:30:5:
external-first ordering admitted an uncovered Call.Bridged state

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

Second isolated run, replacing the test filter with `external_subject_cannot_reject_valid_held_state`:

```text
running 1 test
test external_subject_cannot_reject_valid_held_state ... FAILED

thread 'external_subject_cannot_reject_valid_held_state' (3193560) panicked at crates/specify/ess-domain/tests/subject_state_adversary.rs:36:34:
external outcome ordering cannot change declared Call states: "[unknown_state] command.calls.core.Enrich.outcomes.bridged.when_subject_state: `Bridged` is not a declared state of `calls.core.Observer`"

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

These excerpts preserve assertion output exactly. Complete output, including compiler lines and failure summaries, is retained with path-only redaction in the corresponding `*-public.log`; raw originals and hashes are retained privately.

3. Subsequent affected selection:

`cargo test --offline --locked -p ess-domain --test subject_state --test subject_state_adversary`, exit 101. Complete output: `local-evidence/subject-state/adversary-1/affected-suite-public.log`.

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The before count of 7 comes from the implementor's reported domain selection, not a pre-test baseline run by this reviewer. This one affected selection followed the two new isolated cases. The coordinator's instruction to stop without another affected-suite run arrived after this command had already completed; no further execution occurred. No full gate or conformance runtime rerun was performed.

4. Finding:

| Location | Verdict | Origin | What was measured | What reaches it |
| --- | --- | --- | --- | --- |
| `crates/specify/ess-domain/src/command/subject_state.rs:74` | CONFIRMED | introduced | Selecting the first outcome subject includes External branches and can both admit an uncovered held state and reject a valid declared state of the actual input-selected entity. Both isolated assertions exit 101 after a valid full-specification control succeeds. | Public `RawSpecFile::parse` followed by `Specification::assemble` on an ess/3 command with a separately observable external outcome updating another declared entity; ordinary and external branches are legal, and only their source order changes. |

`validate_shape` explicitly excludes External at lines 25–26 from shared input-selected subject authority; `validate` then takes the first subject without that exclusion at line 74. It passes the wrong entity into both state membership and finite partition validation. Select authority from the same input-selected branch set already validated by `validate_shape` (or directly from a SubjectState outcome), and preserve both new regression assertions. Do not broaden the rule to forbid otherwise legal external effects.

The source module is new in this working-tree implementation (`git show HEAD:.../subject_state.rs` confirms absence at the base); this is an introduced defect, not an inferred pre-existing one. Candidate module SHA256 before correction: `0747f874e4ce1f6a1d74799aaa3c8c563685c0e27ad8413ee3f6118a21e5a872`. Synthesis source SHA256: `f1f5d24b1b8f9de7be337193cc37d1af9b3e886865deebdb75c8cb59a4db1a14`. New test SHA256: `685d948e84b2ce7c7a975a79fbf595f360083aa9d66f809745e9f24735a2b99a`.

5. Bounded source review also traced the declared state/input conjunction, finite joint bound, IR retention, explicit source admission, semantic guard-diff text, state-establishing command route, actual identity/state view observations before and after the command, and existing Rust/Go guard-ignoring mutations. No additional confirmed finding arose from that pass. These source observations do not claim exhaustive runtime correctness or repeat the implementor's runtime validation.

6. Public-path redaction is intentional under the user's publication constraint. `local-evidence/subject-state/adversary-1` aliases the assigned scratch directory; `worktree-state/integration` aliases the managed integration tree; `build-state/integration` aliases its already existing disk-backed compiler cache. Full local paths, raw log hashes and reviewer-written inventory are retained in `inventory-private.md`. Original log bytes are unchanged. Shared sccache may also have received compiler artifacts; its contents are not exclusively owned by this reviewer. The own review lease is released at handback; the coordinator owns all later tree/cache cleanup.

```findings
- file: crates/specify/ess-domain/src/command/subject_state.rs
  line: 74
  category: property
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Selecting the first outcome subject includes External branches and can both admit an uncovered held state and reject a valid declared state of the actual input-selected entity.
```