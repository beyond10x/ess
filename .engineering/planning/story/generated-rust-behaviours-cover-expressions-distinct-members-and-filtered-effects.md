---
format: aep.planning-md/3
id: story:generated-rust-behaviours-cover-expressions-distinct-members-and-filtered-effects
kind: story
status: active
title: Generated Rust and Go behaviours cover expression values, distinct members and counted paths
summary: Three determined behaviour constructs stay obligations in generated Rust and Go (workbench report items 30, 31, 33)
tags:
- backlog
- external-request
- from-consumer-report
- severity-low
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/condition.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/invariant.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:48:15Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
- {from: "proposed", to: "active", at: "2026-11-12T21:48:17Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
---
## Need

Generated command behaviours, Rust and Go alike, leave these constructs as obligations. The reason
text is shared: both targets take it from `crates/generate/ess-synth/src/determined.rs`.

- An expression value in `sets:` (`{expr: "subject.n + input.delta"}`), and the same construct for a
  computed value of a rank command (item 31; `determined.rs`, "an expression value").
- `distinct item in outlines by item.id as string` in a `when:`: distinct list members no generated
  behaviour compares (item 30; `determined.rs`, "distinct list members no generated behaviour
  compares").
- The guard path `title.count`, the character count of a `String` input (item 33; `determined.rs`,
  "the guard path").
- An `affects:` entry whose filter does not pin the row's identity (`where: parent_id == input.id`,
  `moves: <Entity>.<transition>`), a fan-out to every matching row (item 34). beyond10x/ess#547
  (closed, 0.63.0) made a pinned filter (`id == ...`) generate; an unpinned filter stays owed by
  design (`determined.rs`, `fn affects`: "generated storage enumerates no rows by a selector").
  Not a regression of #547.
- The server's test login names an actor but no caller attribute `name`, on 4 commands of one
  whole model (item 27). Not reproduced minimally: the minimal example generates with 0
  obligations.

Reproduction on 0.92.0 (the report's minimal examples), each `ess generate synthesize --target rust
--layout crate --path spec --out rust-out` and `--target go`, exit 0:

| example | capabilities | obligation reason in `PLAN.md` |
|---|---|---|
| 30 | 21 generated, 1 obligation | `distinct item in outlines by item.id as string`, distinct list members no generated behaviour compares, in `duplicated` |
| 31 | 20 generated, 1 obligation | an expression value (`{expr: "subject.n + input.delta"}`) for `n`, in `added` |
| 33 | 18 generated, 1 obligation | the guard path `title.count`, in `no-title` |
| 34 | 23 generated, 1 obligation | effects on filtered rows (`affects:`), in `closed` |
| 27 | 19 generated, 0 obligations | none |

Items 27, 30, 31, 33 and 34 of the 2026-11-09 workbench report.

## Fit review

1. **Need.** A command whose model fully determines its behaviour (a sum of a stored field and an
   input, a duplicate check over a list input, an empty-text guard, a state move over every child
   row) is left for the implementor to write by hand in every generated server. Requester's
   expectation (theirs): each construct generates behaviour. Minimal reproduction: the table above.
2. **Class.** Gap, not defect: each obligation is declared and accounted in `PLAN.md`; the
   behaviour is not wrong, it is missing. Item 34 is a documented limit of the storage port.
3. **Already expressible?** Each construct is valid and checked by the suite and the interpreter;
   only code generation owes it. No idiom removes the obligation.
4. **Fit.** One rule across Rust and Go, since the obligation is decided once in `determined.rs`:
   a construct generates in both targets or stays owed in both. Items 30, 31 and 33 are guard and
   value renderings over values the behaviour already holds (input and subject), which the
   existing renderers extend. Item 34 needs the storage port to enumerate rows by a selector, a
   port contract change, and is a separate unit.
5. **Second adopter.** A ledger command `Credit {amount}` that sets `balance` to
   `subject.balance + input.amount`; an order command refusing a line list with two lines of one
   SKU; a project close that finishes every open task.
6. **Cost.** No source or suite format change. Generated Rust and Go behaviours grow; an adopter
   who implemented one of these obligations by hand sees it generated (the obligation disappears
   from `PLAN.md`). Item 34 changes the generated storage port contract (breaking for a
   hand-written store).
7. **Alternatives.** (a) Change nothing: obligations are honest and accounted. (b) Generate items
   30, 31 and 33 now in both targets; item 34 as its own story over the storage port (chosen).
   (c) One story for all five: couples a port change to renderer work.

## Decisions

Accept, redesigned: Rust and Go both, not Rust only; items 30, 31 and 33 here; item 34 split out to
a story on enumerating rows by a selector in the storage port; item 27 dropped until it is
minimised.

**Accepted under the operator's procedure for external-request drafts (2026-11-12).** He set
it that day: the session that maintains the repository recommends per draft, an independent
adversarial review reads each draft, a draft both read as accept is accepted on his behalf, and
a draft where the two differ goes to him. This draft was number 36 of a second list of 13
(numbers 26 to 38); both read accept. The two transitions carry the correlation
`operator-2026-11-12-round-2` and the executor `agent:b10x`.

The acceptance is for the redesigned form above: items 30, 31 and 33, in generated Rust and Go.
Item 34 is filed as `story:generated-storage-lists-the-rows-a-selector-selects`, a draft. Item 27 is
not filed. The fit review above is the one this story owes; none is outstanding.

Corrected with this acceptance: the title and the summary, which named Rust only and the
filtered effects of item 34; and the tag `defect` is removed, since fit review point 2 reads
"Gap, not defect".

## Acceptance

- `ess generate synthesize --target rust` and `--target go` on examples 30, 31 and 33 report
  `0 obligation(s)`, and the conformance suite each example synthesizes
  (`ess verify conform synthesize`) passes against the generated Rust and Go servers.
- For example 31 the generated behaviour sets `n` to `subject.n + input.delta`; for example 30 a
  list with two members of one `id` takes `duplicated`; for example 33 an empty `title` takes
  `no-title`.

## Scope

- cited: crates/generate/ess-synth/src/determined.rs
- inferred: crates/generate/ess-synth/src/rust/behaviour.rs
- inferred: crates/generate/ess-synth/src/go/behaviour.rs
- inferred: crates/generate/ess-synth/src/rust/invariant.rs
- inferred: crates/generate/ess-synth/src/go/condition.rs

confidence: medium

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
