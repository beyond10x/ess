---
format: aep.planning-md/3
id: story:feature-request-229
kind: story
status: implemented
title: No guard on another entity's state for non-creating commands, and no effect on related records
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#229
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/subject_fact.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/related_guard_moves.rs
- confidence: inferred
  path: docs/design/cross-record-and-stored-field-guards.md
- confidence: inferred
  path: website/docs/reference/predicates.md
- confidence: inferred
  path: website/docs/reference/spec-versions.md
revision: 20
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:14Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-01T06:31:54Z", actor: "human:timo", revision: 19, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:42:56Z", actor: "human:timo", revision: 20, decided_on: {"recorded":{"test_result":1,"review_outcome":3,"verification":1}}}
---
## Outcome

A non-creating command may guard on another entity's state (beyond10x/ess#229, the `state` case).

## Acceptance

- A `when_related` test may read the related row's `state` (for example "a release needs a candidate in state Accepted"); `ess specify validate` no longer refuses it as `unobservable_fact`.
- Synthesis witnesses the guard with a related row in an accepting state and one in a refusing state: `<command>/outcome/<success>` and `<command>/outcome/<refusal>` are synthesized with no ESS-SYNTH refusal for them.
- A mutant that ignores the related row's state fails the synthesized refusal scenario.

## Origin

beyond10x/ess#229; the `state` case added downstream on 0.48.0 (issue comment 5911480054). The issue's other half, an effect on related records, is `story:related-record-effects`, outside this epic.

## Scope

Derived 2026-09-30 by `aep:story-scoper` on 1bd946d6b; **cited** = read in the tree, **inferred** = a reading. `CHANGELOG.md` and `changes/` are the coordinator's at merge and are not scope entries.

- **Domain refusal:** `crates/specify/ess-domain/src/command/related_guard.rs:567` (`check` builds `DomainEnvironment::new(types, &entity.fields)` with no `state`; `expression.rs:529` raises `unobservable_fact`) and `:642-646` (`validate_partition`) — cited
- **Reuse:** `subject_fact::readable_fields`, `admits_state`, `reads_state` (`ess-domain` `command/subject_fact.rs:326-360`), the #204 precedent — cited
- **Also likely:** `ess-domain/src/system.rs` + `primitive_admission.rs` + `schemas/generated/ess.schema.json` if a format gate (ess/20) is chosen — inferred
- **Synthesis:** `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (`selects` :218, `with_row` :778 already pass `row.state`) and `synthesize/subject_fact.rs` (`search` :3293) — inferred
- **Tests:** `crates/specify/ess-domain/tests/related_guard.rs`, `crates/verify/ess-conformance/tests/related_guard_moves.rs` — inferred
- **Documents:** `docs/design/cross-record-and-stored-field-guards.md` (:500-596, plus the effect-on-related-records specification), `website/docs/reference/predicates.md`, `spec-versions.md` — inferred
- **Not touched:** `ess-entity-runtime/src/lib.rs:1278`, `interpret/execute.rs:660`, `ess-gen/src/http.rs:142` — cited
- **Confidence:** high for the domain site; medium for synthesis
- **Would collide with (every in-epic pair `aep plan artifact waves` reports, 2026-09-30):** 266 on `ess-conformance/src/synthesize/subject_fact.rs`; 269 on `ess-domain/src/system.rs`; 270 on `ess-conformance/src/synthesize/related_guard.rs`; 271 on `ess-conformance/src/synthesize/related_guard.rs`, `ess-conformance/src/synthesize/subject_fact.rs`; 272 on `ess-conformance/src/synthesize/related_guard.rs`, `ess-conformance/src/synthesize/subject_fact.rs`
- **Safety fact:** the change only adds a root; an entity cannot declare a stored field named `state` (`subject_fact.rs:330-333`), so no valid document changes meaning — unproven

## Fit review

Fit review from `docs/design/review-external-requests-2026-09.md` (2026-09-30), per `.agents/skills/assessing-external-requests/SKILL.md`.

- Need: a guard on a related row's held state. Class: gap. Existing idiom: none (`unobservable_fact`). Fit: same operand as `state` in `when_subject` (#204), which was gated at a format.

## Decisions

- **accept, redesigned (coordinator, 2026-09-30):** `state` inside `when_related` is admitted at the next format version (ess/20), not silently at ess/18, and ess/20 is bundled with any other syntax this epic adds so the format moves once.
