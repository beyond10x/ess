---
format: aep.planning-md/3
id: story:declaration-order-agrees-with-every-step-pair
kind: story
status: implemented
title: Under format A validation refuses a declaration order that contradicts the step order, for every pair of steps
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#554
relations:
- decomposes: epic:generic-guard-precedence
- serves: vision:O2
- depends_on: story:every-guard-combination-validates-under-the-rule
- informed_by: story:unknown-instance-and-input-guard-agree-on-order
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T03:05:34Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-21T03:05:34Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-21T22:41:23Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

From format A, validation refuses a declaration order that contradicts the step order wherever
two branches' guards can both hold, for every pair of steps of the rule
(`docs/design/generic-language-features.md` § G4 "What changes", second item). A branch of a later
step declared before a branch of an earlier step, where one request (input and stored row) can
satisfy both, is refused as `conflicting_declaration`, naming both branches, their steps and the
order to declare them in. Guards the finite prover shows disjoint may come in either order; a
branch with no guard, and a guard the prover declines, counts as overlapping, as the held-state
rule already reads it. A consumer reading one declaration-order pass (the Go and TypeScript
explorers, a hand-written target checking in declared order) then gives the rule's answer on
every command that validates. Part of https://github.com/beyond10x/ess/issues/554.

It generalises the two rules that check one pair each today and closes the pair that is not
checked:

- the held-state rule of https://github.com/beyond10x/ess/issues/486
  (`crates/specify/ess-domain/src/command.rs:3450-3557`, `validate_held_state_order`);
- the `before_row:` rule of https://github.com/beyond10x/ess/issues/534
  (`command.rs:3361-3448`, `validate_before_row_order`, the "declared after a branch a later step
  answers" arm);
- an input refusal declared after `unknown_instance:`, whose two orders disagree and which nothing
  refuses (`command.rs:3398-3421`; the case `story:unknown-instance-and-input-guard-agree-on-order`
  reports for the explorer).

Below format A only the two existing rules apply, unchanged: a refusal added there would make an
existing valid document invalid (record § "Migration and format consequences", Source).

## Domain relations

- Branch → step, many-to-one, the step is derived from the branch and its command — inferable
  (inferred from `crates/specify/ess-domain/src/command/precedence.rs:401-477`, `place`; no `ess/1`
  document declares this relation).

## Acceptance

At format A no command validates whose declaration order and step order disagree on a request
both branches' guards admit. Each line names the failing test written first.

- `crates/specify/ess-domain/tests/held_state_order.rs`:
  `format_a_refuses_every_step_pair_declared_out_of_order` (table-driven: for each of the ten
  pairs of steps 1 to 5, the later-step branch declared first with an overlapping guard is
  refused, naming both branches and the order to declare them in),
  `format_a_refuses_an_input_refusal_declared_after_unknown_instance`,
  `format_a_admits_out_of_order_declarations_the_prover_shows_disjoint`, and
  `below_format_a_only_the_held_state_and_before_row_rules_apply` (today's messages unchanged).
- `crates/specify/ess-domain/tests/validation_reads_selection_plan.rs`:
  `the_declaration_order_check_follows_the_plan_under_with_phase_order` (the check reads the
  plan's steps through the one test seam, not a second ordering).
- `crates/verify/ess-conformance/tests/explore_guards_first.rs`:
  `on_a_validating_format_a_command_one_declaration_order_pass_gives_the_plans_answer`: over the
  guard-pair corpus of story 2 in declaration order, the interpreter and the Go explorer expect
  the same branch for every scenario.
- Tests are modules of existing test binaries; no new tests/*.rs file.

## Scope

- `crates/specify/ess-domain/src/command.rs`: `validate_before_row_order` (`:3361-3448`) and
  `validate_held_state_order` (`:3450-3557`), replaced at format A by one check over the plan's
  steps, read through `precedence::place` and `Composition::of` (`command.rs:2635`).
- `crates/specify/ess-domain/src/command/finite.rs` (the prover the overlap test reads; read, not
  changed, inferred).
- `docs/design/cross-record-and-stored-field-guards.md`, the paragraph after "The precedence
  order" (`:839-848`, "Validation refuses an accepting or external branch declared before…"),
  stated for every pair of steps at format A; `CHANGELOG.md` `[Unreleased]`.
- Diagnostics: reuses `ConflictingDeclaration`, no new code. A changed message moves pins outside
  the crate: `task test-xtask` and `cargo test -p ess-compiler --test typed_diagnostics`
  (AGENTS.md).
- Adversary: yes (a new refusal on the precedence order).

## Order

After `story:every-guard-combination-validates-under-the-rule`: it rewrites the function whose
`when_related:` arm story 2 removes, and its explorer test runs over story 2's corpus fixture.
Parallel with `story:unknown-instance-beside-preserving-and-update-selection` (no shared file) and
with `story:the-language-policy-is-counted-and-cited`.

## Downstream report 2026-10-19

Downstream case (2026-10-19, finding 12): an accepting `when:` branch (`priority == 7`) declared
before an overlapping input refusal (`priority > 0`) validates (exit 0 on 0.60.0, 0.62.0 and the
integration build) and synthesis refuses it, ESS-SYNTH-003 "… outside too-high (priority > 0), the
input-guarded refusal taken first" (`.engineering/repro/narrow-success-before-refusal/system.yaml`).
The answer order is the documented rule (input refusals step 2, accepting `when:` step 6,
`docs/design/selection-plan.md:24,28`); the defect is that validation admits a branch the rule makes
unreachable. Under format A this order is refused as `conflicting_declaration`
(`docs/design/generic-language-features.md:447`). Idiom today: narrow the refusal,
`when: {all: ["priority > 0", "priority != 7"]}` (`idiom.yaml` beside it: 3 scenarios, 0 refusals).

## State 2026-10-21

Shipped on the 0.67.0 batch branch behind `ess/29` (`ess/27`, format A, shipped in 0.63.0 and is
frozen, so a new refusal there would invalidate valid documents): `CommandSpec::validate_step_order`
replaces the held-state and `before_row:` order rules from `ess/29`; below it both apply unchanged.

Decided while implementing: the branch refused is one read in declaration order within its step
(`Rank::Declared`); the branch above it may be any branch of a later step, a marker included, so
`unknown_instance:` above an input refusal is refused while a marker declared last stays valid.
Overlap reads input guards only, through the finite prover; a stored-row guard is not read, and an
Integer comparison the prover declines counts as overlapping. Step 2 holds only markers, so the
pairs whose earlier step is 2 cannot be refused at their lower branch: seven of the ten pairs are.

Evidence: `crates/specify/ess-domain/tests/held_state_order.rs`
(`format_29_refuses_every_step_pair_declared_out_of_order`,
`format_29_refuses_an_input_refusal_declared_after_unknown_instance`,
`format_29_admits_out_of_order_declarations_the_prover_shows_disjoint`,
`below_format_29_only_the_held_state_and_before_row_rules_apply`,
`format_29_admits_the_step_order_and_markers_declared_last`,
`the_guard_pair_corpus_validates_at_format_29`).

Not done: `validation_reads_selection_plan.rs`
`the_declaration_order_check_follows_the_plan_under_with_phase_order` and the Go explorer test
`explore_guards_first.rs` `on_a_validating_format_a_command_one_declaration_order_pass_gives_the_
plans_answer`.
