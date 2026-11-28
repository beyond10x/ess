---
format: aep.planning-md/3
id: story:at-the-new-format-an-outcome-may-emit-one-event-several-times-in-order-each-with-its-own-payload-and-an
kind: story
status: draft
title: At the new format an outcome may emit one event several times in order, each with its own payload, and an emission may hold only when its predicate does
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 5
---
## Need

Two facts about what one command publishes cannot be stated:

- **One event, several times.** Placing an order passes the row through two states in one command,
  and the service publishes `orders.StatusChanged` twice: `{from: "", to: created}`, then
  `{from: created, to: placed}`. An outcome's `payload:` is keyed by event
  (`crates/specify/ess-domain/src/command.rs:2633-2640`, `Outcome.payload`). A second block for one
  event is refused as `DuplicateDeclaration` ("two blocks for one event",
  `command.rs:8160-8176`). An emission entry's identity among the outcome's events is its event
  name (`crates/specify/ess-domain/src/command/set_effects.rs:286-290`). So the author states the
  last occurrence and drops the others.
- **An event only when a value changed.** `profiles.UpdateProfile` takes `display_name` and
  `enabled`, sets both, and publishes `profiles.Renamed` only when `display_name` differs from the
  stored one, and `profiles.Enabled` or `profiles.Disabled` only when `enabled` changed. Today every
  emission of a branch is unconditional. The idiom is one branch per combination of changes, ordered
  by `when_subject:` predicates. That is 2ⁿ branches for n values (8 for three), so authors leave the
  values unstored and the events unstated.

Minimal reproduction (written fresh; not run):

```yaml
    outcomes:
      - name: placed
        unknown_instance: true
        creates: orders.Order
        instance: order_id
        into: Placed
        emits:
          - {event: orders.StatusChanged, payload: {order_id: input.order_id, from: "", to: created}}
          - {event: orders.StatusChanged, payload: {order_id: input.order_id, from: created, to: placed}}
      - name: updated
        existing_instance: true
        sets: {display_name: input.display_name, enabled: input.enabled}
        emits:
          - {event: profiles.Renamed, when: subject.display_name != input.display_name}
          - {event: profiles.Enabled, when: subject.enabled == false and input.enabled == true}
          - {event: profiles.Disabled, when: subject.enabled == true and input.enabled == false}
```

Today an `emits:` entry is an event name or `{event, each: changed}` (`set_effects.rs:261-266`,
`RawEmit`), so both outcomes are refused at the entry.

## Outcome

Under **AE-D11**, option A (the default), from the new source format an `emits:` entry may be an
occurrence `{event, payload?, when?}`:

- **`payload:`** inline fills that occurrence, with the sources an outcome-level `payload:` takes
  (grid R, row "event `payload:`"). An event listed more than once must have an inline payload on
  every entry, and then has no outcome-level block. Occurrences publish in the order written, as
  `emits` already promises (`command.rs:2625`).
- **`when:`** is a predicate in the language of `when_subject:`. It reads input, the subject's fields
  and `state` before the outcome, the caller and `now` (grid R row `when_subject:`,
  `docs/design/language-grids.md`). The occurrence publishes only when it holds. On a creating
  branch, `subject.` is refused (there is no row before). A related row and a row set are refused by
  name in `when:`: they wait for navigation in guard predicates (G2), the reason the whole column
  stays closed there.
- An outcome whose every emission has `when:` and which sets, moves, creates and deletes nothing is
  refused (`EmptyChange`, `command.rs:3243-3260`). A run where no predicate holds would observe
  nothing, which is the rule's reason.
