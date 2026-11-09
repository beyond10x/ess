---
format: aep.planning-md/3
id: story:generated-rust-behaviours-cover-expressions-distinct-members-and-filtered-effects
kind: story
status: draft
title: Generated Rust behaviours cover expression values, distinct members, counted paths and filtered effects
summary: Several behaviour constructs stay obligations in generated Rust (workbench report items 27, 30, 31, 33, 34)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

Generated Rust leaves these command behaviours as obligations:

- `an expression value ({expr: "subject.n + input.delta"})` in `sets:`, and the same for a computed
  value of a rank command (item 31, example `31`).
- `distinct item in outlines by item.id as string`: distinct list members no generated behaviour
  compares (item 30, example `30`).
- `the guard path \`title.count\`` (item 33, example `33`).
- `effects on filtered rows (\`affects:\`)`: same as #547, closed and claimed fixed in 0.63.0 (item 34,
  example `34`).
- The server's test login names an actor but no caller attribute `name`, on 4 commands (item 27,
  model only).

Items 27, 30, 31, 33 and 34 of the 2026-11-09 workbench report.

## Acceptance

- Each construct above generates behaviour, with no obligation, and the example's suite passes
  against the generated server.
- The generated test login carries the caller attributes the model declares.
- If #547's fix regressed, a regression test pins it.

## Scope

- inferred: `crates/generate/ess-synth/src/rust/behaviour.rs`, the Rust server test login template.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
