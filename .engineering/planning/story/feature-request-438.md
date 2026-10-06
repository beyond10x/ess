---
format: aep.planning-md/3
id: story:feature-request-438
kind: story
status: implemented
title: List-typed view parameters and parameter defaults
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#438
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-427
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/openapi.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: cited
  path: crates/verify/ess-conformance/tests/aggregate_group_selection.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/aggregate-list-parameter.yaml
- confidence: cited
  path: docs/design/aggregate-group-selection.md
- confidence: cited
  path: website/docs/guides/specify/aggregate-views.md
- confidence: cited
  path: website/docs/reference/predicates.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T14:57:43Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-10-05T14:57:43Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:48:15Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":5}}}
---
## Outcome
Resolve beyond10x/ess#438: List-typed view parameters and parameter defaults.

## Origin
beyond10x/ess#438, filed 2026-10-05; an adopter's metrics read API specification (51 view parameters affected), observed on ess 0.35.0.

## Fit review
1. Need: a read selects rows whose key is in a list the caller sends. An empty list means unfiltered. Keys that match nothing select nothing. A Boolean switch has a value when the caller leaves it out. Minimal reproduction, written fresh (`<fit-review scratch>/probe-438/`): entity `Call {queue_id, abandoned, started_at, duration_ms}`, view parameter `queues: List<Integer>`. The requester's syntax, not adopted: `filter: {queue_id: {in: param.queues}}`, `empty: unfiltered`, and `default:` on a view parameter.
2. Class: splits three ways.
   - defect: `queue_id: {in: param.queues}` reads `param.queues` as text. `AnyOf` holds literal values only (`crates/specify/ess-primitives/src/predicate.rs:1306-1311`, `:3080-3087`), and the refusal says "Text literal `param.queues`" (probe a, 0.53.0 debug build). Two places say otherwise: predicates.md:504-505 ("a bare dotted word is a fact path") and view.rs:933-935 (`param.<name>` "is a fact path like any other"). `queue_id: {eq: param.queue}` does read the path (probe i).
   - gap (conformance): the list form that validates is never witnessed. An aggregate view refuses `ESS-SYNTH-017` ("the parameter `queues` is read other than by one top-level `field == param.queues` conjunct", `crates/verify/ess-conformance/src/synthesize/aggregate.rs:1157-1163`). Its scalar sibling `queue_id == param.queue` synthesizes (probe agg).
   - convenience: defaults and the empty-list rule can already be written (question 3).
3. Existing idiom: `ess specify validate` accepts all of these on the 0.53.0 debug build (ess/22) and on installed 0.52.0 (ess/20). The issue's 0.35.0 observation of the `exists` form is out of date.
   - membership: `exists: {in: param.queues, as: q, that: queue_id == q}` (probe b).
   - empty means unfiltered: `any: [param.queues.count == 0, {exists: …}]` (probe c).
   - default: an `Optional<Boolean>` parameter with `any: [{all: ["not defined(param.abandoned)", "abandoned == false"]}, abandoned == param.abandoned]` (probe j).
   - `default:` is refused with "unknown field `default`" (probe e). A command already spells a default as `{input: x, else: <literal>}` (values-and-views.md:56-70).
