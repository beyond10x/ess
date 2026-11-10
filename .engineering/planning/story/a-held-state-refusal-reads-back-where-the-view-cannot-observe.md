---
format: aep.planning-md/3
id: story:a-held-state-refusal-reads-back-where-the-view-cannot-observe
kind: story
status: active
title: A held-state refusal reads the record back where the view cannot observe it
tags:
- defect
- ess-d368-close
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/a_held_state_refusal_reads_back_where_the_view_cannot_observe.rs
- confidence: inferred
  path: docs/design/outcome-shapes.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:34:19Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:34:21Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
---
## Finding

Wave D368 U2's adversary pass (`review-result:adversary-d368-u2-pass-1`) found this, outside that
story's Acceptance and already in the code before the wave. Consider the held-state half of an
input refusal beside held-state branches on a model that has an identity view, where
`observe_selection_subject` still fails for the held state. That happens when the view projects the
identity but not `state`, or is filtered, paged or not `expect`-style. In that case neither the
unchanged-row claim nor the no-view probe runs
(`crates/verify/ess-conformance/src/synthesize/existence.rs:1347-1368`): the probe is taken only
where `identity_views(ir, subject)` is empty (`existence.rs:1527`). That state's send then requires
only the error and no event, so a target that moves the record on refusal and publishes nothing
passes.

## Problem

Reproduced on 0.92.0 with the `TICKETS` model of
`crates/verify/ess-conformance/tests/a_no_view_held_state_refusal_probes_the_record.rs`
(`Advance` with `blank-note: when: note == ""` beside `held`/`closed`/`already-closed` held-state
branches), in three forms:

- no views: each held-state send of `Advance/outcome/blank-note` is followed by a probe
  (`execute_command Advance`, `expect_outcome`);
- plus a view `{ticket_id, state}`: each is followed by `query_view` and
  `expect_complete_subject_unchanged`;
- plus a view projecting only the identity:

  ```yaml
  views:
    - name: demo.ticket.Tickets
      source: demo.ticket.Ticket
      consistency: read_your_writes
      fields:
        - {name: ticket_id, type: String}
  ```

  each held-state send ends with `expect_error` and `expect_no_event` only; nothing reads the record
  back.

## Acceptance

- `crates/verify/ess-conformance/tests/a_held_state_refusal_reads_back_where_the_view_cannot_observe.rs`:
  with the identity-only view above, every held-state send of `Advance/outcome/blank-note` is
  followed by a read-back (the view, where it can observe the record, or the `probe` through a
  branch that runs from that state). The correct interpreted target passes. A target that moves the
  record to another state on the refused send and publishes nothing fails. Red on 0.92.0.
- Where no read-back has a valid send, the scenario is withdrawn and refused as `ESS-SYNTH-001`
  naming the branches tried, as the no-view case is.
- In the same test, the `state-view` form (view `{ticket_id, state}`) synthesizes a suite whose
  SHA-256 equals the digest 0.92.0 gives it, pinned in the test.
- Every committed suite under `suites/generated/` keeps its bytes:
  `cargo test -p ess-conformance --locked --test adversary_map_witness_pass1 adv196_every_example_still_synthesizes_to_its_committed_suite`
  passes with no file under `suites/generated/` changed.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` and
  `--test synthesis_suite_bytes_table_cli_read` pass. Each row of
  `synthesis-suite-bytes-table.tsv` that moves is a fixture whose suite, synthesized before and after
  the change and compared, differs only by read-back steps added after held-state sends of an input
  refusal; no other row moves.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/existence.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/tests/a_held_state_refusal_reads_back_where_the_view_cannot_observe.rs
- inferred: docs/design/outcome-shapes.md

confidence: high

Shared surfaces moved: re-pins the suite-bytes rows of fixtures that gain a held-state read-back,
and proposes its `CHANGELOG.md` sentence.
