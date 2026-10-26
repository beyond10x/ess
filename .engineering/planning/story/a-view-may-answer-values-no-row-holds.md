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
revision: 2
---
## Problem

A view must have a `source`, and every field must be stored or derived from the row (`expr:`) or a related row. A process-held value (limits, capabilities) cannot be answered: ``missing field `source` `` (N20) and `ESS-VIEW-001` for an unstored field (the unstored part of N23). ess#674, ess 0.68.0.

## Blocked

No ADR decides where such a value comes from (a component setting, a declared constant, or an implementation-supplied value the suite cannot check), and it moves `ess/N`. The nearest record, `a-command-guard-reads-a-component-setting` (proposed), covers guards only. A new ADR is needed before this story is scoped.

## Acceptance (provisional)

- `a_view_without_source_validates`: a view with no `source` and fields from the decided origin validates and projects.
- `conformance_holds_a_sourceless_view`: the suite reads the declared value, or records why it cannot.
