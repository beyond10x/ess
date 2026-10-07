---
format: aep.planning-md/3
id: story:feature-request-328
kind: story
status: active
title: Dynamic choices project value and label with view identity defaults
refs:
- provider: github
  reference: beyond10x/ess#328
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T11:42:45Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-04T11:42:45Z", actor: "human:timo", revision: 3}
---
## Outcome

A dynamic choice sends its intended scalar value and displays its intended label even when the receiving field has a different name. Preserve the consumer's full request: default to the view's identity and allow explicit value/label selection, including standalone and filter-bar choices.

## Fit review

1. Need: selected_repository receives repository_id while showing location. Issue328 provides the concrete repository view.
2. Class: remaining capability gap after PR343's delivered same-name-field correction. Choice has no general projection fields (ess-ui/src/model.rs:1301 at55061600).
3. Existing expression: same-name inference and fixed value/label options work. TUI falls back field/id/whole-row (app.rs:2264-2300), React field/id/value/null (choice.tsx:29-35). Reads.key is not honored by these dynamic fallbacks.
4. Fit: an explicit read key should remain authoritative, with a defined model-derived identity default and explicit value/label projection. Do not substitute an explicit-key-only fix for the original request. Choose field-path versus row-expression syntax consistently with existing document vocabulary in a binding design before implementation.
5. Second adopter: destination selects warehouse_id while displaying address.
6. Cost: additive model/schema/check/docs coverage and both renderers; default model inference crosses loading/binding/generation because ViewRoute currently holds only route and parameters (binding.rs:45). Preserve legacy fixture-only behavior when no model identity is available and make invalid explicit projections observable.
7. Alternatives: retain partial same-name inference; honor Reads.key only; selected full projection/default contract after design. The first two alone do not close the issue.

## Decisions

Accept the complete need; implementation waits for a concrete projection/default contract, not for new product approval. Model-aware and fixture-only modes, typed values, missing projections and legacy fallback behavior must be decided explicitly. Group with choice/read work where practical, but do not alter already-published PR381 while its gate runs.

## Acceptance

Fresh red-to-green tests for a differently named receiving field, explicit read key, model identity default without explicit key, explicit value and readable label, typed non-string identity, standalone/filter-bar choices, missing field behavior, and unchanged legacy same-name selection in both renderers. Existing partial PR343 does not discharge these cases.

## Scope

ess-ui model/binding/schema, checker model resolution, React emit/choice runtime, TUI choice_options, both choice_value tests and generated reference. Coordinates with feature-request-365 on choice/read seams.
