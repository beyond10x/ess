---
format: aep.planning-md/3
id: story:generated-storage-lists-the-rows-a-selector-selects
kind: story
status: draft
title: Generated storage lists the rows a selector selects, so an unpinned affects filter generates
summary: An `affects:` filter that pins no identity stays an obligation in generated Rust and Go (workbench report item 34)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: story:generated-rust-behaviours-cover-expressions-distinct-members-and-filtered-effects
revision: 1
---
## Need

An `affects:` entry whose filter does not pin the row's identity (`where: parent_id == input.id`,
`moves: <Entity>.<transition>`) is an effect on every matching row. Generated Rust and Go leave such
a command an obligation, `effects on filtered rows (\`affects:\`)`
(`crates/generate/ess-synth/src/determined.rs:1888`), because generated storage enumerates no rows
by a selector (the doc comment of `fn affects`, `determined.rs:1883-1886`). The same limit keeps a
guard over the rows a selector selects owed (`determined.rs:210`; ess/22, beyond10x/ess#228, #299).

beyond10x/ess#547 (closed, 0.63.0) made a filter that pins the identity generate. This is the other
half, not a regression of #547.

Reproduction on 0.92.0, `.engineering/repro/workbench-2026-11-09/examples/34`:
`ess generate synthesize --target rust --layout crate --path spec --out rust-out`, exit 0,
`24 capabilities: 23 generated, 1 obligation(s), 0 refused`; the obligation is in `closed`.

Item 34 of the 2026-11-09 workbench report. Split out of
`story:generated-rust-behaviours-cover-expressions-distinct-members-and-filtered-effects` by its
fit review (points 4, 6 and 7).

## Acceptance

- `ess generate synthesize --target rust` and `--target go` on example 34 report
  `0 obligation(s)`, and the conformance suite the example synthesizes
  (`ess verify conform synthesize`) passes against the generated Rust and Go servers.
- The generated behaviour of example 34 moves every row the filter selects and no other row.
- The changelog names the change of the generated storage port as breaking for a hand-written
  store and says what such a store adds.

## Scope

- cited: crates/generate/ess-synth/src/determined.rs
- inferred: the storage port the Rust and Go targets generate

confidence: low

## Fit review

Pending for this unit. The fit review of the story it was split from names it a separate unit
because it changes the generated storage port contract (breaking for a hand-written store). An
external request: it is assessed with `.agents/skills/assessing-external-requests/SKILL.md` before
this story leaves draft. One question for it: whether the guard over the rows a selector selects
(`determined.rs:210`) generates through the same port method.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, item 34, with its example under `examples/34`.
