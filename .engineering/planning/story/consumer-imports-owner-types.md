---
format: aep.planning-md/2
id: story:consumer-imports-owner-types
kind: story
status: draft
title: A consumer specification can reference an imported component's types
refs:
- provider: github
  reference: beyond10x/ess#162
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

Name another component's declared type through `composition.yaml` or a `requires:` entry, or assert that a local type conforms to an imported one.

## Acceptance

The #162 case validates with no local copy of the owner's type, or with a conformance assertion that refuses a field drift.
