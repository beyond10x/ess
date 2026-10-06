---
format: aep.planning-md/3
id: story:feature-request-458
kind: story
status: implemented
title: An error field naming the target state versus the current state
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#458
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-429
- depends_on: story:feature-request-427
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/subject_state_source.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/subject_state_source.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/error-payload-sources.yaml
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: website/docs/guides/specify/commands-and-outcomes.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:21Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:21Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:49:06Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":5}}}
---
## Outcome
Resolve beyond10x/ess#458: An error field naming the target state versus the current state.

## Origin
beyond10x/ess#458, filed 2026-10-05; an audit of a downstream specification could not tell whether a 409 answer's state field names the record's current state or the requested one.

## Fit review
1. Need: a refusal's error payload should be able to declare, and have checked, which state a state-typed field names: the state the record holds, or the state the refused move would have entered. Minimal reproduction, written fresh: a `Doc` with states `Draft, Published, Archived`, and `PublishDoc`/`ArchiveDoc` answering `StateConflict {doc_id, current, requested}` on `wrong_state:` (`<fit-review scratch>/probe-458/spec/system.yaml`). The requester asked a question and proposed no syntax.
2. Class: gap for the current state; convenience (already expressible) for the target state. ESS's own normative example declares `InvoiceStateConflict.state` and explains in a comment that it "carries the state the invoice is actually in" (`examples/billing/domains/invoice.yaml:183-189`), but no payload source can say so. The field is left for the implementation to fill (`website/docs/guides/specify/commands-and-outcomes.md:42`, `:324-325`).
3. Existing idiom:
   - Target state: a literal of the entity's `State` enum. `requested: Published` on the `wrong_state:` refusal validates on the installed `ess 0.52.0` (`probe-458/literal/`: `probe v1 — 1 file(s), valid`). The move's `to` state is fixed, so the literal is the declaration. Error payload literals are `ess/19` (`website/docs/reference/spec-versions.md:202-215`).
   - Current state: not expressible. `current: {subject: state}` is refused: "`state` is not a field of `probe.doc.Doc`" (`probe-458/spec/`, 0.52.0). The 0.53.0 source refuses it the same way: `existing_subject_field` looks up only the identity and the declared fields (`crates/specify/ess-domain/src/command/value_expression.rs:952-1010`). An input does not help, because the caller does not know the held state.
4. Fit: admit `state` in the existing `{subject: …}` source. That is the held lifecycle state immediately before the outcome. It is spelled as `when_subject` predicates read it from `ess/18` (`predicates.md:26`), as `when_related` predicates read it from `ess/20` (`spec-versions.md:219-224`), and as invariants read it (`crates/specify/ess-domain/src/entity.rs:886-891`). An entity cannot declare a field named `state` (`spec-versions.md:224`), so no existing document changes meaning. Siblings: `{subject: …}` is shared by error payloads, event payloads and `sets:` (`value_expression.rs:405`), so `state` is admitted in all three, in exactly the positions `{subject: …}` already is: `wrong_state:`, held-state and stored-field refusals, and outcomes on an existing subject. It stays refused where no row is read: input-guarded refusals, `unknown_instance:` and `creates:` (`spec-versions.md:210-212`). The unknown-instance arm of `wrong_state` still carries no fields (`commands-and-outcomes.md:42-48`). The type is `<Entity>.State`, so the existing conversion check holds it (`value_expression.rs:927-946`). Targets that must read or refuse it by name: the domain check and IR (`crates/specify/ess-compiler/src/ir.rs:1057-1062`, `SubjectField`); the interpreter (`crates/verify/ess-conformance/src/interpret/execute.rs`); synthesis (`crates/verify/ess-conformance/src/synthesize.rs`); Rust and Go behaviour (`crates/generate/ess-synth/src/rust/behaviour.rs`, `go/behaviour.rs`); Entity Runtime (`crates/generate/ess-entity-runtime/src/subset.rs:106`, `lib.rs`); and `ess verify diff` (`crates/verify/ess-diff/src/diff.rs`).
5. Second adopter: an account service whose `ActivateAccount` answers `409 AccountConflict {held: Suspended}` while `CloseAccount` answers `{held: Closed, requested: Closed}`. A client branches on the held state ("reactivate first" versus "already closed"), so which reading the field carries is part of the contract.
6. Cost: source format `ess/23`; below it, `{subject: state}` is refused naming `ess/23`. There is no new keyword. One resolved-IR case: either a `SubjectState` variant, or `SubjectField` with `field: state`; the variant is safer, because a reader that looks `field` up among the entity's fields would otherwise fail. That is inferred, decided in the design. It needs an old-reader test, and models that do not use it keep their bytes. Probably one `ess verify diff` classification (a source changed from a field to the held state); I don't know whether existing payload-source change kinds already cover it. Synthesis: every `<entity>/state/<S>/refuses/<command>` scenario already arranges and observes the row in `S` (`docs/design/subject-state-outcome-guards.md:71-77`), so the expected value is the literal `S`. I expect no new `ess-conformance/N` (inferred). It is not a breaking change for any existing document.
7. Alternatives: (a) change nothing and document that a state-typed error field is the implementation's to fill: rejected, because the need is an unchecked contract the normative example already describes; (b) a new source `{current_state: true}` or `{held_state: true}`: rejected, because it is a second spelling for the `state` that predicates and invariants already read; (c) chosen: `{subject: state}` for the current state, and the existing literal for the target state, with no new form; (d) a source `{target_state: true}` resolved from the branch's move: rejected, because a refusal takes no move (`subject-state-outcome-guards.md:65-66`), and a command whose input selects between moves states each refusal separately with `when:` beside `when_subject_state:` (`crates/verify/ess-conformance/tests/state_scoped_refusals.rs:870`), each carrying its own literal.

