---
format: aep.planning-md/3
id: story:synthesis-reaches-a-creator-beside-existing-instance-and-related-guards
kind: story
status: draft
title: Synthesis reaches a creating branch behind an existing_instance refusal and related guards
relations:
- serves: vision:O2
revision: 1
---
## Outcome

`ess verify conform synthesize` witnesses the creating branch of a command that declares an
`existing_instance: true` refusal ahead of `when_related` refusals over the same input reference,
instead of refusing the refusal outcome with ESS-SYNTH-004 because the creating branch is unreached.

## Context

Reported by the loom repository on ESS 0.67.0 (format `ess/29`). Its `loom.run.SelectAction`
creates a `loom.run.Selection` (`instance: selection_id`) and declares, in this order:

1. `selection-exists`: `existing_instance: true`, error `loom.run.SelectionExists`;
2. `catalogue-unknown`: `when_related: {via: input.catalogue_id, exists: false}`;
3. `not-in-catalogue`: `when_related` with a `not exists … in: entries` predicate over the catalogue;
4. `revision-mismatch`: `when_related` with `case_revision != input.case_revision`;
5. `selected`: `creates: loom.run.Selection`.

Without outcome 1 the specification synthesizes. With it, `ess specify validate --strict-requires`
exits 0 and synthesis gives 29 scenarios and 1 refusal:

```
refusal[ESS-SYNTH-004]: outcome loom.run.SelectAction/selection-exists has no scenario `loom.run.SelectAction/outcome/selection-exists`
  it needs an instance of `loom.run.Selection` resting in `Selected`, and the route runs through `loom.run.SelectAction/selected`, which no input reaches
```

The same `existing_instance: true` shape on `loom.run.ProjectCatalogue`, which has no
`when_related` refusal, synthesizes. So the reach of `selected` is lost only when an
`existing_instance` branch sits beside related guards. Inferred from the two commands, not yet
traced in `crates/verify/ess-conformance`. Related: story
`synthesis-covers-input-guards-beside-an-existing-instance-branch` (#479), the same pairing with input
guards.

## Acceptance

- A fixture of the five-outcome shape above synthesizes
  `<command>/outcome/selection-exists` and `<command>/outcome/selected`, with 0 refusals.
- The `selection-exists` scenario first creates the row through `selected`, its input satisfying
  every `when_related` guard, then repeats the id.
- Synthesis of every existing fixture keeps its bytes; the pinned tables move only by the new
  fixture's rows.

## Out of Scope

Changing the format; loom's specification.

## Ambiguities

None.

## Open Questions

None.
