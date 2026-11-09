---
format: aep.planning-md/3
id: story:the-interpreter-and-synthesis-answer-a-guarded-move-before-not-found-alike
kind: story
status: draft
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
revision: 3
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

## Acceptance

- The decision is recorded in `docs/design/typed-literals-and-unknown-instances.md` § 2a (Witness) and in
  `docs/design/input-guard-overlap-precedence.md`, and the sentence "The model interpreter does not yet follow that
  order for this input" is removed.
- `crates/verify/ess-conformance/tests/notfound_after_guarded_move.rs`, case
  `a_fresh_identity_under_the_guarded_move_is_answered_alike`: the authored step above, expecting the decided outcome,
  is admitted by synthesis (no ESS-AUTHOR-041) and passed by the interpreted target; the step expecting the other
  outcome is refused by synthesis's author check.
- Existing cases in `notfound_after_guarded_move.rs`, `precedence_order.rs` and `adversary_overlap_precedence_pass2.rs`
  pass unchanged, and synthesized suite bytes do not move for models without this shape.

## Scope

- cited: crates/verify/ess-conformance/src/interpret/execute.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/tests/notfound_after_guarded_move.rs
- cited: docs/design/typed-literals-and-unknown-instances.md
- inferred: docs/design/input-guard-overlap-precedence.md
- cited: crates/verify/ess-conformance/src/authored.rs (the ESS-AUTHOR-041 check at ~:1800, if the decision is (b))

confidence: medium

## Schedule

Backlog finding (from-finding); no wave takes it before every planned wave is delivered unless the orchestrator
says so.
