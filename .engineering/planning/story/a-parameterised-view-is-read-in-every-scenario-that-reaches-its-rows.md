---
format: aep.planning-md/3
id: story:a-parameterised-view-is-read-in-every-scenario-that-reaches-its-rows
kind: story
status: draft
title: 'A parameterised view is read in every scenario that reaches its rows: ordering bounds, absent references and ranked pages of one referenced row'
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/identity.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/paging.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: inferred
  path: crates/verify/ess-conformance/tests/synthesis.rs
- confidence: inferred
  path: website/docs/reference/diagnostics.md
revision: 7
---
## Outcome

Synthesis binds every view parameter it can read off what the scenario knows, and arranges the
second row a ranked view needs under the same parameter value, so that three common list views no
longer skip their reads (`ESS-SYNTH-005`), drop their order (`ESS-SYNTH-014`) or leave an
invariant unread (`ESS-SYNTH-011`):

1. a cursor page, `seq > param.after` over an `Integer` (or `Decimal`) field;
2. a page of the rows of one referenced row, `parent_id == param.parent_id`, for a row whose
   `Optional` reference was sent absent;
3. the same page with `order_by:`, whose second row must reference the same row.

## Need

A downstream specification on 0.97.3 has 30 `ESS-SYNTH-005`, 13 `ESS-SYNTH-014` and 1
`ESS-SYNTH-011`. 27 of the 005 are `state == param.state` views, covered by the active
`story:state-param-binds-from-post-command-state`. The rest are the three shapes above.
Reproduced on 0.97.3, paths relative to the scratch directory:

- Shape 1, `models/g3-view-params/`: `JobsAfterSeq` (`seq > param.after`, `Integer`) is read in
  no scenario; 5 refusals of the form

  ```text
  refused: refusal[ESS-SYNTH-005]: outcome desk.room.QueueJob/queued has a scenario `desk.room.QueueJob/outcome/queued` that skips its read of `desk.room.JobsAfterSeq`
    `desk.room.JobsAfterSeq` filters on `seq > param.after`, which is undecided for an entity in `Queued`
      - `param.after` is bound by nothing a scenario knows
  ```

  The controls in the same model are read: `JobsAfterInstant` (`at > param.since`, `Timestamp`,
  20 reads, the `period_read` rule of `bound()`), `JobsAtSeq` (`seq == param.seq`, 10 reads) and
  `RunningJobs` (`state == Running`, 14 reads); `JobsInState` (`state == param.state`) has 0, the
  active story's shape. In the downstream specification the only view of an entity is such a
  cursor page, so its invariant is refused as `ESS-SYNTH-011` ("no view … holds an instance in
  `Recorded`"); on a private copy, dropping the cursor conjunct removed the 011.
- Shape 2, `models/g2-optional-parent/`: `FileCard/filed` sends the `Optional` `room_id` absent in
  one run; `CardsOfRoom` (`room_id == param.room_id`) is skipped with "`room_id` is bound by
  nothing a scenario knows" and "`param.room_id` is bound by nothing". The row holds no
  `room_id`, so `room_id == <any value>` is false and the row is decidedly outside every page.
- Shape 3, `models/g2r-required-parent/` (reference required): `CardsOfRoom` with
  `order_by: [filed_at desc]` gives

  ```text
  refused: refusal[ESS-SYNTH-014]: outcome desk.room.FileCard/filed has no scenario `desk.room.FileCard/outcome/filed`
    `desk.room.CardsOfRoom` declares an order and this scenario can put 1 row(s) in it: comparing two rows needs two `desk.room.Card`s, and no declared move reaches it from `Filed`
  ```

  for `FileCard/filed`, `FinishCard/finished` and the `finish` transition, although
  `FileCard` can file a second card into the same room. The same order over a `String` parameter
  (`title == param.title`, `models/g2t-text-param-order/`) gives no 014; without the parameter
  (`models/x-g2r-noparamorder/`) neither. Declaring the parent's relation as `owns` instead of
  `references` (`models/g2o-owns/`) changes nothing. The reason line names a move, which is not
  the cause, and the refusal says "has no scenario" for a scenario that stands (it is in the
  suite without its order assertion).

## Fit review

Adopted, as a gap of synthesis rather than a modelling choice: each view is the natural spelling
of a list endpoint (a cursor page, the rows of one parent, newest first), no other spelling keeps
its contract, and the scenario already knows the value each parameter needs. Covers 1 + 2 of the
downstream 005, its 13 014 and its 011; the 27 state-parameter 005 stay with the active story.
Shape 2 is the parameter form of the active
`story:synthesis-arranges-the-related-rows-a-guard-or-read-needs` part B (an absent `Optional`
compared with a literal); it is listed here because its binding lives in `bound()` beside shape 1.
Considered and rejected: one view per state or per cursor value (changes the adopter's API);
dropping `order_by:` (the help's other repair, which deletes a claim the adopter means).

## Design

- `bound()` in `synthesize.rs` gains an ordering rule beside `period_read`: a top-level conjunct
  ordering an `Integer` or `Decimal` field against a parameter binds the parameter just past the
  settled value on the admitting side (`seq > param.after` sends `seq - 1`), and a second read
  just on the other side expects the row absent, as the `Timestamp` rule does.
- A filter conjunct `f == param.p` over a field the arrangement left absent decides the row as
  not shown for any `p`; the read is made with `p` bound to a value of another row of the
  scenario where one exists, and expects the row absent.
- `arrange_beside` creates a ranked view's further rows with the subject's value in every field a
  bound parameter is compared with, including a reference to an arranged row (the row is reused,
  not created again).
- The 014 reason names the field that kept the further row out, and its headline says "has a
  scenario … without its order" rather than "has no scenario", since the scenario stands.
- No new code, no format consequence: synthesis only; suites of models with these views gain
  reads and order assertions, so their pinned suite bytes move and are re-pinned with this cause.

## Acceptance

- `models/g3-view-params`: no refusal names `JobsAfterSeq`; the suite reads it after each of the
  three commands, once admitting the row and once not (`JobsInState` stays with the active story).
- `models/g2-optional-parent`: no refusal names `CardsOfRoom` with "`room_id` is bound by
  nothing"; the absent-reference run reads `CardsOfRoom` and expects its row absent.
- `models/g2r-required-parent`: no `ESS-SYNTH-014`; the `FileCard/filed` scenario files a second
  card into the subject's room and asserts the order of the two.
- `models/g2t-text-param-order` keeps its suite bytes.
- `synthesis_suite_bytes_table` and `adversary_related_values_all_bytes` keep every row without
  these shapes; rows that move name this story beside the pin.

## Decisions

- D1: (a) one story for the three shapes (recommended: all live in `bound()` and
  `arrange_beside`); (b) one story per shape.
- D2 (shape 2): (a) here, and part B of the active story points to it (recommended); (b) moved
  into part B.
- D3 (shape 1 on `Decimal`): (a) included, the midpoint rule already exists for guards
  (recommended); (b) `Integer` only.
