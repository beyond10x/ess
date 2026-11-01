---
format: aep.planning-md/3
id: story:a-refusal-witness-compares-its-trigger-identities
kind: story
status: draft
title: Binding refusal witnesses compare their trigger event's identities
relations:
- serves: vision:O2
revision: 1
---
## Finding

Out of scope in wave D357 U4 (`review-result:adversary-d357-u4-pass-1`, residue): binding refusal scenarios produced by the refusal-policy witness site (`crates/verify/ess-conformance/src/synthesize/refusal_policy.rs`, `witness`, near :225) still emit a shape-only event step, which may swap in another publisher or an alternative trigger whose branch the site does not carry. The widened sweep in `crates/verify/ess-conformance/tests/adversary_273_pass1.rs` pins the seven input identities left uncompared there in `refusal_witness_residue()`.

## Acceptance

- `<binding>/binding/refusal/<outcome>` scenarios compare the identity fields of the trigger event they expect, as the other binding scenarios do since `story:binding-site-identity-comparisons`, including where the witness swaps the publisher or the trigger.
- `refusal_witness_residue()` is empty and removed; the sweep asserts no binding scenario leaves an identity uncompared.
- A target that publishes a wrong id in such a trigger event fails at least one scenario; the correct interpreted target passes all of them.
