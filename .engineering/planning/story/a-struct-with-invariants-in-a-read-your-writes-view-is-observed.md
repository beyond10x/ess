---
format: aep.planning-md/3
id: story:a-struct-with-invariants-in-a-read-your-writes-view-is-observed
kind: story
status: draft
title: A struct with type invariants in a read_your_writes view is observed as a complete subject
summary: A struct field with invariants in a read_your_writes view breaks complete-subject observation, and leaving it out is ESS-SYNTH-013 (beyond10x/ess#730)
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#730
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

A view with `consistency: read_your_writes` that publishes a field whose type is a struct with
`invariants:` makes every scenario that snapshots the complete subject unsynthesizable. The refusal's
help says to drop the field from the view; doing so is refused as `ESS-SYNTH-013`, whose help says to
publish the field in a view. Each help text leads to the other refusal.

Minimal reproduction, `ess/36`, one domain file (system file: `format: ess/36`, `system: example`,
`version: v1`, `domains: [example.span]`):

```yaml
domain: example.span
summary: A tracked span with a start and an end.
types:
  - name: example.span.Range
    kind: struct
    fields:
      - {name: start, type: Integer}
      - {name: end, type: Integer}
    invariants:
      - start >= 0
      - end >= start
entities:
  - name: example.span.Item
    identity: {name: item_id, type: String}
    fields:
      - {name: range, type: example.span.Range}
    lifecycle:
      initial: Open
      states: [Open, Stale]
      terminal: []
      transitions:
        - {name: mark_stale, from: [Open], to: Stale}
        - {name: confirm, from: [Stale], to: Open}
events:
  - name: example.span.ItemRead
    fields: [{name: item_id, type: String}]
  - name: example.span.ItemStale
    fields: [{name: item_id, type: String}]
  - name: example.span.ItemConfirmed
    fields: [{name: item_id, type: String}]
errors:
  - name: example.span.Exists
  - name: example.span.NoItem
  - name: example.span.NotOpen
  - name: example.span.NotStale
  - name: example.span.OtherRange
commands:
  - name: example.span.ReadItem
    input:
      - {name: item_id, type: String}
      - {name: range, type: example.span.Range}
    outcomes:
      - name: read
        creates: example.span.Item
        instance: item_id
        sets: {range: input.range}
        emits: [example.span.ItemRead]
        payload: {example.span.ItemRead: {item_id: input.item_id}}
      - {name: exists, existing_instance: true, error: example.span.Exists}
  - name: example.span.MarkStale
    input:
      - {name: item_id, type: String}
    outcomes:
      - name: marked
        moves: example.span.Item.mark_stale
        instance: item_id
        emits: [example.span.ItemStale]
        payload: {example.span.ItemStale: {item_id: input.item_id}}
      - {name: unknown, unknown_instance: true, error: example.span.NoItem}
      - {name: wrong-state, wrong_state: true, error: example.span.NotOpen}
  - name: example.span.Confirm
    input:
      - {name: item_id, type: String}
      - {name: end, type: Integer}
    outcomes:
      - name: other-range
        when_subject: {predicate: range.end != input.end}
        error: example.span.OtherRange
      - name: confirmed
        moves: example.span.Item.confirm
        instance: item_id
        emits: [example.span.ItemConfirmed]
        payload: {example.span.ItemConfirmed: {item_id: input.item_id}}
      - {name: unknown, unknown_instance: true, error: example.span.NoItem}
      - {name: wrong-state, wrong_state: true, error: example.span.NotStale}
views:
  - name: example.span.Items
    source: example.span.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: String}
      - {name: state, type: example.span.Item.State}
      - {name: range, type: example.span.Range}
```

`ess verify conform synthesize --path . --out suite.json` with ess 0.92.0:

```text
refused: refusal[ESS-SYNTH-001]: outcome example.span.Confirm/other-range has no scenario `example.span.Confirm/outcome/other-range`
  no witness: `example.span.Items: replay response invariant/reading observer is unsupported` is `complete subject observation`, which complete subject requires a finite exact typed observer
  help: a complete subject compares every field the view publishes, exactly, and the line above names what it cannot compare; drop that field from the view, or give it a type the exact observer compares
(the same for `example.span.Item/state/Open/refuses/example.span.Confirm`,
 `example.span.Item/state/Stale/refuses/example.span.MarkStale` and `example.span.ReadItem/outcome/exists`)
8 scenario(s) (0 authored), 4 refusal(s), written to suite.json
```

With the last view field (`range`) removed, as that help says:

```text
refused: refusal[ESS-SYNTH-013]: type example.span.Range has no scenario
  no view publishes a field position that can answer what `example.span.Range` declares of every value
    - `start >= 0`
    - `end >= start`
  help: publish a field that holds a value of this type in some view, directly or inside a list, a map, a union or an `Optional`: every position a view publishes is observed
11 scenario(s) (0 authored), 1 refusal(s), written to suite.json
```

Controls, same binary: the first document with the struct's two `invariants:` lines removed gives
`11 scenario(s) (0 authored), 0 refusal(s)`, and its four `snapshot_complete_subject` steps carry
`"example.span.Range": {"kind": "struct", "fields": [start: Integer, end: Integer]}`; the interpreter
passes all 11. So the exact observer already compares this struct field by field, and the only thing
that refuses it is the presence of invariants the descriptor does not carry.

The cause is one condition. `declarations_for` refuses every constrained type except a newtype under
the complete-subject profile (`crates/verify/ess-conformance/src/replay.rs:221-224`); the struct
descriptor it would write is `Declaration::Struct { fields }` (`replay.rs:229-234`), which has no
invariant member, exactly as `Declaration::Newtype { of }` has none.

A second, smaller fact: for the constrained types the complete subject keeps refusing (a struct that
declares `undeclared_fields: ignored`, a type with a `reading:`), the help still offers only "drop
that field from the view", which is `ESS-SYNTH-013` when the type has invariants. Publishing the
field in a second view that is not `read_your_writes` clears both today (reproduction variants `d`
and `f`: 12 scenarios, 0 refusals) and the help does not name it.

## Acceptance

- `ess verify conform synthesize` on the reproduction above writes 0 refusals; the suite holds
  `example.span.Confirm/outcome/other-range`, `example.span.Item/state/Open/refuses/example.span.Confirm`,
  `example.span.Item/state/Stale/refuses/example.span.MarkStale` and `example.span.ReadItem/outcome/exists`;
  each `snapshot_complete_subject` step's `shape.declarations` holds `example.span.Range` as
  `{"kind": "struct", "fields": [...]}` with no invariant member; and `provenance.suite_version` is
  the one the same document without the struct's invariants gets (`ess-conformance/34` on 0.92.0).
- `ess verify conform run --path <reproduction> --target interpreted --report-format 2` reports every
  scenario passed, 0 failed, 0 unsupported.
- A target that changes `range.end` of the stored row while answering the `other-range` refusal fails
  `example.span.Confirm/outcome/other-range` on the complete-subject comparison (a Rust runner test
  beside the existing `issue_308_constrained_identity_snapshot_detects_changed_row`).
- The same struct returned by a retained-result replay is still refused at synthesis: the
  retained-result profile is unchanged (existing tests in `retained_replay.rs` stay green unmodified).
- A struct with invariants that also declares `undeclared_fields: ignored`, published by the
  `read_your_writes` view, is still refused with the text naming `undeclared_fields: ignored`, and the
  refusal's `help:` names a third repair besides the two it has: publish the field in a second view of
  the entity that is not `read_your_writes`. Following that repair on the document synthesizes with
  0 refusals.
- A specification whose views publish no constrained struct keeps its suite bytes (the committed
  example suites are unchanged by `ess verify conform synthesize`).

## Scope

- cited: crates/verify/ess-conformance/src/replay.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/subject.rs
- cited: crates/verify/ess-conformance/tests/retained_replay.rs
- cited: docs/design/retained-command-results.md
- cited: website/docs/guides/verify/observations.md
- inferred: CHANGELOG.md

confidence: high

`replay.rs`: widen `structural_newtype` to every constrained body under
`DeclarationProfile::CompleteSubject`; `reading` and `undeclared_fields: ignored` keep refusing.
`synthesize.rs`: `SUBJECT_REPAIR` text. `subject.rs`: doc comments that say "newtype" only.
`retained-command-results.md`: the paragraph "Other constrained declarations still refuse".

## Fit review

1. **The need.** A value object with invariants stored on an entity cannot be published by the
   entity's immediate view without losing every scenario that compares the complete subject, and
   cannot be left out without `ESS-SYNTH-013`. Reproduced on 0.92.0 with the document above
   (`repro/run.txt`, variants `a` and `b`). The requester's proposal, theirs: "the exact observer
   compares a struct of two `Integer` fields field by field; its type invariants are checked
   separately; publishing it should not make complete-subject observation unsupported."
2. **Class: defect.** The design states that a complete subject "compares their actual represented
   values exactly and checks their structural type, without claiming to evaluate those invariants"
   and that the descriptor "carries no invariant authority"
   (`docs/design/retained-command-results.md:233-236`, `:238-244`). That reasoning was applied to
   newtypes only (commit `b05007e493`, `crates/verify/ess-conformance/src/replay.rs:221-224`); the
   struct descriptor is invariant-free in the same way (`replay.rs:229-234`), and the control run
   shows the observer comparing the same struct once its invariants are removed (`repro/run.txt`,
   variant `c`: 0 refusals, 11 passed). Two help texts that send the author to each other's refusal
   (`crates/verify/ess-conformance/src/synthesize.rs:1662-1665`, and the `ESS-SYNTH-013` row of
   `website/docs/reference/diagnostics.md:206`) contradict the purpose of a `help:` line.
3. **Expressible today?** Only with a weaker check. A second view with `consistency: eventual` that
   publishes the struct synthesizes with 0 refusals (`repro/run.txt`, variant `d`), but the
   complete subject then no longer compares `range`, and synthesis itself says of a field only an
   eventual view publishes that it "is checked against an eventually consistent read, which a
   projection that has not yet caught up with a wrong change still passes" (`repro/run.txt`,
   variant `b`, the `note:` lines). The idiom moves a checked figure out of the exact comparison, so
   by the skill's question 3 and `docs/design/generic-language-features.md` § Policy, policy 1, it
   is not an answer.
