---
format: aep.planning-md/3
id: story:a-held-state-guard-is-witnessed-without-a-view-publishing-state
kind: story
status: draft
title: A held-state guard is witnessed without a view publishing state
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#732
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-multi-state-wrong-state-error-branch-is-synthesized
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/held_state_guard_without_state_view.rs
- confidence: cited
  path: docs/design/cross-record-and-stored-field-guards.md
revision: 4
---
## Need

A command with a held-state guard (`when_subject_state:`) has none of its state-selected branches
witnessed unless an unfiltered immediate view publishes the entity's identity and `state`. The
equivalent guard spelled as a stored-field predicate, `when_subject: {predicate: {any: [state ==
Rejected, state == Stale]}}`, is witnessed without such a view: the row is arranged through its
commands and the scenario's outcome, error and events are the evidence about the fields no view
publishes. The design says the two spellings are one guard, and that a view must not be required
for a witness.

Minimal reproduction (`ess/34`, one file; `a` and `c` in the triage reproduction): the
specification of `story:a-multi-state-wrong-state-error-branch-is-synthesized`, whose only view
publishes `item_id`, `source` and `level`.

```console
$ ess verify conform synthesize --path a --out a.json
refused: refusal[ESS-SYNTH-001]: outcome example.catalogue.SetLevel/not-prose has no scenario `example.catalogue.SetLevel/outcome/not-prose`
  no witness: `not-prose.subject_state` is `example.catalogue.Item`, which subject-state selection requires a declared unfiltered immediate view projecting identity and lifecycle state
refused: refusal[ESS-SYNTH-001]: outcome example.catalogue.SetLevel/set has no scenario `example.catalogue.SetLevel/outcome/set`
  (same cause)
refused: refusal[ESS-SYNTH-001]: entity example.catalogue.Item has no scenario `example.catalogue.Item/state/Rejected/refuses/example.catalogue.SetLevel`
  no witness: `example.catalogue.Item.state` is `example.catalogue.Item`, which preservation requires immediate identity views covering every subject field
refused: ... the same for `Stale`, and SetLevel/outcome/wrong-state
23 scenario(s) (0 authored), 5 refusal(s)

$ ess verify conform synthesize --path c --out c.json     # wrong-state: when_subject: {predicate: {any: [state == Rejected, state == Stale]}}
26 scenario(s) (0 authored), 0 refusal(s)
$ ess verify conform run --target interpreted --path c --suite c.json --report-format 2
  26 scenarios: 26 passed, 0 failed, 0 error, 0 unsupported
```

Cause, from the code: `observe_selection_subject`
(`crates/verify/ess-conformance/src/synthesize.rs:8773`) refuses with `NoWitness` where no view
publishes `state`, and its callers propagate it (lines 5131, 5133, 5325, 6195, 6500, 8741); two
callers already tolerate it (line 6084, `crates/verify/ess-conformance/src/synthesize/existence.rs:1347`).
The per-state preservation observation refuses the same way
(`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:8877`).

## Acceptance

- `ess verify conform synthesize` on `a` writes `SetLevel/outcome/not-prose`, `SetLevel/outcome/set`,
  `SetLevel/outcome/wrong-state` and `Item/state/Rejected/refuses/SetLevel` and
  `Item/state/Stale/refuses/SetLevel` with no refusal; each scenario whose arranged `state` no view
  publishes carries a `PartialObservation` note naming `state`.
- `ess verify conform run --target interpreted` passes every scenario of that suite.
- A specification whose view publishes `state` (`b`) keeps its suite bytes.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- cited: crates/verify/ess-conformance/src/synthesize/existence.rs
- cited: docs/design/cross-record-and-stored-field-guards.md
- inferred: crates/verify/ess-conformance/tests/held_state_guard_without_state_view.rs
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A specification of a system that offers no read of `state` cannot have its held-state
   branches witnessed, unless it declares a view the system does not have. Reproduction above,
   brand-free. Requester's proposal: "synthesis treats it like the equivalent `state ==`
   predicate".
2. **Class.** Defect. `when_subject_state: [A, B]` is `subject.state in [A, B]`
   (`docs/design/generic-language-features.md:521-525`), and the view "is not required for the
   witness ... declaring a view to satisfy synthesis would make the specification describe a read
   surface the system does not have" (`docs/design/cross-record-and-stored-field-guards.md:120-131`).
   The `when_subject:` spelling follows that page; the `when_subject_state:` spelling does not.
3. **Already expressible?** Respelling the guard as a `when_subject:` predicate works (`c`), and so
   does declaring a view with `state` (`b`); the first makes one guard's spelling depend on a
   synthesis limit, the second is the read surface the design page says not to invent. Neither is
   an idiom.
4. **Design fit.** No authored surface. Grid R: `state` is read the same way by both spellings; the
   change makes their observation the same, through what the views publish, with
   `Note::PartialObservation` for the rest, as `cross-record-and-stored-field-guards.md` decides for
   stored fields. It applies to every branch of a held-state command (preserves, updates, moves,
   refusals, the per-state family), not only `error:` branches. No target, generator or diff class
   changes.
5. **Second adopter.** `example.ship.Order` with `when_subject_state: [Delivered, Cancelled]` and a
   buyer-only read surface (no view of orders at all): every held-state branch is refused today.
6. **Cost.** No format version, keyword or diagnostic code. Suites that refused gain scenarios and
   notes; suites with a `state` view keep their bytes.
7. **Alternatives.** (a) Change nothing and tell authors to declare a `state` view: contradicts the
   design page. (b) Desugar `when_subject_state:` into a `when_subject:` predicate in the compiler:
   changes IR bytes and every suite of a held-state command, a format matter for a synthesis fix.
   (c) Chosen: observe the held state through views where one publishes it, and note it otherwise.

## Decisions

accept, redesigned: the requester's second option is adopted as the behaviour, but not by rewriting
the guard into a predicate; the held-state observation follows the rule the design page already sets
for stored fields.

## Source

github:beyond10x/ess#732
