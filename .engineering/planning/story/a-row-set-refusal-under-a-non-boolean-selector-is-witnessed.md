---
format: aep.planning-md/3
id: story:a-row-set-refusal-under-a-non-boolean-selector-is-witnessed
kind: story
status: draft
title: A row-set refusal under a non-Boolean selector is witnessed
summary: 'Follow-up of story:a-row-set-command-whose-branches-address-different-rows-passes-its-own-suite: where an Integer or enum input selects the branch, no Boolean flip keeps the refusal''s row set empty and the scenario is refused by name (ESS-SYNTH-001); a send search over the selector would witness it (adversary pass 3, backlog)'
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

`story:a-row-set-command-whose-branches-address-different-rows-passes-its-own-suite` arranges,
beside a row-set refusal's scenario, the row of the accepting branch the send selects, and when
that row would fill the set the refusal needs empty it re-chooses the send by flipping one
`Boolean` the refusal does not read. Where the branches are selected by a non-`Boolean` input (an
`Integer` comparison such as `level >= 5` / `level < 5`, an enum), no flip exists and the scenario
is refused by name as ESS-SYNTH-001 ("no send keeps the row set empty"), though a send selecting
the other branch (`level >= 5`) witnesses it.

Before that story the same scenario was written and failed against the interpreter
(`outcome = unknown`); it is refused by name now, so nothing fails, but the refusal is coverage the
suite could have.

## Acceptance

1. `an_integer_selected_row_set_refusal_keeps_its_witness_or_is_refused` in
   `crates/verify/ess-conformance/tests/adversary_l5_pass1.rs` is tightened to require the
   `Act/refused` scenario, and passes on `--target interpreted`.
2. The send search chooses a value of a non-`Boolean` selector that selects a branch whose row
   does not fill the refusal's set, with the flipped send still selecting the branch the scenario
   is named for.
3. No suite-bytes row moves except for a model with such a selector.
