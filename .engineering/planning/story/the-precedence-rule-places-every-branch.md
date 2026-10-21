---
format: aep.planning-md/3
id: story:the-precedence-rule-places-every-branch
kind: story
status: active
title: Under format A the precedence plan places every branch by the one rule
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#554
relations:
- decomposes: epic:generic-guard-precedence
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T02:54:26Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-21T02:54:26Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

From source format A (the next source format after `ess/25`, its number allocated when this story
lands, `ess/26` if nothing allocates it first), the precedence plan places every branch by the one
rule of `docs/design/generic-language-features.md` § G4 "The rule": a branch answers in the order
of what it must read (the request alone; then whether the addressed record exists; then what it
holds; then the rows around it; then the causes outside the specification), and within one step
the first declared whose condition holds. Visibility issue:
https://github.com/beyond10x/ess/issues/554.

What holds after:

- `precedence::place` and `PrecedencePlan::new` answer, under format A: step 0 `input_absent:`;
  step 1 the input-guarded refusals and `external:` + `before_row: true`; step 2 existence
  (`existing_instance:` on every command, including one reading a related row, then
  `unknown_instance:`); step 3 `when_subject_state:`, `when_state_changes:`, `when_subject:`, then
  `wrong_state:`; step 4 related rows: `exists: false` over a row the input or a stored field
  names, in declaration order, then present-related predicate and row-set refusals, then a row-set
  upsert's `unknown_instance:`; step 5 accepting `when:`, `external:`, accepting `when_related:`
  and row-set branches, in declaration order; step 6 `otherwise:`. The one move from today is
  related-row existence over an input-named row, which today answers first (`RelatedRow`) and
  under the rule answers at step 4.
