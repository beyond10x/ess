---
format: aep.planning-md/3
id: story:a-view-may-publish-a-decimal
kind: story
status: implemented
title: A Decimal inside a list of structs in a view does not cost the scenarios that observe the subject
summary: A view field List<struct with a Decimal member> gives ESS-SYNTH-001 'Decimal and Binary64 replay results are unsupported' on 0.67.0 and 0.68.0 (12 scenarios, unscenarioed 91 -> 103); a top-level Decimal view field passes.
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:19:02Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-10-26T09:19:02Z", actor: "human:timo", revision: 9}
- {from: "active", to: "implemented", at: "2026-10-26T15:50:35Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
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

## Fit review

Reviewed 2026-10-26 on main `a3bc2df3d` (0.69.2) with `ess` 0.69.2. Not fixed on main: the
reproduction below still refuses.

1. **Need.** A view that publishes a `Decimal` (at any depth) must not cost the scenarios that
   check a refusal left its subject unchanged. Minimal reproduction (one entity, one `Decimal`
   field published by its view, a `wrong_state:` refusal): `ess verify conform synthesize` gives
   `refusal[ESS-SYNTH-001]: … Gauge/state/Closed/refuses/…CloseGauge … Decimal and Binary64 replay
   results are unsupported`; the same model with `Integer` gives 5 scenarios, 0 refusals; with the
   field left out of the view, 5 scenarios and a note that the field is unobserved. The trigger is
   not nesting: a top-level `Decimal` fails the same way, and so does `List<struct{value: Decimal}>`.
   It fires wherever synthesis snapshots a complete subject (`preserve`,
   `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:8538-8606`; callers
   `synthesize.rs:5185`, `:5380`, `:14929`, `synthesize/existence.rs:428`, `:595`, `:959`). Why the
   story's earlier four-fixture pass did not reproduce it: I don't know. The requester proposed no
   syntax.
2. **Class.** Gap. The refusal is deliberate in the first retained-result contract
   (`docs/design/retained-command-results.md:106-109`; complete subject shares it, `:221-222`).
3. **Already expressible?** Only by dropping the field from the view (scenario count restored,
   field reported unobserved). That moves a checked figure out of the specification, so it is no
   idiom (`docs/design/generic-language-features.md` § Policy, policy 1).
4. **Fit.** No authored surface. One runner-side check (`ExactShape::check_type`,
   `crates/verify/ess-conformance/src/replay.rs:334-336`) refuses both exact observers, complete
   subject and retained result; opening `Decimal` there opens both siblings. The snapshot compares
   the target with itself, before and after, so no expected `Decimal` is written; the runners
   already hold number tokens (`ts/runtime.ts:133-147` `JsonNumber`, Go `json.Number`,
   `ess-primitives/src/facts.rs:46-60` exact `Number`).
5. **Second adopter.** A refusal to close an invoice must leave its line amounts unchanged, and the
   invoice view publishes them as `Decimal`.
6. **Cost.** No source format. One suite format pair (next free `ess-conformance/58` and `/59`,
   raised only when a shape carries a `Decimal`; a suite without one keeps its bytes); old runners
   refuse such a suite at the version rather than at the step. The comparison is the runner's
   number reader, which on Rust collapses a value beyond binary64 (`facts.rs:189-190`); that limit
   is the one `decision-blocker:decimal-canonical-serialization-crosses-repositories` keeps (cleared
   2026-10-23: persisted bytes do not move). The design page must say so, replacing "do not route
   through binary64" for `Decimal`.
7. **Alternatives.** (a) Change nothing: adopters keep omitting fields. (b) Treat a `Decimal`
   field as unobserved in the snapshot: the runner still compares extra row keys
   (`subject.rs:93`), so it buys nothing over (c) and loses the typed check. (c) Admit `Decimal`
   in both exact observers, compared by value (chosen). (d) Canonical decimal text on the wire:
   refused, it moves persisted bytes (the cleared blocker).

## Decisions

- Accept, redesigned: not "a view may publish a Decimal" (it already may) but "an exact observer
  admits a `Decimal`". The complete-subject and retained-result observers admit `Decimal` at any
  depth and compare it by numeric value (`1.5` equals `1.50`), in the Rust, Go and TypeScript
  runners.
- `Binary64` keeps its refusal in both observers; `story:a-binary64-field-is-admitted-by-a-conformance-run`
  owns it. Both stories edit the same match arm and the same suite pair: land this one first, or
  carry both in one suite pair if they ship in one release.
- No canonical decimal text: the provisional acceptance line "a Decimal round trips by its
  canonical text" is dropped. It conflicts with the 2026-10-23 answer on
  `decision-blocker:decimal-canonical-serialization-crosses-repositories`, and this story does not
  need it.
- `docs/design/retained-command-results.md:106-109` and `:221-222` are amended to state the
  `Decimal` comparison and its binary64 limit on the Rust reader.
- The refusal help text ("give the field a type that has a finite value, or drop it from the
  command's input") is wrong for a view field. It goes away for `Decimal`; for `Binary64` it should
  name the view field.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/verify/ess-conformance/tests/retained_replay.rs`:
  `a_decimal_published_by_a_view_keeps_the_wrong_state_scenario`: the minimal reproduction, with a
  top-level `Decimal` and with `List<struct{value: Decimal}>`, synthesizes
  `<entity>/state/Closed/refuses/<command>` with no ESS-SYNTH-001, at `ess-conformance/58`, and it
  passes on the interpreted target. A target that changes the `Decimal` in the refusal fails
  "complete subject changed".
- Same file: `unsupported_complete_subject_observers_refuse_by_name` keeps only its `Binary64`
  rows, and `replay_response_refuses_floating_representations_recursively` likewise; the `Decimal`
  rows move to `decimal_retained_results_replay_by_value`.
- Same file: `generated_go_replay_runtime_executes_actual_results_and_strict_admission` gains the
  `Decimal` case: the Go runner admits the shape and compares `1.5`/`1.50` equal and `1.5`/`1.25`
  unequal. The TypeScript runner does the same in `src/ts/response.test.ts`.
- `crates/verify/ess-conformance/tests/suite.rs`:
  `a_suite_without_a_decimal_observer_keeps_its_bytes`: a model with no `Decimal` in a complete
  subject keeps its suite version and bytes. An `ess-conformance/56` reader refuses a suite at /58
  by its version.

## Scope

| unit | files | confidence |
|---|---|---|
| exact observer | `crates/verify/ess-conformance/src/replay.rs` `ExactShape::check_type` (`:334-336`), `declarations_for` (`:188`) | cited |
| synthesis | `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` `preserve` (`:8597-8606`, the refusal and its help) | cited |
| suite floor | `crates/verify/ess-conformance/src/scenario.rs` `raise_for_counted_claims` (`:238-250`), a new `ordinary_floor`; coverage counterpart at `coverage_build.rs:627`, `coverage.rs:46`, `count_json.rs:129` | cited (sites), inferred (new floor module) |
| Rust runner | `crates/verify/ess-conformance/src/runner.rs` `compare_complete_subject` (`:2831`, called at `:695`) | inferred (comparison already by `Node` value) |
| Go runner | `crates/verify/ess-conformance/src/go/replay.go` `exactType` (`:468-469`), `responseEqual` | cited / inferred |
| TypeScript runner | `crates/verify/ess-conformance/src/ts/response.ts` (`:376-383`, the `floating` test) | cited |
| design, docs | `docs/design/retained-command-results.md` (`:106-109`, `:221-222`), `website/docs/reference/formats.md` (suite pair row) | cited / inferred |

Crate: ess-conformance only (sizing: M). Format: no source format (`ess/32` not needed); one
suite format pair, `ess-conformance/58`/`59`, shared with
`story:a-binary64-field-is-admitted-by-a-conformance-run`. Test binaries: `retained_replay.rs`,
`suite.rs`, and `src/ts/response.test.ts`. Overlaps: the same `replay.rs` match arm and suite pair
as the Binary64 story; `declarations_for` (`replay.rs:206-216`) is where
`story:a-struct-that-ignores-undeclared-fields-may-declare-none` would meet the exact observers
(inferred).

confidence: medium

## Second report: wrong-state scenario

Second adopter report, 2026-10-26: a wrong-state refusal scenario must show the row unchanged through a view from `ess/7` on, and a top-level `Decimal` view field makes it unreplayable. Neutral probe, 0.69.2: `ess/6` 5 scenarios / 0 refusals, `ess/7` and `ess/25` 4 / 1 (ESS-SYNTH-001 `…/state/Closed/refuses/…`, "Decimal and Binary64 replay results are unsupported"). The unit E tree gives 5 / 0 at `ess/25`; the probe joins this story as a regression fixture. Not beyond10x/ess#646, which is Binary64 admission.
