---
format: aep.planning-md/3
id: story:each-entry-moves-the-rows-its-elements-name
kind: story
status: implemented
title: 'An each: entry moves or deletes the row each element names, by a per-element where:'
summary: One call takes each named row to its own transition by the answer the input carries for it (#596)
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#596
relations:
- decomposes: epic:generic-row-queries
- serves: vision:O2
- informed_by: story:selected-rows-effect-emits-one-event-per-changed-row
- depends_on: task:land-the-2026-10-13-port
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-diff/src
- confidence: cited
  path: docs/design/set-effects-over-filtered-instances.md
- confidence: inferred
  path: schemas/generated/ess.schema.json
- confidence: cited
  path: website/docs/guides/specify/selection-effects.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-19T21:15:17Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-19T21:15:17Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-10-19T21:15:18Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

An `affects:` entry over an input list (`each: {in: input.<list>, as: <x>}`) may move or delete the
row each element names, and may carry a per-element `where:` over the element, so one call takes
each named row to its own transition by the answer the input carries for that row:

```yaml
affects:
  - {entity: demo.batch.Proposal, each: {in: input.answers, as: a, where: a.verdict == applied},
     instance: a.proposal_id, moves: demo.batch.Proposal.apply}
  - {entity: demo.batch.Proposal, each: {in: input.answers, as: a, where: a.verdict == refused},
     instance: a.proposal_id, moves: demo.batch.Proposal.refuse}
  - {entity: demo.batch.Proposal, each: {in: input.answers, as: a, where: a.verdict == invalid},
     instance: a.proposal_id, moves: demo.batch.Proposal.invalidate}
```

An element no entry selects leaves its row as it was; an element whose row rests outside the
transition's `from` states is skipped, as a row under `instances:` is; a refused run moves nothing.

Closes https://github.com/beyond10x/ess/issues/596. The per-row command (one call per row, below)
is the answer the reporter gets today; the one-call batch is this story.

## Fit review

Tree `a500f9f56` (0.62.0); binary run: ess 0.62.0. Minimal reproductions written for this review,
neutral nouns (`perrow.batch`: `Batch`, `Proposal`, `Answer {proposal_id, verdict}`), not the
adopter's document: `.engineering/repro/596/each-moves.yaml` (the refusal) and
`.engineering/repro/596/per-row-settle.yaml` (the idiom), run with
`ess specify validate --path <file>` and `ess verify conform synthesize --path <file> --out <suite>`.

1. **Need.** A batch is applied in one call, and each row of it goes to its own transition,
   decided per row by an answer an external system gave for that row (`applied`, `refused`,
   `invalid`). Reproduction `perrow-each-moves/system.yaml`: `Apply {batch_id, answers:
   List<Answer>}` with `affects: [{entity: Proposal, each: {in: input.answers, as: a}, instance:
   a.proposal_id, moves: Proposal.apply}]` is refused by `ess specify validate`:
   `[conflicting_declaration] command.demo.batch.Apply.outcomes.applied.affects[0].each: outcome
   applied declares each: and moves: in affects[0]; an each: entry creates or updates the row each
   element names, and selects, moves or removes no other (hint: keep each: with instance: and
   sets:, or write the other effect as an entry of its own)` (ESS-COMMAND-004). The issue's three
   other attempts are refused as it reports (three unconditional set moves: ESS-COMMAND-004; one
   set move: ESS-ENTITY-005 `missing_causation`; `when:` on the input: one verdict per call).
   Requester's proposed design, labelled as theirs: "a set move takes each selected row to its own
   outcome, decided per row by a predicate over the row, or by an external answer per row".
2. **Class: gap** for the one-call batch (no spelling moves N rows by N answers in one outcome);
   **convenience** where the answers arrive one row at a time (question 3).
3. **Already expressible?** Partly. (a) One command per row: `Settle {proposal_id, verdict}` with
   three `when: verdict == …` branches, each `moves:` its transition (`perrow-settle/system.yaml`):
   `ess specify validate` answers `valid`, and `ess verify conform synthesize` writes 12 scenarios
   with 0 refusals, among them `Settle/outcome/applied`, `…/refused`, `…/invalid` and the three
   `Proposal/transition/…/by/Settle/…` witnesses, so every per-row verdict is checked. This is the
   ESS-native shape when the external system answers per row, and the answer to the issue's last
   question. (b) The reporter's `Record` + `Apply` workaround validates but adds a stored verdict
   and a command the domain does not have, and turns the external answer into caller input twice.
   (c) `instances: {where: …}` with `when:` on the input: one verdict per call (the issue). None
   states the one-call batch.
4. **Fit.** G2 already reads `each: {in, as}` as `each x in input.list` and `instances: {where: p}`
   as `each r in rows(E where p)` (`docs/design/generic-language-features.md:299-300`); the gap is
   one refusal in the set-effects design: "`each:` beside `where:`, `moves:` or `deletes:` in one
   entry is `conflicting_declaration`" (`docs/design/set-effects-over-filtered-instances.md`, "One
   row per element of an input list", Refusals; `crates/specify/ess-domain/src/command/set_effects.rs:864`
   `each_entry`). Opening `moves:` and `deletes:` under `each:`, with a `where:` over the element,
   reuses the keys `instances:` and `affects:` already take (`moves:`, `deletes:`, `where:`) and
   adds none. It composes with the existing `each:` rules (a declared `distinct:` over the member,
   an element naming the subject skipped, an empty list accepted, a refused run writing nothing),
   with `emits: {event, each: changed}` (`ess/25`, #531) for one event per moved row, with
   bindings on those events, and with #547's identity-pinned `affects:` filters (branch
   the port branch (head `30de58532`)), which cover the single-row case. Siblings: `instances:` keeps its
   one transition per outcome, because its rows carry no per-row answer; the element is what
   carries one. Targets: the interpreter applies the entries element by element in list order;
   Entity Runtime refuses every set effect by name (`SetEffectUnsupported`); Rust, Go, Web and Clap
   owe it as an obligation once #547's D4 lands, and refuse it by name before; `ess verify diff`
   reports it under `outcome-set-effect-changed`; generated docs say one row per element.
