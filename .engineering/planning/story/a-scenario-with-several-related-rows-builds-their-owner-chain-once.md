---
format: aep.planning-md/3
id: story:a-scenario-with-several-related-rows-builds-their-owner-chain-once
kind: story
status: draft
title: A scenario with several related rows builds their owner chain once where the row-set condition allows
summary: The longest scenario of one adopter spec has 281 steps, the same owner chain built about 8 times; one shared chain would make it about 4 times shorter
tags:
- adopter-report
- from-finding
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- informed_by: story:synthesis-time-on-large-specifications-after-0-97-3
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
revision: 2
---
## Outcome

A scenario that needs several related rows builds the chain of the rows' owner once and puts the related rows
under it, where the row-set condition allows, instead of building the same owner chain once per related row.
Such scenarios are shorter, so the interpreted target, every reader of a suite and the synthesis that writes
it spend less on them.

## Problem

Found on an adopter's specification while measuring the cost of synthesis on 0.99.8 (the finding of
story:synthesis-time-on-large-specifications-after-0-97-3, which measures the cost of deciding such scenarios):
the longest scenario of that specification has 281 steps and the scenarios the 0.99.8 change added average 67
steps against 8.6 before. Read from the scenario, as a finding and not yet a measurement: it builds the same
chain of owner rows about 8 times, once per related row, about 34 steps each. Building the related rows under
one shared owner chain, where the row-set condition allows it, would make such a scenario about 4 times
shorter (the reporter's estimate).

The suite changes (fewer steps in those scenarios, same scenario ids, same witnessed outcomes); no format key
and no suite version moves.

## Acceptance

- `a_scenario_with_several_related_rows_builds_their_owner_chain_once`, in an existing synthesis target of
  `ess-conformance`, on a generic model with invented names (a leaf entity whose rows each need an owner row,
  itself created through a two-step chain, and a command whose row-set guard counts N related leaf rows under
  one owner): the scenario that arranges the N rows has the owner chain once, and its step count is at most
  half of what 0.99.8 writes for the same model (the test records both counts; the reporter's estimate is a
  quarter).
- `related_rows_whose_condition_needs_distinct_owners_keep_one_chain_each`: a model whose row-set condition
  needs the related rows under different owners (a count of owners, or a guard that compares the rows'
  owners) keeps one chain per row; its scenario bytes do not change.
- Every scenario id the model had is still written, every one passes `ess verify conform run --target
  interpreted`, and the refusals are the same.
- The suite-bytes pin targets of `AGENTS.md` (the repository-wide byte pins) move only in the rows of models
  that have such a scenario, each re-pinned with its cause named. No test runs over 60 s.

## Decisions

SS-D1, how far the shared chain reaches (recommended: A; no answer yet).

- A (recommended): share the owner chain only where the row-set condition holds with every counted row under
  the one owner (the condition does not distinguish owners); otherwise keep a chain per row.
- B: always share, and drop the case where rows must differ by owner. Loses witnesses of a condition that
  reads the owner.
- C: change nothing; accept the longer scenarios and the cost story's measures only.

## Fit review

1. Need: shorter scenarios for the same witnesses; a measured fact of one adopter's suite, shape described
   without its names.
2. Class: gap in the quality of synthesized suites (not a defect: the scenarios are correct, only long).
3. Expressible today: no; an author cannot shorten a synthesized scenario.
4. Fit: a synthesis arrangement change inside the existing row-set strategy; siblings are
   story:a-branch-the-row-set-strategy-prepares-is-prepared-once-per-model-branch-actors-and-arrangement-context-and
   and the arrangement stories; no authored surface.
5. Second adopter: any specification whose related rows each need an owner chain.
6. Cost: suite bytes of affected models change (no format key, no version), byte-pin rows re-pinned.
7. Alternatives: A, B and C above.

Decision class: accept as proposed (the reporter's proposal is a synthesis behaviour, not authored syntax).
