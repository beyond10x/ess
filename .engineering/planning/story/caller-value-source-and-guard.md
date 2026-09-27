---
format: aep.planning-md/2
id: story:caller-value-source-and-guard
kind: story
status: draft
title: The authenticated caller is a value source and a guard operand
refs:
- provider: github
  reference: beyond10x/ess#168
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

Actor fields, a `{caller: <field>}` source, and guards comparing a caller field with an input or a stored field. Needs the conformance adapter to supply the caller.

## Acceptance

The #168 repro validates; scenarios run as two callers and require the 403 for the one that is not the record's agent.
