---
format: aep.planning-md/3
id: story:an-each-write-may-take-repeated-identities-in-list-order
kind: story
status: implemented
title: An each write may take repeated identities in list order
summary: 'each: {repeated: last_wins} at ess/35 lifts the distinct requirement; elements apply in list order (beyond10x/ess#748 part 3)'
tags:
- external-request
refs:
- provider: github
  reference: beyond10x/ess#748
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src/plan.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/set_effects.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: schemas/generated/ess.schema.json
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T10:55:32Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-11-06T10:55:33Z", actor: "human:timo", revision: 11}
- {from: "active", to: "implemented", at: "2026-11-06T20:28:01Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

An `affects:` entry over an input list may admit repeated identities: `each: {in: input.<list>, as:
<name>, repeated: last_wins}`. Its elements apply one after another in list order, each to the row
as the elements before it left it, so a later element's write to the same row wins. Without the key
the entry still needs a declared `distinct:`, as today. From `ess/35`.

Part 3 of https://github.com/beyond10x/ess/issues/748.

## Fit review

1. **Need.** A service accepts a list that may name one row twice and applies the list in order, so
   the last element for a row wins. Minimal reproduction: `demo.board.Card {id: CardId, priority:
   Integer}`, command `SetCardPriorities {board_id, priorities: List<CardPriority>}` with `affects:
   [{entity: demo.board.Card, each: {in: input.priorities, as: p}, instance: p.id, sets: {priority:
   p.priority}}]` and no `distinct:`. Validate refuses it with `missing_declaration` at the entry's
   `each` ("no `distinct:` holds that member distinct", `crates/specify/ess-domain/src/command/set_effects.rs:3177-3195`).
   Requester's syntax, theirs: an opt-in such as `order: last_wins`.
2. **Class: gap.** The rule exists because "two elements naming one row would leave a result that
   depends on their order" (the refusal's own reason). For a service that defines that order, the
   result is not ambiguous, and the specification cannot say so.
3. **Already expressible?** Only by declaring `distinct:`, which makes the specification refuse a
   request the service accepts; a suite synthesized from it expects a refusal the service never
   answers. Not an idiom: it moves a checked figure out of the specification.
4. **Fit.** The key sits on `each:`, the construct whose rule it relaxes, beside `in`, `as` and
   `where` (`set_effects.rs:133-146`, `RawEach`). The order is the one the language already uses
   for `affects:` entries ("the entries apply in the order written", `ess/27`, `crates/specify/ess-domain/src/system.rs:285`)
   and for the elements the interpreter applies (`crates/verify/ess-conformance/src/interpret/execute/set_effects.rs:146-170`).
   It composes: an upsert entry's repeat updates the row the earlier element wrote; a move entry's
   repeat takes the transition only from a state in its `from`, else it is skipped, the `ess/27`
   skip rule; a delete entry's repeat names no row and is skipped; `where:` inside `each:` selects
   elements before order applies. `{count: changed}` and per-row events count each changed row once.
   Targets: the interpreter applies in order; synthesis writes a scenario with a repeated identity
   that observes the last element's values; generated Rust and Go apply in order or keep the
   command an obligation by name; Entity Runtime lowering refuses it by name; `ess verify diff`
   classifies adding the key as widening and removing it as narrowing.
5. **Second adopter.** A bulk "set stock levels" call where a client may resend one SKU later in
   the same batch; a playlist reorder that lists one track twice and keeps its last position.
6. **Cost.** One key, one value, source format `ess/35`; below it the key is refused as
   `unsupported_format_version` naming `ess/35`. An entry without the key keeps its IR bytes and
   digest. One `ess-diff` line, the reference and design notes, the format history row.
7. **Considered.** (a) change nothing: such a service cannot be specified; (b) drop the `distinct:`
   rule for every entry: silently changes the meaning of every existing specification; (c) the
   requester's `order: last_wins` (changed): `order` reads as a sort of the list; the key names what
   it governs, a repeated identity.

## Decisions

Accept, redesigned: the opt-in is adopted with the key renamed to `repeated: last_wins`, on `each:`.
`last_wins` is the only value in this cut; absent keeps today's `distinct:` requirement.

## Acceptance

Each line names the failing test written first.

- `crates/specify/ess-domain/tests/each_repeated.rs` (new): `repeated_last_wins_validates_without_distinct_at_ess_35`,
  `repeated_is_refused_below_ess_35`, `an_unknown_repeated_value_is_refused`,
  `without_repeated_the_distinct_requirement_holds`.
- `crates/specify/ess-compiler/tests/`: an entry without the key keeps its IR bytes and digest.
- `crates/verify/ess-conformance/tests/each_repeated.rs` (new): the interpreter applies a list naming
  one card twice and the card holds the last priority; `{count: changed}` counts it once;
  synthesis writes a scenario with a repeated identity that asserts the last value, and that
  scenario fails a target that keeps the first.
- Generated Rust and Go: apply in order, or the command is an obligation naming the key; Entity
  Runtime lowering refuses it by name (one case each).
- `crates/verify/ess-diff/`: adding the key is classified widening, removing it narrowing.
- `ess/35` row in `FORMAT_HISTORY`, `cargo xtask schema`, `cargo xtask format-history`, the
  reference and one CHANGELOG line.

## Scope

- cited: `crates/specify/ess-domain/src/command/set_effects.rs:133-146` (`RawEach`), `:151-170`
  (`Each`), `:3177-3195`, `:3316` (the `distinct:` rule); `crates/specify/ess-domain/src/system.rs:53-72, 364`
  (formats); `crates/verify/ess-conformance/src/interpret/execute/set_effects.rs:146-170, 226-340`.
- inferred: `crates/specify/ess-compiler/src/ir.rs`, `resolve.rs` (`ResolvedEach`);
  `crates/verify/ess-conformance/src/synthesize/set_effects.rs`; `crates/generate/ess-synth/src/plan.rs`;
  `crates/generate/ess-entity-runtime/`; `crates/verify/ess-diff/src/diff.rs`;
  `schemas/generated/ess.schema.json`; `website/docs/reference/spec-versions.md`.

## Plan

- unit: U3 of `wave-2026-11-06-ess` (ESS 0.89.0), its own tree.
- adversary: yes (a new semantic across validation, interpreter, synthesis and generated code).
- ess-conformance targets build only with the orchestrator's go for that build.
- shared surfaces with U2: `CHANGELOG.md` only.
- first to leave the wave if it reaches its stop limit; it then becomes the first unit of 0.90.0.
