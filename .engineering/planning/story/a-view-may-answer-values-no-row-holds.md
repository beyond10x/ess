---
format: aep.planning-md/3
id: story:a-view-may-answer-values-no-row-holds
kind: story
status: draft
title: A view may answer values no row holds
summary: 'Adopter reports N20, N23: views need a source and stored fields (ess#674); needs an ADR.'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#674
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: architecture-decision-record:a-command-guard-reads-a-component-setting
revision: 3
---
## Problem

A view must have a `source`, and every field must be stored or derived from the row (`expr:`) or a related row. A process-held value (limits, capabilities) cannot be answered: ``missing field `source` `` (N20) and `ESS-VIEW-001` for an unstored field (the unstored part of N23). ess#674, ess 0.68.0.

## Decisions

Accept, redesigned: option A, staged, decided by the operator on 2026-10-28 ("as you recommend.",
`approval-record:a-view-reads-a-setting-first-cut`). Design: Amendment 1 of
`architecture-decision-record:a-command-guard-reads-a-component-setting`.

- A view field may read `setting.<name>`: a non-secret setting of a component serving the view, of a
  type assignable to the field.
- `source:` is optional when every field is a literal or a setting; such a view answers one row.
- First cut: only settings with a fixed `value:`. A setting without one is refused in a view by
  name. Per-configuration suites stay with the guard ADR's own story.
- Format: the next `ess/N`, shared with the exchange layer when both land in one batch.

## Acceptance

- `a_view_without_source_validates`: a view with no `source`, one literal field and one field reading
  a fixed-value setting validates, compiles and projects (OpenAPI view route, row schema)
  (`cargo test -p ess-domain --locked --test view_settings`, new).
- `a_view_reading_a_setting_without_a_fixed_value_is_refused`: refused by name, the hint naming the
  first cut.
- `a_secret_setting_is_not_readable_in_a_view`: `conflicting_declaration`.
- `a_sourceless_view_below_the_format_keeps_its_refusal`: one format lower, `missing field source`
  and `unobservable_fact` name the new format.
- `conformance_holds_a_sourceless_view`: synthesis emits a read of the view expecting the literal
  and the setting's fixed value, and `ess verify conform run --target interpreted` passes it
  (`cargo test -p ess-conformance --locked --test view_settings`, new).
- A model without either keeps its IR and suite bytes (table tests unmoved).

## Size

M (first cut): ess-domain, ess-compiler (optional view source in the IR), ess-conformance
interpreter and synthesis, ess-gen and ess-http readers of the view IR.
