---
format: aep.planning-md/3
id: story:a-row-set-branch-arranges-rows-past-the-creators-predicate-guard
kind: story
status: implemented
title: A row-set branch arranges rows past the creator's predicate guard
summary: A row-set branch whose rows come from a creator guarded by a related predicate is refused with ESS-SYNTH-001 naming the wrong guard (adopter report 2026-10-25, ess 0.69.2).
tags:
- adopter-report
- defect
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-row-set-branch-arranges-rows-past-the-creators-guard
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:50Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:50Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-26T12:32:46Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Problem

An adopter's read command selects one child row by two inputs through a row-set guard
(`when_related: {entity, where: {all: [parent_id == input.parent_id, child_id == input.id]}, exists:
false}`). The command creating those rows refuses through two `when_related:` guards: `{via:
input.parent_id, exists: false}` and a second one with a `predicate:` over the parent row (a tenant
check, `tenant != input.tenant`). On ess 0.69.2 synthesis refuses the read command's `found` branch
with ESS-SYNTH-001 "no arrangement left a related row", and the refusal names the creator's
`exists: false` guard, which is not the guard that blocked (adopter report, 2026-10-25; reproduced
on 0.69.2).

0.69.0 taught a row-set branch to arrange rows past the creator's `exists: false` guard
(`story:a-row-set-branch-arranges-rows-past-the-creators-guard`, beyond10x/ess#682). Its retry
builds the related row with no inputs (`crates/verify/ess-conformance/src/synthesize/row_set.rs`,
`linked`, about :3410-3450), so a predicate guard over that row refuses the creation, and
`related_link` (about :3378-3388) matches only the absent guard. Without the predicate guard the
same model synthesizes with 0 refusals.

## Fit review

1. **Need.** A row-set branch whose rows are created by a command guarded by a related predicate
   must be witnessed; today it is refused, and the refusal blames the wrong guard. Minimal
   reproduction: one parent entity with an optional tenant, one child entity, a creator with
   `exists: false` plus a predicate guard over the parent, and a reader selecting a child by two
   inputs. No syntax proposed.
2. **Class.** Defect: the 0.69.0 fix states that the row-set branch is arranged past the creator's
   guards, and the predicate guard is one of them.
3. **Already expressible?** Yes, the specification is valid; only synthesis fails.
4. **Fit.** No authored surface. The arrangement satisfies the creator's predicate guard the way a
   subject's creator is steered (`story:a-literal-set-field-steers-the-creator-of-a-subject-row`).
5. **Second adopter.** A booking may be read by room and slot, and the booking's creator refuses a
   room of another site.
6. **Cost.** No format, no suite major for specifications that synthesize today; the read
   command's `found` scenario appears where it was refused.
7. **Alternatives.** (a) Change nothing: adopters keep a known-failing marker. (b) Drop the
   predicate guard from the creator: loses the rule. (c) Arrange the related row so the
   predicate passes (chosen).

## Decisions

- Accept: a defect in the 0.69.0 fix. The arrangement fills the related row and the creator's
  inputs so every `when_related:` guard of the creator passes; where it still cannot, the refusal
  names the guard that refused.

## Acceptance

- `crates/verify/ess-conformance/tests/row_sets.rs`:
  `a_row_set_branch_arranges_rows_past_the_creators_predicate_guard`: on the minimal reproduction
  (creator with `exists: false` and `predicate: tenant != input.tenant` over the parent),
  synthesis gives the reader's `found` scenario with no ESS-SYNTH-001, and the suite passes on the
  interpreted target.
- Same file: `an_unreachable_row_set_branch_names_the_guard_that_refused`: where no arrangement
  passes the predicate guard, the ESS-SYNTH-001 message names the predicate guard, not the
  `exists: false` one.
