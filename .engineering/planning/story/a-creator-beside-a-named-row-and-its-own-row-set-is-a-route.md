---
format: aep.planning-md/3
id: story:a-creator-beside-a-named-row-and-its-own-row-set-is-a-route
kind: story
status: draft
title: A creator beside a named-row refusal and a row-set refusal over its own rows is a route
summary: Creating branch below a via related refusal and a uniqueness row-set refusal is never driven, so every later command on the created entity is refused (weft catalog).
tags:
- defect
relations:
- decomposes: epic:adopter-gaps-synthesis
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/a_row_set_branch_arranges_rows_past_the_creators_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest
revision: 7
---
## Problem

A creating command guarded both by a related row its input names (`when_related: {via:
input.<field>, exists: false}`) and by a row-set refusal over the entity it creates (a uniqueness
check, `when_related: {entity: <created>, where: name == input.name, exists: true}`) cannot be
used to arrange the row it creates. Every later command that needs such a row is refused
ESS-SYNTH-004 "the route runs through `<creator>/<created>`, which no input reaches", and the
row-set refusal itself is refused ESS-SYNTH-001 "its own row set is False".

Consumer: weft (`~/beyond10x/weft/ess/7-services/catalog.yaml`, `PublishOffering`), where this
leaves about 60 % of the outcomes in accounts, formations, catalog, scheduling and nodes without a
scenario.

Reproduction (scratch, outside the repository): the gap9-catalog probe of 2026-11-26, the model
`c.route.yaml` (86 lines, synthesis under 1 s). `RegisterEngine` creates an `Engine`;
`PublishOffering(name, engine_id)` answers `no-such-engine` (`via: input.engine_id, exists:
false`), `name-taken` (row set over `Offering` by `name`), else `published` (creates `Offering`);
`RetireOffering` moves an offering. ess 0.99.2: `4 scenario(s), 4 refusal(s)`: `name-taken`
(ESS-SYNTH-001), `RetireOffering/retired`, `Offering/transition/retire/...` and
`Offering/state/Retired/refuses/...` (ESS-SYNTH-004). Without `no-such-engine`: 7 scenarios, 0
refusals. Without `name-taken`: no refusal on the route either.

## Cause

`related_guard::selects` (`crates/verify/ess-conformance/src/synthesize/related_guard.rs`), asked
which branch a present related row selects for an input, skips no `RelatedSet` branch and reads no
guard of it, so a row-set refusal with no input guard is counted as selected for every input. With
the `Engine` row present it returns `name-taken`, never `published`, so `related_guard::drive`
finds no input for the creating branch, and `created_owned_made` reports it as unwitnessable.

## Fix

A row-set refusal is not this row's to decide. `selects` reads it over no selected rows, which is
what a run that arranges rows of its own leaves: a refusal its test refutes there (`exists: true`,
a count no empty set meets) is passed over, one that holds there (`exists: false`) still answers.
`row_set::contradictions` (the walk of every finished scenario) checks the rows the steps leave.

The first version passed every row-set refusal over. On weft's accounts subset it synthesized
scenarios its own interpreter failed (integrator, 2026-11-26), from two causes:

- `exists: false` refusals (`CommitRevision/empty`) were passed over too, so a run arranging a
  revision for a filtered view sent `committed` on a formation with no component. The one-time
  disclosure scenarios carrying that run are built after the row-set walk and failed.
- Scenarios the fix unblocked (`AddToBorder/no-such-formation` and nine more in formations) met an
  older defect: `related_guard::overlaps` sent every later missing row beside an `exists: false`
  refusal, the addressed subject's row included. Under the one rule (`ess/27`) the addressed row's
  existence is read first, and a command with no `unknown_instance:` answers an identity no row
  carries with no declared outcome. The addressed row is now never sent missing there. On main
  the same scenario fails wherever the component can be arranged (the model without `name-taken`).

A second pass on the same subset left two more causes:

