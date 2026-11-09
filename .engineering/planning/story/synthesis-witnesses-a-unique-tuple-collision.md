---
format: aep.planning-md/3
id: story:synthesis-witnesses-a-unique-tuple-collision
kind: story
status: draft
title: Synthesis witnesses a unique tuple collision, a caller member with two callers
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#638
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-identity-may-be-a-composite-of-fields
revision: 2
---
## Need

Once an entity may declare `unique: [fields]` answered by `existing_instance:`
(`story:an-identity-may-be-a-composite-of-fields`), the suite must show both sides of the tuple:

- a second create with the same tuple is refused, with no event and the first row unchanged;
- two creates that differ in one member are both accepted.

Where a member is written from the caller, the second side is two callers sending the same input.
The interpreted target must enforce the tuple. Today the duplicate witness exists only for an
identity taken from the input (`docs/design/cross-record-and-stored-field-guards.md:596-617`).

Minimal model: `example.g.Grant {grant_id (generated), account_id, app_id}` with
`unique: [account_id, app_id]`, and `example.g.Bind(app_id)` with
`bound: creates, sets {account_id: {caller: account_id}, app_id: input.app_id}` and
`taken: existing_instance: true, error: example.g.AlreadyBound`.

## Acceptance

- `ess verify conform synthesize` on the minimal model reports 0 refusals and writes
  `example.g.Bind/outcome/taken`. That scenario binds `app_id` once, binds it again as the same
  caller, expects `example.g.AlreadyBound` with no event, and expects the view to hold one row.
- `example.g.Bind/outcome/bound` binds one `app_id` as the first caller and then as the second, and
  expects two rows. Its swapped half runs the same with the roles swapped.
- `ess verify conform run --target interpreted` passes both scenarios. An interpreter that checks
  `app_id` alone fails `bound`, and one that checks nothing fails `taken`.
- The same model with both members from `input.` synthesizes the same pair, with one caller.

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize/existence.rs
- inferred: crates/verify/ess-conformance/src/synthesize/caller.rs
- inferred: crates/verify/ess-conformance/src/interpret/execute/existence.rs
- inferred: crates/verify/ess-conformance/tests/unique_tuple.rs
confidence: low

## Fit review

1. **Need.** A declared uniqueness must be checked from both sides. Reproduction: the parent story's
   model, which does not validate today (triage run, `as-written`).
2. **Class.** Gap, following the parent story.
3. **Already expressible?** No. A per-command row-set refusal is the nearest form, and no later
   command can arrange a row through it (triage run `tuple-idiom-input`, ESS-SYNTH-001).
4. **Fit.** The record names this witness shape: "a duplicate tuple on create is witnessed as a
   refusal with no event and the first row unchanged; two rows that differ in one member are both
   created" (`architecture-decision-record:a-compound-key-is-a-uniqueness-constraint-first`,
   § Effect). The two-caller half reuses the caller assignment (`docs/design/caller-values.md`
   § Synthesis).
5. **Second adopter.** A feed item unique on (container, item id).
6. **Cost.** No source format beyond the parent's, and no suite format: the duplicate shape's steps
   and expectations exist. Suites of models without `unique:` keep their bytes.
7. **Alternatives.** (a) Witness only the refusal: rejected, because an implementation that keys on
   one member passes it. (b) Witness both sides: chosen, as the record states.

## Decisions

accept, redesigned, as the parent story. This story carries synthesis and the interpreted target.

## Source

github:beyond10x/ess#638
