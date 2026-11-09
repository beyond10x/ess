---
format: aep.planning-md/3
id: story:a-generated-rust-view-query-applies-its-parameters-and-orders
kind: story
status: draft
title: A generated Rust view query applies its parameters and its orders
summary: 'params: and orders over Optional or Timestamp fields stay obligations in generated queries (workbench report items 25, 26)'
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

The generated Rust server leaves view queries as obligations:

- `a view parameter (\`params:\`), which a generated query does not apply`: 77 obligations in the
  reporter's whole model on 0.92.0 (28 routes), 90 on 0.90.0 (item 25, example `25`).
- `an order over the optional field \`due\`...` and `an order over the \`Timestamp\` field \`created\`,
  which the suite ranks by its instant...` (item 26, examples `26`, `26b`).

Items 25 and 26 of the 2026-11-09 workbench report. This is the largest group in it.

## Acceptance

- A generated Rust view query applies every declared view parameter as the interpreted target does.
- It orders by an Optional field, with absent values placed as the interpreted target places them,
  and by a Timestamp as an instant.
- The examples generate with no such obligation, and their conformance suites pass against the
  generated server.

## Scope

- inferred: `crates/generate/ess-synth/src/view_query.rs`, `crates/generate/ess-synth/src/rust/`.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