- `{event, each: changed}` takes no `when:` (its selection's `where:` is its filter). It is refused
  by name.

Below the new format every new form is refused as `unsupported_format_version`, naming it. IR: from
that format, a resolved outcome carries its occurrences (event, payload, condition) beside today's
`emits`/`payload`. Below it, today's IR bytes are unchanged (`AGENTS.md`, "Determinism and formats").
Format: decided before acceptance. Grid: the effect × target grid of G2, row `emits`
(`docs/design/generic-language-features.md:213`, `:605`), opened whole.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

1. Red today: both outcomes of the reproduction are refused (`ess specify validate` exit 1, the
   refusal at `outcomes.placed.emits[0]`). A case in `crates/specify/ess-domain/tests/` (the crate's
   existing emission tests, chosen at implementation) asserts the refusal.
2. After, at the new format: both validate (exit 0). `ess specify compile` writes the occurrences in
   order with their payloads and conditions. Every committed `examples/` model keeps its IR bytes.
3. Refused, each naming the entry:
   - an event listed twice where an entry has no inline payload;
   - an outcome-level block for an event that also has inline payloads;
   - `subject.` in `when:` on a creating branch;
   - a related row or row set in `when:`;
   - `when:` on a per-row emission;
   - an outcome whose every emission is conditional and that changes nothing;
   - any new form below the new format;
   - a `when:` that is not Boolean or reads an undeclared field (the expression checker's codes).
4. `ess verify diff` reports an added, removed or reordered occurrence and a changed condition, under
   the diff kinds the design page names (an `ess-diff` format move is decided with the source format).
5. `schemas/generated/ess.schema.json` is regenerated (`cargo xtask schema`). `task projection-check`
   passes.
6. `docs/design/generic-language-features.md` § G2 (the effects table) and
   `docs/design/language-grids.md` state the new row. `website/docs/reference/spec-versions.md`
   states the format.

Amendment (ninth gap report, 2026-11-24): a second reproduction holds three occurrences of one event in one outcome, with the payload pairs (empty, created), (created, active), (active, waiting); `ess specify compile` writes them in that order and the witness story asserts the three payloads in that order.

## Scope

- cited: `crates/specify/ess-domain/src/command.rs` (`Outcome` 2600-2660, `keyed_payload` 8155-8190,
  `EmptyChange` 3243-3260)
- cited: `crates/specify/ess-domain/src/command/set_effects.rs` (`RawEmit` 261-291)
- cited: `crates/specify/ess-compiler/src/ir.rs` (resolved outcome `emits` 951, `payload` 958)
- inferred: `crates/verify/ess-diff/src/change.rs`
- cited: `docs/design/generic-language-features.md`, `docs/design/language-grids.md`

## Owned files

`set_effects.rs` (`RawEmit::Occurrence`), `command.rs` (validation of occurrences, `EmptyChange`
widening), `ess-compiler/src/ir.rs` (resolved occurrences), `ess-diff/src/change.rs`,
`schemas/generated/ess.schema.json`, the two design pages, `website/docs/reference/spec-versions.md`.

## Not yours

Synthesis and the interpreter (`ae9b-synthesis-witnesses-every-occurrence-and-both-sides-of-a-condition`),
generated targets and docs (`ae9c-targets-emit-repeated-and-conditional-occurrences`). The format
opener: `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`
or the next decision's opener.

## Tests first

Acceptance 1 (red), 3 (one case each), 2 (IR bytes of a small model).

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes.

```console
task fmt-check
cargo clippy -p ess-domain -p ess-compiler -p ess-diff --all-targets --locked -- -D warnings
cargo test -p ess-domain --locked
cargo test -p ess-compiler --locked --test typed_diagnostics
cargo test -p ess-diff --locked
cargo clippy -p ess-cli --all-targets --locked -- -D warnings
cargo test -p ess-cli --locked --test specify_formats
cargo xtask schema
task projection-check
task test-xtask
```

`ess-cli` is gated because `ess-domain` and `ess-compiler` change. Run `ess-domain`'s targets one at
a time if the crate's run exceeds the step budget.

## Depends on

The opener of the new source format (format: decided before acceptance).

## Changelog

- `ess/<N>`: an `emits:` entry may be an occurrence with its own `payload:` and a `when:` predicate;
  one event may be emitted several times in order, and an emission may hold only when its predicate
  does. Language: G (opens the `emits` row of the effect × target grid).

## Fit review

1. **Need.** One command publishes one event type several times (passing states), and publishes an
   event only when the value it reports changed. The report gave one command that publishes three
   state changes, and change events written only on a change. It proposed no syntax.
2. **Class: gap** for repetition (`DuplicateDeclaration`, `command.rs:8160-8176`). For conditions,
   a convenience at n = 1 that becomes a gap at scale (2ⁿ branches).
3. **Expressible today?** Repetition: no. Condition: by branch product only. The reporting model
   instead dropped the stored values, which moves the figure out of the specification (policy 1).
4. **Fit.** It extends the one existing emission entry (`RawEmit`) rather than adding a construct.
   `when:` reuses the `when_subject:` language and its grid row, and `payload:` reuses the payload
   sources. Composes with `sets:` (reads before), creation (refused `subject.`), per-row emission
   (refused, named), refusals (an error outcome emits nothing, unchanged), and preserving branches
   (`story:an-outcome-that-keeps-its-subject-may-emit`, draft, decides emission on an unchanged row;
   this story adds nothing there). Targets: `ae9b`, `ae9c`.
5. **Second adopter.** An inventory service publishing `stock.level_changed` only when a level
   crosses a threshold, and twice when one command restocks and reserves.
6. **Cost.** A new source format (decided before acceptance), a suite format pair if `ae9b` changes
   step expectations, a diff kind, and IR occurrences beside today's fields.
7. **Considered.** (a) Change nothing. (b) A chain of moves `moves: [A→B, B→C]` with one event each
   (AE-D11 B): it covers repetition only where states exist, and not conditions. (c) Chosen:
   occurrences. (d) `when_changed: <field>` sugar: narrower than a predicate, and it may follow as
   sugar.

## Decisions

Accept, redesigned. **AE-D11** (open, with the owner as AQ4).

- AQ4: stays the owner's.

## Size

L, about 580 lines with tests.
