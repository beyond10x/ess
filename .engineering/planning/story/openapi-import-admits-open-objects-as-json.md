---
format: aep.planning-md/3
id: story:openapi-import-admits-open-objects-as-json
kind: story
status: proposed
title: Openapi import admits open objects as json
summary: 'Adopter report R9: OpenAPI import refuses open objects and nullable anyOf (ess#647).'
tags:
- ess-0.70.0
refs:
- provider: github
  reference: beyond10x/ess#647
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T13:39:56Z", actor: "human:timo", revision: 4, correlation: "ess-0690"}
---
## Problem

`ess infra import openapi` refuses open objects (`additionalProperties: true` or absent), empty schemas (`{}`), properties without a `type`, and nullable `anyOf: [{$ref: X}, {type: 'null'}]`. A protocol whose openness is its forward-compatibility rule cannot be imported, and adopters write converters by hand. Adopter report R9, ess 0.68.0, plus a second adopter with the same refusal classes. Blocked on the proposed decision record `ess-evolution-08-reviewable-source-candidates`, which keeps the refusals.

## Acceptance

- `an_open_object_imports_as_a_struct_that_ignores_undeclared_fields`: `{type: object, properties: {...}}` without `additionalProperties: false` imports with `undeclared_fields: ignored`; the import report names the choice.
- `an_empty_schema_imports_as_json`: `{}` and `{type: object, additionalProperties: true}` import as `Json`.
- `a_nullable_any_of_imports_as_optional`: `anyOf: [{$ref: X}, {type: 'null'}]` imports as `Optional<X>`.
- `an_untagged_object_one_of_stays_refused`: an untagged `oneOf` of objects keeps its refusal by name.
- `a_required_property_not_declared_is_reported_not_refused`: a `required` entry with no property is a coverage gap in the report, not a refusal of the whole document.

## Decisions (2026-10-23)

- Decided on the operator's morning queue (row 33, report rows 44 and 45), option A: the import refusals become one unit in 0.70.0, after 0.69.0 ships. They are `additionalProperties: true` (or absent) and `{}` imported as `Json` or as a struct that ignores undeclared fields, and a nullable `oneOf`/`anyOf` of one schema and `{type: 'null'}` imported as `Optional<X>`. An untagged `oneOf` of objects stays refused by name.
- The acceptance above stands. `a_nullable_any_of_imports_as_optional` also covers the `oneOf` spelling.
- Out of 0.69.0. It does not wait on `ess-evolution-08-reviewable-source-candidates` for these constructs.
