---
format: aep.planning-md/3
id: story:generated-rust-and-go-behaviours-apply-an-affects-each-entry-updating-a-held-row-and-creating-a-missing-one
kind: story
status: draft
title: 'Generated Rust and Go behaviours apply an affects: each: entry, updating a held row and creating a missing one per element'
summary: 'Rust keeps affects: each: owed (determined.rs), so an upsert per element is hand-written in Rust and Go (gap report, 2026-11-24)'
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:generated-rust-behaviours-cover-expressions-distinct-members-and-filtered-effects
- informed_by: story:at-the-new-format-an-outcome-may-emit-one-event-several-times-in-order-each-with-its-own-payload-and-an
revision: 2
---
## Need

A command that registers a second entity as it runs: `check-in` updates a `Visit` and, for each
`Guest` in its input list, updates the `Guest` row if held and creates it if not. A specification states
this today (`affects: [{entity: Guest, each: {in: input.guests, as: g}, instance: g.id, sets: {...}}]`,
ess/23, `crates/specify/ess-domain/src/command/set_effects.rs:108-135`; semantics at
`docs/design/set-effects-over-filtered-instances.md:216-226`). `ess generate synthesize --target rust`
leaves every command with such an entry an obligation, so the service's code for it is written by hand.

What exists (read at `origin/main`):

- `determined::affects` renders an entry only when its filter pins the row's identity and the branch's
  subject is supplied and not created; an `each:` entry "stays owed" (`crates/generate/ess-synth/src/determined.rs:1891-2000`).
  An entry whose list may repeat an identity (ess/35) names the key in the obligation text.
- The generated storage port already has `get` and `put` (a put "replacing what was held",
  `crates/generate/ess-synth/src/rust/behaviour.rs:414`), so an upsert is a get, a merge of the written
  fields and a put, applied per element.
- The interpreter applies each element to `work.next` in order and carries the held row's other fields
  (`set-effects-over-filtered-instances.md:404-406`).
- The active story `story:generated-rust-behaviours-cover-expressions-distinct-members-and-filtered-effects`
  covers identity-pinned filtered entries, not `each:`.

The other half of the report's marker, "an event only when the row is new", is a conditional emission:
the draft `story:at-the-new-format-an-outcome-may-emit-one-event-several-times-in-order-each-with-its-own-payload-and-an`
and its two sibling stories (witness, targets) carry it. `emits: {event, each: changed}` reaches
`instances:` only, so it does not apply here.

## Outcome

Generated Rust and Go behaviours apply an `affects:` `each:` entry: per element, in list order, the held
row is updated (only the written fields change) or the row is created in its initial state, each element
reading the row as the earlier ones left it. A command that has no other owed construct reports
`0 obligation(s)`.

## Acceptance

- `ess generate synthesize --target rust` and `--target go` on a model with an `each:` upsert report
  `0 obligation(s)` for that command (before: the `OWED` obligation), and the suite from
  `ess verify conform synthesize` passes against each generated server.
- A list of two new guests creates two rows; a list naming one held guest updates only the fields the
  entry writes and keeps the others; a list that names one identity twice with `repeated: last_wins`
  leaves the last element's values (the case at `set-effects-over-filtered-instances.md:243`); an empty
  list writes nothing and the branch still answers.
- A target that keeps the first element for a repeated identity fails the suite (a mutant is named in
  the test).
- An `each:` entry whose entity no accepting component stores, and one beside a creating subject, stay
  owed with their reason.
- Generation of a specification with no `each:` entry is byte-identical.

## Scope

- cited: `crates/generate/ess-synth/src/determined.rs` (`affects`, 1891-2000), `set_effects.rs` (the set-effect plan), `rust/behaviour.rs` and `rust/behaviour/` (rendering), `go/behaviour.rs`, `rust/store.rs` (`get`, `put`),
  `crates/specify/ess-domain/src/command/set_effects.rs` (108-135).
- Size: M, about 450 changed lines; Rust and Go go side by side.

## Fit review

1. **Need.** Generate the code of a stated create-or-update per list element. The reporter's marker is an
   obligation in Rust synthesis.
2. **Class: gap** in generator coverage (the construct is owed by name, as designed; the cost is the
   hand-written code).
3. **Already expressible?** The fact: yes (ess/23). The code: no.
4. **Fit.** Grid: the effect by target grid of G2, row `affects`, columns Rust and Go. The interpreter
   already executes it; the port has what it needs.
5. **Second adopter.** A cart that registers each product it sees.
6. **Cost.** No format step, no key; one renderer in two targets.
7. **Alternatives.** Change nothing (obligation stays); render only the single-element list.

## Decisions

AC9-D8: none needed beyond the target list. A: Rust and Go together. B: Rust first. Decided: A, 2026-11-24 (the coordinator, under the owner's delegation); accepted as proposed (generator coverage; no authored surface).
