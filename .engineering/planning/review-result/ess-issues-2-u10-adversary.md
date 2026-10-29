---
format: aep.planning-md/3
id: review-result:ess-issues-2-u10-adversary
kind: review-result
status: active
title: Adversary pass, open-issue batch 2 unit U10 (unreachable-branch refusal)
summary: nothing found
tags:
- ess-issues-2
relations:
- reviews: story:validate-refuses-a-branch-implied-by-an-earlier-input-refusal
revision: 1
---
## Scope

One adversary pass over unit U10 of open-issue batch 2 (branch `unit/ess-issues-2-u10`, change
fc3f6821a: #702). Cases in `ess-domain/tests/adversary_ess_issues_2_u10.rs`, committed as
3114bc23b.

## Findings

No false refusal found. Seven cases, each with a refused counterpart: an accepting branch declared
above its refusal, `any`/`not`/`any_of`/`!=`, struct members, an Optional input with `defined`, an
external `before_row` refusal, two refusals with equal guards, a row-set command. The wording
matches the story and the ESS-SYNTH-003 text for the same model.

Notes kept, not fixed:
- Validation stays silent where synthesis still reports, outside the story's single-refusal,
  `when:`-only scope: two refusals that together cover the later branch, an accepting
  `when_subject:` plus `when:` branch, an `external:` plus `when:` branch.
- Five adapted conformance tests (`adversary_refstate_pass1`, `adversary_refstate_pass2`, two in
  `mutation_identical_answer`, `witness_directed_search`) keep a dead branch that validation misses
  only because the prover declines; they break if the prover improves.

## Result

At 3114bc23b: 7 adversary cases green, with `implied_input_guard` 6, `adversary_refstate_pass1` 5
and `refusal_beside_state` 9.
