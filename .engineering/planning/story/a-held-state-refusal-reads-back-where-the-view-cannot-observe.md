---
format: aep.planning-md/3
id: story:a-held-state-refusal-reads-back-where-the-view-cannot-observe
kind: story
status: draft
title: A held-state refusal reads the record back where the view cannot observe it
tags:
- defect
- ess-d368-close
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
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
- Models whose identity view projects `state` (the `state-view` form) and every committed suite under
  `suites/` keep their bytes; any test-fixture suite that moves is re-pinned and named in the change.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/existence.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/tests/a_held_state_refusal_reads_back_where_the_view_cannot_observe.rs
- inferred: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- inferred: docs/design/outcome-shapes.md

confidence: high