- Request-only refusals stay before existence and the held state (record § "Open questions,
  answered", answer 1); the table in `docs/design/cross-record-and-stored-field-guards.md` § "The
  precedence order" pins it.
- Below format A every model keeps today's plan and today's answers, and its IR bytes and digest.
- Every consumer that reads the plan answers a moved branch where the plan places it: the
  interpreter, synthesis, the generated Rust and Go behaviour, and Entity Runtime lowering, which
  lowers it in plan order or refuses it by name (answer 10), recorded in the lowered-definitions
  table.
- Format A is open: its `FORMAT_HISTORY` row (release `None`) names the rule; no suite format, IR
  type or `RawSpecFile` key changes with it (record § "Migration and format consequences": suite
  format pairs are named for G1, G2 and G3 only).
- `docs/design/selection-plan.md` no longer says no consumer reads the plan (lines 3-5), its phase
  table states the format-A placement beside today's, and its "Where the plan's reading and the
  written ones part" case (lines 135-142) is settled by step 2.

## Domain relations

- Command → branch (outcome), one-to-many, the command owns its branches; a branch's step is a
  function of its own shape and of what its command declares — inferable (inferred from
  `crates/specify/ess-domain/src/command/precedence.rs:401-477`, `place` over `BranchShape` and
  `Composition`; no `ess/1` document declares this relation).
- Command → precedence plan, one-to-one, derived on demand and never stored, so it has no
  lifecycle of its own — inferable (inferred from
  `crates/specify/ess-compiler/src/ir/precedence.rs:26-66`, `PrecedencePlan::new`; no `ess/1`
  document declares it).

## Acceptance

Under format A every consumer answers a command's branches in the rule's step order, and every
model below format A answers exactly as before. Each line names the failing test written first.

- `crates/specify/ess-domain/tests/precedence_classification.rs`:
  `format_a_places_input_named_absent_rows_after_the_held_state`,
  `format_a_places_existing_instance_at_existence_on_a_related_row_command`,
  `format_a_keeps_input_refusals_and_before_row_externals_before_existence` (answer 1), and
  `below_format_a_every_placement_is_unchanged` (every `ConditionShape` × composition, against
  today's `place`).
- `crates/specify/ess-compiler/tests/precedence_plan.rs`:
  `precedence_plan_reads_the_rule_from_format_a`.
- `crates/specify/ess-compiler/tests/selection_precedence_table.rs` (existing test): the table
  pins the new fixture `crates/specify/ess-compiler/tests/fixtures/precedence-rule-format-a.yaml`
  under the rule, and every line and digest it pinned before is byte-identical.
- `crates/verify/ess-conformance/tests/precedence_plan_answers.rs`:
  `format_a_answers_a_missing_input_named_row_after_unknown_instance_and_held_state` (the
  interpreter, on the fixture above) and `synthesis_plans_read_the_documents_format_not_the_newest`
  (the `newest()` call sites below, on a below-A model, once format A is the newest).
- `crates/verify/ess-conformance/tests/selection_precedence.rs`:
  `format_a_fixture_suite_passes_on_the_interpreted_rust_and_go_targets`.
- `crates/generate/ess-entity-runtime/tests/selection_precedence.rs`:
  `format_a_related_row_existence_lowers_in_plan_order_or_is_refused_by_name`.
- `crates/specify/ess-domain/tests/format_history.rs`: `format_a_shipped_in_0_63_0_and_names_the_rule`.
- Tests are modules of existing test binaries; no new tests/*.rs file.

## Scope

- `crates/specify/ess-domain/src/command/precedence.rs`: `place` (`:401-477`), `Composition::new`
  (`:282-316`, carries the format), the `Phase` docs and `Phase::step` (`:23-72`, step numbers
  under format A).
- `crates/specify/ess-domain/src/system.rs`: `SUPPORTED_FORMATS`, `FORMAT_HISTORY` (`:240-258`),
  the new `FormatVersion` constant; `website/docs/reference/spec-versions.md` through
  `cargo xtask format-history`.
- `crates/specify/ess-compiler/src/ir/precedence.rs`: `PrecedencePlan::new` (signature unchanged).
- `crates/verify/ess-conformance/src/interpret/execute.rs:581-610` (the `Phase::RelatedRow` arm
  that calls `related_absent`) and `interpret/execute/related.rs:36-52`
  (`branches(Phase::RelatedRow)`): read input-named `exists: false` where the plan places it.
- `crates/verify/ess-conformance/src/synthesize/precedence.rs:53, 72, 98` (`72` and `98` build the
  plan at `newest()`, which becomes format A) and `synthesize/related_guard.rs:949, 983`.
- `crates/generate/ess-synth/src/rust/behaviour.rs:1258-1283`, `go/behaviour.rs:1843-1881`
  (`related_row`), `crates/generate/ess-synth/src/plan.rs:727`.
- `crates/generate/ess-entity-runtime/src/lib.rs:1968`, `src/subset.rs` (lowered-definitions
  table).
- Messages naming a step number (`crates/specify/ess-domain/src/command.rs:3361-3448`,
  "at step 2 of the precedence order") name the step of the document's format; if a diagnostic's
  text changes, run `task test-xtask` and `cargo test -p ess-compiler --test typed_diagnostics`.
- Docs: `docs/design/cross-record-and-stored-field-guards.md:810-841` (the rule's table for
  format A beside today's order for the formats below it), `docs/design/selection-plan.md:3-5,
  17-60, 135-142, 199-209`; `CHANGELOG.md` `[Unreleased]`.
- Adversary: yes (changes the precedence order).

## Order

First in the wave: it opens format A, and stories 2, 3 and 4 gate their admissions on it. It runs
in parallel with story 5 (`story:the-language-policy-is-counted-and-cited`), which shares no file
with it except `CHANGELOG.md` `[Unreleased]` (different lines).

## Format A after 0.62.0

`ess/26` shipped in 0.62.0 (#587) and `ess/27` is the format `release-plan:ess-063-downstream` opens for its
batch (U3, the #542–#552 port, #594, #551, #596). Format A is therefore `ess/27` if this story's
wave lands before 0.63.0 closes it, and `ess/28` otherwise; the integrator decides at dispatch and
the `FORMAT_HISTORY` row names the rule either way. This wave runs after
`task:land-the-2026-10-13-port` and `task:salvage-the-related-rows-quantifier-tree`, which rewrite
`crates/specify/ess-domain/src/command.rs`, `command/related_guard.rs`, `command/row_set.rs` and
`crates/verify/ess-conformance/src/synthesize/precedence.rs`'s neighbours.

## State 2026-10-21

Shipped in 0.63.0 under `ess/27` (format A is `ess/27`). Every acceptance test named above passes
on the 0.67.0 batch branch, with one deviation: the suite of the format-A fixture is run against
the interpreted target only (`selection_precedence.rs`,
`format_a_fixture_suite_passes_on_the_interpreted_target`), not against generated Rust and Go;
the generated behaviours of the rule are covered by `ess-synth`'s own tests. Entity Runtime keeps
refusing `when_related:` by name, the reason recorded in
`docs/design/generic-language-features.md` § G4 "Entity Runtime and `when_related:`".
