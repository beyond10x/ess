---
format: aep.planning-md/3
id: story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row
kind: story
status: draft
title: existing_instance arranges the prior row when the creating branch reads a related row
summary: 'existing_instance: cannot arrange the prior row when the creating branch guards on a related row, though another command creates it unguarded (beyond10x/ess#733)'
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#733
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/existing_instance_beside_related_guards.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs
- confidence: inferred
  path: docs/design/outcome-shapes.md
revision: 5
---
## Need

A command creates an entity under an input identity, declares `existing_instance:` beside it, and
creates only after `when_related` guards on a related row pass. Synthesis refuses the
`existing_instance:` branch (`ESS-SYNTH-004`) although the creating branch's own outcome scenario
reaches that branch, and an authored scenario that sends the command twice passes on the
interpreter. The refusal also names the wrong state and gives a help line that does not apply.

Minimal reproduction (ess 0.92.0, `ess/35`): `Decision` (identity `decision_id`, field `item`,
lifecycle `Current -> Stale`); `RecordDecision` creates one unguarded; `ConfirmDecision` declares:

```yaml
- {name: already-exists, existing_instance: true, error: example.review.AlreadyExists}
- {name: missing, when_related: {via: input.confirms, exists: false}, error: example.review.Missing}
- {name: other-item, when_related: {via: input.confirms, predicate: item != input.item}, error: example.review.OtherItem}
- {name: not-stale, when_related: {via: input.confirms, predicate: state != Stale}, error: example.review.NotStale}
- {name: confirmed, creates: example.review.Decision, instance: decision_id, sets: {item: input.item}}
```

```text
$ ess verify conform synthesize --path spec --out suite.json
refused: refusal[ESS-SYNTH-004]: outcome example.review.ConfirmDecision/already-exists has no scenario `example.review.ConfirmDecision/outcome/already-exists`
  it needs an instance of `example.review.Decision` resting in `Current`, and the route runs through `example.review.ConfirmDecision/confirmed`, sent here for an input chosen before the row of `example.review.Decision` it reads, and no row the declared commands create lets that input pass over `example.review.ConfirmDecision/missing`, `example.review.ConfirmDecision/other-item` and `example.review.ConfirmDecision/not-stale`
  help: let the command creating the related row set each field those guards compare from its own input (`sets: <field>: input.<field>`), or cover the state with an authored scenario (ess-scenario/1)
10 scenario(s) (0 authored), 1 refusal(s), written to suite.json
```

The same suite holds `example.review.ConfirmDecision/outcome/confirmed`, so the creating branch is
reached. Three faults:

1. The existence segment's first call takes an input chosen before the related row
   (`creating_input`, `crates/verify/ess-conformance/src/synthesize/existence.rs:1997`) and then
   searches a row toward it (`steered`, `existence.rs:1818`), which reaches a related row only in
   its initial state; the guard needs it `Stale`. The run the creating branch's own scenario is
   built from is not reused.
2. With the entity's initial state `Draft` and `confirmed` creating `into: Current`, the refusal
   says "resting in `Draft`": `stored_first` and `creating_input` name `lifecycle.initial`
   (`existence.rs:1796`, `existence.rs:2024`), not the state the creating branch creates into.
3. The help line asks for `sets: item: input.item`, which `RecordDecision` already declares
   (`crates/verify/ess-conformance/src/synthesize.rs:3028`).

