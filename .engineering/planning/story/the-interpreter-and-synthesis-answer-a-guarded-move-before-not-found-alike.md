---
format: aep.planning-md/3
id: story:the-interpreter-and-synthesis-answer-a-guarded-move-before-not-found-alike
kind: story
status: active
title: The interpreter and synthesis answer a guarded move before a not-found alike
summary: After 0.87.0 U1, synthesis gives a fresh identity with the guarded move to the accepting branch declared first, while the model interpreter answers not-found (from-finding)
tags:
- backlog
- defect
- from-finding
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/notfound_after_guarded_move.rs
- confidence: cited
  path: docs/design/input-guard-overlap-precedence.md
- confidence: cited
  path: docs/design/typed-literals-and-unknown-instances.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:47:43Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:44Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Found while implementing `story:a-not-found-branch-after-a-guarded-move-is-refused-with-its-repair` (0.87.0 wave, U1)
and still present on 0.92.0. For a command whose accepting branch `when: move == Open` is declared before an
`external:` not-found branch, synthesis follows the precedence order (`docs/design/input-guard-overlap-precedence.md`;
`docs/design/typed-literals-and-unknown-instances.md` § 2a, Witness, which already says so and names this story): an
input with a fresh identity and `move: Open` is answered by the accepting branch. The model interpreter answers the
same input with `not-found`: it consults `unknown_instance` before any `when:` guard
(`crates/verify/ess-conformance/src/interpret/execute.rs:2460-2477`, through `synthesize::declared_not_found` at
`crates/verify/ess-conformance/src/synthesize.rs:15879`).

No synthesized scenario asserts either answer, because synthesis no longer builds that witness. But the two halves
contradict each other on an authored scenario: synthesis's author check admits only `opened` for this input, and the
interpreter fails exactly that admitted scenario. No authored scenario for this input passes the reference target.

## Reproduction (0.92.0)

Model: `SHELF` in `crates/verify/ess-conformance/tests/notfound_after_guarded_move.rs` (`ess/34`; `HandleBin {bin_id,
move}` with `opened` (`when: move == Open`, `moves: Bin.open`) declared before `not-found` (`external:`, error
`BinNotFound {bin_id}`)). One authored `ess-scenario/1` step, no arrangement:

```yaml
- {at: 2026-01-05T09:00:00Z, command: demo.shelf.HandleBin, input: {bin_id: "00000000-0000-4000-8000-0000000000ab", move: Open}, outcome: opened}
```

```text
$ ess verify conform synthesize --path spec --scenarios expect-opened --out suite.json
5 scenario(s) (1 authored), 2 refusal(s)
$ ess verify conform run --target interpreted --path spec --suite suite.json --report-format 2
  failed demo.shelf/authored/fresh-bin-open-answers-opened
ESS-CF-OUTCOME expected ['outcome = opened'] observed ['outcome = not-found']
ESS-CF-ERROR expected ['no error'] observed ['error = demo.shelf.BinNotFound']
```

The same step expecting `outcome: not-found, error: {name: demo.shelf.BinNotFound}`:

```text
refusal[ESS-AUTHOR-041]: ... `demo.shelf.HandleBin` is expected to answer `not-found` for {bin_id: "…00ab", move: "Open"},
which its guards decide otherwise: `opened` (move == Open) answers it first, so `not-found` is not taken
```

## Decision needed

Which answer the precedence order gives for `{fresh identity, move: Open}`: (a) `opened`, as § 2a and the author check
say today, which leaves open what a move on a record that does not exist does; or (b) the not-found branch, as the
interpreter answers, which reads "an identity nobody holds" before step 6 for acting branches and moves § 2a, the
author check and the synthesis witness with it. Q-A of `approval-record:ess-0870-wave` kept the precedence order;
it did not rule on the missing record.

## Decisions

