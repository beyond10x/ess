---
format: aep.planning-md/3
id: story:an-invariant-over-a-lists-count-is-held-by-a-synthesized-scenario-not-refused-ess-synth-011
kind: story
status: draft
title: An entity invariant over a list's .count is held by a synthesized scenario on command input, not refused ESS-SYNTH-011
summary: validate accepts members.count >= 1 and synthesis refuses its scenario on 0.99.2 and 0.99.9; the bound is held where the list is built
tags:
- gap
- gap-reports-2026-11-28
relations:
- serves: vision:O2
- informed_by: story:a-list-count-invariant-is-projected-as-min-and-max-items
revision: 2
---
## Need

A downstream specification of reusable building blocks (ess 0.99.2, checked on 0.99.9 and on
`main` at `298790bd0`) declares an invariant on the count of a list an entity stores:
`invariants: [columns.count >= 1]` on an entity with `columns: List<…>`. `ess specify validate`
accepts it. `ess verify conform synthesize` refuses the entity's invariant scenario:

```text
refused: refusal[ESS-SYNTH-011]: entity shelf.sets.Set has no scenario `shelf.sets.Set/invariant/after/shelf.sets.CreateSet/created`
  `members.count >= 1` reads what no view row of `shelf.sets.Set` can be held to
    - `members.count` is a list's `.count`, which a view row does not assert
  help: a view row does not assert a list read this way; the value is held to it where it is built, on command input and setup. A rule of every element is observed on rows when the element's type declares it: an `invariants:` on the element's newtype or struct
```

Neutral reproduction on 0.99.9:
`.engineering/repro/an-invariant-over-a-lists-count-is-held-by-a-synthesized-scenario-not-refused-ess-synth-011/system.yaml`
(`ess/37`: a `Set` with `members: List<String>`, `invariants: [members.count >= 1]`, a creating
command `CreateSet` taking `members` and storing it, one identity view). `validate` says valid;
`synthesize` writes 1 scenario and 1 refusal, `ESS-SYNTH-011`. 0.99.2 and `main`
refuse it the same way (0.99.9 adds the "which a view row does not assert" line). The help names
the place the bound is held: command input and setup.

The refusal is true to what a view row can show: a view publishes the list, and the row's list
has a count the observer could read. The scenario a suite owes the invariant is the other half:
a command input that breaks the bound (an empty list) is refused, one that meets it is accepted.
That half is "held where it is built, on command input", and no scenario sends such an input for
a bound a stored `.count` holds: the refusal leaves the invariant with no scenario at all.

The schema half is a separate story:
`story:a-list-count-invariant-is-projected-as-min-and-max-items` publishes the bound as
`minItems` and `maxItems` and does not touch synthesis.

## Outcome

An entity invariant over a list's `.count` against a literal gets a synthesized scenario: a
creating or updating command whose input sets the list is sent a list that breaks the bound, and
the interpreter answers the invariant's refusal; a list that meets the bound is accepted and the
view row shows the list. Where the list is not set from an input (a computed or stored value) the
refusal `ESS-SYNTH-011` stays and says so.

## Acceptance

- On the reproduction above, `ess verify conform synthesize` writes a scenario for
  `Set/invariant/after/CreateSet/created` (and the refused-input one) and
  `ess verify conform run --target interpreted` passes them.
- A bound on a `.count` of a list no command input sets keeps `ESS-SYNTH-011` with a help line
  naming the missing input.
- No committed suite changes bytes except by the rows of the new fixture.

## Fit review

1. Need: an invariant a model declares and validates gets a scenario or an honest refusal.
   Reproduction above.
2. Class: gap in synthesis, with a validate-accepts, synthesize-refuses split that an author meets
   only at synthesis.
3. Already expressible: an authored scenario for the bound; the `invariants:` of the list's
   element type for a rule of every element (the refusal's own help).
4. Fit: the same witness synthesis writes for a bound over a scalar input, applied to the length
   of an input list.
5. Second adopter: any entity whose list must not be empty (a set, a board with columns, a
   route with stops).
6. Cost: scenarios gained; no format key; a rule on what a model-side count reads from an input.
7. Considered: keep the refusal (today); witness the bound on command input (this story); have
   `validate` refuse the invariant at the entity, which would break models that validate today.

## Decisions

- CD1 (open): A, witness the bound on command input where a command sets the list from an input;
  B, make `validate` name the refusal at the entity. Recommended: A, because B refuses documents
  that validate today.

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/tests/
- inferred: CHANGELOG.md
