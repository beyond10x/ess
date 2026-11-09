---
format: aep.planning-md/3
id: story:a-generated-rust-view-query-applies-its-parameters-and-orders
kind: story
status: draft
title: A generated Rust view query applies its parameters and its orders
summary: 'params: and orders over Optional or Timestamp fields stay obligations in generated queries (workbench report items 25, 26)'
tags:
- backlog
- defect
- external-request
- from-consumer-report
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: .engineering/repro/workbench-2026-11-09/examples/25
- confidence: inferred
  path: crates/generate/ess-synth/src/go/
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/
- confidence: cited
  path: crates/generate/ess-synth/src/view_query.rs
revision: 5
---
## Need

The generated Rust server leaves view queries as obligations (`crates/generate/ess-synth/src/view_query.rs`,
reasons around lines 87, 157 and 399); the generated Go server does the same:

- `a view parameter (\`params:\`), which a generated query does not apply`: 77 obligations in the
  reporter's whole model on 0.92.0 (28 routes), 90 on 0.90.0 (item 25, example `25`).
- `an order over the optional field \`due\`, which has no order against a present value` (item 26,
  example `26`, also `note`) and `an order over the \`Timestamp\` field \`created\`, which the suite
  ranks by its instant and a generated query does not` (example `26b`).

Items 25 and 26 of the 2026-11-09 workbench report. This is the largest group in it.

Minimal reproduction (ess 0.92.0), example `25`: an entity `gap.demo.Item` with a `String` field
`title`, and the view

```yaml
- name: gap.demo.ByTitle
  source: gap.demo.Item
  consistency: read_your_writes
  params:
  - name: title
    type: String
  filter: title == param.title
  fields:
  - name: id
    type: String
  - name: title
    type: String
```

`ess generate synthesize --target rust --layout crate --path spec --out rust-out` (and
`--target go`) exits 0 with `21 capabilities: 20 generated, 1 obligation(s)`, the obligation being
`view query gap.demo.ByTitle … kept an obligation by a view parameter (params:)`. Example `26`
(views ordered `due desc` over `Optional<Timestamp>` and `note asc` over `Optional<String>`) gives
2 obligations; example `26b` (`created desc` over `Timestamp`) gives 1. Each validates (exit 0).

## Acceptance

- `ess generate synthesize --target rust --layout crate` on examples `25`, `26` and `26b` reports
  `0 obligation(s)`; the generated `ByTitle` query returns only rows whose `title` equals the
  parameter, as the interpreted target does.
- The generated query orders by an Optional field with absent values placed as the interpreted
  target places them, and by a `Timestamp` as an instant (two texts of one instant in different
  offsets rank equal; `2026-01-01T01:00:00+01:00` ranks before `2026-01-01T00:30:00Z`).
- The conformance suite synthesized for each example passes against the generated Rust server.
- Go: the same three examples report `0 obligation(s)` with `--target go`, or the Go half is split
  into its own story named here.

## Scope

- cited: crates/generate/ess-synth/src/view_query.rs
- inferred: crates/generate/ess-synth/src/rust/ (the emitted query)
- inferred: crates/generate/ess-synth/src/go/ (the same obligations on Go)
- inferred: .engineering/repro/workbench-2026-11-09/examples/25, 26, 26b (fixtures)

confidence: medium

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