- An `exists: false` branch over the row the command addresses (`RemoveFromBorder/no-such-component`,
  `via: input.component_id` on an `updates:` of that component) is never reached under the one
  rule, for the same reason. Beside a row set it was arranged by the row-set family with the
  component present and failed. It is now refused before any family arranges it
  (`related_guard::absent_addressed`), as `AddToBorder/no-such-component` already was (then
  ESS-SYNTH-008, now ESS-SYNTH-001 with its own cause and help).
- The one-time disclosure scenarios (`disclosure::augment`) are built after the row-set walk, so
  a run they carry that a row set answers otherwise was never checked
  (`RevokeTenantOwner/wrong-state`, a row set over the addressed binding by its identity). The
  walk now runs again after them. This one fails on main as well.

## Acceptance

In `crates/verify/ess-conformance/tests/a_row_set_branch_arranges_rows_past_the_creators_guard.rs`,
on the inline model `CATALOG` (the reproduction):

- `a_creator_beside_a_named_row_and_its_own_row_set_is_a_route_for_every_later_command`: synthesis
  records no refusal, and `PublishOffering/outcome/{no-such-engine,name-taken,published}`,
  `RetireOffering/outcome/retired`, `Offering/transition/retire/by/RetireOffering/retired` and
  `Offering/state/Retired/refuses/RetireOffering` are synthesized and pass the interpreted target.
- `the_name_taken_refusal_is_sent_the_name_an_offering_was_published_under`: the `name-taken`
  scenario publishes an offering under the name its last send repeats.
- On `FORMATIONS`: `every_scenario_through_the_fixed_creator_passes_its_own_interpreter` (every
  synthesized scenario passes, `AddToBorder/outcome/no-such-formation` included) and
  `no_send_names_a_component_no_row_carries`.
- On `REVISIONS`: `a_row_set_refusal_that_holds_over_no_rows_still_answers_an_arranging_run`: no
  refusal, and every scenario, the one-time disclosure ones included, passes.
- On `FORMATIONS`: `an_absent_branch_over_the_addressed_row_is_refused_as_never_reached`, for
  `AddToBorder/no-such-component` and `RemoveFromBorder/no-such-component`.
- On `BINDINGS`: `a_disclosure_scenario_is_held_to_the_row_sets_its_sends_read`: every scenario
  passes the interpreted target.

## Result

Built `ess` (release) before and after, on the reproduction and on weft's catalog with its probe's
stub neighbours (`--jobs 4`), each suite then run with `--target interpreted`:

| model | before | after |
|---|---|---|
| reproduction (`c.route.yaml`) | 4 scenarios, 4 refusals | 8 scenarios, 0 refusals, 8 passed |
| weft catalog (`catalog.yaml` + stubs) | 47 scenarios, 180 refusals, 47 passed | 76 scenarios, 151 refusals, 76 passed |

`synthesis_suite_bytes_table`'s slow lane (`every_repository_model_keeps_its_synthesized_bytes`,
`every_repository_component_keeps_its_synthesized_bytes`) passes: no repository model's suite
moves. Only `tests/fixtures/synthesis-sources.digest` is re-stamped.

## Not covered

- `story:a-creator-under-a-row-set-and-when-is-arranged` (active, no branch or pull request on
  2026-11-26) has the same symptom from another shape: the creating branch carries the row set
  itself (`exists: false`, with a `when:`), no related row. Its cause is the plain pass of
  `arrange_toward_with`, not `related_guard::selects`.
- The probe's `ess/c.yaml` writes the related guard as `{entity: Engine, where: engine_id ==
  input.engine_id, exists: false}`, a row set over the related entity's generated identity. That
  form is refused with or without `name-taken` (ESS-SYNTH-001, "let a command that creates the
  entity write each field the filter reads from an input"), so it is a different limit, not this
  defect; weft writes the `via:` form.
- A creator of `Device` beside a row set over `Device` (`known`) and a `via:` guard over an
  optional self-reference (`parent_device_id`), the integrator's probe `probe-inv/q` of
  2026-11-26: 2 scenarios, 5 refusals before and after (`ReportDevice/reported` ESS-SYNTH-003:
  a parent row is arranged where the absent optional reference should be left out). Another cause.
- weft's `PublishOffering/engine-wrong-kind` (a `not exists k in kinds` predicate) stays refused
  ESS-SYNTH-003; another cause.
