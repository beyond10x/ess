---
format: aep.planning-md/3
id: story:a-multi-state-wrong-state-error-branch-is-synthesized
kind: story
status: implemented
title: A wrong-state error branch over several states is synthesized in each of them
summary: 'when_subject_state: [A, B] on an error branch validates, but synthesis refuses it as selected in a held state (beyond10x/ess#732)'
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#732
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_listed_state_refusal_pass1.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_listed_state_refusal_pass2.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: inferred
  path: crates/verify/ess-conformance/tests/listed_state_refusal_tries_its_states.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:38:27Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:38:28Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-10T17:53:42Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Need

A refusal guarded by a held-state list, `when_subject_state: [Rejected, Stale]` on an `error:` branch,
is witnessed by trying every lifecycle state of the entity in name order, not only the states the
list names, and the refusal kept is the first state's. So where the listed states cannot be
witnessed, the refusal says the branch was not selected in a state the list does not name
(`Accepted`), and hides the real cause.

Minimal reproduction (`ess/34`, one file; `a` in the triage reproduction): `example.catalogue.Item`
with states `[Extracted, Accepted, Rejected, Stale]`, `Accepted` reached both by a creation
`into: Accepted` and by a move; the only view publishes `item_id`, `source` and `level`, not `state`.

```yaml
  - name: example.catalogue.SetLevel
    input:
      - {name: item_id, type: example.catalogue.ItemId}
      - {name: level, type: example.catalogue.Level}
    outcomes:
      - {name: not-prose, when_subject: {predicate: source != Prose}, error: example.catalogue.NotSettable}
      - name: wrong-state
        when_subject_state: [Rejected, Stale]
        error: example.catalogue.WrongState
        payload: {example.catalogue.WrongState: {state: {subject: state}}}
      - {name: set, updates: example.catalogue.Item, instance: item_id, sets: {level: input.level}, ...}
      - {name: unknown, unknown_instance: true, error: example.catalogue.NotFound}
```

```console
$ ess verify conform synthesize --path a --out a.json
refused: refusal[ESS-SYNTH-003]: outcome example.catalogue.SetLevel/wrong-state has no scenario `example.catalogue.SetLevel/outcome/wrong-state`
  no candidate of the 1 tried satisfies `wrong-state selected in held state Accepted`
```

`ess specify validate --path a` reports the document valid, as it should: an `error:` branch
guarded by a state list is documented (`website/docs/guides/specify/guards-and-predicates.md:47-54`)
and admitted at every step-3 composition from `ess/27` (`website/docs/reference/spec-versions.md:69`).

Cause, from the code: `run_as` (`crates/verify/ess-conformance/src/synthesize.rs:4774`) and its
sibling at line 6273 loop over `ir.entity(..).lifecycle.states`, a `BTreeSet`
(`crates/specify/ess-domain/src/entity.rs:351`), so `Accepted` is tried first, and keep the first
failure. `prepare_state_input` (line 8697) already narrows to the states the condition admits.

## Acceptance

- `ess verify conform synthesize` on `a` no longer reports `selected in held state Accepted` (or
  any state outside `[Rejected, Stale]`) for `example.catalogue.SetLevel/outcome/wrong-state`; its
  refusal, where it still has one, names the cause met in `Rejected`.
- With `{name: state, type: example.catalogue.Item.State}` added to the view (`b`), the suite has
  `SetLevel/outcome/wrong-state` arranged in `Rejected` or `Stale` and no refusal, and
  `ess verify conform run --target interpreted` passes it.
- The default (`otherwise:`) state refusal still tries every lifecycle state; existing tests of it
  pass unchanged, and a suite that synthesized before keeps its bytes.

## Scope

Delivered in turn 1 of the zero-open-issues plan (lane story 4). The paths below are the ones
its commits changed, read from the diff, so every entry is cited.

- cited, confirmed: `crates/verify/ess-conformance/src/synthesize.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/listed_state_refusal_tries_its_states.rs`
- cited, added: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs`
- cited, added: `crates/verify/ess-conformance/tests/adversary_listed_state_refusal_pass1.rs`
- cited, added: `crates/verify/ess-conformance/tests/adversary_listed_state_refusal_pass2.rs`
- cited, added: `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv`

Shared surface written by the coordinator, not a scope entry: `CHANGELOG.md`.

## Fit review

1. **Need.** A held-state-list refusal whose witness fails is reported against a state the list does
   not name. Reproduction above, brand-free. The requester proposed either refusing the construct
   in validation or synthesizing it like the equivalent `state ==` predicate.
2. **Class.** Defect. The guide promises the branch is arranged in each listed state
   (`guards-and-predicates.md:52-54`), and `prepare_state_input` already admits only listed states
   ("Falling back to the whole lifecycle for a condition that narrows it is the silent-admit
   failure", `synthesize.rs:8716-8719`); `run_as` does the opposite for a refusal.
3. **Already expressible?** The equivalent `when_subject: {predicate: {any: [state == Rejected,
   state == Stale]}}` synthesizes (`repro/run.txt`, case c), but the two spellings are one guard by
   design (`docs/design/generic-language-features.md:521-525`); telling authors to avoid one is no
   idiom.
4. **Design fit.** No authored surface. Validation stays as it is (the requester's first option is
   refused: the construct is documented and admitted from `ess/18`, beyond10x/ess#201, and at every
   step-3 composition from `ess/27`). The witness loop uses the states the condition admits
   (`admits_held_state`, `synthesize.rs:8590`), as the outcome path already does.
5. **Second adopter.** `example.ship.Order` refuses `Gone` with `when_subject_state: [Delivered,
   Cancelled]` (the guide's own example); with an entity whose states sort before those, a failed
   witness is reported against the wrong state.
6. **Cost.** No format, keyword or diagnostic. Refusal text changes only where the witness failed.
7. **Alternatives.** (a) Refuse the construct in validation: contradicts the guide and
   `spec-versions.md:69`, and breaks valid documents. (b) Keep the loop and report every state's
   cause: still tries states the branch cannot be selected in. (c) Chosen: loop over the admitted
   states only.

## Decisions

accept, redesigned: of the requester's two options, synthesis is fixed and validation is left
alone. The witness of a listed held-state refusal tries only the states it lists. That the listed
states need a view publishing `state` at all, where the equivalent `when_subject:` predicate does
not, is `story:a-held-state-guard-is-witnessed-without-a-view-publishing-state`. The ESS agent
plugin's `specifying` reference (`skills/specifying/references/later-formats.md`, "Two limits")
still says `when_subject_state:` is refused on an `error:` branch; that page lives outside this
repository and needs the same correction.

## Source

github:beyond10x/ess#732
