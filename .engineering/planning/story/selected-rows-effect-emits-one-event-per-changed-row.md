---
format: aep.planning-md/3
id: story:selected-rows-effect-emits-one-event-per-changed-row
kind: story
status: draft
title: A selected-rows effect emits one event per changed row
relations:
- depends_on: story:create-or-update-selects-update-branches-by-stored-fields
revision: 1
---
## Outcome

An outcome with a selected-rows effect (`moves:` or `updates:` with `instances: {where: …}`) can
emit one event per row it changed, with a payload read from that row (its identity, a stored
field, its held state before the move), optionally only for the rows that were in named states.

Closes https://github.com/beyond10x/ess/issues/531.

## Fit review

1. **Need.** Logging out a team moves every selected seat to `Closed`; for each seat that was
   `Busy` with a call reference the service also publishes `Left {user_id, call_ref}` for that
   seat. Minimal reproduction: the issue's `closed-all` outcome, ess/24: `moves:
   demo.desk.Seat.close`, `instances: {where: team == input.team}`, `emits: [TeamClosed]` with
   `closed: {count: changed}`. Requester's syntax: none proposed beyond "an event once per changed
   row, optionally only for rows in named states".
2. **Class: gap.** `docs/design/set-effects-over-filtered-instances.md` "The constructs": a set
   outcome's `payload:` admits `{count: changed}` and refuses every single-row read (`{subject: …}`,
   `{related: …}`) under a set effect by name; an outcome's `emits:` publishes each listed event
   once (`crates/specify/ess-domain/src/command/set_effects.rs:10`, `:1172-1190`). Nothing reads a
   changed row into an event.
3. **Already expressible?** No. A binding on `TeamClosed` cannot recover which rows changed; a
   list-valued payload field cannot be filled from the selection (only `{count: changed}` is
   admitted). Per-row commands lose the atomic team logout.
4. **Fit.** Design to settle on a page: an entry of `emits:` that is per changed row, for example
   `emits: [{event: demo.desk.Left, each: changed, when: {state: Busy}}]` with payload fields
   `{changed: user_id}` / `{changed: call_ref}` / `{changed: state}`, reusing the `{subject: …}`
   value shape (state before the move, as `{subject: state}` reads it,
   `crates/specify/ess-domain/src/system.rs:231`). It must compose with `affects:` (a secondary
   set, same question), with bindings (each per-row event is a cause like any other), and with
   conformance `expect_event` (several events of one name, ordered by identity because "the order
   in which rows change" is out of scope in the set-effects page). Every target handles it or
   refuses it by name: Entity Runtime already refuses set effects (`SetEffectUnsupported`); Rust,
   Go, Web and Clap synthesis refuse set effects as `MissingRepresentation`, so only the
   interpreter, synthesis and `ess verify diff` (`outcome-set-effect-changed`) change in this cut.
5. **Second adopter.** Archiving every overdue invoice of a customer emits one `InvoiceArchived
   {invoice_id, amount}` per archived invoice for the ledger.
6. **Cost.** A new key shape under `emits:` and a new value source (`{changed: …}`): source format
   `ess/25`; suite format bump if `expect_event` gains a multiplicity rule (to settle on the page);
   one `ess-diff` classification line. No migration.
7. **Considered.** (a) change nothing: the adopter models one summary event and loses the
   per-row facts; (b) a list-valued payload filled from the selection (`{changed: [user_id,
   call_ref]}` into `List<struct>`): one event, no per-row binding; (c) per-row events (chosen
   pending the design page): each row's fact is an event a consumer can bind to, as the issue asks.

## Decisions

Accept, redesigned: the need is adopted; the spelling is chosen on a design page before code,
not taken from the issue (it proposes none).

## Acceptance

Each line names the failing test written first.

- Design page `docs/design/set-effects-over-filtered-instances.md` § "One event per changed row"
  (ess/25) chooses the key, the state filter, the value source, the event order and the suite
  format consequence.
- `crates/specify/ess-domain/tests/set_effects_per_row_events.rs`
  `per_row_event_with_state_filter_validates_at_ess_25` and
  `per_row_event_is_refused_below_ess_25_and_on_a_single_instance_outcome`.
- `crates/verify/ess-conformance/tests/interpreted_set_effects.rs`
  `interpreter_emits_one_left_per_busy_selected_row`: three selected seats, two `Busy` with a call
  reference, answer one `TeamClosed {closed: 3}` and two `Left` events.
- `crates/verify/ess-conformance/tests/set_effects.rs`
  `synthesis_witnesses_a_per_row_event_and_a_row_its_state_filter_excludes`: the suite fails a
  target that emits for the `Open` row or omits one `Busy` row.
- `crates/verify/ess-diff` test (file inferred) `per_row_event_change_is_classified`.

## Scope

- cited: `crates/specify/ess-domain/src/command/set_effects.rs` (payload sources, `{count:
  changed}`), `docs/design/set-effects-over-filtered-instances.md`.
- inferred: `crates/specify/ess-domain/src/command.rs` (`emits:` reader),
  `crates/specify/ess-compiler/src/ir.rs` (IR of the emit entry),
  `crates/verify/ess-conformance/src/interpret/` (set effect execution),
  `crates/verify/ess-conformance/src/synthesize/row_set.rs`, `crates/verify/ess-diff/`,
  `schemas/generated/ess.schema.json` via `cargo xtask schema` (a `RawSpecFile` change).

## Plan

- depends_on: the unit that opens `ess/25`.
- du estimate: 10G (ess-domain, ess-compiler, ess-cli, ess-conformance one `--test` at a time).
- adversary: yes (new construct across interpreter and synthesis).
- shared surfaces: `command.rs` `emits:` reader and `RawSpecFile` with every other ess/25 key in
  the wave (serialize the `wire.rs` edits); `synthesize/row_set.rs` with no other wave m story.
