---
format: aep.planning-md/3
id: story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view
kind: story
status: draft
title: A branch may acknowledge and leave its addressed record unchanged without a read_your_writes view
summary: 'preserves: with instance: is refused at synthesis without an immediate identity view, and updates: with no change or accepts: nothing with instance: are refused at validation (adopter item 6, 2026-10-22); same on 0.67.0.'
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/preserve_without_view.rs
revision: 6
---
## Need

Reported by an adopter on 2026-10-22 (item 6 of the second batch, relayed by the orchestrator; adopter names kept out). In a
pub/sub protocol contract, `Publish` on a `/service/...` channel is acknowledged and changes nothing: the addressed `Client`
exists, nothing is stored, nothing is emitted. The contract has no view. The only spelling, `preserves:` with `instance:`, is
refused at synthesis because preservation is checked through a `read_your_writes` view the contract does not have; the two
alternatives are refused at validation.

Needed: an acknowledged no-op on an existing record that synthesis can check without the author declaring a view the contract
does not have (for example through the next command's own observable effect, or an outcome-local read-back), or a refusal that
names an alternative the author can write.

## Reproduction

Brand-free fixture, `format: ess/29`, domain `demo.server`: entity `Client` (identity `client_id`, field `published: Integer`),
`Handshake` creating it, and

```yaml
- name: demo.server.Publish
  input: [{name: client_id, type: demo.server.ClientId}, {name: channel, type: String}]
  outcomes:
    - name: service-channel
      when: {channel: {starts_with: "/service/"}}
      preserves: demo.server.Client
      instance: client_id
    - {name: delivered, updates: demo.server.Client, instance: client_id, emits: [demo.server.Published], ...}
    - {name: no-client, unknown_instance: true, error: demo.server.UnknownClient}
```

`ess verify conform synthesize --path spec.yaml --suite-format 5 --out suite.json`, exit 1 on 0.68.0 and 0.67.0, identical:

```text
refused[ESS-SYNTH-001]: refusal[ESS-SYNTH-001]: outcome demo.server.Publish/service-channel has no scenario `demo.server.Publish/outcome/service-channel`
  no witness: `demo.server.Client.client_id,published,state` is `demo.server.Client`, which preservation requires immediate identity views covering every subject field
  help: declare a `read_your_writes` view of the entity that projects its identity, its state and the fields named; an `eventual` view cannot be read at the moment this observation is made
```

The alternatives, re-probed on 0.68.0 and 0.67.0 (identical): `updates:` with no event and no `sets:` is
`[empty_change] ... neither emits an event nor names an error` (ESS-COMMAND-007); `accepts: nothing` with `instance:` is
`[missing_declaration] ... declares instance: client_id and none of creates, moves or updates` (ESS-COMMAND-005).

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:8088`: the preservation witness requires immediate identity
  views covering every subject field.
- Related, not the same: `story:an-outcome-that-keeps-its-subject-may-emit` (a preserving outcome that emits) and
  `story:a-delete-may-have-a-no-op-branch-that-emits-nothing` (a delete of an absent row).

## Regression status

Not a 0.68.0 regression: identical refusals on 0.67.0; no commit in `0.67.0..52bd110dd` touches `subject_fact.rs`. Gap. Does
not block 0.68.0.

## Decisions

2026-10-22: **accept, mechanism chosen; not a refusal.** The preserving branch's scenario is
emitted without a `read_your_writes` view through the fallback that already exists for refusals a
stored row selects: `preserve_selected_subject`
(`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:7944`, `Observation::Published`,
beyond10x/ess#496). The preserving branch reaches preservation at
`crates/verify/ess-conformance/src/synthesize.rs:4231` through `subject_fact::preservation`
(`subject_fact.rs:7901`, `Observation::Legacy`), which refuses in `covered`
(`subject_fact.rs:8088`) when no immediate view covers every subject field; the fix sends it to
the `Published` observation instead. The row is arranged through its creating command and the
preserving branch is sent again for it, so the scenario's outcome, error and events show the row
still exists; every subject field no view reads is named in the scenario's unobserved note.

A field without any read path stays unobserved by design: the suite cannot see it and says so in
the note, it does not invent a read. The validation-side alternatives in the Reproduction
(`updates:` with no change, `accepts: nothing` with `instance:`) are not changed here.

Boundary with `story:an-outcome-that-keeps-its-subject-may-emit` (G4): that story depends on this
one and builds its emitting preserve on this mechanism; this story covers a preserving branch that
emits nothing.

## Acceptance

Fixture: the Reproduction's `demo.server` model (no view), written by the test.

- **The scenario is emitted.** Transition: `ess verify conform synthesize` on the fixture. Before:
  exit 1, `refused[ESS-SYNTH-001] ... preservation requires immediate identity views covering every
  subject field`. After: exit 0, and the suite contains
  `demo.server.Publish/outcome/service-channel`. Test:
  `crates/verify/ess-conformance/tests/preserve_without_view.rs` (new), case
  `a_preserving_branch_without_a_view_is_synthesized`. Failing before: the case finds the refusal
  on the current tree.
- **Its note names the unobserved fields.** Transition: the same synthesis. After: the scenario's
  unobserved note names exactly `published` (the subject's fields other than identity and state,
  none of which a view reads). Same file, case `its_note_names_the_unobserved_fields`. Mutation
  check: an empty note fails the case.
- **It runs.** Transition: the suite run against the interpreted target. After: the scenario
  passes. Same file, case `the_preserving_branch_passes_on_the_interpreted_target`.
- **Models with a view keep their bytes.** Transition: the fixture with a `read_your_writes` view
  projecting `client_id`, `state` and `published`. After: the suite is byte-identical to the
  current tree's (the immediate-view observation is still chosen). Same file, case
  `a_preserving_branch_with_a_view_keeps_its_observation`; existing cases in
  `tests/subject_fact_complete_refusal.rs` keep their verdicts.

## Schedule

Release 0.70.0 (gap, wave A unit A10, with `story:a-delete-may-have-a-no-op-branch-that-emits-nothing`).
The release tag on the artifact marks it.
