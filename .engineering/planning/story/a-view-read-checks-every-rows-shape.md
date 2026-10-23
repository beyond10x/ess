---
format: aep.planning-md/3
id: story:a-view-read-checks-every-rows-shape
kind: story
status: draft
title: A view read checks every returned row against the view fields
summary: 'Row shape check on synthesized and authored view reads, split from #680 for 0.70.0; design page first.'
tags:
- adopter-report
- ess-0.70.0
refs:
- provider: github
  reference: beyond10x/ess#680
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A synthesized or authored view read checks every row the target returns against the view's declared fields: a missing required field or a mistyped field fails the scenario, not only the fields the scenario claims.

## Evidence

Split from `story:a-view-read-checks-its-rows-shape` (beyond10x/ess#680) for 0.70.0. Traced in 0.69.0: `ExpectView` and `EventuallyView` (`crates/verify/ess-conformance/src/scenario.rs`) carry no shape; the event payload shape is the model. A shape on view reads needs a new suite format pair and a row check in the Rust runner and the Go and TypeScript runtimes.

## Acceptance

- A design page under `docs/design/` settles which view reads carry a shape and the format consequence, before code.
- A view read refuses a row missing a required field, and a row with a mistyped field.
- A suite whose view reads carry no shape keeps its bytes.
