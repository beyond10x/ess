---
format: aep.planning-md/3
id: story:a-row-set-refusal-is-witnessed-on-rows-whose-creator-needs-a-related-row-the-send-names
kind: story
status: active
title: A row-set refusal is witnessed on rows whose creator needs a related row the send names
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/a_row_set_branch_arranges_rows_past_the_creators_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T13:42:33Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-11-22T13:42:35Z", actor: "human:timo", revision: 8}
---
## Outcome

A row-set branch that needs selected rows present (`exists: true`, a `count` above zero) gets its
scenario when the command creating those rows refuses a missing related row
(`when_related: {via: input.x, exists: false}`) and the selector compares the same input
(`x == input.x`): the related row is created first, the rows are created naming it, and the send
names it. This holds whether the related row is the command's addressed subject, an unaddressed
row, or the row set's rows are created by the same command.

## Need

Two `ESS-SYNTH-001` refusals of a downstream specification on 0.97.3: a command that refuses to
move its subject while child rows of that subject are open, and a command that refuses a child row
whose sequence number is not above every earlier child of the same parent. Both children are
created by commands that refuse a missing parent. Reproduced on 0.97.3, paths relative to the
scratch directory:

- `models/g5h-guarded-creator-nosubject/`: `AddItem` refuses a missing `Batch` (`no-batch`) and
  creates `Item {batch_id: input.batch_id}`; `SealBatch` (no addressed subject, it creates a
  `Seal`) refuses `items-pending` on `Item where batch_id == input.batch_id`, `exists: true`.
  `SealBatch/outcome/sealed` is synthesized, the refusal branch is not:

  ```text
  refused: refusal[ESS-SYNTH-001]: outcome desk.room.SealBatch/items-pending has no scenario `desk.room.SealBatch/outcome/items-pending`
    no witness: `desk.room.SealBatch/items-pending: `batch_id` created by `desk.room.AddItem`, whose `no-batch` refuses a missing `desk.room.Batch` (`when_related: {via: input.batch_id, exists: false}`)` is `row set`, which has no arrangement of rows its selector selects: the command creating them refuses a missing related row through the input the selector compares, and no arrangement left a related row for the rows and the send to name
  5 scenario(s) (0 authored), 1 refusal(s), written to suite.json
  ```

  Without the creator's guard (`models/g5g-nosubject/`) the branch is synthesized: `3 scenario(s)
  (0 authored), 0 refusal(s)`.
- `models/g5-rowset-child/`: the same refusal for `AddItem/out-of-order` (row set over `Item`
  rows `AddItem` itself creates, `seq >= input.seq`) and for `SealBatch/items-pending` where
  `SealBatch` moves the `Batch` it addresses; the accepting `AddItem/added` then counts as reached
  by no input for three `ESS-SYNTH-004` refusals of `FinishItem`, while its own scenario is in the
  suite.
- `models/g5e-nostate/` and `models/g5d-unguarded-child/`: with the creator unguarded but the
  selector reading the addressed subject's identity, the refusal is `its own row set is Unknown`.

The 0.69.0 entry of `CHANGELOG.md` (beyond10x/ess#682) fixed the accepting branch of this shape
only; the help line of the refusal ("let a declared command create that related row in a state
the creator accepts") is already true of every model above.

## Fit review

Adopted as a defect, as an extension of the active
`story:synthesis-arranges-the-related-rows-a-guard-or-read-needs` part A, which names the
subject-addressed case (its example 21b) and diagnoses it as "that row must be the subject". The
models above show the narrower diagnosis is incomplete: the refusal branch fails with no addressed
subject at all (`g5h`), and with the rows created by the command under test (`g5`). Covers the two
downstream `ESS-SYNTH-001` refusals of this shape and the follow-on `ESS-SYNTH-004` refusals.
Considered and rejected: an authored-scenario-only answer (the help already promises synthesis).

## Design

- The row-set arrangement for a branch that needs rows present follows the #682 path the accepting
  branch takes: arrange the related row through its creating branch in a state every creator
  guard accepts, create each selected row naming it, and send the command naming it; where the
  command addresses that row as its subject, the subject is that row.
- Where the rows are created by the command under test, the arranging sends of that command are
  steps of the arrangement and are checked against the branch they take, as row-set arrangements
  already are.
- No new code, no format consequence (synthesis only; suites of models with these shapes gain
  scenarios).

## Acceptance

- `ess verify conform synthesize --path models/g5h-guarded-creator-nosubject --out suite.json`
  prints `0 refusal(s)` and holds `desk.room.SealBatch/outcome/items-pending`; the interpreted
  target passes it, and its arranged `Item` carries the `batch_id` the send names.
- `models/g5-rowset-child` prints `0 refusal(s)`: `AddItem/out-of-order`,
  `SealBatch/items-pending` and the three `FinishItem`/`Item` scenarios are in the suite.
- The acceptance rows of the active story's part A still hold.
- `synthesis_suite_bytes_table` keeps every pinned row; new rows pin g5 and g5h.

## Decisions

- D1: (a) fold this into the active story's part A as two further acceptance rows (recommended:
  same arrangement path, one implementor); (b) a separate story.

## A second consumer, 2026-11-28: "no row for this pair of keys yet"

A downstream meta-model specification (ess 0.99.9) reported that a guard "no row for this pair of
keys yet" (a uniqueness over two inputs, written as a row-set refusal) cost two scenarios of a
sibling command with `ESS-SYNTH-001`, and turned the guard into an `external:` cause. **Not
reproduced on 0.99.9** in the forms tried; its specification is private.

Neutral model, `.engineering/repro/a-row-set-refusal-is-witnessed-on-rows-whose-creator-needs-a-related-row-the-send-names/system.yaml`
(`ess/37`): `Member` rows reference a `Person` and a `Group`; `Join(person_id, group_id)` refuses
`unknown-person` and `unknown-group` (`when_related: {via, exists: false}`), then
`already-member` (`when_related: {entity: Member, where: {all: [person_id == input.person_id,
group_id == input.group_id]}, exists: true}`), then creates the `Member`; a sibling `Promote`
addresses a `Member` (`unknown_instance:`, `wrong_state:`, a move). `validate` says valid;
`synthesize` writes 10 scenarios and 0 refusals, and `ess verify conform run --target interpreted
--report-format 2` passes all 10, including `Promote/outcome/promoted`, whose row `Join` creates
under the guard.

What the neutral model lacks is the reporter's pair: a pair whose keys the creator copies or
derives (not sends), or a pair in which one key is a reference whose row only a provided entity
holds. The next reproduction should vary where the keys come from, not the guard. Until then the
story's own cause (the creator needs a related row the send names) is the only one known for
`ESS-SYNTH-001` beside a row set, and the model above passes it.
