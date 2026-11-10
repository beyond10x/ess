---
format: aep.planning-md/3
id: story:an-owner-link-guard-beside-an-owner-existence-guard-is-witnessed
kind: story
status: active
title: A guard comparing an owned row's link with an input is witnessed when the owner input is guarded too
tags:
- adopter-report
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/related_guard_owner_link_beside_owner_guard.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:40:44Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:40:46Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

A command that refuses an unknown owner (`when_related: {via: input.board_id, exists: false}`) and
also refuses a related row filed under another owner (`when_related: {via: input.node_id, predicate:
board_id != input.board_id}`) gets a scenario for its accepting branch: the owner row arranged for
`input.board_id` is the one the related row is filed under.

## Fit review

Finding covered: downstream specification report of 2026-10-19, finding 7.

1. **Need.** Binding a node is refused when the board is unknown, when the node is unknown, and when
   the node belongs to another board; otherwise it binds. Reproduction:
   `.engineering/repro/owned-via-compared-with-input/system.yaml` (Board owns Node via `board_id`;
   `Bind` has `no-board`, `no-node`, `other-board`, `bound`). Still present on 0.92.0:

   ```text
   $ ess verify conform synthesize --path .
   refused: refusal[ESS-SYNTH-003]: outcome wiring.board.Bind/bound has no scenario `wiring.board.Bind/outcome/bound`
     no candidate of the 1 tried satisfies ``wiring.board.Board` stored related row selecting this branch, over the rows 1 bounded arrangements left`
   6 scenario(s) (0 authored), 1 refusal(s), nothing written
   $ ess verify conform run --path . --target interpreted   → 6 passed (bound has no scenario)
   ```

   Without the `exists: false` guard on the board the same model synthesizes every scenario with 0
   refusals (probe on 0.62.0 integration build).
2. **Class.** Defect: the owner-link comparison is supported (`subject_fact::naming_owners`,
   beyond10x/ess#271) and fails only when the owner input is also addressed by its own
   `when_related`.
3. **Already expressible?** Yes; the model validates. Synthesis is what fails.
4. **Fit.** No new surface. Inferred from code, not traced: the Board row for the owner guard is
   searched separately (`related_guard.rs` `search_rows` :4280, `subject_fact.rs` `search_rows`
   :5723) and is not the row the Node is filed under; the arrangement should pin both guards' rows
   to one owner, as `Setup::bound` already does for a single guard (`related_guard.rs` :4717-4721,
   the `naming_owners` call).
5. **Second adopter.** An order line is accepted only on its own order, and an unknown order is
   refused first.
6. **Cost.** Synthesis only; suites change only for models that refuse today. No format change.
7. **Alternatives.** (a) Change nothing: adopters drop either guard. (b) Fix the arrangement
   (chosen).

## Decisions

Accept as a defect.

## Acceptance

- `an_owner_link_guard_beside_an_owner_existence_guard_is_witnessed`: the reproduction synthesizes
  all four `Bind` branches with 0 refusals and passes `--target interpreted`; before the fix
  `Bind/bound` is refused ESS-SYNTH-003 as above.
- `tests/related_guard_owner_link.rs` (the single-guard case) keeps its verdicts and suite bytes.

## Scope

- inferred: `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (`searched_inputs` :4700-4728, `search_rows` :4280)
- inferred: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (`naming_owners` :2026, `search_rows` :5723)
- inferred: `crates/verify/ess-conformance/tests/related_guard_owner_link_beside_owner_guard.rs` (new)

confidence: medium
