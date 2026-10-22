---
format: aep.planning-md/3
id: story:an-existence-outcome-passes-a-related-list-membership-guard
kind: story
status: draft
title: An existence outcome is synthesized beside a related list-membership guard
summary: A downstream specification's existing_instance outcome is refused with ESS-SYNTH-004 because the steered search finds no related row whose list holds the input chosen first; reproduced on 0.68.0 and the 0.69.0 build.
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

A downstream specification (public repository beyond10x/loom, `ess/domains/run.yaml:313`, commit 8cfac2cc, reported on ESS 0.67.0) declares `loom.run.SelectAction` with an `existing_instance: true` outcome (`selection-exists`) beside three `when_related` branches on `input.catalogue_id`: `catalogue-unknown` (`exists: false`), `not-in-catalogue` (`not: {exists: {in: entries, as: entry, that: entry.action == input.action}}`) and `revision-mismatch` (`case_revision != input.case_revision`). `ess specify validate` and `ess specify compile` exit 0, but synthesis refuses the `selection-exists` scenario with ESS-SYNTH-004. The `selected` scenario itself is synthesized: there the catalogue row is arranged first and the action is chosen from its `entries`. The existence scenario fixes the input first (the identity is sent twice), and no catalogue row is then searched whose `entries` hold that action, although `ProjectCatalogue/projected` sets `entries` and `case_revision` from its own input.

## Reproduction

Add to the downstream `run.yaml` an error `loom.run.SelectionExists {selection_id}` and, first under `SelectAction`'s `outcomes:`, the outcome `selection-exists` with `existing_instance: true`, that error and payload `selection_id: input.selection_id`. Then `ess verify conform synthesize --path ess`.

| release | result |
|---|---|
| 0.67.0 (downstream) | 29 scenarios, 1 refusal: "the route runs through `loom.run.SelectAction/selected`, which no input reaches" |
| 0.68.0 | 29 scenarios, 1 refusal (line below) |
| 0.69.0 integration build (commit 52bd110dd, which still prints `ess 0.68.0`) | identical to 0.68.0 |

```
refused: refusal[ESS-SYNTH-004]: outcome loom.run.SelectAction/selection-exists has no scenario `loom.run.SelectAction/outcome/selection-exists`
  it needs an instance of `loom.run.Selection` resting in `Selected`, and the route runs through `loom.run.SelectAction/selected`, sent here for an input chosen before the row of `loom.run.ActionCatalogue` it reads, and no row the declared commands create lets that input pass over `loom.run.SelectAction/catalogue-unknown`, `loom.run.SelectAction/not-in-catalogue` and `loom.run.SelectAction/revision-mismatch`
```

Probe on 0.69.0 (verified): with `not-in-catalogue` removed, synthesis writes 29 scenarios and 0 refusals; with `revision-mismatch` removed instead, the refusal stays and names `catalogue-unknown` and `not-in-catalogue`. The list-membership guard is what the steered search cannot pass; the scalar `case_revision` comparison is not.

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/existence.rs:1013` (`stored_first`) creates the row for the input already chosen; on failure `:1049` (`steered`) retries through `related_guard::drive_toward`, and `:1074` maps its failure to `Unreachable::Unsteered`, which prints the line above (`synthesize.rs:2396`).
- `crates/verify/ess-conformance/src/synthesize/related_guard.rs:1484` (`drive_toward`) and `:1690` (`searched`): the search for a related row toward a fixed input. Inferred, not traced: it does not construct a list element (`entries[*].action`) equal to the fixed `input.action`.

Not the cause of `story:seeded-state-reaches-related-rows`: that chain needs a related row in a lifecycle state (`Compiled`) reached by a walk that an empty list witness breaks, and surfaces as ESS-SYNTH-003 in `unsatisfied` with an `Unwitnessable` cascade. Here the related row needs no state, the list witness works (the `selected` scenario carries matching `entries`), and the refusal comes from the existence family's steered search.

## Acceptance (provisional)

- The refused-selection scenario `loom.run.SelectAction/outcome/selection-exists` is synthesized for the specification above and executes: it arranges an `ActionCatalogue` row whose `entries` contain the action of the input sent twice, and its `case_revision` matches that input.
- A conformance scenario in this repository holds a minimal specification of the same shape (existence outcome beside a `when_related` list-membership guard over a field the related creator sets from its input) to that synthesis.
- Where no creator can set the list field from its input, the refusal still names the guard it cannot pass.
