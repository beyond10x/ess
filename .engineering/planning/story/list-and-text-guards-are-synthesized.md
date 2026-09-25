---
format: aep.planning-md/2
id: story:list-and-text-guards-are-synthesized
kind: story
status: active
title: List and text-ordering input guards get synthesized scenarios
relations:
- serves: vision:O2
revision: 3
---
## Outcome

A command-input guard over a list (`.count`, `exists`, `forall`) or a text ordering (`caller < "m"`)
that passes validate also gets synthesized scenarios for both of its branches.

## Why

GitHub issue beyond10x/ess#94 (`ess` 0.32.0; same class as #74). Validate accepts
`tags.count > 0`, `exists: {in: tags, as: t, that: t == vip}`, `forall: {…}` and `caller < "m"`;
synthesize refuses each with `ESS-SYNTH-002`, because the witness builder makes every list `[]`
(`witness.rs:39`), the input flattener publishes no count or element facts for input lists, and
text has no declared scale.

## Decision (operator default, 2026-09-25)

**Synthesize**, not refuse: a one-element list built from the guard's own literal (`[vip]`
satisfies, `[]` and a list of the path's own text refute), `.count` published for input lists the
way `FactSource::cardinality` does for observed ones. Text ordering is **byte-wise lexicographic**,
identical in the Rust, Go and TypeScript evaluator lanes.

## Acceptance

- Red first: the four guards in the issue's table each get a satisfying and a refuting scenario.
- A case per lane (Rust, Go, TypeScript) asserts the same order for a pair of texts whose byte
  order differs from a locale order (e.g. `"B" < "a"`).
- The witness module doc states the list and text rules.
