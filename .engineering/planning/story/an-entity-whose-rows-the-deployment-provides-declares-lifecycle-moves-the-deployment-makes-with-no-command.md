---
format: aep.planning-md/3
id: story:an-entity-whose-rows-the-deployment-provides-declares-lifecycle-moves-the-deployment-makes-with-no-command
kind: story
status: draft
title: An entity whose rows the deployment provides declares lifecycle moves the deployment makes, with no command
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 4
---
## Need

A system keeps records it creates and moves by itself, such as a carrier's shipments going from
`Packed` to `InTransit` to `Delivered`. Its callers only read them. To declare the records' states, a
specification must give every lifecycle transition a command outcome that takes it, and every such
outcome an event. The author invents commands and events that describe no caller's request.

- Every declared transition needs an outcome that takes it: "`<t>` moves `<Entity>` to `<S>`, and
  no command outcome takes it, so nothing in this specification can make that state change happen"
  (`missing_causation`, `crates/specify/ess-domain/src/entity.rs:1312-1333`). The reason is in the
  function's comment: a move nothing can trigger is an unkeepable promise (`entity.rs:1183-1216`).
  The same comment declines the matching rule for creation, because entities may "arrive from a
  migration or from a system outside this document" (`:1204-1207`).
- An outcome that neither emits nor names an error is `empty_change`
  (`crates/specify/ess-domain/src/command.rs:3242-3264`).
- Two constructs come close:
  - `provided: true` on an entity (`ess/33`) says the deployment provides its rows. A conformance
    fixture prelude names a provided row `{entity, state}`, and the target's provider makes or finds
    one in that state (`entity.rs:1015`, `:1094-1106`;
    `website/docs/reference/spec-versions.md:951-954`; the cleared
    `decision-blocker:provided-entity-rows-for-conformance`, option B).
  - `system: true` on an actor (`ess/27`) makes a command system-issued: conformance awaits it
    rather than driving it. Its outcomes still emit events, and its routes are still served
    (`docs/design/system-issued-commands.md:1-20`, `:402`).
- `validate_lifecycle_causes` does not read `provided` (`entity.rs:1217-1333`), so a provided
  entity's transitions still need commands.

Confirmed. It is not a defect: the rule is documented and deliberate.

Minimal reproduction (fresh, not run here), at `format: ess/37`:

```yaml
entities:
  - name: shop.delivery.Shipment
    provided: true
    identity: {name: id, type: shop.delivery.ShipmentId}
    lifecycle:
      initial: Packed
      states: [Packed, InTransit, Delivered]
      transitions:
        - {name: dispatch, from: [Packed], to: InTransit}
        - {name: deliver, from: [InTransit], to: Delivered}
```

`ess specify validate` reports `missing_causation` at `transitions[0]` and `transitions[1]`.

The requester's proposal, restated: an entity or a transition declared as recorded by its
owner (`caused_by: owner`), with no command, written `lifecycle: {initial: Packed, recorded_by:
owner, transitions: [...]}`.

## Outcome

From `ess/39`, a transition of an entity marked `provided: true` that no outcome takes is a move
the deployment makes. It is not refused. A command outcome may still take it. Such a transition is
listed in the compiled IR as the deployment's (`caused_by: deployment` on the transition, derived,
never written). A reader of the IR, the generated documentation and `ess verify diff` can tell it
apart. Below `ess/39`, and on an entity without `provided: true`, `missing_causation` is unchanged.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

- Red first, in `crates/specify/ess-domain/tests/provided_rows.rs`: the reproduction at
  `format: ess/39` validates. It fails before the change. At `ess/37` and `ess/38` it is refused
  with today's message, and the hint adds "or mark the entity `provided: true` at ess/39, if the
  deployment makes this move".
- The same transitions on an entity without `provided: true` are refused at every format, and the
  hint names `provided: true`.
- `ess specify compile` writes the derived `caused_by: deployment` on exactly the uncaused
  transitions. A model without such a transition keeps its IR bytes and digest. An old-reader
  compatibility test shows that a reader of `ess/38` IR refuses the key by name.
- `ess verify diff` classifies a transition moving between "taken by a command" and "made by the
  deployment" as a changed transition. It is not left unclassified.
- `website/docs/reference/spec-versions.md` (the `ess/39` row) and the lifecycle guide state the
  rule. `CHANGELOG.md` names it.

## Fit review

Tree `bcfc51555`, read-only. Second report, G13.

1. **Need.** A specification describes records a system keeps by itself, with their states and
   legal moves, without inventing the requests that would cause them. Reproduction above.
2. **Class.** *Gap*.
3. **Already expressible?** Partly. Commands granted to a `system: true` actor say that the system
   takes the step, and conformance awaits it. Each still needs an event, and the commands are still
   served routes. The requester declared such commands and events for each move.
4. **Fit.** `provided: true` already says "this entity's rows come from the deployment, not from
   this document's commands". Its moves are the same fact about the same entity. Extending the
   marker adds no word. The requester's `recorded_by: owner` would be a second spelling of
   `provided`, which the skill's first red flag refuses. Conformance stays honest: a provided row in
   any state is arranged by the provider fixture (`ess-conformance/60`), not by driving a move.
5. **Second adopter.** A payments ledger whose settlements a bank moves from `Pending` to
   `Settled`. The service reads them and offers commands only to annotate them.
6. **Cost.** A format gate: a document refused before validates, so it waits for a format. By
   policy 2 it does not join `ess/38`, whose row is the member address of a struct input
   (`.engineering/waves/wave-2026-11-24.md` § Decision on the format). It joins `ess/39`, opened by
   the first report's `g15-ess39-opens`. One derived IR key. A diff classification. A hint change
   in `missing_causation` (no new code, so `macro-guards.json` does not move).
7. **Considered.** (a) Decline with the idiom (`system: true` commands plus events). (b) The
   requester's lifecycle key. (c) Chosen: `provided: true` covers the moves.

## Decisions

**Accept, redesigned.** The existing `provided` marker carries the moves.

- **SL-D11 (open, with the owner as AQ3). How does a specification declare moves only the deployment makes?** (A) Through
  `provided: true`: its uncaused transitions are the deployment's, from `ess/39`. (B) Decline with
  the idiom: commands granted to a `system: true` actor, with their events, and a `missing_causation`
  hint naming it. (C) The requester's `recorded_by: owner` on the lifecycle or a transition, from
  `ess/39`. **Recommended: A.**

- AQ3: stays the owner's.

## Scope

- cited: `crates/specify/ess-domain/src/entity.rs` (`validate_lifecycle_causes_after` 1229-1333, `provided` 1015, 1094-1106)
- cited: `crates/specify/ess-domain/tests/provided_rows.rs`
- inferred: `crates/specify/ess-compiler/src/` (the derived IR key), `crates/verify/ess-diff/src/change.rs`,
  `website/docs/reference/spec-versions.md`, the lifecycle guide under `website/docs/guides/specify/`,
  `CHANGELOG.md`
- Size: M, about 330 changed lines (unit 1 of 2).

## Depends on

`g15-ess39-opens` (first report, P2), which opens `ess/39`.

## Source

A downstream application, 2026-11-24, second report, G13.
