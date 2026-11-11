---
format: aep.planning-md/3
id: story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view
kind: story
status: draft
title: A branch may acknowledge and leave its addressed record unchanged without a read_your_writes view
summary: 'preserves: with instance: is refused at synthesis without an immediate identity view, and updates: with no change or accepts: nothing with instance: are refused at validation (adopter item 6, 2026-10-22); same on 0.67.0.'
tags:
- adopter-report
- defect
- ess-0.70.0
- ess-0.70.0-first
- gap
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
revision: 9
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

## Where it lives

- `crates/verify/ess-conformance/src/synthesize.rs:5111`: a preserving branch reaches
  `subject_fact::preservation` (`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:8690`,
  `Observation::Legacy`).
- `subject_fact.rs:8859` (`covered`) refuses with "preservation requires immediate identity views
  covering every subject field" (`:8877`) when no immediate view covers every subject field.
- The fallback that already exists for refusals a stored row selects: `preserve_selected_subject`
  (`subject_fact.rs:8733`, `Observation::Published`, beyond10x/ess#496).
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

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize.rs`
- cited: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs`
- inferred: `crates/verify/ess-conformance/tests/preserve_without_view.rs` (new)

confidence: high
