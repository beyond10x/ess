---
format: aep.planning-md/3
id: story:synthesize-stored-field-guards-without-a-view
kind: story
status: active
title: Synthesize subject-fact scenarios for an entity no view observes, noting the unobserved fields
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-10T18:36:42Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-10T18:36:43Z", actor: "human:timo", revision: 4}
---
## Outcome

A command whose branches are selected by a stored field of its subject (`when_subject:`) gets
synthesized scenarios for every branch even when the specification declares no view of that entity,
because the system it models offers no way to read the record back. Synthesis already arranges such
a row through the creating command and its `sets:` mapping; today it refuses the scenario only
because no view can observe the arranged row. After this story the scenario runs without that
observation, the outcome and events stay as its evidence, and a note names the fields no view
observes. Requested in https://github.com/beyond10x/ess/issues/496.

## Fit review

1. **Need, apart from the syntax.** A record bound to whoever created it (an order bound to the
   buyer who placed it) is guarded on that stored field (`placed_by != input.collector`), and the
   system offers no read of the record, so the specification declares no view. ESS validates the
   model but synthesis refuses every scenario that needs the stored field. Minimal reproduction:
   `.engineering/repro/496/orders.yaml` (`ess 0.56.0`):
   ```
   $ ess specify validate --path orders.yaml
   catalog v1 — 1 file(s), valid
   $ ess verify conform synthesize --path orders.yaml --out suite.json
   refused: refusal[ESS-SYNTH-001]: outcome catalog.orders.Collect/wrong-collector has no scenario `catalog.orders.Collect/outcome/wrong-collector`
     no witness: `catalog.orders.Order.placed_by` is `catalog.orders.Order`, which subject fact selection requires an immediate unfiltered identity/state/fact view, or an `eventual` one it waits for, and no view projects them
   refused: refusal[ESS-SYNTH-001]: outcome catalog.orders.Collect/collected ...
   refused: refusal[ESS-SYNTH-001]: transition catalog.orders.Order/collect ...
   refused: refusal[ESS-SYNTH-001]: entity catalog.orders.Order has no scenario `catalog.orders.Order/state/Collected/refuses/catalog.orders.Collect`
   2 scenario(s) (0 authored), 4 refusal(s), written to suite.json
   ```
   (`.engineering/repro/496/synth-noview.txt`). The requester's proposal, labelled as theirs:
   "where a stored field is set only from the creating command's input and the identity is carried
   by an emitted event, let synthesis arrange the subject the way the authored scenario does: run the
   creating command with inputs it chooses, capture the identity, and use the chosen inputs as the
   stored facts."

2. **Class.** A gap in generated coverage, not in the language. ESS does what its design says:
   "The view is part of the rule … without one the witness is refused"
   (`docs/design/cross-record-and-stored-field-guards.md:120-126`), and `covered` refuses when
   "nothing observing the row at all" (`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:7176-7213`).
   So this is not a defect. The fix adds no authored surface, so the class gate on new surface does
   not apply.

3. **Can it already be expressed?** The model, yes: it validates as written (above). Coverage today
   has two idioms. (a) Declare a view: `.engineering/repro/496/orders-with-view.yaml` adds one
   `read_your_writes` view and gets `6 scenario(s) (0 authored), 0 refusal(s)`, and all 6 pass
   against `--target interpreted` (`synth-view.txt`). That view claims a read surface the modelled
   system does not have, so the specification would describe something false. (b) An authored
   `ess-scenario/1` scenario (the issue shows one passing). That works but leaves the adopter to
   write by hand what synthesis already knows how to build. The arrangement synthesis would need
   already exists. Goal-directed input choice through `sets:` (`cross-record-and-stored-field-guards.md:233-241`)
   and arranging through the creating command plus `capture_instance` (`docs/design/owned-subject-arrangement.md:35-74`)
   are both implemented: the view-backed suite's `wrong-collector` scenario runs `Place`,
   `capture_instance` and then `Collect`. Removing every view step from that suite and running it
   against the view-less model passes all 6 scenarios (`.engineering/repro/496/suite-hand-noview.json`:
   "6 scenarios: 6 passed, 0 failed, 0 error, 0 unsupported"). The only obstacle is the observation.

