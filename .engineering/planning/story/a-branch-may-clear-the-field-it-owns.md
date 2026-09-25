---
format: aep.planning-md/2
id: story:a-branch-may-clear-the-field-it-owns
kind: story
status: draft
title: A branch may clear the field it owns
relations:
- serves: vision:O2
revision: 1
---
# Story: a branch may clear the field it owns

## Outcome

An author can write that an outcome leaves an `Optional` field holding nothing, and the synthesized
scenario asserts that rather than saying nothing.

## Context

`sets:` admits two sources — `input.<field>` and a literal (`command.rs:645-666`,
`PayloadSource::parse`). Neither can say "after this branch the field holds nothing", and plenty of
branches do exactly that: a `leave` that ends a membership, a `dispose` that finishes with a lead, a
teardown that drops a media handle.

Measured on an adopter's model (2026-09-16, ESS 0.25.0), the three ways to write it today and what
each costs:

- **Say nothing.** The field keeps whatever an earlier act determined, so the suite requires the old
  value. The adopter's `DisposeCall/disposed` scenario asserted a lead the server had cleared, and
  passed or failed depending on whether an asynchronous backend read had put it back — a red
  scenario in one run of three.
- **Write a literal.** `metrics: none` compiles and is then dropped: `settled` (`synthesize.rs`)
  abstains on a literal source because reading it at the field's declared type is a parse no
  declaration specifies. So the document says one thing and the suite another, with no diagnostic.
  The author cannot tell it did nothing without diffing the generated suite.
- **Write `""`.** Types as a `String` and means an empty string, which is a different reading from
  absent — and impossible for a struct-typed `Optional`.

The invalidation half is already fixed: a branch that writes a field drops any earlier determination
of it. That stops the wrong assertion. It does not give the author the right one.

## Acceptance

- `sets:` accepts an explicit source meaning "cleared", alongside `input.<field>` and a literal, in
  the same shape the grammar already uses for its other explicit source (`RawPayloadSource::Explicit`).
- It is refused on a field whose type is not `Optional<…>`, naming the field and its type: a required
  field cannot hold nothing, and a specification that says it can is wrong rather than surprising.
- Synthesis asserts the absence rather than abstaining — the scenario reads the view and requires the
  field to be empty, so a target that leaves the old value fails.
- `ess/1` documents it beside `input.` and the literal, with the reason a literal is not a substitute.
- A test over a two-act fixture: act one fills the field from its input, act two clears it, and the
  synthesized expectation requires it empty rather than omitting it or requiring act one's value.

## Out of Scope

A conditional `sets:` — "this value when the input says X, nothing otherwise". The adopter needs that
too (`CampaignStatusRefresh/created` carries metrics only when the reading is idle) and it is a
larger construct: a guard over a field assignment rather than over a branch. Its own story.

## Ambiguities

None.

## Open Questions

None.
