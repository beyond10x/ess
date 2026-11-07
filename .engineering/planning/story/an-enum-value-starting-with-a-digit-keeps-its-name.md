---
format: aep.planning-md/3
id: story:an-enum-value-starting-with-a-digit-keeps-its-name
kind: story
status: draft
title: An enum value that starts with a digit keeps its name in Rust and has a TypeScript value object
summary: 'spindle wave 12: 2xx..5xx become V0..V3 and get no TS value object (ess 0.88.0)'
tags:
- backlog
- from-consumer
- queue-0.91.0
- rank-5
relations:
- serves: vision:O2
revision: 1
---
## Need

Reported by spindle wave 12 against ess 0.88.0. An enum whose values start with a digit (`2xx`,
`3xx`, `4xx`, `5xx`) is generated as the Rust variants `V0`..`V3`, and the TypeScript target emits
no value object for it. Example: spindle `ui/src/api/generated/types.ts:254`. The UI keeps its own
copy of the values (`ui/src/api/shapes.ts`).

## Acceptance

- The Rust variant of a value that starts with a digit keeps the value in its name, for example
  `S2xx` or `Status2xx`, by a rule the reference page states. Its serde rename is the value.
- The TypeScript target emits the value object for such an enum, as it does for any other enum.
- Generated output for enums whose values do not start with a digit is byte-identical.

## Scope

- inferred: `crates/generate/ess-gen/src/` (Rust and TypeScript type targets, identifier
  mangling).
