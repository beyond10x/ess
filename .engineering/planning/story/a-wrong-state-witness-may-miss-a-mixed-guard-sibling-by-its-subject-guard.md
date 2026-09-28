---
format: aep.planning-md/3
id: story:a-wrong-state-witness-may-miss-a-mixed-guard-sibling-by-its-subject-guard
kind: story
status: implemented
title: A wrong-state witness may miss a mixed-guard sibling by its subject guard
refs:
- provider: github
  reference: beyond10x/ess#192
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/mixed_guard_wrong_state.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T13:35:30Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-09-28T13:35:30Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-09-28T15:58:15Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":6,"verification":1}}}
---
# Story: a wrong-state witness may miss a mixed-guard sibling through its subject guard

## Outcome

`verify conform synthesize` writes the `wrong_state` scenario for a command whose sibling branch
combines `when:` over an input with `when_subject:` over the subject: the witness may avoid that
branch by falsifying either guard, so complementary input guards on two siblings no longer make
the scenario unsatisfiable (ESS-SYNTH-003).

## Why

beyond10x/ess#192 (a downstream adopter, 2026-09-28): regression from 0.36.0 to 0.38.0; the
adopter loses two scenarios. Reproduction shape in the issue (`Confirm` with `already-confirmed`
`when_subject` + `when: token != ""`, `token-required` `when: token == ""`, `gone`
`wrong_state: true`): refusal `no candidate of the 2 tried satisfies none of: token != "", token
== ""`. Removing only the sibling's `when:` restores it. Cause (found by the implementor, 2026-09-28): `subject_fact::refusal_input`, added in `e86e2f9db0` for #173 (0.38.0), made the input half of every sibling false, including the `when:` of a branch that also has a `when_subject:`. The issue's guess (#178 precedence) was wrong; that code is not on this path.

## Acceptance

- The issue's shape synthesizes `Order/state/Closed/refuses/Confirm`; the witness sends
  `token != ""` and arranges the subject so `already-confirmed`'s `when_subject` is false.
- A branch whose selection needs both an input guard and a subject guard is avoided by falsifying
  either; a branch guarded by an input alone is still avoided only through its input.
- #178's precedence (an input-guarded refusal wins over an accepting branch it overlaps) is
  unchanged; its tests stay green.
- A red-first test reproduces the issue on the base.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs` (wrong_state witness / `refused_here` / `plain_guards`) — inferred
- `crates/verify/ess-conformance/tests/` (new test) — inferred
