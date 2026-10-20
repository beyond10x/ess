---
format: aep.planning-md/3
id: story:payload-source-reads-the-head-of-an-ordered-selection
kind: story
status: implemented
title: A payload source reads the head of an ordered selection
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-19T21:15:01Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-19T21:15:02Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-19T21:15:02Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

A payload and `sets:` source reads one field of the first row of an ordered selection:
`{related: {entity, where, order_by: [{field, direction}...], first: <field>}}`, with a total order
required, so a model states which ranked element an outcome takes and synthesis arranges a witness
where the head differs from insertion order.

Closes https://github.com/beyond10x/ess/issues/543.

## Fit review

1. **Need.** A matching command pops the highest-scored present membership of each side and
   publishes both keys. `{related: {entity, where, field}}` (ess/22) supplies no value when the
   selector matches several rows, so the fields fall back to `{generated: true}`: the suite checks
   presence and type, not which elements were paired (3 unmapped fields and 2 known failures in one
   adopter).
2. **Class: gap.**
3. **Already expressible?** No: `order_by` exists on views only (`ViewSpec.order_by`); in a payload
   source it is refused ("a list is read only as the `via:` …").
4. **Fit.** Reuses the view layer's ordering vocabulary at the related source. `first:` replaces
   `field:` (one of the two, never both). The order must be total: it must end in a key that is
   unique over the selected rows (the entity's identity, or a declared unique field), otherwise it
   is refused, so the head is deterministic for every evaluator. An empty selection supplies no
   value, as a selector matching no row does today.

## Decisions

- Syntax as the requester proposed (`order_by` list of `{field, direction}`, `first: <field>`).
- Totality: refused unless the last key, or a prefix of keys, is unique over the entity (identity
  fields, or a unique constraint); the refusal names the missing tie-break.
- Format `ess/25`; old-reader test.
- Reversible: a new optional source form.

## Acceptance

Each line names the failing test written first.

- `crates/specify/ess-domain/tests/ordered_related_source.rs`
  `an_ordered_head_source_validates_at_ess_25`, `an_order_without_a_unique_tie_break_is_refused`,
  `first_beside_field_is_refused`.
- `crates/verify/ess-conformance/tests/ordered_related_source.rs`
  `synthesis_arranges_a_witness_whose_head_is_not_the_first_inserted_row` and
  `a_target_taking_the_first_inserted_row_fails_a_scenario`.
- same file, `the_interpreter_publishes_the_ranked_head`.
- `crates/generate/ess-synth` (file inferred) `generated_behaviour_reads_the_ranked_head` (Rust, Go).

## Scope

- inferred: `crates/specify/ess-domain/src/command/` related sources, `wire.rs`,
  `crates/specify/ess-compiler/src/ir.rs`, `crates/verify/ess-conformance/src/{synthesize/related*,
  interpret/execute/related.rs}`, `crates/generate/ess-synth/src/{rust,go}/behaviour.rs`,
  `crates/generate/ess-entity-runtime` (refuses by name if it cannot lower), schema projection.

## Plan

- depends_on: the unit that opens `ess/25`.
- du estimate: 10G.
- adversary: yes.
- shared surfaces: related-source code with `story:generated-behaviour-reads-several-related-rows`
  (not in this release).

## Design pass

Design pass (2026-10-13, unit commit 441b8a35f, merged on the 2026-10-13 integration branch as 09f0846d8): `docs/design/ordered-related-head.md`. Its Fit review answers all seven questions of `.agents/skills/assessing-external-requests/SKILL.md` with citations. Decision: **accept, redesigned**.

Departures from the decisions above, accepted by the coordinator:

- Order keys are spelled as a view spells them (`[element_score desc, membership_id]`).
- The identity is the last key, and nothing may follow it.
- Generated Rust and Go report a named obligation; generating selector reads becomes its own story.
- Unit U5 is added: synthesis with several selectors per command.
- Scope is corrected to `command.rs`, `synthesize/row_set.rs`, `interpret/execute/row_set.rs` + `values.rs` and `determined.rs`.
