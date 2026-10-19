---
format: aep.planning-md/3
id: story:minted-head-refuses-only-its-branch
kind: story
status: implemented
title: Synthesis drops every branch of a command once one branch reads a head over a minted key
summary: A minted-head branch is refused alone; sibling branches keep their scenarios
owner: timo
tags:
- adopter-request
refs:
- provider: github
  reference: beyond10x/ess#552
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-19T21:15:11Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-19T21:15:11Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-19T21:15:12Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

A branch of a command whose sibling reads the head of an ordered selection keeps its scenario
whenever rows do not bear on it. A head-reading branch whose first ranking key the target mints
(`{generated: true}`) is refused alone, by name, with a help line toward writing the key from the
creating command's input or an authored scenario.

Closes https://github.com/beyond10x/ess/issues/552.

## Fit review

1. **Need.** A matching specification moved a picked row from the input to
   `{related: {entity, where, order_by: [element_score desc], ties: unordered, first}}`. Its
   `no-pair` branch, which reads no head, lost its scenario (ESS-SYNTH-001, "changes rows of the
   selector's entity in a way synthesis does not follow").
2. **Class: defect** in synthesis routing, plus a refusal wording gap.
3. **Mechanism (measured).** Any head read makes `row_set::uses(command)` true, and `routes()`
   sent every branch of the command through the row-set arrangement. Under a binding anywhere in
   the model, `walk` marks every row opaque, so a branch the rows never decide was refused. The
   same model reading the key from the input kept the branch.
4. **Fit.** No format change, no new code. Route a branch through the row-set arrangement only
   where rows bear on it (own row-set guard, a selected-row or head read, or a row-set guard the
   precedence plan answers before it).

## Decisions

- A branch rows do not bear on is witnessed by the ordinary families; `consistent()` passes it
  without reading rows.
- A minted first key refuses the head-reading branch with its own reason (`the target mints this
  key`); every unarranged-head reason carries a help line toward an authored scenario. Code stays
  ESS-SYNTH-001.
- Not taken: arranging the order under `ties: unordered` by creation order. A target that breaks
  no ties may answer any tied row, so creation order states nothing a scenario can hold it to.

## Acceptance

- `crates/verify/ess-conformance/tests/minted_head_siblings.rs`
  `a_branch_reading_no_head_keeps_its_scenario_beside_a_minted_head` (red before: the issue's
  message), `a_head_over_a_minted_key_is_refused_by_name_toward_an_authored_scenario` (red before:
  generic help), `a_row_set_guarded_branch_keeps_its_scenario_beside_a_minted_head`,
  `a_steered_key_keeps_both_branches`.
- `synthesis_suite_bytes_table` written before and after: identical for every repository model.

## Not done

- A row-set-guarded branch (`when_related: {entity, where, exists: false}`) in a model with any
  binding is still refused for opacity, with or without a head (`Rows.all_opaque =
  !ir.bindings().is_empty()`). This predates #543 and is not head-specific. Its own story.

## Implemented

Unit commit 6d167d757 on `impl/ess-542-u4`.
