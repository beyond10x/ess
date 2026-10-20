---
format: aep.planning-md/3
id: story:related-rows-quantifier-in-a-selector
kind: story
status: implemented
title: A row selector cannot test each candidate's related rows
summary: 'exists: walks an entity''s rows (in: {entity: E}, as, that) in value selectors and row-set guards, from ess/25'
owner: timo
tags:
- adopter-request
refs:
- provider: github
  reference: beyond10x/ess#551
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-19T21:15:15Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-19T21:15:16Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-19T21:15:17Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

A selector may test each candidate's related rows: the quantifier `exists:` takes the rows of an
entity as its collection, `exists: {in: {entity: <E>}, as: <b>, that: <P>}`, with bare names still
reading the candidate and `<b>.` the related row. A model then picks the best slot of a group among
the slots whose two sides each hold a `Present` membership, refuses by name when no slot qualifies,
and synthesis arranges a witness in which a higher-scored slot failing the test is present, so a
target that ignores the test fails.

Closes https://github.com/beyond10x/ess/issues/551. Design: `docs/design/related-rows-quantifier.md`.

## Fit review

All seven questions are answered with citations in `docs/design/related-rows-quantifier.md`,
"Fit review". In short:

1. **Need.** A command picks a row among candidates each of which must have related rows meeting a
   condition, and refuses by name when none qualifies. Measured with the binary at `3fdd9479e`: the
   requester's correlated `exists: {entity, where}` is a parse refusal ("a quantifier takes `in`,
   `as` and `that`"), a counter kept by `affects:` on the creating command is `ESS-COMMAND-009`, and
   a quantifier over a declared relation is `unobservable_fact`.
2. **Class: gap.** Selector operands are the candidate, `input.`, `subject.`, `now`
   (`ess-domain/src/command/row_set.rs:12-16`); a quantifier's collection is a fact path
   (`ess-primitives/src/predicate.rs:2961-2966`). `published-value-operand.md` names the gap.
3. **Already expressible? No.** All three idioms are refused (above). Naming the slot in the input
   is #544's problem; a counter states one fact twice.
4. **Fit.** Reuses the quantifier `{in, as, that}` and the selector's `entity:` key; no new root,
   no new keyword. Admitted in value selectors and row-set guards; every other predicate site
   refuses by default through the type environment. Composes with #549's `payload.` operand.
   `forall` refused with its `not exists` rewrite; `count` named as not in this design.
5. **Second adopter.** A support desk assigns to the highest-priority team that has an available
   agent; a warehouse picks the nearest bin holding a lot with stock.
6. **Cost.** Rides `ess/25`. No keyword, no `ValidationCode` (about 9 messages), no IR shape
   change, schema description only, no suite or diff version, no migration. Public Rust API:
   `Quantified::over` becomes a `Collection` enum and `FactSource` gains a defaulted `rows`; about 85
   mechanical arms in 17 files; no sibling repository reads `over`.
7. **Alternatives.** Change nothing; the requester's `{entity, where}` with a `candidate.` root
   (changed); a declared relation as the collection (later); a maintained aggregate (declined); an
   author-kept counter (declined); the rows quantifier (chosen).

## Decisions

- **accept, redesigned.** The per-candidate `exists` is kept; the spelling is the existing
  quantifier with a rows collection; the maintained aggregate is declined.
- Spelling `exists: {in: {entity: <E>}, as: <b>, that: <P>}`; bare names stay the candidate.
- A tie `<b>.<f> == <candidate field>` is required as a top-level conjunct of `that:`.
- Admitted in `{related: {entity, where, field | order_by+first}}` selectors and row-set
  `when_related:` `where:`/`forall:`; refused by name everywhere else (set-effect filters deferred
  with #547; views and invariants until generated targets enumerate rows).
- `exists` only (`forall` refused with the rewrite); one level (no nesting); the binder may not
  shadow a candidate field, `state`, a root or a binder in scope.
- `Quantified::over` becomes `Collection { Fact, Rows { entity } }` so every arm decides;
  `FactSource::rows` defaults to `None`, evaluating `Unknown`.
- No IR shape change, no new code, no suite or `ess-diff` version; rides `ess/25`.
- Generated Rust and Go stay owed under the existing selector obligation; the story
  `generated-behaviour-reads-several-related-rows` gains a scope line.
- Reversible: the shape was refused at every format before.

## Acceptance

Each line names a failing test written first (full list in the design, "Decisive acceptance").

- `crates/specify/ess-primitives/tests/related_rows_quantifier.rs`: parse and print,
  `in` shape refusal, `Unknown` without a rows source, Kleene disjunction over rows, binder routing.
- `crates/specify/ess-domain/tests/related_rows_quantifier.rs`:
  `a_head_over_matchable_slots_validates_at_ess_25`,
  `a_row_set_guard_refusing_no_matchable_slot_validates`,
  `the_rows_test_below_ess_25_is_refused_naming_ess_25`, `an_untied_rows_test_is_refused`,
  `a_binder_naming_a_candidate_field_is_refused`,
  `forall_over_rows_is_refused_with_the_not_exists_rewrite`, `a_nested_rows_test_is_refused`,
  `every_other_predicate_site_refuses_a_rows_test` (census).
- `crates/specify/ess-compiler/tests/related_rows_ir.rs`:
  `the_selection_carries_the_rows_test_in_its_predicate`,
  `a_model_without_a_rows_test_keeps_its_ir_bytes`.
- `crates/verify/ess-conformance/tests/related_rows_quantifier.rs`:
  `the_interpreter_skips_a_higher_scored_slot_with_an_empty_side`,
  `no_matchable_slot_is_refused_by_name`,
  `synthesis_arranges_a_higher_scored_candidate_that_fails_the_test`, and faulty targets each
  failing a scenario: ignoring the test, testing one side, dropping the state conjunct, dropping the
  side conjunct, dropping the tie; named gaps for an untied creation and an unsteerable first key.

## Scope

Units R1-R5, files and gates per `docs/design/related-rows-quantifier.md`, "Implementation units":

- R1 `ess-primitives` collection, evaluation and every quantifier arm (17 files; cited).
- R2 `ess-domain` sites, format, tie, binder, `forall`, nesting; IR pin test (cited; the claim
  that every predicate site is typed through `check_predicate` is **inferred**, checked by the
  census test).
- R3 interpreter (`interpret/execute/{row_set,set_effects,caller}.rs`, `history/facts.rs`; cited).
- R4 synthesis (`synthesize/row_set.rs`, new `synthesize/related_rows.rs`; cited).
- R5 documentation.

Order R1 → R2 → R3 → R4 → R5, based after #549 U2; R2-R4 overlap #549 U1/U3/U4 (high).