- (b): an identity no record carries is answered by the declared not-found outcome, also where an
  accepting `when:` branch declared before it claims the input. Why: § 2a's own Rule ("whose
  `instance:` names no record, is answered with the command's not-found outcome") and G4's order
  (`docs/design/generic-language-features.md` § G4: the request, then whether the addressed record
  exists, then acceptance) already say so; the interpreter answers a selected acting branch whose
  row is missing with the command's unknown-instance answer
  (`crates/verify/ess-conformance/src/interpret/execute.rs:2286-2289`); and (a) would need a
  meaning for `moves:` on a record that does not exist, which the model has nowhere.
- What this reverses: for this shape the 0.87.0 refusal of
  `story:a-not-found-branch-after-a-guarded-move-is-refused-with-its-repair` (`ESS-SYNTH-003` with
  the `before_row: true` repair on `not-found`) no longer fires, because a fresh identity now
  witnesses `not-found`. Its `before_row: true` form stays valid and synthesized. The interpreter
  does not change.

## Acceptance

Every test bullet runs `cargo test -p ess-conformance --locked --test notfound_after_guarded_move`
on `SHELF`.

- `docs/design/typed-literals-and-unknown-instances.md` § 2a (Witness) and
  `docs/design/input-guard-overlap-precedence.md` (the paragraph ending "does not fire on a cause
  that lies in the specification") say that a fresh identity is answered by the declared not-found
  outcome even where an accepting `when:` branch declared before it claims the input. The § 2a
  sentences "Where an accepting `when:` branch declared before the not-found outcome claims …" and
  "The model interpreter does not yet follow that order for this input" are gone
  (`grep -c "does not yet follow that order"` on the file prints `0`).
- Test `a_fresh_identity_under_the_guarded_move_is_answered_alike`: the authored step above with
  `outcome: not-found, error: {name: demo.shelf.BinNotFound}` is admitted by synthesis (no
  `ESS-AUTHOR-041`) and passes on `--target interpreted`; the same step with `outcome: opened` is
  refused `ESS-AUTHOR-041`, naming `not-found` as the outcome that answers it.
- Same target: synthesis of `SHELF` writes `demo.shelf.HandleBin/outcome/not-found` with a fresh
  `bin_id`, `move: Open`, nothing arranged, expecting `not-found`, error `demo.shelf.BinNotFound`
  and no `demo.shelf.BinOpened`. No refusal names `not-found`. This replaces the assertions of
  `a_not_found_after_a_guarded_move_is_refused_once_with_its_repair` and the refusal count of
  `an_error_field_named_like_the_input_builds_no_step_the_precedence_refutes`.
- Same target: `a_not_found_declared_first_before_the_row_is_synthesized` and
  `a_reachable_not_found_after_a_guarded_move_is_still_synthesized` pass unchanged.
- `cargo test -p ess-conformance --locked --test precedence_order` and `--test
  adversary_overlap_precedence_pass2` pass unchanged. In `--test synthesis_suite_bytes_table`, a
  row moves only for a model with an accepting `when:` branch declared before a declared not-found
  outcome that claims the acting input.

## Scope

Re-pins suite-bytes rows for models of that shape, and proposes its `CHANGELOG.md` sentence (the
0.87.0 refusal no longer fires for this shape).

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/authored.rs
- cited: crates/verify/ess-conformance/tests/notfound_after_guarded_move.rs
- cited: docs/design/typed-literals-and-unknown-instances.md
- cited: docs/design/input-guard-overlap-precedence.md

confidence: medium

`synthesize.rs`: the § 2a witness's exception for a claiming accepting branch, and the
`ESS-SYNTH-003` repair path near :9131-9210. `authored.rs`: the `ESS-AUTHOR-041` check (~:1800).
`interpret/execute.rs` leaves the scope: the interpreter's answer is the decided one.

## Schedule

Backlog finding (from-finding); no wave takes it before every planned wave is delivered unless the orchestrator
says so.