4. **Fit with what is there.**
   - Vocabulary: the change adds no key, keeps the step kinds and refusal codes, and introduces no new
     concept. It broadens the existing `Note::PartialObservation`
     (`crates/verify/ess-conformance/src/synthesize.rs:286-295`, Display at `:435-450`). That note
     already lets a wrong-state refusal observe only what views publish (beyond10x/ess#132,
     `CHANGELOG.md:1870-1872`).
   - Composition: guards `when_subject:` (predicate and `{field, equals}`), the wrong-state witness
     (`subject_fact::refusal_witness`) and the success and transition scenarios all reach the same
     `observe_fields`/`covered` paths (`subject_fact.rs:324,1998,2462,4913,5723,5888,6818,6884`), so
     one rule covers every sibling family. The **absent-subject** witness keeps its immediate-view
     requirement (`subject_fact.rs:5398-5404`). Without a view it cannot prove that no row exists,
     and that requirement is correct. The replay family's `observe` (`subject_fact.rs:5250-5262`)
     is out of scope and keeps refusing.
   - Siblings: one rule covers both cases. With no view at all, or with views that publish only some
     of the guarded fields, synthesis observes what is published and names the rest. Today #132
     allows that for refusals only, and success branches refuse. That asymmetry goes away.
   - Targets: only synthesis changes. The interpreter, generated Rust/Go/TypeScript/web, Entity
     Runtime and `ess verify diff` see no new construct (inferred: no IR or format change). The suite
     carries fewer `query_view`/`expect_view`/`snapshot_complete_subject` steps, all of them
     existing kinds.

5. **Second, unrelated adopter.** A review queue where an approval request is bound to the reviewer
   assigned when it is filed, and only that reviewer may approve it. Requests are submitted and
   approved through write-only endpoints and never listed. Its shape is `sets: {reviewer: input.reviewer}`
   and `when_subject: {predicate: reviewer != input.approver}`, with no view. The same refusal
   applies.

6. **Cost.** No `ess/N` bump and no new keyword, because the authored language does not change. No
   new refusal code or diff classification. The suite format stays at `ess-conformance/34`
   (inferred). Notes are not persisted in the suite: `suite.json` provenance holds only
   `suite_version, system, specification_version, spec_digest, contract_digest, scenario_initial_state`.
   Suites regenerated from models that were refused before now hold more scenarios. Suites whose
   models have a covering view keep their bytes. The #172 precedent (eventual-view fallback,
   `CHANGELOG.md:1810-1812`) also added scenarios without gating on format. One existing test flips:
   `without_a_view_observing_every_guarded_field_the_arrangement_is_refused`
   (`crates/verify/ess-conformance/tests/stored_field_guards.rs:552-563`). The documented contract
   changes: the design paragraphs at `cross-record-and-stored-field-guards.md:120-126` and `:261-272`
   are rewritten.

7. **What else was considered.**
   - *Change nothing* (authored scenario, or declare a view). Rejected as the answer. The view
     idiom makes the specification assert a read surface the system lacks. The authored idiom leaves
     uncovered a branch that synthesis can already arrange.
   - *The requester's proposal* (a new arrangement path for "set only from input, identity carried by
     an event"). Changed. The arrangement it describes already exists for every `sets:`-from-input
     field, so a second path would duplicate it. The narrower condition in the proposal would also
     leave a view with partial coverage still refusing.
   - *A new authored marker for unobservable fields.* Rejected because it adds surface for something
     synthesis can infer from the absence of a covering view.
   - *Chosen:* drop only the observation requirement for subject-fact arrangement. Observe what the
     declared views publish and name the remainder in `Note::PartialObservation`. This adds no
     surface and extends an existing construct.

## Decisions

**Accept, redesigned.**

- Design: where subject-fact selection (success, refusal, transition and `state/<S>/refuses/<command>`
  scenarios) has no immediate or `eventual` unfiltered view covering identity, state and the guarded
  fields, synthesis still writes the scenario. It arranges the row through the creating command and
  its `sets:` mappings, as it does today, and omits the observation steps no view can make. It
  observes whatever a declared view does publish. Every guarded or subject field left unobserved is
  named in `Note::PartialObservation`, broadened from refusals to every subject-fact scenario and
  reworded so it reads for both. The scenario's evidence is its outcome, error and events.
- Kept: the absent-subject witness still requires an immediate view, and the replay family's
  `observe` keeps its refusal.
- Changed from the request: no new arrangement path and no conditions on how the identity is
  carried. The existing arrangement is reused. The proposal's "view stays the authority where one
  exists" holds because a covering view is still used whenever one exists.
