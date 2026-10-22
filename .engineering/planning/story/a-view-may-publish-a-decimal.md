---
format: aep.planning-md/3
id: story:a-view-may-publish-a-decimal
kind: story
status: draft
title: A Decimal inside a list of structs in a view does not cost the scenarios that observe the subject
summary: A view field List<struct with a Decimal member> gives ESS-SYNTH-001 'Decimal and Binary64 replay results are unsupported' on 0.67.0 and 0.68.0 (12 scenarios, unscenarioed 91 -> 103); a top-level Decimal view field passes.
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Need

A downstream specification (public repository beyond10x/patchback, commit 69e87fd) leaves `nodes` out of the `Patches` view (`spec/domains/patch.yaml:894-900`) because `nodes` is `List<NodeDecl>`, `NodeDecl.parameters` is `List<ParameterValue>` and `ParameterValue.value` is a `Decimal`. Its comment: "ESS 0.67.0 refuses Decimal replay results in a view ('Decimal and Binary64 replay results are unsupported'), which would cost the generated ValidatePatch and SetParameter scenarios (tested: unscenarioed 91 -> 103)". `spec/domains/score.yaml:573-574` omits `NoteEvent.velocity` and `ControlEvent.normalized_value` for the same reason.

Corrected facts (downstream repro, 0.67.0, confirmed on 0.68.0 below): publishing `nodes` takes unscenarioed from 91 to 103 and unanswered from 28 to 12, so it answers 16 questions and costs 12 scenarios. The earlier relayed "23 unanswered" was wrong. A top-level `Decimal` field in a view is fine; the refusal needs the Decimal nested in a list of records.

Needed: a view may publish a `Decimal` nested in a list of structs without costing the scenarios that observe its subject.

## Reproduction

Minimal, top-level Decimal (passes, for contrast): a two-file spec with one `Decimal` view field. 0.67.0 and 0.68.0 both: `valid`, 1 scenario, 0 refusals.

Failing case, the downstream specification with one change: `nodes: List<patchback.patch.NodeDecl>` added to view `patchback.patch.Patches`. `ess verify conform synthesize` on 0.68.0 (pin edited to 0.68.0 in a copy, so the dispatcher does not delegate to 0.67.0):

```text
247 scenario(s) (0 authored), 103 refusal(s)
validate completeness: {"unscenarioed":103,"outside":0,"unanswered":12}
12 x refusal[ESS-SYNTH-001]: outcome patchback.patch.ValidatePatch/unknown-node has no scenario ...
  no witness: `patchback.patch.Patches: Decimal and Binary64 replay results are unsupported` is `complete subject observation`, which complete subject requires a finite exact typed observer
```

| | unscenarioed | unanswered | Decimal ESS-SYNTH-001 |
|---|---|---|---|
| 0.67.0, 69e87fd without `nodes` | 91 | 28 | 0 |
| 0.67.0, with `nodes` | 103 | 12 | 12 |
| 0.68.0, with `nodes` | 103 | 12 | 12 |

Same on both releases. Not reduced further than "Decimal nested in a list of records in a view"; an earlier pass (four fixtures with a Decimal entity field, a `preserves` outcome and a `List<struct with a Decimal member>` view field set from an input) did not reproduce it, so a further condition of the downstream specification triggers the complete-subject path. Which one is not established.

## Where it lives

- `crates/verify/ess-conformance/src/replay.rs:334-336`: `ExactShape::check_type` returns "Decimal and Binary64 replay results are unsupported" for `Primitive::Decimal | Primitive::Binary64`; it recurses through `Named` (via `declarations`, :306-315), `List` and `Optional` (:325-327) and `Map`, so a Decimal anywhere under a published field reaches it (traced).
- `crates/verify/ess-conformance/src/subject.rs:40-56`: `SubjectShape::of` builds the complete-subject shape from every field of the view with `declarations_for(..., DeclarationProfile::CompleteSubject)`, then `validate()` (traced).
- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:8025-8034`: the `Err` becomes `RefusalCause::NoWitness` with path `<view>: <reason>` and reason "complete subject requires a finite exact typed observer". This is the ESS-SYNTH-001 raised for the nested Decimal (traced).
- Why the top-level Decimal repro does not take this path was not traced (inferred: a view that does not cover the whole subject, or a different `Observation`, selects the non-complete `SnapshotSubject` at :8036).
- Go and TypeScript runners carry the same refusal: `crates/verify/ess-conformance/src/go/replay.go:469`, `crates/verify/ess-conformance/src/ts/response.ts:382`.

## Acceptance (provisional)

- The downstream specification with `nodes` in the `Patches` view validates with unscenarioed no higher than 91 (the 12 ValidatePatch and SetParameter scenarios stay scenarioed), in the interpreter and in the Go and TypeScript runners.
- A Decimal round trips by its canonical text, not through binary64 (see `story:primitive-canonical-serialization` and `decision-blocker:decimal-canonical-serialization-crosses-repositories`).
