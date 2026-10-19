---
format: aep.planning-md/3
id: story:an-owner-link-guard-beside-an-owner-existence-guard-is-witnessed
kind: story
status: draft
title: A guard comparing an owned row's link with an input is witnessed when the owner input is guarded too
tags:
- adopter-report
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/related_guard_owner_link_beside_owner_guard.rs
revision: 2
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
   `.engineering/repro/owned-via-compared-with-input/system.yaml` (Board owns Node via `board_id`).
   On 0.60.0, 0.62.0 and the integration build (0.62.0 + the 2026-10-13 port): `6 scenario(s), 1
   refusal(s)`, `ESS-SYNTH-003 … wiring.board.Bind/bound … no candidate of the 1 tried satisfies
   `wiring.board.Board` stored related row selecting this branch, over the rows 1 bounded
   arrangements left`. Without the `exists: false` guard on the board the same model synthesizes
   every scenario with 0 refusals (probe on the integration build).
2. **Class.** Defect: the owner-link comparison is supported (`naming_owners`,
   `crates/verify/ess-conformance/src/synthesize/related_guard.rs:3177-3185`, beyond10x/ess#271) and
   fails only when the owner input is also addressed by its own `when_related`.
3. **Already expressible?** Yes; the model validates. Synthesis is what fails.
4. **Fit.** No new surface. Inferred from code, not traced: the Board row for the owner guard is
   searched separately (`synthesize/subject_fact.rs:4851-4866`, `search_rows`, called from
   `related_guard.rs:3212-3224`) and is not the row the Node is filed under; the arrangement should
   pin both guards' rows to one owner, as `Setup::bound` already does for a single guard.
5. **Second adopter.** An order line is accepted only on its own order, and an unknown order is
   refused first.
6. **Cost.** Synthesis only; suites change only for models that refuse today. No format change.
7. **Alternatives.** (a) Change nothing: adopters drop either guard. (b) Fix the arrangement
   (chosen).

## Decisions

Accept as a defect.

## Acceptance

- `an_owner_link_guard_beside_an_owner_existence_guard_is_witnessed`: the reproduction synthesizes
  all four branches with 0 refusals and passes `--target interpreted`.

## Scope

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (~3160-3260) — inferred.
- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (`search_rows`, ~4851) — inferred.
- Test: new `crates/verify/ess-conformance/tests/related_guard_owner_link_beside_owner_guard.rs`.

Medium; self-contained, but `related_guard.rs` is where the U6 unit and the port also landed, so
it runs after both are merged.
