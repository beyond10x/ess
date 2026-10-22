---
format: aep.planning-md/3
id: story:every-guard-combination-validates-under-the-rule
kind: story
status: active
title: Under format A every guard combination refused for want of an order validates
tags:
- adopter-report
- ess-0.70.0-first
refs:
- provider: github
  reference: beyond10x/ess#282
- provider: github
  reference: beyond10x/ess#554
relations:
- decomposes: epic:generic-guard-precedence
- serves: vision:O2
- depends_on: story:the-precedence-rule-places-every-branch
- informed_by: story:one-command-combines-identity-and-row-set-related-guards
- informed_by: story:feature-request-282
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T02:54:27Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-21T02:54:27Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

From format A, every guard combination that ESS-COMMAND-004 (`ConflictingDeclaration`) refuses
today because "which of the two answers first is not stated" validates, and the rule of
`story:the-precedence-rule-places-every-branch` orders it; overlap within one step is resolved by
declaration order (`docs/design/generic-language-features.md` § G4 "What changes";
`docs/design/input-guard-overlap-precedence.md:45-49`). Closes the remainder of
https://github.com/beyond10x/ess/issues/554 and generalises
https://github.com/beyond10x/ess/issues/282 beyond its `ess/22` composition.

Admitted at format A, each keeping its refusal below format A with a hint naming format A:

- `when_related:` beside `when_subject:`, `when_subject_state:`, `when_state_changes:`,
  `unknown_instance:` or `input_absent:` (https://github.com/beyond10x/ess/issues/554;
  `crates/specify/ess-domain/src/command/related_guard.rs:543-596`, `other_authority` and
  `validate_shape`);
- `when_related:` beside `wrong_state:` outside the `ess/22` composition: an accepting related
  branch that moves, or present-related branches that do not all refuse
  (https://github.com/beyond10x/ess/issues/282; `related_guard.rs:866-918`,
  `orders_present_refusals`);
- `before_row: true` beside any `when_related:` (`crates/specify/ess-domain/src/command.rs:3383-3396`,
  in `validate_before_row_order`), left undecided by
  `docs/design/cross-record-and-stored-field-guards.md:927-931`;
- `when_subject:` beside a lifecycle guard (`crates/specify/ess-domain/src/command/subject_fact.rs:208-218`,
  "subject fact and lifecycle guards cannot be combined") and explicit subject-state guards beside
  `wrong_state:` (`crates/specify/ess-domain/src/command/subject_state.rs:168-175`), all step 3,
  with `wrong_state:` last;
- two present-related refusals over one related row that both hold: the first declared answers,
  where today they "stay ambiguous" (`related_guard.rs:47`, `:1785-1791`, `validate_joint_partition`;
  inferred from the doc comments, the refusal they lead to was not reproduced).

Not here: ESS-COMMAND-009's one-kind-of-related-row refusal (an identity-addressed `when_related:`
beside a row set, https://github.com/beyond10x/ess/issues/473,
`crates/specify/ess-domain/src/command/row_set.rs:295-330`), which the record lifts with G2
(wave 4, item 4). A row set beside `when_subject:` or a lifecycle guard is refused by the same
`beside` function and stays with it.

Every consumer reads the admitted commands through the plan: the interpreter, synthesis, the
generated Rust and Go behaviour, mutation, and Entity Runtime lowering, which lowers each or
refuses it by name. One corpus fixture holds one command per pair of steps these refusals blocked,
and a corpus test runs the interpreter, synthesis and every generated target over it (the epic's
acceptance).

## Acceptance

Every command of the guard-pair corpus validates at format A and is answered in the rule's order
by the interpreter, synthesis and the generated targets, while each pairwise refusal stays below
format A. Each line names the failing test written first.

