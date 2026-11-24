---
format: aep.planning-md/3
id: story:a-later-event-of-a-creating-outcome-carries-the-minted-identity
kind: story
status: active
title: A later event of a creating outcome carries the minted identity
summary: A creating outcome that mints its identity and emits two events has no payload source giving the second event the new row's id; two generated fields are two mints on 0.94.0 (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
- depends_on: story:the-suite-pair-ess-conformance-66-and-67-is-open-read-wherever-64-and-65-are-and-written-by-nothing-yet
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:30Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:32Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Need

A creating outcome that mints its identity (`creates:` with `instance: <field>` and no input of
that name) and emits two events cannot say that the second event carries the new row's identity.
Both payload fields are `{generated: true}`, each a mint of its own; the synthesized suite ties the
created row to the first listed event's field and checks the second event's field for its shape
only. An implementation that publishes two different identities passes.

Reported by an adopter on 0.90.0 and 0.93.0 (2026-11-12); reproduced on the 0.94.0 release binary.
The adopter's workaround is one event and a read of the row; a caller-supplied `input.<field>`
gives equal values but moves the choice of the identity to the caller.

Model (`format: ess/37`), entity `demo.t.Note` with identity `id`, events `demo.t.Opened` and
`demo.t.Announced`, each with a field `id`:

```yaml
- name: demo.t.Open
  input:
    - {name: text, type: String}
  outcomes:
    - name: opened
      creates: demo.t.Note
      instance: id
      sets: {text: input.text}
      emits: [demo.t.Opened, demo.t.Announced]
      payload:
        demo.t.Opened: {id: {generated: true}}
        demo.t.Announced: {id: <source>}
```

`ess verify conform synthesize` per `<source>`:

| `<source>` | result |
|---|---|
| `{generated: true}` | 1 scenario, 0 refusals; `expect_event` for `demo.t.Announced` checks only the shape of `id`; `expect_view` reads the row's `id` as the value observed in `demo.t.Opened.id` |
| `{payload: demo.t.Opened.id}` | refused, `type_mismatch` (ESS-COMMAND-002): "`{payload: <Event>.<field>}` stores a value this outcome publishes, and is a `sets:` source only" |
| `{subject: id}` | refused, `conflicting_declaration` (ESS-COMMAND-004): "a `creates:` outcome has no row before it" |
| `demo.t.Opened.id` | refused, `type_mismatch`: read as a literal |

`docs/design/minted-value-sources.md` (`ess/27`) gave `sets:` the source `{payload: <Event>.<field>}`
so that a stored value is the published one. It has no counterpart between two payload fields of
one outcome, and its "Not done" list does not name one.

beyond10x/ess#823 (2026-11-24, 0.98.0) asks exactly acceptance 1, two events of one creating outcome carrying one generated value, and beyond10x/ess#816 item 5 asks the same for sibling generated values related by an expression (`not_before == issued_at`); the fit review decides both together with story:stored-field-equals-returned-response-value (beyond10x/ess#498), in one design.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

1. From `ess/38` an event payload field may read another field this outcome publishes, `{payload: <Event>.<field>}`, and an `{expr:}` in `sets:` or `payload:` may read `payload.<Event>.<field>`; below `ess/38` both are refused as `unsupported_format_version` naming `ess/38`.
2. Synthesis expects the two event fields equal (the second read as the value observed in the
   first), and the row's identity equal to both; a target that publishes two different values
   fails the scenario.
3. The interpreter publishes one value in every field that reads it; generated Rust and Go keep the command an obligation named by the read (D103-1 A); Entity Runtime lowering refuses it by name.
4. A source that names an event the outcome does not emit, a field of another type, or itself is
   refused by name. Reads that close a cycle are refused as `conflicting_declaration` naming the cycle.
5. `docs/design/minted-value-sources.md` and the reference page for payload sources state the form.
6. #816 item 5: `expires_at: {expr: "payload.<Event>.issued_at + 300s"}` validates at `ess/38`, the interpreter stores the offset value, and synthesis expects it as the observed `issued_at` plus 300 s (suite pair `/66`, `/67`, D103-2 A).

## Gate

One at a time; every test under 60 s, the test step under 120 s, no job in the build pool over 10
minutes.

- `task test-xtask`
- `cargo test -p ess-compiler --locked --test typed_diagnostics`
- `cargo xtask schema` and `task projection-check`, only if `RawSpecFile` changes (it should not:
  both spellings parse today)
