---
format: aep.planning-md/3
id: story:published-value-operand-in-a-selector
kind: story
status: implemented
title: A head of an ordered selection cannot be scoped by another value the same outcome publishes
summary: 'payload.<Event>.<field> as an operand in a value read''s where:'
owner: timo
tags:
- adopter-request
refs:
- provider: github
  reference: beyond10x/ess#549
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-19T21:15:09Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-19T21:15:09Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-19T21:15:10Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

A selector of a value source may read a value its own outcome publishes:
`payload.<Event>.<field>[.<member>…]` in the `where:` of `{related: {entity, where, field}}` and
`{related: {entity, where, order_by, first}}`. A model then states nested heads (the best slot of a
group, then the head of each side of that slot), and synthesis arranges a witness in which a target
that scopes the inner read by the wrong outer row fails.

Closes the first need of https://github.com/beyond10x/ess/issues/549. Design:
`docs/design/published-value-operand.md`.

## Fit review

All seven questions are answered with citations in `docs/design/published-value-operand.md`,
"Fit review". In short:

1. **Need.** A command reads a ranked row, then a ranked row related to it, and publishes both. The
   inner selector cannot name the outer row. Minimal reproduction with the binary at `3fdd9479e`:
   `payload.demo.nested.PairMade.slot_id` reads `payload`, which is not a declared observable root
   (`unobservable_fact`).
2. **Class: gap.** Selector operands are the row, `input.`, `subject.` and `now`
   (`ess-domain/src/command/row_set.rs:12-16`). `ordered-related-head.md:336-339` names the
   exclusion.
3. **Already expressible? No.** The caller-names-the-slot workaround validates, but it checks
   nothing about the implementation's choice (#544).
4. **Fit.** It reuses #545's `{payload: <Event>.<field>}` meaning and lookup as a predicate root
   beside `input.` and `subject.`. Guards refuse it by name (precedence). `affects[].where` refuses
   it by name until #547. Every target executes, owes or refuses it.
5. **Second adopter.** A support desk: the highest-priority open queue of a team, then the oldest
   waiting ticket in that queue. A warehouse: the fullest bin of a product, then the oldest lot in
   it.
6. **Cost.** It rides `ess/25`. No new keyword, no new `ValidationCode` and about 9 messages. One
   optional IR field (`ResolvedRowSelection.reads`), so old bytes are unchanged. Schema, suite
   format and generated API are unchanged. No migration.
7. **Alternatives.** Change nothing, a named intermediate, an inline source as an operand, the
   requester's mapping spelling, and the chosen root (reasons in the design).

## Decisions

- **accept, redesigned.**
- The spelling is the predicate root `payload.<qualified Event>.<field>`. The requester's mapping
  form `where: {slot_id: {payload: PairMade.slot_id}}` is not used: `where:` is a predicate, and
  event names are qualified, as in #545.
- An operand, not a named intermediate. An intermediate can be added later without conflict.
- The read field must be one this outcome emits and fills, with a snapshot-value source: no mint,
  no response, no `{count: changed}`, not the created identity.
- Admitted only in value-source selectors. Guards, `affects[].where`, `instances:` and error
  payloads refuse it as `unsupported_construct`.
- Cycles among an outcome's payload fields are refused as `self_reference`.
- The IR lists the published reads on `ResolvedRowSelection` explicitly, so no evaluator decides
  `payload.` by the candidate entity's fields.
- Generated Rust and Go stay owed under the existing row-set obligation.
- It rides `ess/25` while that format is unreleased.
- Reversible: a new operand, refused below `ess/25`.
- **Not covered: "the slot can match"** (best slot among those whose sides are both non-empty).
  This is a correlated row-set test per candidate, a separate construct needing a decision. It was
  measured refused both inside a selector and as a counter maintained through `affects:`.

## Acceptance

Each line names the failing test written first.

- `crates/specify/ess-domain/tests/published_value_operand.rs`:
  `a_nested_head_reading_the_published_slot_validates_at_ess_25`,
  `the_operand_below_ess_25_is_refused_naming_ess_25`,
  `a_selector_over_an_entity_with_a_payload_field_keeps_its_meaning`,
  `the_operand_in_a_row_set_guard_is_refused`, `the_operand_in_an_affects_where_is_refused`,
  `an_operand_reading_a_mint_is_refused`,
  `an_operand_reading_an_event_the_outcome_does_not_fill_is_refused`,
  `a_cycle_between_two_payload_fields_is_refused_as_self_reference`,
  `a_field_whose_selector_reads_itself_is_refused`.
- `crates/specify/ess-compiler/tests/published_value_operand_ir.rs`:
  `the_selection_lists_its_published_reads_in_dependency_order`,
  `a_model_without_the_operand_keeps_its_ir_bytes`.
- `crates/verify/ess-conformance/tests/nested_ordered_heads.rs`:
  `the_interpreter_takes_each_side_of_the_slot_it_chose`,
  `an_empty_outer_selection_publishes_nothing`,
  `the_event_and_the_stored_copy_hold_the_operand_value`,
  `synthesis_arranges_cross_decoys_in_the_other_slots`,
  `a_target_scoping_by_the_first_inserted_slot_fails_a_scenario` (and the last-inserted and
  slot-ignoring targets), `a_minted_scope_is_a_named_gap`.

## Scope

- cited: `crates/specify/ess-domain/src/command/{row_set.rs,set_effects.rs,value_expression.rs,minted_value.rs}`, `command.rs`; new `command/published_operand.rs`.
- cited: `crates/specify/ess-compiler/src/{ir.rs,resolve.rs}` (`ResolvedRowSelection`, constructed at `resolve.rs:3152`, `:3207`, `:5469`).
- cited: `crates/verify/ess-conformance/src/interpret/execute.rs` (`payload_reads`, `publish_reads`), `interpret/execute/{row_set.rs,set_effects.rs}`.
- cited: `crates/verify/ess-conformance/src/synthesize/{row_set.rs,ordered_head.rs}`.
- inferred: #543 U7's author tie check, if it does not evaluate selectors through `Reader`.
- cited: docs `website/docs/guides/specify/values-and-views.md`, `website/docs/reference/spec-versions.md`, `CHANGELOG.md`, `docs/design/ordered-related-head.md`.

## Plan

- depends_on: #543 U7 (struct identities in an order, declared-unordered ties).
- units: U1 domain → U2 IR → U3 interpreter → U4 synthesis → U5 docs, strictly serial.
- du estimate: U1 and U2 build no `ess-conformance`. U3 and U4 each take about 10G (the #543
  story's estimate for an `ess-conformance` unit).
- adversary: yes (U3 and U4).
- shared surfaces: `ess-domain` `value_expression.rs` and `set_effects.rs`, and the head's
  interpreter and synthesis, with #543 U7. `ResolvedRowSelection` with the #547 follow-up.
