---
format: aep.planning-md/3
id: story:one-host-component-stands-in-many-slots-that-receive-the-same-thing-each-slot-identified-by-its-node-path
kind: story
status: draft
title: One host component stands in many slots that receive the same thing, each slot identified by its node path
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-host-slot-in-a-reused-template-is-one-registration
revision: 1
---
## Need

A host-drawn cell, for example a carrier's logo with its name, appears in the same column of three
tables and in one field of a detail page. Each place must be a separate host slot with its own
`host` name, because a host name is unique in the document. One component therefore registers
under four names.

- `check_host_slots` refuses a second slot of one `host`: "host slot `<H>` is declared twice, here
  and at `<path>`: a host name is unique in the document" (`crates/ui/ess-ui/src/path.rs:476-497`).
  It walks the expanded document, so a YAML alias or anchor of one slot is refused too.
- The rule is in the schema (`schemas/ui/ess-ui.schema.yaml:1799`,
  `be_the_only_slot_of_its_host_name`) and the reference (`website/docs/reference/ess-ui.md:2007`,
  `:2013`).
- The reason is in `design:ui-host-slots` § Generators: the React registry has "exactly one entry
  per declared `host` name", "typed by what that slot receives": the data adapter narrowed to the
  slot's reads and commands, and the scope state of its position
  (`.engineering/planning/design/ui-host-slots.md:29`, `:44`, § Generators).
- The active `story:a-host-slot-in-a-reused-template-is-one-registration` admits one declaration
  inside a widget or page kind used many times. Its acceptance keeps "two slots with the same `host`
  that come from different declarations … still refused".

Minimal reproduction (fresh, not run here): two collections, each with the column
`{field: carrier, host: CarrierCell, stands_in_for: text, reason: draws the carrier logo}`. `ess ui check`
reports `host_slots … declared twice`.

The requester's proposal, restated: the slot's node path identifies the slot, and `host` names the
component, which may stand in many slots.

## Outcome

A `host` name may stand in several slots when every slot of that name receives the same thing:
the same scope kind (page, section, overlay, tab or field), the same `stands_in_for`, the same
`reads` and the same `commands`. The registry keeps one entry per `host` name, typed by that shared
shape. Each slot is identified by its node path in the report, the coverage count and the
generated pages. Slots of one name that differ in any of those are refused, naming the first
difference and both paths.

## Acceptance

- Red first, in `crates/ui/ess-ui-check/tests/host_slots.rs`: the reproduction's document checks
  with exit 0 and lists `CarrierCell` at both paths. It fails before the change.
- The same document through a YAML alias for the second column checks the same way.
- Two `CarrierCell` slots whose `stands_in_for` differ (`text` and `badge`), or whose `reads` differ,
  or one a field slot and one a section slot, are refused as `host_slots`, naming the key that
  differs and both paths.
- `ess generate ui --target react` writes one `CarrierCell` entry in `src/hosts.ts`, and each
  generated use passes its own path and row (`crates/ui/ess-ui-react/tests/host_slots.rs`).
- The terminal draws the labelled placeholder at both paths (`crates/ui/ess-ui-tui/tests/host_slots.rs`).
- With `--model`, a slot's reads count as host-covered from each path, once per row of the count.
- `design:ui-host-slots`, the schema's `host_slots` rule and the reference say: a `host` names a
  component, a slot is its node path, and the slots of one name agree on what they receive.

## Fit review

Tree `bcfc51555`, read-only. Second report, G7.

1. **Need.** One host component, many places. Reproduction above.
2. **Class.** *Gap*. Uniqueness is the designed rule, so it is not a defect.
3. **Already expressible?** Partly. One widget holding the slot, used in each place, is admitted
   by the active story once it ships. A column inside three collections is not a widget body, and
   a widget cannot stand in a column's place, so the column case stays refused.
4. **Fit.** The design's reason for uniqueness is a typed registry entry. Requiring the slots of a
   name to agree on the type's inputs keeps that reason. The active story's instance-path identity
   is generalised from "instances of one declaration" to "slots of one name".
5. **Second adopter.** A map pin component drawn in a list column, a record field and an overlay
   (three scope kinds would be refused; the same field-slot shape in three tables is admitted).
6. **Cost.** No key and no version. The meaning widens: a document refused before now checks. The
   active story's acceptance on different declarations is replaced, once that story has shipped.
7. **Considered.** (a) Change nothing: four names for one component. (b) Uniqueness per
   declaration only, the active story: the column case stays refused. (c) Chosen: one name, many
   slots, one received shape.

## Decisions

**Accept, redesigned.** The requester's rule is kept, with the condition that the slots agree on
what they receive.

- **SL-D4. How does this meet the active story?** (A) A follow-up story landing after
  `story:a-host-slot-in-a-reused-template-is-one-registration`, replacing its last-but-one
  acceptance. (B) Widen that active story now. **Recommended: A.** The active story keeps its
  planned scope.

## Scope

- cited: `crates/ui/ess-ui/src/path.rs` (`check_host_slots` 476-497)
- cited: `crates/ui/ess-ui-check/src/hosts.rs`, `schemas/ui/ess-ui.schema.yaml` (1799)
- cited: `.engineering/planning/design/ui-host-slots.md`
- inferred: `crates/ui/ess-ui-react/src/emit.rs` (registry, one entry per name),
  `crates/ui/ess-ui-check/tests/host_slots.rs`, `crates/ui/ess-ui-react/tests/host_slots.rs`,
  `crates/ui/ess-ui-tui/tests/host_slots.rs`, `website/docs/reference/ess-ui.md`
- Size: M, about 330 changed lines.

## Depends on

`story:a-host-slot-in-a-reused-template-is-one-registration` (active).

## Source

A downstream application, 2026-11-24, second report, G7.