- Spec first: the change goes into `docs/design/cross-record-and-stored-field-guards.md` (Observation
  paragraphs, lines 120-126 and 261-272) before code. No `ess/N`, IR or suite-format change.

- 2026-10-08: `synthesize/subject_fact.rs` changed on `integrate/selection-plan` (32296a1e92, stored-row synthesis reads the precedence plan) before this story starts. The unit branches from that commit, lands in its own wave after `integrate/selection-plan` reaches `main`, and does not touch `synthesize/related_guard.rs`, `related_guard/stored.rs` or `synthesize/row_set.rs`.

## Acceptance

- `ess verify conform synthesize` on a view-less model shaped like `.engineering/repro/496/orders.yaml`
  reports `6 scenario(s) (0 authored), 0 refusal(s)`, with a `PartialObservation` note naming
  `placed_by`. A new test in `crates/verify/ess-conformance/tests/stored_field_guards.rs` checks this
  (e.g. `without_any_view_every_subject_fact_branch_is_synthesized_and_noted`).
- That suite passes against `--target interpreted`, with 6 passed (same test, or an `ess-cli`
  integration test running `ess verify conform run --target interpreted --report-format 2`).
- An implementation that ignores the stored guard fails the view-less `wrong-collector` scenario. A
  mutant test in `crates/verify/ess-conformance/tests/connective_and_source_mutants.rs` drops the
  `when_subject:` predicate and the suite kills it.
- `without_a_view_observing_every_guarded_field_the_arrangement_is_refused` is rewritten to assert
  that the scenario exists, observes the published fields and notes `weight_kg`.
- The absent-subject witness still refuses without an immediate view (an existing or new test in
  `stored_field_guards.rs`).
- Committed generated suites in `examples/` are unchanged (`task check`'s projection/drift check).
- `cargo test -p ess-conformance --locked` and `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
  pass.

## Scope

- `docs/design/cross-record-and-stored-field-guards.md`, Observation paragraphs at :120-126 and
  :261-272 (cited).
- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs`: `observe_fields` (:5270),
  `covered` (:7176), the callers at :324, :1998, :2462, :4913, :5723, :5888, :6818, :6884 (cited).
  `absent` (:5398) and `observe` (:5250) stay unchanged (cited).
- `crates/verify/ess-conformance/src/synthesize.rs`: `Note::PartialObservation` doc and Display
  (:286-295, :435-450), and the `ESS-SYNTH-001` help text that points at declaring a view (cited).
- `crates/verify/ess-conformance/tests/stored_field_guards.rs` and
  `crates/verify/ess-conformance/tests/subject_fact_complete_refusal.rs`, which assert on the note
  (cited; the second is inferred to need a text update).
- `CHANGELOG.md` `[Unreleased]` (inferred).
- Held files:
  - `crates/verify/ess-conformance/src/synthesize.rs`: **yes**.
  - `src/synthesize/**`: **yes** (`subject_fact.rs`).
  - `src/interpret/execute.rs`: no.
  - `interpret/execute/{related,existence}.rs`: no.
  - `crates/specify/ess-domain/src/command.rs` and `command/**`: no.
  - `crates/specify/ess-compiler/src/ir.rs` and `ir/**`: no.

## Related finding

ESS-SYNTH-008 ("`TestStrategy` and `OutcomeCondition` have drifted apart") reproduces on 0.56.0. The
trigger is `one_time_response:` beside an **`unknown_instance:`** sibling, not `wrong_state:`.
`.engineering/repro/496/synth008.yaml` (both siblings) and `synth008-no-wrongstate.yaml` both
refuse:

```
refused: refusal[ESS-SYNTH-008]: outcome catalog.items.Publish/published has no scenario `catalog.items.Publish/disclosure/published/share_key/command/catalog.items.Publish/unknown/as/actor/catalog.items.Editor`
  its strategy is `send_unknown_identity` and it declares no guard
  help: `TestStrategy` and `OutcomeCondition` have drifted apart in `ess-domain`
```

`synth008-no-unknown.yaml` (keeping `wrong_state:`, dropping `unknown_instance:`) gives
`12 scenario(s) (0 authored), 0 refusal(s)`. The disclosure family appears to build a scenario per
sibling outcome and has no arm for `send_unknown_identity` (inferred; the `help:` falls through at
`crates/verify/ess-conformance/src/synthesize.rs:1198-1200`).
