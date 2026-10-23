---
format: aep.planning-md/3
id: story:an-input-keeps-missing-and-null-apart
kind: story
status: draft
title: A command input keeps a missing member and an explicit null apart
summary: 'Three-state input: missing, null and a value, readable by guards and sets:'
refs:
- provider: github
  reference: beyond10x/ess#678
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---

- A served API accepts an input member that is missing and the same member sent as `null`, and the two mean different things (for example: missing keeps the stored value, `null` clears it).
- In ESS an `Optional<T>` input that declares no `presence:` admits both spellings as one absent value; one that declares `presence:` admits only one spelling. No input type keeps "missing" and "null" apart for a guard, a `sets:` mapping or a scenario.

## Expected

A way to declare an input whose three states (missing, `null`, a value) are distinct and can each be read by guards and `sets:` (for example a clear-to-absent mapping), with the published schema and conformance payloads following it.

## Status

Needs a design page before code: it adds a state to the input model and touches `sets:`, synthesis and the projections. A minimal reproduction from the reporter is requested.
