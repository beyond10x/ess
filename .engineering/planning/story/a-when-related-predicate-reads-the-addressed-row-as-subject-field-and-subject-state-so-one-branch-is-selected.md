---
format: aep.planning-md/3
id: story:a-when-related-predicate-reads-the-addressed-row-as-subject-field-and-subject-state-so-one-branch-is-selected
kind: story
status: draft
title: A when_related predicate reads the addressed row as subject.<field> and subject.state, so one branch is selected by the held state and a related row together
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:at-ess-38-instance-and-each-in-name-a-member-of-a-struct-input-and-the-documents-list-every-addressing
revision: 1
---
## Need

A branch cannot be selected by the addressed row's held state (or a stored field) and a related row
at once. `when_related:` beside `when_subject:` or `when_subject_state:` on one branch is refused
(`crates/specify/ess-domain/src/command/related_guard.rs:510-536`, `alone`) as
`conflicting_declaration` (ESS-COMMAND-004): "outcome `held` declares `when_related` beside
`when_subject_state`; a branch has one selection authority", hint "keep the related guard and an
optional `when:` over the input, or the other key". The doc of `alone` gives the reason: "on one
branch each is a second selection authority, and a replay takes the answer its origin gave". And a
`when_related` predicate reads its own row and the input only
(`related_guard.rs:123-125`; `docs/design/language-grids.md` grid R, row "`when_related:`
predicate", column "subject field / state": "no").

Since `ess/27` the one precedence rule lets a related guard and held-state guards share a command
on different branches (`related_guard.rs:764-766`), with held-state branches answering at step 3
before any related row at step 4 (`docs/design/generic-language-features.md:484-497`). That covers a
case where the held state alone decides first. It does not cover two branches that read the same
related row and differ by the held state, because a step-3 branch answers whatever the related row
holds.

Minimal reproduction, written for this story (not run here). `demo.ship.Load(shipment_id)` on
`demo.ship.Shipment {shipment_id, carrier_id}` with lifecycle `Packing -> Loaded -> Departed`, and
`demo.ship.Carrier` with lifecycle `Active <-> Suspended`:

```yaml
outcomes:
  - name: no-shipment
    unknown_instance: true
    error: demo.ship.ShipmentNotFound
  - name: no-carrier
    when_related: {via: carrier_id, exists: false}
    error: demo.ship.CarrierNotFound
  - name: held
    when_related: {via: carrier_id, predicate: {all: [state == Suspended, subject.state == Packing]}}
    error: demo.ship.CarrierSuspended
  - name: recalled
    when_related: {via: carrier_id, predicate: {all: [state == Suspended, subject.state == Loaded]}}
    moves: demo.ship.Shipment.recall
    instance: shipment_id
  - name: loaded
    moves: demo.ship.Shipment.load
    instance: shipment_id
  - {name: wrong-state, wrong_state: true, error: demo.ship.WrongState}
```

Today `subject.state` in the predicate is refused (its exact message not established by reading),
and the requester's spelling, `when_related:` beside `when_subject_state:` on one branch, is refused
as quoted above. The requester's workaround copies the related row's state into the addressed row
when it is created, which holds only while the related row never changes afterwards.

## Outcome

From the format named under Decisions, a `when_related:` `predicate:` (one row, and each element
of `when_related: {all: […]}`) reads the addressed subject as it was before the branch:
`subject.<field>` for a declared stored field and `subject.state` for its held lifecycle state,
the reading a row-set selector already gives `subject.`
(`crates/specify/ess-domain/src/command/row_set.rs:12-16`). Bare names keep reading the related
row. The branch stays a related-row branch: it answers at step 4 (a refusal) or step 5 (an
accepting branch), after the addressed row's existence (step 2) and every held-state branch
(step 3). `when_related:` beside `when_subject*` on one branch stays refused, its hint naming the
new reading.

## Acceptance

- Red first: `crates/specify/ess-domain/tests/` gains, in an existing binary of related guards,
  `a_related_predicate_reads_the_addressed_rows_state_at_the_new_format`: the reproduction
  validates at the new format; on the base `held` and `recalled` are refused.
- The same text below the new format is refused as `unsupported_format_version` naming the format
  ("a `when_related` predicate reading the addressed subject requires specification format
  ess/<n>"), its hint `FormatVersion::declaration_hint`.
- Refused at every format, a test each: `subject.` on a command whose branches address no common
  existing subject (a command that only creates, or two branches naming different subjects), the
  refusal reusing the row-set selector's message for the same case
  (`docs/design/filtered-related-reads.md:75-80`, `subject_fact::common_subject`); `subject.` naming
  an undeclared field (`undeclared_reference`, naming the declared fields).
- The validation partition of a command's related rows crosses the subject's stored fields and
  held state with the related row's fields and the input: in the reproduction `held` and
  `recalled` do not overlap, and changing `recalled`'s predicate to `subject.state != Departed`
  makes the pair overlap on `Packing` and is refused as overlapping refusal and acceptance, as
  today for two predicates over one row.
- A branch reading `subject.state == S` declared after a step-3 branch selected by `S` alone is
  refused as unreachable, naming the step-3 branch (the existing reachability refusal, generalised).
- `alone` keeps refusing `when_related:` beside `when_subject:` and `when_subject_state:` on one
  branch; from the new format its hint reads "read the addressed row inside the related
  predicate: `subject.state == <State>`, `subject.<field>`"; below it the hint is unchanged.
- If a row-set selector does not yet read `subject.state`, it does from the new format, with one
  test (the same column of grid R; the closing rule, `generic-language-features.md:211-216`).
- The interpreter binds `subject.` when it evaluates a related predicate: a case in an existing
  `ess-conformance` binary executes the reproduction against a hand-built store, no synthesis:
  `Packing` + `Suspended` answers `held`, `Loaded` + `Suspended` answers `recalled`, `Packing` +
  `Active` answers `loaded`, `Departed` + `Suspended` answers `wrong-state`. The history checker
  reads the same.
- IR: no new shape. The predicate carries fact paths under `subject.` as a row-set selector's does;
  an old-reader test shows a model without such a path compiles to the same bytes as before.
- `website/docs/reference/predicates.md` § "A guard over another entity's row" (from line 1370)
  lists `subject.` among what the predicate reads; `docs/design/language-grids.md` grid R cell
  becomes "yes (`ess/<n>`)"; the format row in `formats.md` and the format history in `system.rs`
  name it; `CHANGELOG.md` `### Added` gets one line with its grid cell (policy 3).
- Every new test under 5 s or named with its time; no new test binary.

## Fit review

Tree `bcfc51555` (`integration/wave-2026-11-24`), read 2026-11-24.

1. **Need.** A branch selected by the addressed row's held state and a related row together:
   "a suspended carrier holds a packing shipment and recalls a loaded one". Reproduction above. The
   requester's words: "a related-row guard beside a held-state guard" (report item 11); their
   spelling: `when_related:` beside `when_subject_state:` on one branch.
2. **Class.** Gap. The refusal is a branch-shape rule, not a missing order: the precedence rule
   orders the two guards across branches but gives no meaning to both on one branch.
3. **Already expressible?** Partly. A command whose held-state branches decide before the related
   row reads it with the precedence rule since `ess/27` (inferred for the requester's own command:
   its other branches select on the held state first, so its related branch would need no held
   state of its own; not run). Two branches over one related row that differ by held state cannot
   be stated; the copied-state workaround goes stale when the related row moves.
4. **Fit.** Grid R row "`when_related:` predicate" × column "subject field / state" is "no"; the
   column's target is "every command position" (`docs/design/language-grids.md`, Target). The
   row-set selector already reads `subject.` (`row_set.rs:12-16`), and `ess/18` reads `state` in a
   `when_subject` predicate (`crates/specify/ess-domain/src/command/subject_fact.rs:20-28`), so the
   spelling exists twice and this reuses it. A sibling cell of the same row, the caller, is
   `story:caller-attributes-are-readable-in-a-when-related-predicate` (active). The G1/G4 design
   reads `when_subject_state: [A, B]` as `subject.state in [A, B]`
   (`generic-language-features.md:526-527`): this is that reading inside the related predicate.
   Composes with `all:` (each element's predicate), `exists: false` (no change: a missing related
   row answers first), stored-field `via`, `wrong_state:` and every step-3 branch.
   Targets: interpreter here; synthesis in `g11-subject-read-synthesis`; generated Rust and Go and
   Entity Runtime in `g11-subject-read-targets`.
5. **Second adopter.** A refund is refused while the order is `Open` and its payment `Pending`,
   and reversed while the order is `Shipped` and its payment `Disputed`.
6. **Cost.** One format gate; no key, no diagnostic code, no IR shape; a widened partition.
7. **Considered.** (a) Change nothing: rejected for the general case (above). (b) The requester's
   spelling, `when_subject*` beside `when_related:` on one branch, conjunctive, as `external:`
   beside `when_subject:` at `ess/29` (`generic-language-features.md:529-545`): workable, but a
   second spelling of one conjunction, and a new IR field. (c) Chosen: the related predicate reads
   `subject.`, one authority per branch, the spelling the row-set selector already has.

## Decisions

**Accept, redesigned**: the requester's two keys on one branch become one predicate reading
`subject.`. Report item 11 covered. Open: `P2A-D3` (shape; default A, `subject.` in the related
predicate) and `P2A-D6` (format).

## Scope

- cited: `crates/specify/ess-domain/src/command/related_guard.rs` (`alone` 510-536; the
  predicate's environment; `validate_joint_partition` from 2120; reachability)
- inferred: `crates/specify/ess-domain/src/expression.rs` (the related predicate's environment
  admits the `subject` root)
- inferred: `crates/specify/ess-domain/src/command/subject_fact.rs` (`common_subject`)
- inferred: `crates/specify/ess-domain/src/command/row_set.rs` (only if `subject.state` is not
  read there yet)
- inferred: `crates/specify/ess-domain/src/system.rs` (format history row, gate)
- inferred: `crates/verify/ess-conformance/src/interpret/execute/related.rs`,
  `interpret/execute/history.rs`
- inferred: `crates/specify/ess-compiler/tests/` (old-reader bytes case in an existing binary)
- cited: `website/docs/reference/predicates.md`, `docs/design/language-grids.md`,
  `website/docs/reference/formats.md`, `CHANGELOG.md`
- Size: M to L (about 500 changed lines). If the partition change runs past 600, split it as
  `g11-related-reads-subject` (validation, refusals, docs) and a second unit for the partition and
  the interpreter.
- Overlap: s1 and s2 of the wave of 2026-11-24 change `related_guard_ir.rs`, `resolve.rs`,
  `ir.rs` and `spec.rs`; this unit lands after s2.

## Depends on

- `story:at-ess-38-instance-and-each-in-name-a-member-of-a-struct-input-and-the-documents-list-every-addressing`
  (s2, shared files).

## Source

A downstream application, 2026-11-24, report item 11 (its G3).
