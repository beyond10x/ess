---
format: aep.planning-md/3
id: story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view
kind: story
status: draft
title: 'A read-only command that addresses its row with preserves: and instance: is witnessed on a row created first, without a view covering every field of the entity'
summary: 'preserves: with instance: is refused at synthesis without an immediate identity view, and updates: with no change or accepts: nothing with instance: are refused at validation (adopter item 6, 2026-10-22); same on 0.67.0.'
tags:
- adopter-report
- defect
- ess-0.70.0
- ess-0.70.0-first
- gap
- gap-p1
- gap-reports-2026-11-24
- severity-medium
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/preserve_without_view.rs
revision: 11
---
## Need

An adopter reported this on 2026-10-22 (item 6 of the second batch). In a pub/sub protocol contract,
`Publish` on a `/service/...` channel is acknowledged and changes nothing: the addressed `Client`
exists, nothing is stored and nothing is emitted. The contract has no view. The only way to write
this, `preserves:` with `instance:`, is refused at synthesis, because preservation is checked
through a `read_your_writes` view the contract does not have. The two alternatives are refused at
validation.

What is needed: an acknowledged no-op on an existing record that synthesis can check without the
author declaring a view the contract does not have, or a refusal that names an alternative the
author can write.

### The seventh report's case (added)

A second case, brand-free, from the seventh report (attempt 3).

A downstream application has a command that only reads one row by its id. It is refused with
404 when no row has that id, and otherwise answers the row and changes nothing. Five of its
commands have this shape. Today the "not found" branch is `external:` and the success branch has
no subject. So synthesis sends the success branch a fresh witness id that no step created, and on
a real target that scenario gets the 404.

The form that says "this input names an existing row of `Card`" is `preserves: Card` with
`instance: card_id`, with `unknown_instance: true` on the refusal. It validates. Synthesis refuses
the success branch, and the `invariant/after` scenario built on it, with the cause this story
already names:

```text
refused: refusal[ESS-SYNTH-001]: outcome demo.cards.ReadCard/read has no scenario `demo.cards.ReadCard/outcome/read`
  no witness: `demo.cards.Card.label,owner,state` is `demo.cards.Card`, which preservation requires immediate identity views covering every subject field
  help: declare a `read_your_writes` view of the entity that projects its identity, its state and the fields named; an `eventual` view cannot be read at the moment this observation is made
refused: refusal[ESS-SYNTH-001]: entity demo.cards.Card has no scenario `demo.cards.Card/invariant/after/demo.cards.ReadCard/read`
  (same cause)
```

This form also settles which entity is meant: `preserves:` names it. So it is not affected by an
identity type that two entities share, which is what blocks `when_related: {via: input.card_id}`
(ESS-COMMAND-004, see the story on the refusal of an input whose identity type several entities share).

## Reproduction

Brand-free fixture, `format: ess/29`, domain `demo.server`: entity `Client` (identity `client_id`,
field `published: Integer`, one state), `Handshake` creating it (emits `HandshakeDone`), and

```yaml
- name: demo.server.Publish
  input: [{name: client_id, type: demo.server.ClientId}, {name: channel, type: String}]
  outcomes:
    - {name: no-client, unknown_instance: true, error: demo.server.UnknownClient}
    - name: service-channel
      when: {channel: {starts_with: "/service/"}}
      preserves: demo.server.Client
      instance: client_id
    - {name: delivered, updates: demo.server.Client, instance: client_id, sets: {published: {increment: 1}}, emits: [demo.server.Published], ...}
```

On 0.92.0 (same refusal as on 0.67.0 and 0.68.0; the exit status is now 0, with the suite written
without the scenario):

```text
$ ess verify conform synthesize --path spec.yaml --out suite.json
refused: refusal[ESS-SYNTH-001]: outcome demo.server.Publish/service-channel has no scenario `demo.server.Publish/outcome/service-channel`
  no witness: `demo.server.Client.client_id,published,state` is `demo.server.Client`, which preservation requires immediate identity views covering every subject field
  help: declare a `read_your_writes` view of the entity that projects its identity, its state and the fields named; an `eventual` view cannot be read at the moment this observation is made
3 scenario(s) (0 authored), 1 refusal(s), written to suite.json
```

The alternatives, as re-probed on 0.68.0: `updates:` with no event and no `sets:` is
`[empty_change]` (ESS-COMMAND-007), and `accepts: nothing` with `instance:` is
`[missing_declaration]` (ESS-COMMAND-005).

## Where it lives (refreshed at `bcfc51555`, `integration/wave-2026-11-24`)

