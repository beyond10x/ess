---
format: aep.planning-md/3
id: story:a-required-list-with-no-route-is-sent-empty-where-its-row-is-understood
kind: story
status: draft
title: A required list with no route is sent empty wherever its stored row can be decided
summary: 'Follow-up of story:accepted-input-arranges-referenced-list-rows: the fail-closed no-route check refuses by name when a field is set in a form it does not read exactly (else naming an input, struct, conversion, expression, generated, an update''s untouched field), though the empty list would pass (turn-3 split, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:accepted-input-arranges-referenced-list-rows` sends the empty list for a `cardinality: many`
reference no command creates, and decides it in one place, `empties_keep_branch`
(`crates/verify/ess-conformance/src/synthesize/referenced_input.rs`). The check fails closed: it
applies every unarrangeable list's `[]` at once, and refuses every such list's scenario by name
unless the precedence order still selects the branch, `invariant_broken_by` finds nothing broken,
and every invariant that reads a list, a field the branch sets, or `state` is decided true over
the row the branch would store.

The row is read exactly only for a copied input, a literal, `{input: x, else: <literal>}`, the empty
list, and (on a creating branch) the state it lands in. Any other form a branch sets a field in is
undecided, never absent, so an invariant that reads it refuses the scenario even where `[]` passes.

## Problem

Wrongly refused, each with a route-less list and an invariant over the field:

- `else:` naming another input (`{input: note, else: input.alt}`).
- A struct value (`cfg: {backup: input.sources}`) or a member path target.
- A field stored through a conversion, an expression or `{generated: true}`.
- A branch that does not create, reading a field it leaves in place (`updates:`, `moves:`): the old
  row's value is not known to synthesis.
- A held-state branch, whose selection falls back to `keeps_branch` one list at a time.
- An invariant reading `state` on a branch that does not create.

## Design rule

Bind each of these forms to the value the branch stores (follow an `else:` to its input, expand a
struct source, evaluate an expression over the input), and bind a non-creating branch's unset
fields from the row the scenario arranged for it, so the invariant is decided instead of refused.
Keep failing closed for any form still not followed.

## Acceptance

1. A route-less `sources` list with `else:` naming another input, under an invariant `[]` keeps,
   has an accepted scenario that passes the interpreter.
2. The same for a struct-valued copy and for an update that leaves the invariant's other fields in
   place.
3. `adversary5_an_else_naming_another_input_is_not_read_absent` and
   `adversary5_an_invariant_over_a_struct_member_copy_of_the_list_is_kept` keep passing (neither
   is ignored today: each passes by refusal by name, which this story makes the fail-closed
   result).
4. No case of `referenced_list_input` is ignored for this backlog item: the fail-closed rule
   changed none of their outcomes.

## Notes for whoever picks it up

- `stored_by` / `stored_value` hold the forms followed; every other `ResolvedPayloadValue` lands
  in `StoredRow::undecided`.
- The undecided read is the only reason such a scenario is refused; widening `stored_value` is the
  whole change.
