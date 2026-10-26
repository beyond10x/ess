---
format: aep.planning-md/3
id: story:an-identity-guard-beside-a-row-set-validates
kind: story
status: active
title: A row-set guard beside an identity-addressed when_related validates and answers in the rule's step-4 order
tags:
- adopter-report
- ess-0.70.0
refs:
- provider: github
  reference: beyond10x/ess#473
- provider: github
  reference: beyond10x/ess#554
relations:
- decomposes: epic:generic-guard-precedence
- serves: vision:O2
- informed_by: story:one-command-combines-identity-and-row-set-related-guards
- depends_on: story:declaration-order-agrees-with-every-step-pair
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src/determined.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/row_set.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute/row_set.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:07:58Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-21T22:07:58Z", actor: "human:timo", revision: 3}
---
## Outcome

From `format: ess/30`, one command may guard on an identity-addressed related row
(`when_related: {via: input.<id>, exists | predicate}`) and on a row set
(`when_related: {entity, where, exists | count | forall}`) together. Today validation refuses the
pair as ESS-COMMAND-009, "one command reads one kind of related row in this cut"
(`crates/specify/ess-domain/src/command/row_set.rs:330-350`, the message at `:341`). The answer
order is the one precedence rule (`docs/design/generic-language-features.md` § G4, "The rule",
step 4): existence of every named row first, in declaration order, then predicate and row-set
refusals over present rows; an accepting row set answers at step 5. The design lifts this refusal
with G2 (§ G4 "What changes", first item: "ESS-COMMAND-009's one-kind-of-row refusal is lifted
with G2"). Below `ess/30` the refusal stays, its hint naming `ess/30`.

Part of https://github.com/beyond10x/ess/issues/554 and the 2026-10-13 comment on
https://github.com/beyond10x/ess/issues/473. It carries forward the row-set half of
`story:one-command-combines-identity-and-row-set-related-guards` (archived); the
`when_subject`/`unknown_instance` halves shipped under `ess/27`
(`story:every-guard-combination-validates-under-the-rule`).

Reproduction: `.engineering/repro/identity-beside-row-set/system.yaml` (a wire may be laid only
between two existing ports of the right direction and only where no wire already leaves the
source port: four identity-addressed guards at `:56-65`, a row set at `:68-71`). On the
integration build (e44d2b75c) it refuses ESS-COMMAND-009 (sizing pass 2, 2026-10-21).

## Fit review

The archived story's fit review (2026-10-13, ess 0.56.0) holds and is not repeated; the points
that change:

1. **Need.** Refuse when an input-named row is missing, when it is in the wrong state, and when a
   row set holds rows, on one command. Splitting the command loses the rule.
2. **Class.** Gap; the refusal names itself a cut (`row_set.rs:341`, "in this cut").
3. **Already expressible?** No.
4. **Fit.** The rule's step 4 already orders both kinds; no new key, no second order. The
   requester's "declaration order decides" (#473) stays rejected: declaration order decides only
   within a step.
5. **Second adopter.** Book a slot only for a room that exists and when no booking of that room
   overlaps the slot.
6. **Cost.** A format gate (`ess/30`); the interpreter, synthesis and the generated-behaviour
   obligation read the combination; no new IR shape, suite shape or diagnostic code expected.
7. **Alternatives.** (a) Change nothing. (b) Declaration order across kinds: rejected. (c) Admit
   and let step 4 order it (chosen).

## Decisions

- Accept, as the binding design states: lift ESS-COMMAND-009 for the identity-plus-row-set pair
  from `ess/32` (the first unreleased format; `ess/30` and `ess/31` shipped without it); below it
  the message is unchanged and the hint names `ess/32`.
- Order within step 4 is existence of identity-addressed rows, then predicate and row-set
  refusals over present rows, each in declaration order (§ G4 table, step 4). Corrected by unit A
  (2026-10-26): a row-set refusal declared above an identity-addressed `exists: false` is not
  refused; it validates. `story:declaration-order-agrees-with-every-step-pair`'s check does not
  pair those two steps, and this story adds no rule for them.
- Generated Rust and Go keep such a command a named obligation (row sets are not generated,
  `docs/design/generic-language-features.md` § G2 "Cost"); Entity Runtime keeps
  `RelatedGuardUnsupported` (§ G4 "Entity Runtime and `when_related:`").

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/specify/ess-domain/tests/row_sets.rs`:
  `from_ess_30_an_identity_guard_beside_a_row_set_validates` (the reproduction validates) and
  `below_ess_30_an_identity_guard_beside_a_row_set_names_ess_30` (ESS-COMMAND-009, message
  unchanged, hint naming `ess/30`).
- `crates/verify/ess-conformance/tests/precedence_plan_answers.rs`:
  `identity_and_row_set_guards_answer_in_step_4_order`: the interpreter answers a missing named
  row before a wrong-direction row before an occupied source port, on each branch of the
  reproduction.
- `crates/verify/ess-conformance/tests/row_sets.rs`:
  `identity_and_row_set_guards_synthesize_without_refusal`: synthesis refuses nothing on the
  reproduction, the suite passes against the interpreted target, and a mutant dropping either
  guard turns a scenario red.
- `crates/generate/ess-synth/tests/declared_behaviour.rs`:
  `identity_and_row_set_guards_stay_an_obligation`: generated behaviour names the command as an
  obligation.

## Scope

| unit | files | confidence |
|---|---|---|
| validation | `crates/specify/ess-domain/src/command/row_set.rs` (`:330-350`), the `ess/30` gate beside the other `ess/30` admissions | cited |
| interpreter | `crates/verify/ess-conformance/src/interpret/execute/row_set.rs`, `execute/related.rs` | inferred |
| synthesis | `crates/verify/ess-conformance/src/synthesize/row_set.rs`, `synthesize/related_guard.rs` | inferred |
| generation | `crates/generate/ess-synth/src/determined.rs` | inferred |
| docs | `website/docs/reference/predicates.md`, `website/docs/reference/spec-versions.md` (`ess/30` row) | inferred |

Crates: ess-domain, ess-conformance, ess-synth (sizing: L). Shares the related-guard and row-set
code of ess-domain with `story:enforced-reference-needs-no-missing-target-branch` and the
navigation stories; serialize with them.

## Order

After `story:declaration-order-agrees-with-every-step-pair` (its step-order check is what refuses
an out-of-order pair here). Lane L4a: this story, then
`story:enforced-reference-needs-no-missing-target-branch`, then
`story:a-race-above-a-held-state-branch-is-witnessed`.
