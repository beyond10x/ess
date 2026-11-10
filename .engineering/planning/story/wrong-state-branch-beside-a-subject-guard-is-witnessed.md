---
format: aep.planning-md/3
id: story:wrong-state-branch-beside-a-subject-guard-is-witnessed
kind: story
status: active
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
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:49:22Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:49:24Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":3}}}
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
- A held state that neither the plain wrong-state row nor a guard can reach is refused by name. Model: the corpus
  with a terminal state `Scrapped` added to `Seat` and a transition `scrap: {from: [Released], to: Scrapped}` that no
  command moves through, so the lifecycle validates and no arrangement reaches `Scrapped`. Synthesis writes no
  `demo.pairs.Seat/state/Scrapped/refuses/demo.pairs.StateBesideWrongState` scenario and records one refusal for that
  id with code `ESS-SYNTH-004`, the code the control `Release` gets for its own `Scrapped` id. Same file, case
  `an_unreachable_wrong_state_beside_a_subject_guard_is_refused_by_name`, run with
  `cargo test -p ess-conformance --locked --test wrong_state_beside_subject_guard`.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes. Each row of
  `synthesis-suite-bytes-table.tsv` that moves is a fixture with a subject guard beside `wrong_state:` whose suite,
  synthesized before and after the change and compared, differs only by added `<entity>/state/<S>/refuses/<command>`
  scenarios or refusals; no other row moves.

## Decisions

- The unreachable state is refused as `ESS-SYNTH-004`: `refused_here` arranges the row through
  `refusal_arrangement`, which answers a state no arrangement reaches with `RefusalCause::InstanceRequired` (code 4,
  `synthesize.rs:1700`), and `wrong_state_scenario` passes that refusal on when no guard answers the state either.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/tests/explore_guards_first.rs
- inferred: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- inferred: crates/verify/ess-conformance/tests/wrong_state_beside_subject_guard.rs

confidence: medium

Shared surfaces moved: re-pins the suite-bytes rows of fixtures with a subject guard beside `wrong_state:`, and
proposes its `CHANGELOG.md` sentence.

