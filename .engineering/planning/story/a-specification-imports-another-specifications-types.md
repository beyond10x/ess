---
format: aep.planning-md/3
id: story:a-specification-imports-another-specifications-types
kind: story
status: draft
title: A specification imports another specifications types
summary: 'Adopter report N9: No way to import types across specifications (ess#663).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#663
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Problem

A specification cannot reference another specification's types: `system.yaml` has no import key and `ess-inputs` refuses files outside the root. The designed path is a local copy proved with `compose conformances:` (`docs/design/composition-type-conformance.md`). Adopter report N9, ess 0.68.0. Blocked on a new decision record: whether a source may import types by pinned digest, and how ownership (ESS-DOMAIN-004), `source_digest` and generated code treat them.

## Acceptance

Written after the decision record is accepted. Scenario names reserved:

- `an_imported_type_resolves_by_pinned_digest`
- `an_imported_type_changed_upstream_is_refused_until_repinned`
- `generated_code_references_the_imported_type_once`

## Further reports

2026-10-26, ess 0.69.2: two more downstream specifications hold another system's versioned document type as `Json` (a sibling system's agent resource document) or declare its types field for field and convert one to one, for the same reason.
