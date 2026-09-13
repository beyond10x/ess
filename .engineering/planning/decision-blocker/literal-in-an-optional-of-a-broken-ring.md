---
format: aep.planning-md/1
id: decision-blocker:literal-in-an-optional-of-a-broken-ring
kind: decision-blocker
status: open
title: A literal into an Optional of a broken ring is refused by nobody, and the two stories disagree about whether it should be
relations:
- blocks: story:literal-representation-walk-exhaustion
revision: 1
---
## Finding

Measured by `aep-drive:implementor` while correcting
`story:structured-ring-is-refused-by-two-passes` in ESS wave 23, and deliberately **not** fixed
there. Recorded with the reproducer in the `Landing::Ring` doc of
`crates/specify/ess-domain/tests/literal_representation_adversary_pass2.rs`.

```yaml
Alpha = newtype of Optional<Beta>
Beta  = newtype of Gamma
Gamma = newtype of Beta
```

A literal written into an input typed `Alpha` draws two `self_reference` lines — about `Beta` and
about `Gamma` — and **nothing at the mapping site**. But `Alpha` is inhabited: its one value is
absence, which no literal spells. So the literal is refused by nobody.

**36 shapes of this structure** in the corrected 2744-shape matrix. It is the same class the wave-23
story just closed for the struct and union arm, in the **ring** arm, which belongs to
`story:literal-representation-walk-exhaustion`.

## Why it was not fixed in wave 23

Three reasons, and the second is the one that makes this a decision rather than a task:

1. it is the other story's arm;
2. **the fix would make the walk's rule identical to the mutant that story's own case
   `counting_optionals_outside_the_ring_would_move_shapes_the_matrix_asserts_on` exists to rule
   out** — so that case would become unsatisfiable and would have to be weakened to make this one
   pass;
3. the two stories genuinely disagree about whether a broken ring underneath a good type is one
   mistake or two.

The implementor confirmed the mechanism by direct measurement both ways before stopping. Satisfying
both cases as written is not possible.

## What a person has to decide

Which reading is right:

- **a broken ring is one mistake.** `check_inhabitation` speaks about `Beta` and `Gamma`, that is
  the mistake, and the literal into `Alpha` is a consequence not worth a second diagnostic. Then the
  36 shapes are correct as they stand and this story closes as a documentation change.
- **the literal is its own mistake.** `Alpha` has a value and the literal cannot spell it, so this
  pass owns it regardless of what is wrong underneath. Then the ring arm takes the
  `base_cases == 0` pairing, and
  `counting_optionals_outside_the_ring_would_move_shapes_the_matrix_asserts_on` is re-written to
  assert whatever the new rule makes true.

## Acceptance

The decision is recorded, one of the two readings is implemented, and the case that currently rules
out the other is re-written rather than weakened — with its new assertion naming this story.
