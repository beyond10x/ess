---
format: aep.planning-md/3
id: story:a-required-list-field-may-be-set-to-the-empty-list
kind: story
status: active
title: A required list field may be set to the empty list
summary: 'From ess/31 a sets: entry may store [] in a required List field, so a view publishing it can be filled (adopter row 16); 0.69.0.'
tags:
- adopter-report
- ess-0.69.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T21:27:56Z", actor: "human:timo", revision: 2, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-22T21:27:56Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
---
## Need

Reported by an adopter (tracker row 16, 2026-10-22): a required `List<…>` field that no creator
sets leaves every view publishing it unfillable, and the interpreted run reports those scenarios
unsupported. The adopter cannot set the field to an empty list, because a `sets:` entry has no way
to say `[]`.

## Change

From `format: ess/31` a `sets:` entry may store the empty list `[]` in a required `List<…>` field.
Earlier formats refuse it as before. Generated Rust and Go store an empty list; the interpreter
stores one and a view publishing the field is filled.

## Acceptance

- A specification at `ess/31` with `sets: { tags: [] }` on a creator validates and compiles; the
  interpreted view over the entity publishes `tags: []`.
- The same specification at `ess/30` is refused by validate, naming the format.
- Generated Rust and Go store an empty list for the field.
- An adversary pass (`*_empty_list_adversary.rs`) finds no refusal or mis-store left open.

Companion of `story:an-unfillable-required-field-is-named-by-validate-and-the-view`, which names
the field when nothing fills it.
