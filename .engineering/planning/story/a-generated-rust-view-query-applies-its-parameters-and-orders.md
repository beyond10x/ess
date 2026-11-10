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
  path: .engineering/repro/workbench-2026-11-09/examples/26
- confidence: inferred
  path: .engineering/repro/workbench-2026-11-09/examples/26b
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour/query.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour/query.rs
- confidence: cited
  path: crates/generate/ess-synth/src/view_query.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/generated_view_queries.rs
revision: 9
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

All examples are `.engineering/repro/workbench-2026-11-09/examples/<n>/spec`.

- `ess generate synthesize --target rust --layout crate --path <example>/spec --out rust-out` on
  examples `25`, `26` and `26b` prints `0 obligation(s)` in its capabilities line.
- The same command with `--target go` (the reproduction's Go invocation) on the same three
  examples prints `0 obligation(s)`.
- `cargo test -p ess-synth --locked --test generated_view_queries` passes a new case that builds
  the Rust and the Go servers for example `25` over the harness storage port, holds rows with
  `title` `a`, `b` and `a`, and asks `ByTitle` with `title=a`: both answer exactly the two rows whose
  `title` is `a`, the rows the interpreted target answers.
- The same test target passes a case over example `26`, Rust and Go: with one row lacking `note`
  and rows with `note` `x` and `y`, `note asc` answers the absent row first; with one row lacking
  `due` and two rows with `due`, `due desc` answers the absent row last (the interpreted target's
  `interpret::views::compare` ranks an absent value below every present one).
- The same test target passes a case over example `26b`, Rust and Go: rows with `created`
  `2026-01-01T01:00:00+01:00` and `2026-01-01T00:30:00Z` answer under `created desc` with the
  `00:30:00Z` row first, and two rows whose `created` texts name one instant in different offsets
  rank equal (they keep the port's order).
- The conformance suite synthesized for each of the three examples passes against the generated
  Rust and the generated Go server in that test target (`assert_suite_passes` and `go_suite`).

## Decisions

- Go is in this story, not split off: `view_query.rs` is the one gate the planner and both
  emitters ask, so admitting these constructs there makes the Go plan generate them too, and the
  Go query must render them in the same change.
- An absent value ranks below every present one, because that is what the interpreted target's
  `compare` does (`Option<&Node>`, `Node::Null` its least variant); the generated queries follow it.

## Scope

- cited: crates/generate/ess-synth/src/view_query.rs
- cited: crates/generate/ess-synth/src/determined.rs
- cited: crates/generate/ess-synth/src/rust/behaviour/query.rs
- cited: crates/generate/ess-synth/src/rust/behaviour.rs
- cited: crates/generate/ess-synth/src/go/behaviour/query.rs
- cited: crates/generate/ess-synth/src/go/behaviour.rs
- cited: crates/generate/ess-synth/tests/generated_view_queries.rs
- inferred: .engineering/repro/workbench-2026-11-09/examples/25
- inferred: .engineering/repro/workbench-2026-11-09/examples/26
- inferred: .engineering/repro/workbench-2026-11-09/examples/26b

confidence: medium

Shared surfaces: no format row and no suite-bytes row; it re-pins only the `ess-synth` `go` and
`served_codec_names` byte-pin rows whose generated query bytes it changes.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
