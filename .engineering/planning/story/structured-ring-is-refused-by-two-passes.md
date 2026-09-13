---
format: aep.planning-md/1
id: story:structured-ring-is-refused-by-two-passes
kind: story
status: implemented
title: A ring closing through a struct or a union draws three diagnostics for one mistake
relations:
- serves: vision:O2
- informed_by: story:literal-representation-walk-exhaustion
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
revision: 6
---
## Finding

Measured by `aep-drive:adversary` pass 2 in ESS wave 22
(`review-result:adversary-wave22-unit1-pass-2`), against `wt-8fb8a43f6a81`.

`story:literal-representation-walk-exhaustion` adopted a rule for the representation walk: **a
mistake another pass already owns gets silence here.** It is applied to the ring answers —
`Uninhabited` is silent and left to `check_inhabitation`, `Cyclic` is refused. It is **not** applied
to `Structured`.

So a ring that closes through a struct or a union draws three diagnostics for one mistake:

```yaml
Alpha = newtype of Pick
Pick  = struct { only: Alpha }
```

```
- [self_reference] types.notifications.core.Alpha: no value of `notifications.core.Alpha` can exist …
- [self_reference] types.notifications.core.Pick:  no value of `notifications.core.Pick`  can exist …
- [type_mismatch]  binding.reject-on-refusal.mapping.reason: `…reason` is `notifications.core.Alpha`,
                   which has structure, and a literal in a binding is one piece of text …
```

`check_inhabitation` refuses both names; the walk meets the struct before the second `Alpha`,
answers `Structured`, and the literal check refuses again. The same shape holds through a union.
Mirrored on the payload consumer at `crates/specify/ess-domain/src/command.rs:2186`.

## Why this is not that story's defect

The double report predates it. At the base commit the `Structured` arm behaved exactly as it does
now; what changed is that the neighbouring arms adopted an ownership rule the `Structured` arm does
not share, which is what made the asymmetry visible. Classified `pre-existing` by the pass that
found it.

## What reaches it

Any document whose binding writes a literal into an input of a self-referential struct- or
union-backed type. No flag, no opt-in.

## Acceptance

One mistake draws one diagnostic. Either the `Structured` arm takes the same ownership test the ring
answers took — silence when the named type is uninhabited — or the taxonomy gains a fifth answer
that says so. Both consumers, and a red-first case for the struct shape and the union shape.

## The case that is already written

`crates/specify/ess-domain/tests/literal_representation_adversary_pass2.rs`, case
`a_ring_closing_through_a_struct_or_union_is_not_reported_by_both_passes`, asserts the wanted
behaviour and is **red**. It is rewritten in wave 22 to assert today's three diagnostics with a
message naming this story, so the gate stays readable and the case goes red the moment the gap
widens. Narrowing it back is the change this story makes.
