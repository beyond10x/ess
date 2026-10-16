---
format: aep.planning-md/3
id: story:state-param-binds-from-post-command-state
kind: story
status: draft
title: A view parameter compared with the lifecycle state is bound from the asserted state
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Outcome

A view parameter compared with the entity's lifecycle state (`state == param.p`) is bound by
synthesis from the state the generated scenario asserts after its command, so such a view no longer
blocks the outcome and transition scenarios that reach the entity.

## Evidence

Reported by an adopter on ess 0.59.0 and reproduced on 0.60.0: a view filtering
`group_id == param.group_id and state == param.state` makes synthesis refuse every outcome reaching
the entity with ESS-SYNTH-005 ("`param.state` is bound by nothing a scenario knows"); the same spec
without the state parameter synthesizes with 0 refusals. `bound()` in
`crates/verify/ess-conformance/src/synthesize.rs` binds parameters from string comparisons,
identities and settled fields, never from the lifecycle state, which its caller already knows. The
refusal's help text suggests filtering on the state, which the view already does.

## Acceptance

- `state_param_bound_from_asserted_state_on_create`
- `state_param_bound_after_transition`
- `state_param_view_excludes_row_in_other_state`

The help text of the remaining ESS-SYNTH-005 refusal no longer suggests what the view already does.
