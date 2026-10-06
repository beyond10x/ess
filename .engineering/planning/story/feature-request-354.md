---
format: aep.planning-md/3
id: story:feature-request-354
kind: story
status: implemented
title: Nested reading nodes and record headers participate in live updates
refs:
- provider: github
  reference: beyond10x/ess#354
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T12:53:10Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-04T12:53:10Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:25Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

Reading composites inside tabs and headers receive declared live updates, and a header title can read the intended page record. Both parts of issue354 remain required.

## Source assessment

At55061600 NodeCommon has no live field; Section owns it (ess-ui/model.rs:685-713). TUI app.rs:141 and React emit.rs:1799 walk section live behavior. Header.live already describes channel lifecycle display (model.rs:1420-1423), so silently giving it another meaning would break existing documents. Header titles are literal in React emit.rs:1990/core.tsx:488 and TUI view.rs:365-380. Assessment executed no tests.

## Design work required

Specify which read supplies the header record when a page has several reads, ownership and lifetime of nested read subscriptions including inactive tabs, applicability of each live effect to records/metrics/lists, and an unambiguous compatibility rule for the existing Header.live field. Then write red cases for tab composite updates and record-derived header titles in both renderers. A title-expression-only patch does not close the request.

## Decisions

Retain the full request as pending design and implementation. Existing channel-status indicators or section live behavior are not a substitute. No source implementation assigned yet; the next design must resolve the concrete ambiguity above before expanding model/schema/renderers.