- `crates/specify/ess-domain/tests/related_guard.rs`:
  `format_a_admits_when_related_beside_every_subject_and_existence_guard`,
  `format_a_admits_when_related_beside_wrong_state_outside_the_ess_22_composition`,
  `format_a_two_refusals_over_one_row_answer_in_declaration_order`,
  `below_format_a_the_pairwise_refusals_stay_and_name_format_a`, and
  `incident_ledger_guard_combinations_validate` (taken from
  `story:one-command-combines-identity-and-row-set-related-guards`: a neutral command with
  `unknown_instance:`, `when_subject:` and an input-addressed `when_related:` validates, and the
  interpreter answers each branch in the rule's order).
- `crates/specify/ess-domain/tests/subject_fact_lifecycle_mix.rs`:
  `format_a_admits_when_subject_beside_when_subject_state`.
- `crates/specify/ess-domain/tests/subject_state.rs`:
  `format_a_admits_wrong_state_beside_explicit_subject_state_guards`.
- `crates/specify/ess-domain/tests/held_state_order.rs` (where the `before_row:` tests live):
  `format_a_admits_before_row_beside_when_related`.
- `crates/verify/ess-conformance/tests/precedence_plan_answers.rs`:
  `guard_pair_corpus_answers_by_the_rule` over the new fixture
  `crates/verify/ess-conformance/tests/fixtures/guard-pair-corpus.yaml`: synthesis refuses no
  branch, and the interpreted target and the generated Rust and Go behaviour pass the suite.
- `crates/verify/ess-conformance/tests/mutation_arms.rs`:
  `guard_pair_corpus_mutant_dropping_either_guard_turns_a_scenario_red`.
- `crates/generate/ess-entity-runtime/tests/selection_precedence.rs`:
  `guard_pair_corpus_lowers_or_refuses_each_command_by_name`.
- Tests are modules of existing test binaries; no new tests/*.rs file.

## Scope

- `crates/specify/ess-domain/src/command/related_guard.rs`: `other_authority` (`:542-566`),
  `validate_shape` (`:575-600`), `orders_present_refusals` (`:866-918`), `validate_joint_partition`
  (`:1792`).
- `crates/specify/ess-domain/src/command.rs`: the `when_related:` arm of
  `validate_before_row_order` (`:3383-3396`) only; the rest of that function and
  `validate_held_state_order` are story 3's.
- `crates/specify/ess-domain/src/command/subject_fact.rs:208-218`;
  `crates/specify/ess-domain/src/command/subject_state.rs:168-175`.
- `crates/verify/ess-conformance/src/interpret/execute.rs`, `interpret/execute/related.rs`,
  `synthesize/related_guard.rs`, `synthesize/precedence.rs` where an admitted pair meets a path
  that assumed it could not occur (inferred).
- `crates/generate/ess-synth/src/{rust,go}/behaviour.rs`, `crates/generate/ess-synth/src/determined.rs`
  (`related_composition`, inferred), `crates/generate/ess-entity-runtime/src/subset.rs`.
- Docs: `website/docs/reference/predicates.md` (the admitted combinations),
  `docs/design/cross-record-and-stored-field-guards.md:927-931`, `CHANGELOG.md` `[Unreleased]`.
- A changed or removed diagnostic: `task test-xtask` and
  `cargo test -p ess-compiler --test typed_diagnostics` (AGENTS.md). Refusals the rule removes are
  removed, not renamed (record § "Migration and format consequences").
- Adversary: yes.

## Order

After `story:the-precedence-rule-places-every-branch` (format A and the rule's placement). Before
`story:declaration-order-agrees-with-every-step-pair`, which shares
`crates/specify/ess-domain/src/command.rs` `validate_before_row_order` and builds on the corpus
fixture, and before `story:unknown-instance-beside-preserving-and-update-selection`, which shares
`crates/specify/ess-domain/src/command/subject_fact.rs`. Parallel with
`story:the-language-policy-is-counted-and-cited`.

## State 2026-10-21

Shipped in 0.63.0 under `ess/27`. Every acceptance test named above passes on the 0.67.0 batch
branch. The pair this story left to ESS-COMMAND-004 for an external branch, `external:` beside
`when_subject:`, is admitted from `ess/29` by `story:external-branch-narrowed-by-subject-guard`.
Entity Runtime refuses each `when_related:` command of the guard-pair corpus by name
(`guard_pair_corpus_lowers_or_refuses_each_command_by_name`); why no lowering is taken is recorded
in `docs/design/generic-language-features.md` § G4 "Entity Runtime and `when_related:`".
