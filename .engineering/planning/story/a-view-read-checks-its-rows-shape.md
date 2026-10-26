---
format: aep.planning-md/3
id: story:a-view-read-checks-its-rows-shape
kind: story
status: implemented
title: A view read checks each row's declared shape
summary: Rows of a view read are checked for required fields and types; the AUTHOR-042 hint stops claiming it for view rows
tags:
- ess-0.69.0
refs:
- provider: github
  reference: beyond10x/ess#680
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T18:26:24Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T18:26:25Z", actor: "human:timo", revision: 4, correlation: "ess-0690"}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:27Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Problem

A view read checks no row shape, and the ESS-AUTHOR-042 hint claims it does (ess#680).

## Acceptance

1. The AUTHOR-042 hint stops claiming a presence and type check for a view row, and names `satisfies: defined(<field>)` and `$observed` there.
2. A view read checks every returned row against the view's declared fields: each required field present, each present field of its declared type, as an event payload is checked. This is a new suite format member, selected the way earlier additive suite formats were, so suites that do not use it keep their bytes.

Scenario names: `the_generated_value_hint_names_satisfies_for_a_view_row`, `a_view_read_refuses_a_row_missing_a_required_field`, `a_view_read_refuses_a_mistyped_field`, `a_suite_without_view_shapes_keeps_its_bytes`.
