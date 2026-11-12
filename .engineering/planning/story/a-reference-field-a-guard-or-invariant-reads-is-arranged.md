---
format: aep.planning-md/3
id: story:a-reference-field-a-guard-or-invariant-reads-is-arranged
kind: story
status: draft
title: A reference field a guard or an invariant reads is arranged too
summary: 'Follow-up of story:accepted-input-arranges-referenced-list-rows: a references input a guard of the command or an invariant of the entity reads is still sent the witness identity, naming no created row; seven adversary passes showed re-deciding it needs the selection and invariant machinery to read the sent input (turn-3 split, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:accepted-input-arranges-referenced-list-rows` arranges the rows an input names, and sends the
empty list for an unrouted `many`, only where nothing reads the field
(`crates/verify/ess-conformance/src/synthesize/referenced_input.rs`, `read`). Where any guard of
the command (input guard, `when_subject:` in any form, `when_related:`, row set, external guard,
`before_row`) or any invariant of an entity the branch writes reads the field, including through a
copy of the input into another field, the suite sends the witness value unchanged, as before the
story. A target that resolves the references it stores refuses that scenario.

## Problem

- `sources.count > 0` as the accepting branch's own guard (the branch is skipped whole when the
  arranged list is decided again, so it is not re-decided today).
- A refusal beside the accepting branch comparing the list or an arranged identity
  (`sources.count < 3`, `primary == other`, ordering, pattern, quantifiers).
- An invariant over the list, a copy of it or a struct member of it (`sources.count <= limit`).
- A creating command whose guards or created entity's invariants read the reference it is given,
  for a row chained below the command.
- A stored-row guard reading the input (`SubjectField.predicate`, `SubjectPredicate.input`).

## Design rule

Decide the input again as it will be sent, with the arranged identities as values no other value
equals, and keep the arrangement only where every guard and invariant that reads the field still
selects the branch and holds. Seven adversary passes found a new shape of that re-decision each
time (a stored-row guard over the arranged list, a defaulted or struct copy, an `else:` naming
another input, an identity compared by order, a chained row swapping identities under a sibling
guard, a `SubjectField.predicate` never re-decided, `nodes_of` minting a fake UUID unlike the real
`id-1663200`); decide it in one place that evaluates the real branch over the real arranged values,
instead of a fail-closed approximation per shape.

## Acceptance

1. `adversary2_an_accepting_branch_guarded_on_the_list_count_names_arranged_rows`,
   `adversary2_two_relations_to_a_guarded_creator_name_distinct_arranged_rows` and
   `adversary3_invariants_over_the_list_decide_the_empty_list` lose their
   `#[ignore = "backlog: read-reference-arrangement"]` and pass with their assertions unchanged.
2. Every case that passes by "interpreter or refused by name" keeps passing.
3. No pinned byte table moves except for a model that reads a reference field it writes.

## Notes for whoever picks it up

- `read` is the single predicate that today keeps a field out of the arrangement; replacing it
  with a re-decision is the whole change.