## Decisions
Accept, redesigned. The question is answered with existing vocabulary, with no new key. The target state is a literal (works today, `ess/19`). The current state is `{subject: state}`, admitted from `ess/23` wherever `{subject: …}` is admitted, for error payloads, event payloads and `sets:`. Synthesis compares it with the state each refusal scenario arranges. Generated Rust and Go read the held row's state. Entity Runtime refuses it by name: `ValueExpressionUnsupported` on the `` `{subject: …}` `` row (`crates/generate/ess-entity-runtime/src/subset.rs:73`, `:106`), as it refuses every `{subject: …}` value today (`lib.rs:2879-2890`). There is no `$from_state` payload lowering to reuse: `$from_state` is a condition operand only (`lib.rs:2203-2292`, `:3836-3876`). Format: `ess/23`, introduced by #429. This story depends on #429 (edge recorded) and extends `ess/23`. It does not edit `system.rs` or `spec-versions.md`: the `ess/23` gate for this source sits in `primitive_admission.rs` and reads the `FormatVersion::V23` constant #429 adds, and the implementor reports the one sentence for the `ess/23` row under `## Changelog lines` for the coordinator to merge into the row #429 writes. No `ess-conformance/N` is expected; if the design shows one is needed, it uses the `/44`/`/45` pair #427 introduces, and it depends on #427 (edge recorded) so that pair exists first. #459 depends on this story (edge recorded): both edit `value_expression.rs`, this story `check_subject` (927-949) and `existing_subject_field` (952-1010), #459 `validate_affect` (283-333). Sourcing `examples/billing` `InvoiceStateConflict.state` is a separate follow-up, because it moves the recorded site data (`cargo xtask site-data`).