The reporter's larger specification also shows the variant "the route runs through `<creating
branch>`, which no input reaches" (`Unreachable::Unwitnessable`, `synthesize.rs:3004`) for creating
branches that have their own scenario. It did not reduce to a minimal specification; it has the
same cause (the first call is not the creating branch's own witnessed run) and the same fix.

## Acceptance

Named tests in `crates/verify/ess-conformance/tests/existing_instance_beside_related_guards.rs`,
each failing on 0.92.0 first:

- `existence_reuses_the_creating_branchs_own_run`: for the reproduction above,
  `ess verify conform synthesize` reports `0 refusal(s)` and the suite holds
  `example.review.ConfirmDecision/outcome/already-exists`; its first `ConfirmDecision` call is sent
  after a `RecordDecision` and an `ExpireDecision` of the related row; `ess verify conform run
  --target interpreted` passes every scenario.
- `existence_refusal_names_the_created_state`: in a specification where the creating branch
  creates `into: Current` from initial `Draft` and no run reaches the branch, the `ESS-SYNTH-004`
  text says "resting in `Current`".
- `existence_refusal_help_names_no_fix_already_present`: where every related creator already sets
  the compared fields from its input, the help line does not ask for `sets: <field>: input.<field>`;
  it names the authored-scenario route only.
- Every synthesis byte pin that moves (`tests/synthesis_suite_bytes_table*.rs`) is re-pinned in the
  same change, with the reason in its commit.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/existence.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/tests/existing_instance_beside_related_guards.rs
- inferred: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs
- inferred: docs/design/outcome-shapes.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A creation guarded by a related row cannot get its `existing_instance:` scenario,
   though the creation itself is reached; the refusal also misnames the state and the fix. Brand-free
   reproduction above (triage `repro/run.txt`, cases A and B). The requester proposed arranging the
   prior row through any branch that creates it (`RecordDecision/recorded` here), and the command's
   own success scenario sent twice for the other two cases, and naming the `into:` state.
2. **Class: defect.** `docs/design/outcome-shapes.md:388-394` promises the `existing_instance:`
   branch a scenario with one segment per creating branch; the comment on `steered`
   (`existence.rs:1809-1817`) says the refusal "although its own scenario reaches it" is the fault
   it exists to remove; synthesis emits `ConfirmDecision/outcome/confirmed` in the same run.
3. **Already expressible?** Only by an authored scenario: `repro/spec/scenarios/confirm-twice.yaml`
   passes on the interpreter (`11 scenarios: 11 passed`, run.txt case C), but the refusal stays
   (`1 refusal(s)`), which is beyond10x/ess#735. No idiom in the specification removes it.
4. **Fit.** No authored surface; grids R and O are untouched (`docs/design/language-grids.md`).
   It is the synthesis column for `existing_instance:` beside `when_related:` (G4 precedence,
   `docs/design/generic-language-features.md:448-526`, `existing_instance:` answers at step 2,
   before the related row at step 4). The design reuses the run the creating branch's own scenario
   is built from (`run_as`, `synthesize.rs:4758`, as `one_row` already does at
   `existence.rs:1588`), so it composes with row sets (`on_rows`, `existence.rs:2108-2125`) and
   with every creating branch of the command. Targets: synthesis only; runners execute existing
   steps.
5. **Second adopter.** A booking command that refuses a slot whose parent schedule is not
   `Published` (`when_related: {via: input.schedule_id, predicate: state != Published}`), and
   refuses a booking id already taken. Same refusal, same fix.
6. **Cost.** No source format, no new key, no new diagnostic code; the `ESS-SYNTH-004` text changes
   for this case. Suites of specifications that hit it gain one scenario where a refusal stood,
   built of existing steps; suite format unchanged. Synthesis byte pins may move.
7. **Alternatives.** (a) Change nothing: the specification keeps an unretirable refusal and an
   authored scenario is the only witness. (b) The requester's first proposal, arrange the prior row
   through another command's creating branch: it weakens the documented witness, which sends the
   identity twice through the same creating path so a target that looks up the stored row on one
   path but not another is caught (`outcome-shapes.md:388-394`); refused as the primary route.
   (c) Chosen: the segment's first call is the creating branch's own witnessed run, which is the
   requester's second proposal generalised to every case, and keeps the documented witness.

## Decisions

Accept, redesigned. The existence segment's first call reuses the run that witnesses the creating
branch's own outcome (its arrangement and input), then sends the same identity again; the refusal
that remains names the state the creating branch creates into (`into:` or the initial state) and
offers only a help line that applies. Arranging the prior row through another command's creator is
not adopted, because it drops the per-creating-path check the design requires. Design page to
extend first: `docs/design/outcome-shapes.md`, the `existing_instance:` paragraph (one sentence on
where the first call comes from). No format bump.

## Source

github:beyond10x/ess#733
