---
format: aep.planning-md/3
id: story:feature-request-323
kind: story
status: implemented
title: TUI form overlays submit every overlay parameter
refs:
- provider: github
  reference: beyond10x/ess#323
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/tests/tui.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T09:25:39Z", actor: "human:timo", revision: 4, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T09:25:39Z", actor: "human:timo", revision: 5, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "active", to: "implemented", at: "2026-10-02T10:36:58Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

Submitting a terminal form overlay sends its resolved overlay parameters together with its typed draft fields, with the same draft-over-parameter precedence as React.

## Fit review

1. Need: an overlay opened for a record identified by ambiguity_id must send that identifier with resolution. Issue323 gives a minimal ResolveAmbiguity form; it proposes no new syntax.
2. Class: renderer defect. The issue contrasts React's {...params, ...draft} with TUI's id-only copy. Source at candidate28aeddddf crates/ui/ess-ui-tui/src/app.rs:2321-2337 corroborates submit reading only overlay_params().get("id").
3. Existing expression: overlay.params already declares arbitrary named values and open_overlay resolves them (app.rs:2640-2691). Renaming every entity key to id is not an equivalent idiom.
4. Fit: use the existing resolved overlay parameter map, then typed form fields. Preserve parameter value types, nested typed draft handling from329, and live command delivery from377. Align with React's existing form input composition; no new command/model semantics or target vocabulary.
5. Second adopter: an edit-reservation overlay passes reservation_number and warehouse_id with an edited note. Same arbitrary-parameter contract.
6. Cost: generated/submitted values become correct; no authored format, field, generated API or migration. Regression must establish collision precedence explicitly and cover non-string parameters.
7. Alternatives: retain id-only behavior; special-case another consumer identifier; selected general map composition already used by the sibling renderer. No new syntax is warranted.

## Decisions

Accept as proposed. Apply within the UI consolidation tree after its source merges, before package verification. Existing PR345 owns nested typing; this story adds missing overlay parameters rather than duplicating that implementation.

## Acceptance

- overlay_form_sends_non_id_parameters: form over ambiguity_id sends that exact value together with resolution through the public TUI/test seam.
- overlay_form_preserves_typed_parameters: multiple scalar/nested parameter values retain their types.
- overlay_form_draft_overrides_matching_parameter: intentional edited fields win collisions, matching React.
- Ordinary forms and id-based overlays retain existing behavior, including live-refusal/draft preservation and nested typed form values.

## Scope

Cited: crates/ui/ess-ui-tui/src/app.rs submit/open_overlay; crates/ui/ess-ui-tui/tests/tui.rs existing public harness. Rust regression fixtures only. Source cause verified; fresh red execution required before changing submit.
