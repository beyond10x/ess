---
format: aep.planning-md/3
id: story:validate-refuses-unsatisfiable-guards
kind: story
status: active
title: Validation refuses an unsatisfiable guard, a contradictory when and an unanswered held state
tags:
- adopter-report
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/finite.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/
- confidence: inferred
  path: crates/specify/ess-primitives/src/error.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:48:59Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:49:01Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

`ess specify validate` refuses a `when:` guard that can never hold — a value compared with itself
(`window.low != window.low`) or conjuncts that contradict each other — with its own code naming
the branch. Today such a branch validates, and the first sign of it is synthesis refusing it
(`ESS-SYNTH-003`, "no candidate … satisfies", with a hint to supply a fixture, which cannot help).

## Problem

Validation refuses only a refusal whose guard always holds
(`crates/specify/ess-domain/src/command.rs`, `validate_always_refusal`, beyond10x/ess#489);
nothing refuses a guard that never holds.

Minimal reproduction (ess 0.92.0), starting from the committed fixture
`crates/verify/ess-conformance/tests/fixtures/refusal-beside-state.yaml` (validates, exit 0) and
changing only the `too-short` refusal of `demo.secrets.RotateSecret`:

1. Self-contradiction: add `{name: demo.secrets.Window, kind: struct, fields: [{name: low, type: Integer}, {name: high, type: Integer}]}`
   to `types:`, the input `{name: window, type: demo.secrets.Window}` to `RotateSecret`, and write
   `when: window.low != window.low`. `ess specify validate` prints `valid`, exit 0.
2. Contradictory conjuncts: `when: {all: [secret.count < 12, secret.count > 20]}` and
   `when: {all: [secret == "a", secret == "b"]}`. Both `valid`, exit 0.

`ess verify conform synthesize` on each then reports
`refusal[ESS-SYNTH-003]: outcome demo.secrets.RotateSecret/too-short has no scenario … no candidate of the N tried satisfies …`.

Control: `when: true` on the same refusal is refused as `ESS-COMMAND-004` ("a refusal whose `when:`
always holds").

The third shape of the original report, a held state no outcome answers, is already refused on
0.92.0: removing the `not-configured` default from `RotateSecret` gives `ESS-COMMAND-005`
`non_exhaustive_branches` "held state Pending and input [] select 0 branches" (and `Revoked`). It
is out of this story unless the reporter's own shape is reproduced.

## Acceptance

- Shape 1 above: `ess specify validate` exits 1 with one error at
  `command.demo.secrets.RotateSecret.outcomes.too-short.when`, a code of its own (not
  `ESS-COMMAND-004`), saying the guard never holds.
- Shape 2 above, both guards: the same refusal, naming the contradicting conjuncts.
- A guard the finite prover cannot decide (over 64 joint assignments, or an operator it does not
  read) is accepted, not refused: a test asserts one such model validates.
- The committed fixture and every example under `examples/` still validate unchanged.
- One validation test per shape in `crates/specify/ess-domain/tests/`.

## Scope

- cited: crates/specify/ess-domain/src/command.rs
- inferred: crates/specify/ess-domain/src/command/finite.rs
- inferred: crates/specify/ess-primitives/src/error.rs (a new validation code)
- inferred: crates/specify/ess-domain/tests/

confidence: medium

## Fit review

1. Need: a branch whose guard never holds is dead; validation accepts it and synthesis reports it
   later with a misleading fixture hint. Reproduced above, brand-free.
2. Class: defect. Validation already refuses the dual (`when:` always holds, beyond10x/ess#489),
   and synthesis treats the branch as unwitnessable (`ESS-SYNTH-003`).
3. Already expressible: not applicable; the author wants to be told, not to express something new.
4. Fit: reuses the finite prover (`command/finite.rs`) that validation and `ess verify conform
   mutate` (`unsatisfiable_guard`) already run; applies to `when:` on every branch, accepting and
   refusing alike. No new authored surface.
5. Second adopter: any author who writes a range with its bounds swapped.
6. Cost: one new validation code; a model accepted today with a dead branch is refused afterwards
   (it already failed synthesis), so no format bump.
7. Considered: change nothing (synthesis keeps reporting it, with a misleading hint); a warning
   instead of a refusal (keeps a dead branch in a model that claims every branch is reachable).
   The refusal wins.

## Decisions

- Accept, redesigned: refuse a never-holding `when:` with its own code. The reported "unanswered
  held state" shape is dropped: it is already `ESS-COMMAND-005` on 0.92.0.
