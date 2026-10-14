---
format: aep.planning-md/3
id: story:text-literal-partition-coverage
kind: story
status: draft
title: The coverage prover partitions a text fact by the literals its branches compare
revision: 2
---
## Context

At `ess/25` a create-or-update command may select its update branches with `when_subject:`
predicates over stored fields (beyond10x/ess#530). The coverage prover proves a stored-row
partition only over closed sets: the held state, enums, Booleans. A partition over a text field
written as complementary literal comparisons, `call_ref == ""` in one branch and
`call_ref != ""` in another, is refused as `non_exhaustive_branches` ("`call_ref` has no closed
set of values"), even though the two comparisons cover every value. Authors work around it by
leaving one branch unguarded.

## Outcome

The prover partitions a text fact by the literals the branches compare it with: each literal is
one class, and every other value is one more class. `== L` against `!= L`, or a set of `in:`
literals against its complement, is then proved exhaustive with no default branch.

## Acceptance

- `ess specify validate` accepts the #530 reproducer as filed at `ess/25`, the one whose branches
  guard `call_ref == ""` and `call_ref != ""` with no unguarded branch. Test: a case in
  `crates/specify/ess-domain/tests/subject_fact_create_or_update.rs` (an existing binary).
- A partition that leaves a literal class uncovered, for example only `call_ref == ""` with no
  default, is still refused as `non_exhaustive_branches`, naming the fact.
- The synthesized suite witnesses each literal class and the class of every other value.
