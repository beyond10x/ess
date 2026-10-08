---
format: aep.planning-md/3
id: story:one-command-combines-identity-and-row-set-related-guards
kind: story
status: draft
title: One command may guard on an identity-addressed related row and on a row set, and a related guard may sit beside when_subject
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#473
relations:
- serves: vision:O2
revision: 2
---
## Outcome

https://github.com/beyond10x/ess/issues/473: three everyday rules become expressible. Today an
identity-addressed `when_related` and a row-set `when_related` in one command are refused
(`ESS-COMMAND-009`), and a `when_related` beside `when_subject` is refused (`ESS-COMMAND-004`).

## Constraints

- Validation lives in `crates/specify/ess-domain/src/command/`. This story waits until another
  session's selection-plan wave 3 has merged; it does not edit `crates/specify/ess-domain/src/command.rs`,
  `crates/specify/ess-compiler/src/ir.rs`, `src/command/subject_state.rs` or
  `src/command/related_guard.rs` before then.
- Spec first: the admitted combinations are stated in the specification language's own reference and
  validated with the newest `ess` before validation or synthesis changes.
- A fit review per `.agents/skills/assessing-external-requests/SKILL.md` is owed before acceptance
  is written; the issue's reproduction (`shop.orders.AddLine`) is reproduced on the newest release
  first.

## Fit review

Reproduced on ess 0.56.0 (domain `shop.orders`, `ess/23`): identity `when_related` beside a row set refuses `ESS-COMMAND-009` (`ess-domain/src/command/row_set.rs:296-330`); two row sets validate but synthesis refuses all three outcomes with `ESS-SYNTH-001` (`synthesize/row_set.rs:140-165`); `when_related` beside `when_subject` refuses `ESS-COMMAND-004` (`command/related_guard.rs:542-596`), and beside `unknown_instance` too.

1. **Need.** Refuse when an input-named row is missing and when a row set holds rows; guard the addressed subject beside a related row.
2. **Class.** Gap; the refusals call themselves a cut ("in this cut", `row_set.rs:36-38`).
3. **Already expressible?** No; splitting the command loses the rule.
4. **Fit.** The requested "declaration order decides" would be a second order beside the one precedence plan (`docs/design/selection-plan.md:23-27`): `exists: false` is `RelatedRow`, `unknown_instance` `Existence`, `when_subject` `HeldState`, a row-set refusal `PresentRelated`, then `Accepting`. Declaration order stays the order within a phase.
5. **Second adopter.** Book a slot only for a room that exists and when no booking of that room overlaps the slot.
6. **Cost.** No new key, IR, suite or diff shape. The admission is gated at source format `ess/24`, as the `ess/22` relaxations were; below it the refusals stay and name `ess/24`.
7. **Alternatives.** (a) Change nothing. (b) The requester's declaration order: rejected. (c) Admit and let the plan's phases answer.

## Decisions

- Accept, redesigned: the precedence plan's phases decide; `unknown_instance` is admitted beside `when_related`; gated at `ess/24`, so the story ships in 0.59.0 with that format.
- Generated behaviour names such commands as obligations until generation reads the selection plan.

## Acceptance

- At `ess/24` the reproduction commands validate; at `ess/23` they refuse with `ESS-COMMAND-009`/`ESS-COMMAND-004` naming `ess/24`.
- The interpreter answers in the plan's phase order on every branch of a combined fixture (`precedence_plan_answers.rs`, case `combined_related_guards`).
- Synthesis on the combined fixture and on the two-row-set command refuses nothing; the suite passes against the interpreted target; a mutant dropping either guard turns a scenario red.
- Generated behaviour names the command as an obligation (`declared_behaviour.rs` case).

## Scope

| unit | files |
|---|---|
| U1 spec first | `website/docs/reference/predicates.md`, `spec-versions.md`, `docs/design/cross-record-and-stored-field-guards.md`, `docs/design/filtered-related-reads.md:82-94` |
| U2 validation | `ess-domain/src/command/row_set.rs`, `command/related_guard.rs`, new `tests/related_guard_combinations.rs`, fixture `ess-conformance/tests/fixtures/combined-related-guards.yaml` |
| U3 interpreter | `ess-conformance/src/interpret/execute.rs`, `execute/row_set.rs`, `execute/related.rs` |
| U4 synthesis | `ess-conformance/src/synthesize/row_set.rs`, `synthesize/related_guard.rs` |
| U5 generation | `ess-synth/src/determined.rs` (`related_composition`), `tests/declared_behaviour.rs` |