## Acceptance
- subject_state_source_validates_on_wrong_state: `current: {subject: state}` on a `wrong_state:` refusal validates under `ess/23`, and the IR carries the held-state source.
- subject_state_source_refused_below_ess23: the same document under `ess/22` is refused with `unsupported_format_version` naming `ess/23`.
- subject_state_source_refused_without_a_row: refused on an input-guarded refusal, on `unknown_instance:` and on `creates:`, with the existing `{subject: …}` diagnostics.
- subject_state_source_type_checked: a target field that is not of the entity's `State` type is `type_mismatch`.
- wrong_state_scenarios_compare_current_and_requested: each `<entity>/state/<S>/refuses/<command>` scenario requires `current == S` and `requested == <literal>`. A target that answers the requested state in `current` fails, and so does one that answers the current state in `requested`.
- subject_state_source_in_event_payload_and_sets: on a `moves:` branch, an event payload reading `{subject: state}` carries the state before the move.
- generated_rust_and_go_fill_current_state: generated Rust and Go behaviours fill `current` from the held row's state, and the `wrong_state_scenarios_compare_current_and_requested` scenario passes against both.
- entity_runtime_refuses_subject_state_by_name: lowering a model whose `wrong_state:` payload reads `{subject: state}` reports `ValueExpressionUnsupported` naming the `` `{subject: …}` `` construct at `<command>.<outcome>`, and lowers nothing for that outcome. A case of that name in `crates/generate/ess-entity-runtime/tests/subject_state_source.rs`.
- commands_and_outcomes_page_states_the_source: `website/docs/guides/specify/commands-and-outcomes.md` at the current-state field (:42) and the source list (:302-331) names "`{subject: state}`", "`ess/23`" and "the state before the move". A case of that name in `crates/specify/ess-domain/tests/subject_state_source.rs` reads the page and fails naming the missing phrase.
- ess_23_row_names_subject_state: the coordinator's merged `ess/23` row of `website/docs/reference/spec-versions.md` names `{subject: state}` as a value source. Checked at integration by the coordinator, not by a unit test.
- old_models_keep_ir_bytes: models without the source keep their compiled digest.

## Scope
- crates/specify/ess-domain/src/command/value_expression.rs  cited — `check_subject` (927-949) and `existing_subject_field` (952-1010), which refuses `state` today; #459 edits `validate_affect` (283-333) after this story (edge recorded)
- crates/specify/ess-domain/src/primitive_admission.rs  inferred — the `ess/23` gate for the source
- crates/specify/ess-compiler/src/ir.rs  cited — `ResolvedPayloadValue::SubjectField` (lines 1057-1062)
- crates/specify/ess-compiler/src/resolve.rs  inferred — disjoint from #448 (`family_of_kind`, `family_of`, a later wave); `expression_field` (2777-2889; `PayloadSource::SubjectField` becomes `ResolvedPayloadValue::SubjectField` at 2788-2811) resolves the held-state source; `payload_constant_source` (5244-5271) lists it among non-constant sources
- crates/verify/ess-conformance/src/interpret/execute.rs  cited — consumes `SubjectField` payloads
- crates/verify/ess-conformance/src/synthesize.rs  cited — `wrong_state_scenario` (10187-10249) through `refused_here` (10285-10416) builds the refusal scenario; its expected payload values come from `expression_value_at` (7417-7502) and `determined_fields` (6506-6569), which read `SubjectField`. Disjoint from #455 (`sibling_refusals` 6017, `boundary_inputs` 11934, `overlap_inputs` 12175) and from #429/#452 (`deletion_witness` 10736)
- crates/generate/ess-synth/src/rust/behaviour.rs  cited — reads `SubjectField`
- crates/generate/ess-synth/src/go/behaviour.rs  cited — reads `SubjectField`
- crates/generate/ess-entity-runtime/src/subset.rs  cited — `Source::SubjectField` maps to the `{subject: …}` row (73, 106); the refusal reaches `{subject: state}` unchanged
- crates/generate/ess-entity-runtime/src/lib.rs  cited — payload lowering refuses `SubjectField` with `ValueExpressionUnsupported` (2879-2890); no `$from_state` payload lowering is added
- crates/verify/ess-diff/src/diff.rs  cited — payload source changes
- crates/verify/ess-conformance/tests/fixtures/error-payload-sources.yaml  cited — the existing error-payload fixture to extend
- website/docs/guides/specify/commands-and-outcomes.md  cited — lines 42 and 302-331 describe the current-state field and the sources
- crates/generate/ess-entity-runtime/tests/subject_state_source.rs  inferred — the named refusal case
- crates/specify/ess-domain/tests/subject_state_source.rs  inferred — validation cases and the guide-page case