- `crates/verify/ess-conformance/src/synthesize.rs:6055-6079`: a preserving branch calls
  `subject_fact::preservation`. Only where `has_subject_guards(command)` (`:10046-10053`, held
  state or a state change) does a `NoWitness` fall back to `preserve_selected_subject`
  (beyond10x/ess#732). A preserving branch on any other command keeps the refusal.
- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:9997-10014`: `preservation` →
  `preserve_subject` → `Observation::Legacy`. `:10166-10186` (`covered`) raises "preservation
  requires immediate identity views covering every subject field".
- `subject_fact.rs:10040-10047`: `preserve_selected_subject` (`Observation::Published`,
  beyond10x/ess#496) is the fallback the decision of 2026-10-22 chose.
- `crates/verify/ess-conformance/src/synthesize.rs:21052-21101` (`invariants`): the
  `invariant/after` family builds on the branch's scenario and repeats its refusal.
- The principle: `docs/design/cross-record-and-stored-field-guards.md:120-131`. "Declaring a view
  to satisfy synthesis would make the specification describe a read surface the system does not
  have". It is applied there to guarded rows, held state and refusals (`:277-285`). The accepting
  `preserves:` branch is the remaining position.

- Related, not the same: `story:an-outcome-that-keeps-its-subject-may-emit` (a preserving outcome
  that emits) and `story:a-delete-may-have-a-no-op-branch-that-emits-nothing` (a delete of an absent row).

## Fit review

1. **Need.** An acknowledged no-op on an existing record must be checkable without a view the
   contract does not have. The reproduction above is brand-free. The requester proposed no syntax.
2. **Class: gap.** `preserves:` with `instance:` is valid (`ess specify validate` passes). Synthesis
   cannot witness it without a view, and the help asks the author to add surface the contract
   does not have.
3. **Already expressible?** Only by declaring a `read_your_writes` view, which adds a read surface
   the protocol does not have. `updates:` with no change and `accepts: nothing` are refused (above).
   That idiom moves a checked figure into surface the specification's reader never sees, so it is
   no answer (SKILL question 3).
4. **Fit.** It reuses an existing observation (`preserve_selected_subject`, beyond10x/ess#496)
   instead of adding a key. It applies to every preserving branch without a covering view, not only
   to this adopter's command.
5. **Second adopter.** A message broker's "ping" or "heartbeat" command that is acknowledged for a
   known session and stores nothing.
6. **Cost.** No authored surface and no format version. Suites of models that already have a
   covering view keep their bytes. Models without one gain a scenario where there was a refusal.
7. **Alternatives.** (a) Change nothing, and keep the refusal with its view hint. (b) Admit
   `updates:` with no change: that reverses `empty_change`. (c) Chosen: the `Published` observation.

### The seventh report (added)

- Report: the seventh report, attempt 3. The requester's option (c) is taken: `preserves` with
  `instance:` works without a full read-your-writes view.
- Class: gap, unchanged.
- Second adopter: a "fetch one record by id, 404 when absent" endpoint. Most REST services have
  one per resource.
- Cost: no authored surface and no format. The suite major does not move, because the scenario
  uses the existing snapshot steps and the existing `PartialObservation` note. A model that
  synthesized cleanly keeps every byte. A model that had this refusal gains the scenario. The
  repository's own models may move rows of `synthesis-suite-bytes-table.tsv` where one of them
  had this refusal. That was not established by reading.
- Not counted under policy 3 of `docs/design/generic-language-features.md`, because no source
  spelling changes.

## Decisions

2026-10-22: **accept, mechanism chosen; not a refusal.** The preserving branch's scenario is
emitted without a `read_your_writes` view, through `preserve_selected_subject`
(`Observation::Published`). The row is arranged through its creating command and the preserving
branch is sent for it, so the scenario's outcome, error and events show that the row still exists.
Every subject field that no view reads is named in the scenario's unobserved note. A field with no
read path at all stays unobserved by design: the suite cannot see it, says so in the note, and
does not invent a read. The validation-side alternatives are not changed here.

Boundary with `story:an-outcome-that-keeps-its-subject-may-emit`: that story depends on this one and
builds its emitting preserve on this mechanism. This story covers a preserving branch that emits
nothing.

2026-11-09: the earlier revision scheduled this for 0.70.0. It did not ship. 0.92.0 still refuses
as above, and `tests/preserve_without_view.rs` does not exist.

2026-11-24: the seventh report's attempt 3 is a second case of this story. The decision of
2026-10-22 stands. The fallback is taken for every preserving branch, not only where the command
has held-state guards.

## Acceptance

Fixture: the Reproduction's `demo.server` model (no view), written by the test.

- **The scenario is emitted.** Transition: `ess verify conform synthesize` on the fixture. Before:
  `refused: refusal[ESS-SYNTH-001] ... preservation requires immediate identity views covering every
  subject field`, and the suite lacks `demo.server.Publish/outcome/service-channel`. After: no
  refusal, and the suite contains that scenario. Test:
  `crates/verify/ess-conformance/tests/preserve_without_view.rs` (new), case
  `a_preserving_branch_without_a_view_is_synthesized`.
- **Its note names the unobserved fields.** After: the scenario's unobserved note names exactly
  `published`. Same file, case `its_note_names_the_unobserved_fields`. Mutation check: an empty note
  fails the case.
- **It runs.** After: the scenario passes `ess verify conform run --target interpreted
  --report-format 2`. Same file, case `the_preserving_branch_passes_on_the_interpreted_target`.
- **Models with a view keep their bytes.** Transition: the fixture with a `read_your_writes` view
  projecting `client_id`, `state` and `published`. After: the suite is byte-identical to the
  current tree's, because the immediate-view observation is still chosen. Same file, case
  `a_preserving_branch_with_a_view_keeps_its_observation`. The existing cases in
  `tests/subject_fact_complete_refusal.rs` keep their verdicts.

### Acceptance, the seventh report (added)

The fixture is written by the test: `format: ess/37`, domain `demo.cards`, with no view.

- Entity `Card`: identity `card_id: demo.cards.CardId` (a newtype of `String`), fields
  `label: String` and `owner: String`, one state, and one invariant `label != ""`.
- Entity `Badge`: identity `card_id: demo.cards.CardId`. It shares the type on purpose.
- `CreateCard`: `creates: Card`, its identity taken from `input.card_id`.
- `ReadCard`: input `card_id: demo.cards.CardId`, and two outcomes:
  `{name: unknown-card, unknown_instance: true, error: demo.cards.UnknownCard}` and
  `{name: read, preserves: demo.cards.Card, instance: card_id, returns: true}`.

The acceptance checks:

- **The read is synthesized on a created row.** This is the failing test the story turns green.
  Transition: `ess verify conform synthesize` on the fixture. Before: the two refusals above.
  After: there is no refusal for `ReadCard`. The scenario `demo.cards.ReadCard/outcome/read`
  sends `CreateCard` before `ReadCard`, and its `card_id` names that created instance. Test:
  `crates/verify/ess-conformance/tests/preserve_without_view.rs`, case
  `a_read_only_command_reads_the_row_its_scenario_created`.
- **The refusal is sent an id no row has.** After: `demo.cards.ReadCard/outcome/unknown-card`
  sends an identity that no step of its scenario created. Same file, case
  `the_unknown_row_refusal_is_sent_an_identity_no_row_carries`.
- **The invariant scenario stands.** After: `demo.cards.Card/invariant/after/demo.cards.ReadCard/read`
  is in the suite, and no refusal names it. Same file, case
  `the_invariant_after_a_read_stands_without_a_view`.
- **Its note names the unobserved fields.** After: the read scenario's unobserved note names
  exactly `label` and `owner`. Same file, case `a_read_names_the_fields_no_view_shows`.
  Mutation check: an empty note fails the case.
- **It runs.** After: both `ReadCard` scenarios pass
  `ess verify conform run --target interpreted --report-format 2`. Same file, case
  `the_read_passes_on_the_interpreted_target`.
- **A model with a covering view keeps its bytes.** The existing acceptance is unchanged.
  `tests/subject_fact_complete_refusal.rs` and the held-state cases of beyond10x/ess#732 keep
  their verdicts.

Each case runs synthesis on a model of a few commands, so its expected time is well under 5 s.
The test step's bounds still apply: 60 s per test, 120 s for the step, and 10 minutes for any
build-pool job.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize.rs` (`:6055-6079`: the fallback is taken
  for every preserving branch, not only where `has_subject_guards`)
- cited: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (`:9997-10047`)
- inferred: `crates/verify/ess-conformance/tests/preserve_without_view.rs` (new: the existing
  story's four cases and the five above)
- `CHANGELOG.md` `## [Unreleased]` `### Fixed`: one line.

Size: M, about 300 changed lines, of which about 30 are source and the rest are the test.

## Depends on

Nothing. It runs beside the units of the wave of 2026-11-24:

- s3 and s4 edit other functions of `synthesize.rs` (`supply`, `fresh_identity`).
- s4 edits `subject_fact.rs` in `absent` and `unknown_identity_refusal` only.
- None of them edits the preserving-branch block or `subject_fact.rs:9997-10047`.
Delivery: decision RD-D1.
