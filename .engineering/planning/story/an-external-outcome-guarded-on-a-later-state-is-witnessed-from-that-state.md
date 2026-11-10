---
format: aep.planning-md/3
id: story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state
kind: story
status: active
title: An external outcome guarded on a later state is witnessed from that state
summary: An external outcome with when_subject state == S is witnessed from the initial state, so the model selects wrong-state and every implementation fails it (beyond10x/ess#739)
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#739
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
  path: crates/verify/ess-conformance/tests/external_narrowed_by_subject.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:39:28Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:39:29Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

An `external:` branch narrowed by `when_subject: {predicate: P}` (`ess/29`) must be witnessed
forced on a row that `P` selects (`docs/design/generic-language-features.md` § "An external branch
narrowed by a subject guard", Consumers). Suppose that command has no other branch guarded on a
stored field. Synthesis then keeps the plain witness: it creates the subject, which rests in the
lifecycle's initial state, forces the branch and expects it. `P` is false there, so the model
answers `wrong-state`, and every correct target fails the scenario, the interpreter included.

Minimal reproduction, ess 0.92.0. Parcel lifecycle `Packed -> Sealed -> Shipped`. `ShipParcel`
declares `held` (`external:`, `when_subject: {predicate: state == Sealed}`, error `ShipmentHeld`),
`shipped` (`moves: ship`, from `Sealed`) and `wrong-state`.

```console
$ ess verify conform synthesize --path spec --out suite.json
8 scenario(s) (0 authored), 4 refusal(s), written to suite.json
```

`depot.parcel.ShipParcel/outcome/held` sends `PackParcel`, forces `held` and sends `ShipParcel`,
with no `SealParcel` in between:

```console
$ ess verify conform run --target interpreted --path spec --report-format 2 --report-out report.json
  failed depot.parcel.ShipParcel/outcome/held
  8 scenarios: 7 passed, 1 failed, 0 error, 0 unsupported
      expected: outcome = held        observed: outcome = wrong-state
```

Cause: `unclaimed_external` (`crates/verify/ess-conformance/src/synthesize.rs:5253`) routes an
external branch to the row search `subject_fact::external_witness` only when
`subject_fact::uses(command)` holds. `uses` (`synthesize/subject_fact.rs:50-57`) counts
`SubjectField` and `SubjectPredicate` but not `ExternalSubject`. With any sibling branch guarded on
a stored field (`when_subject: {predicate: label == "oversize"}`), the same `held` scenario sends
`PackParcel`, `SealParcel`, then `ShipParcel`, and all 11 scenarios pass (`repro/run.txt`,
control).

## Acceptance

All cases run in `cargo test -p ess-conformance --locked --test external_narrowed_by_subject`.

- On the reproduction specification above, synthesis writes
  `depot.parcel.ShipParcel/outcome/held` with `SealParcel` (`sealed`) before the forced
  `ShipParcel`, and the interpreted target passes every scenario of the suite (0 failed,
  0 unsupported).
- On the reproduction specification without the `SealParcel` command (the `seal` transition stays
  declared, so the lifecycle validates, but no command takes it and no arrangement reaches
  `Sealed`), synthesis writes no `depot.parcel.ShipParcel/outcome/held` scenario and
  records one refusal for it with code `ESS-SYNTH-003` whose text names `held`.
- The tests in `external_narrowed_by_subject.rs` before the change pass unchanged.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` and
  `--test external_beside_held_guard` pass. Each row of `synthesis-suite-bytes-table.tsv` that moves
  is a fixture whose suite, synthesized before and after the change and compared, differs only in
  the scenario of an `external:` branch narrowed by `when_subject: {predicate: …}`; no other row
  moves.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs (`unclaimed_external`, lines 5245-5254: also route an outcome with `subject_fact::narrowing(&outcome.condition).is_some()` to `external_witness`)
- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs (`uses` line 50, `narrowing` line 99, `external_witness` line 2580, `leaves_external` line 2994: the narrowed search already exists)
- cited: crates/verify/ess-conformance/tests/external_narrowed_by_subject.rs (new tests: the narrowed branch is the command's only stored-row guard, reachable and unreachable)

confidence: high

Shared surfaces moved: re-pins the suite-bytes rows of fixtures with a narrowed external branch
whose scenario moves, and proposes its `CHANGELOG.md` sentence (Fixed).

## Fit review

1. **Need.** A synthesized scenario for an external branch narrowed by a subject guard must put the
   subject in a state the guard selects before forcing the branch. Today it does not when the
   narrowed branch is the command's only stored-row guard, so the scenario is one no correct
   implementation passes (`repro/run.txt`). The requester proposed moving the subject into the
   state first, or refusing with a code if no arrangement reaches it. That is what the design
   already says.
2. **Class.** Defect. `docs/design/generic-language-features.md` (§ `ess/29`, Consumers) says
   "Synthesis witnesses the branch forced on a row `P` selects". The interpreter answers
   `wrong-state` on the initial row (`repro/run.txt`).
3. **Already expressible?** The specification is already correct, and nothing in it needs to
   change. The only workaround is adding an unrelated subject-guarded sibling branch (the control
   in `repro/run.txt`), which changes the domain. That is no idiom.
4. **Design fit.** No authored surface and no grid cell changes (grid R, `when_subject` row,
   `docs/design/language-grids.md`). The fix makes the gate agree with the row search that
   already handles narrowing (`subject_fact.rs:2595`, `:2635`, `:3009`). Sibling constructs:
   `when_subject_state:` and `when_state_changes:` beside an external branch go through the same
   `unclaimed_external` gate, via `has_subject_guards`. The test should include one
   `when_subject: {predicate: state == S}` external branch with a later state, and no sibling.
   Targets are unaffected: only synthesis changes. The interpreter, generated Rust and Go, and
   Entity Runtime already read `P` before the provider (same design section).
5. **Second adopter.** `example.order.Order` with `Placed -> Paid -> Dispatched`. `Dispatch`
   declares `courier-declined` as `external:` with `when_subject: {predicate: state == Paid}`,
   and no other stored-row guard. Today its scenario is also sent on a `Placed` order and fails.
6. **Cost.** None in formats: same steps, same suite major. A suite with such a branch changes the
   bytes of that one scenario, from one every target fails to one the interpreter passes. No new
   diagnostic.
7. **Alternatives.** (a) Change nothing: every adopter with a sole narrowed external branch carries
   a known failure. (b) Widen `subject_fact::uses` to count `ExternalSubject`: this would also
   route every sibling branch of such a command through the stored-field arrangement (`routes`,
   `subject_fact.rs:142`), changing the bytes of scenarios that are correct today. (c) Chosen:
   route only the narrowed branch itself to `external_witness`, the search its design already
   names.

## Decisions

Accept as proposed. The narrowed external branch is witnessed on a row its predicate selects,
reached through the lifecycle's transitions, or refused by name where none is reachable. No format
change. The design section already decides this behaviour, so no design page is needed first.

- The unreachable case is `ESS-SYNTH-003`: `external_witness_searched` tries the reachable `Packed`
  row, finds no input the branch answers on, and returns `external_claimed`, a
  `GuardUnsatisfiable` cause (code 3, `synthesize.rs:1696`); `ESS-SYNTH-004` is reached only when
  no row at all can be arranged, and the initial row always can.

## Source

github:beyond10x/ess#739
