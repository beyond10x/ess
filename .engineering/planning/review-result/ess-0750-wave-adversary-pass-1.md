---
format: aep.planning-md/3
id: review-result:ess-0750-wave-adversary-pass-1
kind: review-result
status: active
title: The 0.75.0 wave adversary pass 1
summary: 'NEEDS-CHANGE then fixed: F1, F2 and J1 fixed in 5d715c4fd; J2 accepted'
relations:
- reviews: story:a-seed-says-it-does-not-serve-a-set-move-filter
- reviews: story:the-workspace-compiles-on-its-declared-rust-version
- reviews: story:every-set-move-only-refusal-names-the-state
- reviews: story:a-seeded-scenario-notes-its-skipped-reversed-caller-run
revision: 1
---
## Scope

One adversary pass over integrate/ess-0750 at b789ab489: the four unit merges (U4 e862b8570, U3
d7f2e8485, U1 7d6a83a97, U2 e7fed52c2) and the integration re-pin.

## Findings

| id | unit | severity | test | outcome |
|---|---|---|---|---|
| F1 | U1 | medium | `an_unwritten_filter_refusal_names_the_seed_row_of_its_entity` | fixed in 5d715c4fd: the filtered entity is resolved from the field path for the unwritten-filter refusal |
| F2 | U4 | low | `a_seeded_single_caller_scenario_is_not_noted_as_a_mixed_caller_witness` | fixed in 5d715c4fd: `CrossCallerUnswapped` carries `mixed`, and a single-caller seeded scenario is worded as running once |
| J1 | U1 | low | `a_seed_applied_elsewhere_is_refused_only_for_this_filter` | fixed in 5d715c4fd: "does not apply to this filter" |
| J2 | U1 | low | none | accepted: a seeded refusal keeps the cause text whole but replaces its help with the seed help, which points to the run without seeds |

Pinned and passing: P1 arrives-only entry names two seed rows in order; P2 an undriven entry and an
instances refusal both name the seed row; P3 a seed applied to a subject read stays applied; P4 the
seeded note is deterministic.

## Result

`adversary_0750_pass1`: 4 passed, 2 failed at b789ab489; 7 passed at 5d715c4fd. Repros unchanged
(r6 four variants 10/0, r3a-tel1 2/1, r5e 9/1, r8 seeded 2/1 naming the seed).