4. **Fit.** No authored construct, key or value is added, so no grid cell of
   `docs/design/language-grids.md` opens or closes; the change is in which declared types the
   complete-subject observation profile admits. Siblings: the profile already admits a constrained
   newtype (`replay.rs:221-222`); this admits the other constrained body, a struct
   (`crates/specify/ess-compiler/src/ir.rs:376-386` lists exactly these two as constrained), so the
   row "constrained declaration in a complete subject" is opened whole. The two remaining refusals
   stay and name their reason: `reading` (`replay.rs:222`) and `undeclared_fields: ignored`
   (`replay.rs:211-219`, `docs/design/typed-response-outcome-payloads.md:80`). The retained-result
   profile stays strict by design (`retained-command-results.md:236-237`). Targets: the step
   `snapshot_complete_subject` with a `kind: struct` declaration is already written and executed
   (`repro/run.txt`, variant `c`, suite `ess-conformance/34`), so Rust, Go, TypeScript and the
   interpreter read nothing new; `ess verify diff`, generated code, generated docs, the entity
   runtime and authoring see no change because no source or IR meaning changes.
5. **A second adopter.** A booking system: `example.booking.Stay` is a struct with `nights >= 1`
   and `checkout > checkin`, stored on `example.booking.Reservation`, published by the reservation
   view that a `wrong_state` cancellation is checked against. Any value object with a rule, held by
   an entity that has a refusal, meets this.