- the sources stamp, rewritten in this change:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`
- the byte pins of `AGENTS.md` ("A gate of named test targets includes the repository-wide byte
  pins"), each alone
- the bounded slow lane:
  `SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table -- --ignored every_repository_model_keeps_its_synthesized_bytes`

## Depends on

- `story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess`
- `story:the-suite-pair-ess-conformance-66-and-67-is-open-read-wherever-64-and-65-are-and-written-by-nothing-yet`
  (D103-2 A: the offset's suite value is written at `/66` and `/67`)

## Size

L (validation, compiler, interpreter, synthesis, a suite value in three runners, docs).

## Fit review

1. **Need, apart from syntax.** One value an outcome mints must reach several places of that
   outcome: two events of a creating outcome carry the created row's identity (beyond10x/ess#823:
   an account and its owner announced by two events, each naming the other's new identity), and
   stored timestamps are related to one minted instant (beyond10x/ess#816 item 5: `not_before ==
   issued_at`, `expires_at == issued_at + 300 s`). Today each `{generated: true}` is a mint of its
   own, and the suite checks the second field's shape only, so a target that publishes two
   identities passes. Minimal reproduction `run/mint/system.yaml` (`format: ess/37`): `mint.t.Open`
   creates `mint.t.Note`, emits `Opened {id: {generated: true}, issued_at: {generated: true}}` and
   `Announced {id: {payload: mint.t.Opened.id}}`:

   ```console
   $ ess specify validate --path mint
     - error[ESS-COMMAND-002]: `{payload: <Event>.<field>}` stores a value this outcome publishes, and is a `sets:` source only
   ```

   The requesters' syntax, theirs: `{payload: <Event>.<field>}` inside another event's payload "or
   a named generated value both can reference", with a cycle check (#823); a relation between
   sibling generated values (#816 item 5).
2. **Class.** Gap. No source names a value the same outcome publishes from a payload
   (`docs/design/minted-value-sources.md:46-62` admits `{payload: …}` in `sets:` only, and refuses it
   in a payload, row 1 of its table), and `{expr:}` reads `input.<path>` and an existing subject only
   (`website/docs/reference/spec-versions.md`, `ess/27` row: "reading `input.<path>` and, on an
   existing subject, `subject.<field>`"). Not a defect: the refusal is documented.
3. **Already expressible?** In part.
   - Stored copies of one mint: yes. `run/mint-sets/` and `run/mint-inv/` store
     `issued_at: {payload: mint.t.Opened.issued_at}` and `not_before: {payload:
     mint.t.Opened.issued_at}` with the entity invariant `not_before == issued_at`: valid, 2
     scenarios, both pass on the interpreted target. So #816 item 5's equality half has an idiom
     whose figure the suite checks (`expect_view` reads both stored values).
   - Two events carrying one mint (#823): no (refusal above). The one-event workaround moves the
     figure out of the second event, so it is no answer (policy 1).
   - A stored value offset from a mint (`expires_at == issued_at + 300 s`): no. `{expr:}` cannot read
     the mint, and `now` is refused as a value (same row).
   - The created identity by name: the row's identity is the first listed event's `{generated:
     true}` field already (story table, `expect_view` reads the row's `id` as the value observed
     in `demo.t.Opened.id`), so naming that payload field names the identity; no source of its own
     is needed.
4. **Fit.** Grid R (`docs/design/language-grids.md:15-61`) has no column for "a value this outcome
   publishes". Two cells of it exist without a column: `sets:` reads `{payload: <Event>.<field>}`
   (`ess/27`, #545) and a selector's `where:` reads `payload.<Event>.<field>` (`ess/27`, #549, "values
   that read each other are refused"). The decision opens the column whole, under the name "own
   payload":
   - yes: `sets:` (exists), selector `where:` (exists), event `payload:` (new: a field may read a
     field of another event the outcome fills, or another field of the same event), and `{expr:}`
     in `sets:` and `payload:` (new: the path `payload.<Event>.<field>`, the spelling selectors use);
   - n/a: every guard (`when:`, `when_subject:`, `when_related:`, row-set `where:`), because a guard
     decides before the outcome publishes anything; error `payload:` (a refusal publishes no event);
     views, invariants and binding positions (no outcome there).
   It reuses both existing spellings and adds none. It composes with the response column
   `story:stored-field-equals-returned-response-value` (#498, step 104) opens (`{response: f}` in
   `sets:`): two columns, "own payload" and "own response", one per thing an outcome emits. Rules:
   the read names an event the outcome fills and a top-level field given a source there
   (`undeclared_reference` otherwise, as in `sets:`); the types are assignable without a
   conversion (`type_mismatch`); a field never reads itself, and reads that close a cycle are
   refused as `conflicting_declaration` naming the cycle (the selector rule's wording). Targets:
   the interpreter publishes one value in every place; synthesis expects the reading field equal
   to the value observed in the read one (`ScenarioValue::Observed`,
   `crates/verify/ess-conformance/src/scenario.rs:1965`), and an offset as that value plus the
   duration; generated Rust and Go keep the command an obligation named by the read, as they do
   for `{payload: …}` in `sets:` (`minted-value-sources.md:94-95`); Entity Runtime lowering
   refuses it by name; `ess verify diff` reports a changed payload source with the existing
   payload-change kind (I don't know its name; the story's implementor names it, or the DIFF
   choice of `FORMAT.md` applies if a new kind is needed).
5. **A second, unrelated adopter.** An order system whose `PlaceOrder` creates an order and a first
   shipment and emits `OrderPlaced {order_id, shipment_id}` and `ShipmentCreated {shipment_id,
   order_id}`; a warehouse binding reacts to the second event and must receive the order's id.
6. **Cost.** Source format `ess/38` (D77, the pull request's one format). No new key, no new
   spelling: two existing spellings admitted at more positions. No new `ValidationCode`; refusal
   texts change ("is a `sets:` source only" goes). Suite: if `expect_event` already admits an
   `Observed` value, the equality needs no suite format; the offset needs an observed value plus a
   duration, which is new vocabulary, written at `ess-conformance/66` and `/67` (step 49). IR: the
   existing `ResolvedPayloadValue` payload-read variant at a new position, plus the path in the
   expression IR; models without it keep their bytes.
7. **Considered.**
   - Change nothing, document the one-event workaround: rejected; the second event's figure is
     unchecked (policy 1).
   - A source for the created identity (`{created: id}`): rejected; it covers #823's identity case
     only, not a second minted value (`person_id`) nor #816 item 5, and adds a spelling.
   - A named generated value both places reference (the requester's second option): rejected; a
     new declaration for what a payload field already is.
   - Admit `{payload: …}` in payloads and `payload.<Event>.<field>` in `{expr:}` (chosen): the
     requester's first option taken as asked, widened to the expression position so #816 item 5
     is covered by the same column.


## Evidence

Model, script and output: session scratch `ess/adopter8/` (`run.sh`, `run.txt`; not in the
repository).

## Decisions

- **Accept, redesigned.** #823's form is taken as proposed (`{payload: <Event>.<field>}` in
  another event's payload, cycle refusal). It is widened to the whole "own payload" column of grid
  R: `{expr:}` in `sets:` and `payload:` reads `payload.<Event>.<field>`, which covers #816 item 5
  (equality already has the `sets:` idiom; the offset needs the expression). #498 keeps its own
  form (`{response: f}` in `sets:`, step 104); the two land in the same format.
- Changed from the request: no named generated value; the created identity is named through the
  payload field that publishes it.
- Changed from the story's acceptance 3: generated Rust and Go keep the command an obligation named
  by the read (precedent `minted-value-sources.md:94`), where the story asked them to publish one
  value. Choice D103-1 below.
- Format: `ess/38` (D77). Below it each new position is refused as `unsupported_format_version`
  naming `ess/38`.
- D103-1 (decided: A, 2026-11-24): generated Rust and Go.
  - A (decided): an obligation named by the read, as for `{payload: …}` in `sets:`.
  - B: generate it (render the minted value into a local before the row and the events, the
    "labelled later step" of `minted-value-sources.md:94-95`), for `sets:` and payloads together.
- D103-2 (decided: A, 2026-11-24): the offset (`expires_at == issued_at + 300 s`).
  - A (decided): in this story, with its suite value at `/66` and `/67`.
  - B: equality only here; the expression read is a follow-up story, and #816 item 5 stays open
    for its offset half.
- Tell the requesters: #823 accept, redesigned (its form, plus the expression read); #816 item 5
  accept, redesigned (equality: the `sets:` idiom works today; offset: the expression read).

