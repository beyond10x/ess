---
format: aep.planning-md/3
id: story:an-ordered-selection-may-be-counted-as-a-value-with-rows-before-this-one-and-its-head-may-be-the-subject-of-a
kind: story
status: draft
title: An ordered selection may be counted as a value, with rows before this one, and its head may be the subject of a system command
summary: 'A selection count exists as {count: changed} only, and the head of an ordered selection cannot be the subject of a system command (gap report, 2026-11-24)'
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 2
---
## Need

A waiting line: a customer reads their position, and a system step serves the customer at the head.
Two facts over one ordered selection of rows:

- a returned value counted over the rows ahead of this one in the order (the position), and
- a command whose subject is the first row of the ordered selection (serve the head).

Reproduction (brand-free): entity `Ticket {id, queue_id, taken_at, status}`; command `position`
answering the number of waiting tickets of the same `queue_id` taken before this one; system command
`serve-next` that sets `status: served` on the oldest waiting ticket of a queue.

What exists (read at `origin/main`):

- A selection's consumers `count(sel)`, `first(sel)`, `forall`, `exists` are listed in
  `docs/design/generic-language-features.md:227-234` (G2). Of them, the value forms in a branch are a
  `{related: {entity, where, field}}` read and, from `payload-source-reads-the-head-of-an-ordered-selection`
  (implemented), `{related: {entity, where, order_by, first}}` in a `payload:`, a `sets:` entry or an
  `affects[].sets:` entry (`docs/design/published-value-operand.md:78-82`).
- A count as a value exists for changed rows only: `{count: changed}`
  (`crates/specify/ess-domain/src/command/value_expression.rs:504, 616`). A count of a selection is a
  guard (`count` in `where`/`forall`, `row_set.rs:115-126`), never a value in a response, payload or
  `sets:`.
- The effect on the head row is named as the candidate follow-up in both pages: "An effect on the head
  row ... a candidate follow-up, with the same order and totality rules"
  (`docs/design/ordered-related-head.md:414`), and `affects[].where` and `instances:` read no value
  source (`published-value-operand.md:80`). `instance:` is one input name (`command.rs:7556`).
- Today's idiom for the head: the caller of `serve-next` names the ticket id. That moves the "which
  one is first" figure out of the specification into the caller, which a suite then cannot check
  (fix-by-idiom fails). A position is not expressible at all.

Reporter's syntax (theirs): `{count: {entity, where, before: {order_by}}}` and
`instance: {first: {where, order_by}}`.

## Outcome

A selection with a declared order may be counted as a value, with an optional "before this row" bound,
and may be the subject of a system command (its head). Both reuse the selection syntax the guards and
the head operand already read.

## Acceptance

Format (D77): both are grid-cell openings of the G2 selection-consumer row; they land together in the
source format that decision opens (not `ess/38` unless it names the row).

- At the new format a `{count: {entity, where, before: {order_by}}}` value validates in a `response:`
  field, a `payload:` field, a `sets:` entry and an `affects[].sets:` entry; below it each is refused as
  `unsupported_format_version`. A `before` without a total `order_by` (a declared tie-break), a count in
  a guard position, and a non-Integer target each refuse by name.
- `instance: {first: {where, order_by}}` on a system-issued command validates; an unordered selection and
  a selection whose order is not total refuse by name; an empty selection answers `unknown_instance:`.
- Synthesis witnesses the position with three rows (positions 0, 1, 2 are returned for the three) and
  the head step with the head and a later row (only the head changes). A row inserted ahead changes the
  position of the others in the next witness.
- The interpreter and the generated Rust and Go servers return the same positions and serve the same
  head; `ess generate synthesize --target rust` reports no obligation for either command.
- The IR bytes of a specification that uses neither are unchanged.

## Scope

- cited: `crates/specify/ess-domain/src/command/value_expression.rs` (sources 504, 616), `command.rs`
  (`instance` 7556), `command/row_set.rs` (selection 115-126), `crates/specify/ess-compiler/src/ir.rs`
  (`ResolvedInstance` 828), `crates/generate/ess-synth` (storage and determined behaviours),
  `docs/design/published-value-operand.md`, `ordered-related-head.md`, `language-grids.md`.
- Size: L, about 800 changed lines; split into the value half and the subject half if it overruns.

## Fit review

1. **Need.** Position in an order, and an effect on the head of an order. The reporter's syntax is
   labelled theirs.
2. **Class: gap** (both halves are named follow-ups in the design pages).
3. **Already expressible?** The head's value only (payload). A count as a value and the head as a
   subject: no. The caller-names-the-head idiom fails the fix-by-idiom test.
4. **Fit.** Grid R cells `payload`/`sets:`/`response:` for the count operand and the `instance:`
   position for the head; the siblings (`affects[].where`, `instances:`) read the same selection and
   stay refused, with the page's reason, until the effect follow-up. Targets: all.
5. **Second adopter.** A leaderboard rank; "run the oldest queued job".
6. **Cost.** Two operands, about six codes, one format step shared with its row.
7. **Alternatives.** Change nothing; caller-supplied position and head; count only, head later.

## Decisions

AC9-D7: the order of landing. A: both at the next G2 format step in one story. B: the count first, the head with `affects[].where`. C: decline, caller-supplied values. Decided: A, 2026-11-24 (the coordinator, under the owner's delegation); accepted as redesigned (the selection syntax, not the reporter's two spellings).
