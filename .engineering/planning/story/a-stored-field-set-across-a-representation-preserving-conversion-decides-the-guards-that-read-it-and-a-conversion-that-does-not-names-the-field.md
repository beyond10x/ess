---
format: aep.planning-md/3
id: story:a-stored-field-set-across-a-representation-preserving-conversion-decides-the-guards-that-read-it-and-a-conversion-that-does-not-names-the-field
kind: story
status: draft
title: A stored field set across a representation-preserving conversion decides the guards that read it, and a conversion that does not names the field
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/
- confidence: inferred
  path: crates/verify/ess-conformance/tests/main.rs
- confidence: cited
  path: docs/design/cross-record-and-stored-field-guards.md
revision: 8
---
## Outcome

A `when_related:` or stored-field guard over a field that a creating or changing branch fills
from an input through a declared conversion is decided wherever the conversion carries the value
unchanged, so the related row is arranged and the scenarios behind it exist. Where synthesis
cannot follow the conversion, the refusal names the field and the conversion, not a search that
"tried 1 candidate".

## Need

Found while checking a downstream specification's report of a scenario drop between two
releases. The cause was in the specification, not the release: a capped newtype was introduced for
one request member, and the stored field it fills kept the plain type, so the specification
declared the conversion `Optional<Capped> -> Optional<String>`. After that one change, 94 synthesis
refusals cascaded from one guarded branch, among them dozens of `<entity>/state/<S>/refuses/<command>`
scenarios of an entity two relations away.

Reproduced on 0.97.1 and 0.97.3 alike, byte-identical suites (`model-e/run.txt`):

- `model-e`: `Widget.model: Optional<String>` is set from `input.settings.model:
  Optional<Label>`, `Label` a newtype over `String` (with or without an invariant,
  `model-e1`), crossing the declared conversion `Optional<Label> -> Optional<String>`. `Bind`
  refuses `not defined(model)` on the related widget, then creates a `Binding`; `Ticket` is opened
  through a binding and has wrong-state refusals.

  ```
  refused: refusal[ESS-SYNTH-003]: outcome shop.desk.Bind/bound has no scenario `shop.desk.Bind/outcome/bound`
    no candidate of the 1 tried satisfies ``shop.desk.Widget` stored related row selecting this branch, over the rows 1 bounded arrangements left`
  refused: refusal[ESS-SYNTH-004]: entity shop.desk.Ticket has no scenario `shop.desk.Ticket/state/Closed/refuses/shop.desk.CloseTicket`
    it needs an instance of `shop.desk.Ticket` resting in `Closed`, and the route runs through `shop.desk.OpenTicket/opened`, which no input reaches
  6 scenario(s) (0 authored), 14 refusal(s)
  ```

- `model-e2`: the same model with `Widget.model: Optional<Label>` and no conversion:
  `14 scenario(s) (0 authored), 6 refusal(s)`.

The cause is `settled` in `synthesize.rs`: a `sets:` entry with a conversion is skipped unless it
is `Optional<T> -> T` (`unwraps_unchanged`). The skipped field is neither settled nor unwritten, so
the row's `model` is unknown and neither `not defined(model)` nor the branch after it is selected.
The skip comes before the rule that an omitted `Optional` input leaves an `Optional` field absent,
so the absent side is lost too, although no conversion makes a value out of an absent input.

Two defects:

1. A conversion between a newtype and its own representation, `Optional`-wrapped or not, in the
   direction where the target admits every source value (the source's invariants only narrow
   it), stores the value it was sent. Synthesis treats it as computing an unknown value.
   `binding-mapping-bounded-accessor.md` already follows a conversion between newtypes over one
   representation for bindings; `settled` does not.
2. The refusal is the generic `ESS-SYNTH-003` whose help line ("write the branch's condition over
   values a candidate can carry, or supply a fixture") points nowhere.
   `cross-record-and-stored-field-guards.md` (part 1) says a field written through a declared
   conversion is refused with `ESS-SYNTH-001` naming the field and the reason.

The evidence model names the newtype differently; `Label` is this story's name for it.

## Fit review

Adopted, redesigned as a generic synthesis rule. The report asked why 0.97.3 dropped scenarios.
It did not. 0.97.3 synthesizes byte for byte what 0.97.1 does on the generic model, and `settled`
and `unwraps_unchanged` are the same text at both tags. On the downstream specification, 0.97.3
gives the same refusals and scenario ids before and after the reported import. Retyping the one
stored field brings back 81 scenarios. What is adopted is the
over-strict reading of conversions, which any specification that caps a request type and stores
the value under its plain type runs into. Declined: the downstream workaround (type the stored
field as the newtype) as the only answer, because a cap on the request alone, with the stored
field kept wider, is a legitimate design. Rejected: deciding `defined()` across every conversion,
because a conversion from a non-`Optional` source into an `Optional` target may yield an absent
value.

## Design

- `settled` keeps a `sets:` entry across a conversion when the conversion preserves the value:
  source and target resolve to the same representation through newtypes, with `Optional` on both
  sides or on neither, and the target declares no invariant the source does not imply. Kept with
  `Optional<T> -> T` from an input, as before.
- An `Optional` input the invocation left out leaves an `Optional` target field absent (`null` in
  the row) across any conversion from an `Optional` source, so `not defined(field)` and
  `{exists: false}` stay arrangeable.
- A field still skipped for a conversion is recorded, and a stored-field or related-row search
  whose guard reads it refuses with `ESS-SYNTH-001` naming the entity field, the `sets:` entry and
  the conversion, with the help "type the field as the source, or declare a conversion that hands
  the value on". No new code: the design page assigns `ESS-SYNTH-001`.
- Format: no new `ess/N`. No persisted meaning, identity, reference or canonical form changes. Suites
  of affected models gain scenarios and lose refusals. That is a Changed entry, and committed suites
  are re-pinned.

## Acceptance

- `model-e` synthesizes the same scenario set as `model-e2`, ids `shop.desk.Bind/outcome/bound`,
  `shop.desk.Bind/outcome/no-model` and `shop.desk.Ticket/state/Closed/refuses/shop.desk.CloseTicket`
  among them, checked by a new `ess-conformance` integration test over that model.
- The same model with `Label -> String` declared from a non-`Optional` source into an
  `Optional` target still refuses the guarded branch, with `ESS-SYNTH-001` naming `Widget.model` and
  the conversion. The same test checks the text.
- A conversion whose target declares an invariant the source does not imply is not followed.
  Asserted in the same test.
- `synthesis_suite_bytes_table` and `component_suites` keep every pinned row except rows of models
  with such a conversion. Each moved row names this story as its cause.

## Decisions

- D1: which conversions count as value-preserving. A: newtype to and from its representation and
  between newtypes over one representation, invariants only narrowing toward the target
  (recommended). B: newtype to representation only. C: every conversion, with presence only.
- D2: the refusal for a conversion still not followed. A: `ESS-SYNTH-001` as the design page says
  (recommended). B: keep `ESS-SYNTH-003` and only rewrite its cause and help.
