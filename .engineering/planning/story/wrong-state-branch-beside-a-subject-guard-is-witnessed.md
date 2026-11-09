---
format: aep.planning-md/3
id: story:wrong-state-branch-beside-a-subject-guard-is-witnessed
kind: story
status: draft
title: A wrong-state branch beside a subject guard is witnessed
summary: 'Synthesis''s wrong-state family skips a command that also has a subject guard: StateBesideWrongState/not-held gets no scenario and no refusal at ess/27 and ess/30, so the suite is silent on a branch the explorer reaches; explore_guards_first.rs pins the gap as expected. On origin/main since 9572af9ba. (adversary pass 2026-10-22, F2).'
tags:
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/explore_guards_first.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/wrong_state_beside_subject_guard.rs
- confidence: inferred
  path: synthesis
revision: 5
---
## Problem

Found by the 0.68.0 release adversary pass (2026-10-22, finding F2; `review-result:release-0680-adversary-pass-1`),
still present on 0.92.0. Synthesis's wrong-state family (`<entity>/state/<S>/refuses/<command>`) skips every command
that also has a subject guard (`when_subject_state:` or `when_state_changes:`): the `movers` map in
`crates/verify/ess-conformance/src/synthesize.rs` filters such commands out (`.filter(|driver|
!has_subject_guards(driver.command))`, ~:15119; `has_subject_guards` at :8537). A wrong state that no subject guard
answers then gets neither a scenario nor a refusal, so the suite is silent on a branch a target can answer wrongly.
`crates/verify/ess-conformance/tests/explore_guards_first.rs:450-454` pins the gap as expected
(`beyond == ["demo.pairs.StateBesideWrongState/not-held"]`).

Minimal reproduction: `crates/verify/ess-conformance/tests/fixtures/guard-pair-corpus.yaml`, command
`StateBesideWrongState` on `Seat` (states `Held`, `Released`, `Retired`; `release` moves `Held -> Released`):

```yaml
outcomes:
  - {name: unknown-seat, unknown_instance: true, error: demo.pairs.UnknownSeat}
  - {name: parked, when_subject_state: Released, error: demo.pairs.SeatReleased}
  - {name: not-held, wrong_state: true, error: demo.pairs.SeatNotHeld}
  - {name: released, moves: demo.pairs.Seat.release, instance: seat_id, ...}
```

`ess verify conform synthesize` at `ess/27`, `ess/30` and `ess/32` (0.92.0): `56 scenario(s) (0 authored), 0
refusal(s)`. The wrong-state family holds `Seat/state/Released/refuses/StateBesideWrongState` (answered by `parked`)
but no `Seat/state/Retired/refuses/StateBesideWrongState`, and no refusal names it. Control: `Release` (the same
branches without `parked`) gets both `Seat/state/Released/refuses/Release` and `Seat/state/Retired/refuses/Release`.
The interpreter already answers `not-held` on a retired seat
(`crates/verify/ess-conformance/tests/precedence_plan_answers.rs::guard_pair_corpus_answers_by_the_rule`).

## Acceptance

- Synthesis over the guard-pair corpus at `ess/27`, `ess/30` and `ess/32` emits
  `demo.pairs.Seat/state/Retired/refuses/demo.pairs.StateBesideWrongState` expecting `not-held`, and the interpreted
  target passes it. Test: `crates/verify/ess-conformance/tests/wrong_state_beside_subject_guard.rs`, case
  `a_wrong_state_no_subject_guard_answers_is_witnessed`.
- A faulty interpreted target that accepts (or answers `parked`) on a retired seat fails that scenario. Same file,
  case `a_target_ignoring_the_wrong_state_beside_a_subject_guard_fails`.
- States a subject guard already answers keep their scenarios unchanged (no new wrong-state scenario where the guard
  selects; `Seat/state/Released/refuses/StateBesideWrongState` bytes unchanged).
- `explore_guards_first.rs` drops the `beyond` pin: the explorer reaches no branch the scenarios do not expect.
- Where a held state can be reached by neither the plain wrong-state row nor a guard, a refusal names it instead of
  silence.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/tests/explore_guards_first.rs
- inferred: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- inferred: crates/verify/ess-conformance/tests/wrong_state_beside_subject_guard.rs
- inferred: synthesis byte pins of fixtures with a subject guard beside `wrong_state:` (suite bytes move)

confidence: medium