6. **Cost.** No source format, no suite format, no new diagnostic, no keyword, no migration, no
   diff classification, no generated-API change. One help text changes wording. Suites of
   specifications that were refused gain the scenarios they were missing; a suite that synthesized
   before keeps its bytes (the descriptor and step are the existing ones,
   `retained-command-results.md:239-242` makes the same argument for newtypes).
7. **Alternatives.** (a) Change nothing: the author keeps the eventual-view workaround and the
   weaker check, and two help texts stay circular. (b) Only reword the help to name the second
   view: breaks the circle but keeps the weaker check, and leaves newtype and struct treated
   differently for no stated reason. (c) Carry the invariants in the descriptor and evaluate them on
   the observed row: a new suite format and work in every runner, for an obligation
   `ESS-SYNTH-013` already assigns to the value-invariant scenarios. (d) Chosen: admit the
   constrained struct structurally under the complete-subject profile, as newtypes are, and reword
   the help for the cases that stay refused. It is the requester's proposal, taken unchanged, plus
   the help wording.

## Decisions

accept as proposed.

The complete-subject profile admits a struct with invariants by its declared fields, as it admits a
newtype with invariants; the invariants stay the obligation of the value-invariant scenarios and are
not evaluated by this observation. The retained-result profile is unchanged. Added to the request:
the help of the remaining complete-subject refusals names the second-view repair, so no help text
leads to `ESS-SYNTH-013`.

No source format version and no suite format version: no meaning, identity, reference,
canonicalization, name or envelope changes. `docs/design/retained-command-results.md` (the paragraph
at lines 230-244) is extended in the same change, before the code, to say "newtype and struct".

## Source

github:beyond10x/ess#730