5. **Second adopter.** A bulk import whose validator answers `accepted` or `rejected` per line and
   the importer records both in one call; a payroll run where the bank answers `paid` or `bounced`
   per transfer.
6. **Cost.** No new key, no new diagnostic code, no suite format (the `each:` scenario family's
   steps exist). One relaxation and one new key position (`where:` inside `each:`) in
   `set_effects.rs`, so `cargo xtask schema` regenerates the document schema. Source-format gate:
   `ess/27`, the format this batch opens (`release-plan:ess-063-downstream`); below it the entry keeps today's
   refusal naming `ess/27`. Synthesis: per entry one element whose row rests in the `from` state
   and one whose row rests outside it, plus a decoy row no element names.
7. **Alternatives.** (a) Change nothing: the per-row command stands, the one-call batch stays
   unexpressed, and a specification of an atomic batch keeps an UNMAPPED marker. (b) The
   requester's "a set move takes each selected row to its own outcome": per-row branch selection
   inside one outcome, a construct every consumer (precedence plan, interpreter, synthesis, diff,
   generators) would need to learn; refused. (c) `each:` admitting `moves:`/`deletes:` with an
   element `where:` (chosen): the element carries the per-row answer, the existing construct
   carries the effect.

## Decisions

- **Accept, redesigned** into (c). The requester's per-row outcome selection is not taken; the
  per-row answer lives in the input element.
- Design first: a section "Moves and deletes per element" in
  `docs/design/set-effects-over-filtered-instances.md` decides, before code: an element naming no
  row (refused by name as `missing_declaration`-class validation cannot see it, so a run-time
  answer: skipped, with the count of skipped elements readable as `{count: skipped}`, or a declared
  refusal; the page chooses one and says why), the order in which entries apply, and that the
  `distinct:` requirement is unchanged.
- Format: `ess/27` while `release-plan:ess-063-downstream` keeps it open; the next open format otherwise.
- Reply to the requester: the per-row command is the way to write it today (validated, witnessed);
  the one-call batch lands with this story.

## Acceptance

Each line names the failing test written first.

- `docs/design/set-effects-over-filtered-instances.md` § "Moves and deletes per element": the
  decisions above, with the reproduction and its refusal quoted.
- `crates/specify/ess-domain/tests/set_effects.rs` (module of the existing binary):
  `each_entry_with_moves_validates_at_ess_27`, `each_entry_with_an_element_where_validates`,
  `each_entry_with_deletes_validates_at_ess_27`,
  `below_ess_27_each_with_moves_keeps_its_refusal_naming_ess_27`,
  `each_beside_where_on_the_entry_stays_refused`.
- `crates/verify/ess-conformance/tests/interpreted_set_effects.rs`:
  `interpreter_moves_each_named_row_by_its_elements_verdict` (three answers, three states, a
  fourth row untouched, an element whose row is already `Applied` skipped).
- `crates/verify/ess-conformance/tests/set_effects.rs`:
  `synthesis_witnesses_each_entry_moves_and_a_skipped_element`: a target that moves every named
  row by the first verdict fails; a target that moves the skipped row fails.
- `crates/verify/ess-diff` (existing test binary): `each_moves_change_is_classified` under
  `outcome-set-effect-changed`.
- The reproduction model, neutralised, as `crates/verify/ess-conformance/tests/fixtures/each-moves.yaml`.

## Scope

- cited: `crates/specify/ess-domain/src/command/set_effects.rs` (`each_entry` at `:864`),
  `docs/design/set-effects-over-filtered-instances.md`,
  `website/docs/guides/specify/selection-effects.md` ("Write one record per element of an input
  list").
- inferred: `crates/specify/ess-compiler/src/ir.rs`,
  `crates/verify/ess-conformance/src/interpret/execute.rs`,
  `crates/verify/ess-conformance/src/synthesize/set_effects.rs`, `crates/verify/ess-diff/src/`,
  `schemas/generated/ess.schema.json` (`cargo xtask schema`), `CHANGELOG.md`.

## Order

Wave 3 of `release-plan:ess-063-downstream`, after `task:land-the-2026-10-13-port` (which rewrites
`set_effects.rs` and `synthesize/set_effects.rs` for #547) and after the in-flight unit U1 of
`integrate/ess-downstream-20261018` (same synthesis file). Parallel with
`task:salvage-the-related-rows-quantifier-tree` only if the two do not both edit
`synthesize/set_effects.rs`; the scoper decides at dispatch.
