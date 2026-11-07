---
format: aep.planning-md/3
id: decision-blocker:typescript-types-target-runtime-decoder
kind: decision-blocker
status: cleared
title: Does the TypeScript types target gain a runtime decoder, or refuse constrained types?
relations:
- blocks: story:generated-decoders-enforce-declared-value-constraints
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-11-07T18:38:08Z", actor: "human:timo", revision: 3}
---
# Does the TypeScript types target gain a runtime decoder, or refuse constrained types?

story:generated-decoders-enforce-declared-value-constraints requires every generated decoder to
enforce each constraint its type publishes (`x-ess-alphabet`, a length, a bound, a prefix, an
invariant the predicate language evaluates), or the generator to refuse the type by name. The Rust
half shipped; the generated Go server decoder is wave 1 of the 0.91.0 plan. The TypeScript types
target emits type declarations only: there is no decoder in it to hold a constraint.

## Options

| option | what it does | cost |
|---|---|---|
| A | The TypeScript types target gains a runtime decoder per type (`decode<Type>(value): Result`) that enforces every published constraint and names the path, agreeing with the Go decoder (`go_and_typescript_decoders_agree`) | new generated runtime code in the TypeScript target, byte-pinned output moves, adopters gain an API; size L |
| B | The TypeScript types target refuses, by name, every type that publishes a constraint it cannot hold in a declaration | generation fails for adopters whose types carry an alphabet, prefix or invariant until they drop the target or the constraint; size S, breaking |
| C | Declarations only, with each unenforced constraint listed in the target's coverage report as an obligation | contradicts the story's "never left to the application"; size S |

## Recommendation

A: a declaration target that refuses most real types (B) is worse than no target, and C leaves the
promise the story exists to keep unkept. A also makes the Go/TypeScript agreement test meaningful.

## What waits

Wave 3 of the 0.91.0 plan (decoders part B). Waves 1 and 2 proceed without it.

## Decision

2026-11-07: option A. The TypeScript types target gains a runtime decoder per type that
enforces every published constraint and names the path, agreeing with the Go decoder. Additive;
it is decoders part B, wave 3 of the 0.91.0 plan.
