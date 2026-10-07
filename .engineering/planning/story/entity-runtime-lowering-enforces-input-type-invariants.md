---
format: aep.planning-md/3
id: story:entity-runtime-lowering-enforces-input-type-invariants
kind: story
status: draft
title: Entity Runtime lowering enforces, or refuses by name, the invariants of an unstored input's type
relations:
- serves: vision:O2
revision: 1
---
## Finding

Measured by adversary pass 1 of wave 2026-10-07b
(`review-result:adversary-wave-20261007b-u1-pass-1`, F3), pre-existing on `main`: the Entity Runtime
lowering visits the invariants of entity identity and stored field types only
(`crates/generate/ess-entity-runtime/src/lib.rs:936`). A command input whose type carries
`invariants:` and is not stored has them dropped, so entity-core accepts an input ESS rejects.
Example: a `Label` newtype with `value.count <= 3`; ESS rejects `"abcd"`, the lowered definition
takes it.

## Acceptance

Each invariant of an unstored input's type either lowers to a precondition entity-core enforces, or
is refused by name with a lowering code; none is dropped. The adversary case
`adv_an_input_only_text_length_invariant_…` in
`crates/generate/ess-entity-runtime/tests/adversary_alphabet_count_pass1.rs` flips from asserting
today's behaviour to asserting the new one.