4. Fit:
   - Refuse a membership operand that is a `param.`/`input.` dotted word, and say why. The refusal names the quantifier form, so there is one spelling for membership in a fact. It adds no `Predicate` variant, so no evaluator changes. Precedent: `subject.note` is refused in every format (values-and-views.md:46-49). The same hint covers a comparison operand that starts with a parameter path and continues as text: `param.limit_s * 1000` is read as text today (probe-441 a; see #441).
   - Synthesis: extend aggregate group selection (aggregate-group-selection.md:33, aggregate.rs:1150-1163) so a top-level `exists: {in: param.L, as: b, that: <field> == b}` counts as a selector. The scenario sends a one-element list holding the arranged key and keeps a decoy group outside it. Where a sibling disjunct is `param.L.count == 0`, it also reads with `[]` and asserts every scoped group.
   - The interpreter already binds a parameter by its declared type (`interpret/views.rs:36-55`). Generated Rust and Go queries keep a quantified predicate as a named obligation (conditional-aggregate-measures.md:125), and nothing is dropped silently.
   - A row view with any parameter refuses `ESS-SYNTH-005`, scalar parameters included (probes h, hs). That limit is not specific to lists, so it stays out of this story.
5. Second adopter: an order board filtered by `statuses=[Open, Paid]`, where an empty list shows every order. A ticket list has `include_archived`, false when omitted.
6. Cost:
   - No new source key and no ess/23.
   - One refusal changes in every format: a membership list holding the text `param.x` is refused. No diff classification changes.
   - Suite: whether the Rust, Go and TypeScript runners accept a JSON array as a view parameter value — I don't know. If they do not, it rides `ess-conformance/44`/`/45`, which #427 introduces.
   - OpenAPI emits a list parameter's declared type with no `style`/`explode` (`crates/generate/ess-gen/src/openapi.rs:1766-1800`; grep finds neither key). So the `queues=a,b,c` comma encoding is unstated (inferred). Settle it in this story.
7. Alternatives:
   - (a) Change nothing: the misread stays and list views stay unwitnessed.
   - (b) The requester's form: admit `{in: param.L}` as membership in a fact. That is a new `Predicate` variant in every evaluator, generated query and Entity Runtime lowering, and a second spelling beside `exists`.
   - (c) `empty: unfiltered`: duplicates the `.count == 0` disjunct.
   - (d) `default:` on a view parameter: a second spelling of `else:`, which commands already use.
   - Chosen: a refusal with a hint, the synthesis extension and the documented idioms.

## Decisions
accept, redesigned — Built:
1. A membership operator (`in`, `any_of`, `one_of`, `not_in`, `none_of`) whose operand is a `param.`/`input.` dotted word is refused as `type_mismatch` with a hint naming `exists: {in: param.<x>, as, that}`. A comparison operand that starts with such a path and continues as text gets a hint too, naming the one-constant offset form and a parameter declared in the stored unit. This story owns that refusal and its hint, `param.limit_s * 1000` included (coordinator decision 2026-10-05); #441 only documents it.
2. Aggregate synthesis arranges a list parameter read through that quantifier: a one-element list, a decoy group, and `[]` where the `.count == 0` disjunct exists.
3. The guide section `## A list parameter, an empty list and a default` in `website/docs/guides/specify/aggregate-views.md` documents the list, empty-list and default idioms.
4. OpenAPI states a `List<T>` view parameter as `in: query`, `style: form`, `explode: true` (`queues=1&queues=2`). That is OpenAPI's default made explicit, and it stays unambiguous for any element type. The Go and TypeScript runners send a list parameter the same way.

Not built: `default:`, `empty:` and `{in: param.L}` membership. No ess/23. Suite: depends on #427 (edge recorded). If list values need a suite change, it rides `ess-conformance/44`/`/45`, which #427 introduces; this story does not edit the format lists in `scenario.rs`, `admission.rs` or `count_json.rs`. #439 depends on this story (edge recorded) and builds its range selector on this story's change to the parameter selectors in `aggregate.rs`.

## Acceptance
- membership_operand_naming_a_parameter_is_refused_with_quantifier_hint: `queue_id: {in: param.queues}` is refused, naming the `exists` form; the same holds for `not_in`, `none_of`, `any_of`, `one_of` and an `input.<x>` operand.
- comparison_operand_with_trailing_text_after_a_parameter_is_refused: `duration_ms > param.limit_s * 1000` is refused `type_mismatch`, and the hint names the one-constant offset form and declaring the parameter in the stored unit.
- parameter_offset_stays_admitted: `duration_ms > param.limit_ms + 1000` validates with unchanged IR bytes.
- membership_literal_lists_keep_meaning_and_bytes: `queue_id: {in: [1, 2]}` and `channel: [Web, Store]` validate with unchanged IR bytes.
- list_parameter_quantifier_validates: the committed fixture `crates/verify/ess-conformance/tests/fixtures/aggregate-list-parameter.yaml` validates. It holds the three fit-review shapes as views: `exists: {in: param.queues, as: q, that: queue_id == q}`; `any: [param.queues.count == 0, exists: {…}]`; and the `Optional<Boolean>` default `any: [all: [not defined(param.abandoned), abandoned == false], abandoned == param.abandoned]`.
- list_parameter_guide_section_states_the_idioms: `website/docs/guides/specify/aggregate-views.md` has the heading `## A list parameter, an empty list and a default`. The section shows `exists: {in: param.<list>, as, that}`, `param.<list>.count == 0` and `not defined(param.<flag>)`, and says `default:` and `{in: param.<list>}` are not admitted. Each fenced model in the section validates. A case in `crates/verify/ess-conformance/tests/aggregate_group_selection.rs` reads the page and fails on a missing heading, phrase or invalid model.
- aggregate_list_parameter_selects_listed_groups: the scenario sends `[key_A]`, asserts group A exactly and decoy B absent.
- aggregate_list_parameter_empty_reads_unfiltered: with the `.count == 0` disjunct, a `[]` read asserts both groups; without the disjunct, no `[]` read is synthesized.
- aggregate_list_parameter_faults_fail: a target that ignores the list, reads `[]` as matching nothing, or matches only the first element fails the named observation in the native interpreter and the Go and TypeScript runners.
- list_view_parameter_openapi_encoding_is_stated: the OpenAPI parameter of a `List<T>` view parameter carries `in: query`, `style: form`, `explode: true` and an array schema of `T`. A case in `crates/generate/ess-gen/tests/openapi.rs` checks it; a scalar parameter's bytes are unchanged.

## Scope
- crates/specify/ess-primitives/src/predicate.rs  cited — `AnyOf` operand parsing, 3080-3087
- crates/specify/ess-domain/src/expression.rs  cited — `AnyOf` mismatch and its hint, 2842-2870
- crates/verify/ess-conformance/src/synthesize/aggregate.rs  cited — parameter selectors, 1150-1163
- crates/verify/ess-conformance/tests/aggregate_group_selection.rs  cited — existing refusal control, 1361; guide-section case
- crates/verify/ess-conformance/tests/fixtures/aggregate-list-parameter.yaml  inferred — the fit-review list-parameter shapes, committed
- crates/verify/ess-conformance/src/interpret/views.rs  cited — typed parameter binding, 36-55
- crates/verify/ess-conformance/src/go/runtime.go  inferred — list parameter values in the Go runner
- crates/verify/ess-conformance/src/ts/runtime.ts  inferred — list parameter values in the TypeScript runner
- crates/generate/ess-gen/src/openapi.rs  cited — `view_parameters`, 1766-1800
- crates/generate/ess-gen/tests/openapi.rs  cited — list-parameter encoding case
- docs/design/aggregate-group-selection.md  cited — selector rule, 33
- website/docs/guides/specify/aggregate-views.md  cited — parameter scoping, 409-416; new section
- website/docs/reference/predicates.md  cited — operand rule, 504-505
