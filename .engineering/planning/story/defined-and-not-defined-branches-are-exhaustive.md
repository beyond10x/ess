---
format: aep.planning-md/3
id: story:defined-and-not-defined-branches-are-exhaustive
kind: story
status: draft
title: Branches over defined(x) and not defined(x) are proved exhaustive
summary: 'Adopter report R4: two update branches over one stored Optional field draw ESS-COMMAND-005.'
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: inferred
  path: crates/generate/ess-synth/tests/finite_presence.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/finite.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/subject_guard_input.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/finite_enum.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/negated_defined_beside_when_subject.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/when_subject_witness.rs
- confidence: cited
  path: docs/design/closed-enum-outcome-coverage.md
- confidence: cited
  path: website/docs/guides/specify/fields-and-invariants.md
revision: 6
---
## Report

An adopter retrofitting an existing service (2026-10-23, ess 0.67.0, ess/29): two `when_subject` update branches guarded `defined(token)` and `not defined(token)` over one `Optional` field are refused as ESS-COMMAND-005 (coverage not proved); dropping the second guard validates.

## Fit review

1. **Need.** One branch partition names both sides of whether an Optional value is present: a stored field (`when_subject:`) or an input (`when:`). The requester's syntax is the pair `defined(token)` / `not defined(token)`, which needs no new syntax. Reproductions on 0.74.0, kept outside the repository: the adopter's create-or-update shape, a stored-pair shape with an existence refusal, and an input-pair shape. All three are refused as ESS-COMMAND-005.

2. **Class.** Convenience. Making one side the default works (point 3). Not a defect: `docs/design/closed-enum-outcome-coverage.md:14` documents that Optional paths cannot prove coverage.

3. **Already expressible.** Yes, as a default. On 0.74.0, a stored-default shape with the absent case and one with the present case each validate with 6 scenarios, 6 passed; the input-default shapes (absent: `token` left out of the input; present: `token` sent) also pass 6 of 6. Accepted anyway, because it adds no surface (point 6).

4. **Grids and targets.** Grid O's `defined()` cell (`docs/design/language-grids.md:77`); no new vocabulary. Cause: `input_paths` sends `Predicate::Defined` to `_ => Err(Decline::Unsupported)` (`crates/specify/ess-domain/src/command/finite.rs:152`), and `analyze_inputs` declines every `resolved.optional` path (finite.rs:216). Siblings:
   - Stored (`crates/specify/ess-domain/src/command/subject_fact.rs:601-605`), input (`crates/specify/ess-domain/src/command.rs:4114`) and related (`crates/specify/ess-domain/src/command/related_guard.rs:1890-1894`, `:2141`, `:2394`) use the closed mode only when there is no default, and refuse when the proof declines. Included.
   - Held state (`crates/specify/ess-domain/src/command/subject_state.rs:332-336`) is excluded. When its proof declines it drops an input refusal and proves the rest (subject_state.rs:348), which can admit the command. Adding presence there could newly refuse a valid document: the doubled domain can pass the 64-assignment cap, or an overlap can appear.
   - Step-order checks (`command.rs:3836`, `:3908`) and synthesis (`crates/verify/ess-conformance/src/witness.rs:595`) call `analyze`, which keeps declining presence. Unchanged.
   
   Targets: no IR change. Generated Rust, Go and web already handle defaultless stored partitions (`crates/generate/ess-synth/tests/finite_boolean.rs`) and `defined` guards (the default variants). The interpreter passes both sides. `ess verify diff` sees no new IR. `mutate` already decides `defined` (CHANGELOG 0.56.0, #501).

5. **Second adopter.** An account with an optional `reset_token` whose ResendReset command answers "resent" when it is set and "issued" when it is not; or a Checkout command with an optional input `coupon`.

6. **Cost.** No format, syntax, ValidationCode or IR bytes. Code: a presence domain in `finite.rs`; `command.rs:4119-4134` must also defer an Optional input root read only through `defined` (today it defers only Named or Boolean types, which is why the input refusal carries no reason). Public Rust API: absence must appear in `Case` / `FieldCase` (values hold `FactValue`, which has no "absent"), the kind of change `closed-enum-outcome-coverage.md` accepted for Booleans. The uncovered and overlap messages must render "absent" and "present".

7. **Alternatives.** (a) Change nothing, answer with the idiom: works, but leaves a two-value domain unproved. (b) Gate at `ess/33`: rejected; no meaning changes, the Boolean precedent was ungated (commit `d1e3bed41`, 0.52.0, "prove closed Boolean outcome partitions"), and a format for one request is a red flag. (c) `Optional<Enum>` as {absent} plus variants, mixed reads included: larger; out of scope. (d) Presence only, no default only, held state excluded: chosen.

## Decisions

**Accept, redesigned. No format, no new syntax.**

- **Rule.** In the closed (`booleans`) mode of `analyze_inputs`, a path whose resolution is Optional and which every guard reads only through `defined(…)` (bare, or under `not`, `all`, `any`) has the domain {absent, present}. In the prover, absent is an unset fact and present is any set placeholder (`FactStore::observed_presence`, `crates/specify/ess-primitives/src/facts.rs:1186`). A path also read by value, or a required path read by `defined`, keeps its decline.
- **Unchanged callers.** `analyze` (synthesis, step order) and `analyze_with_states` (held state) keep today's behaviour. `paths` admits a `defined`-only path syntactically, for the shape-time deferral.
- **Format.** None. No `ess/N` gate and no below-format hint. The earlier `ess/31` gate was stale: `ess/31` shipped in 0.69.0 and `ess/32` in 0.70.0 (`crates/specify/ess-domain/src/system.rs:330-350`), so a gate would now mean `ess/33`.
- **Siblings.** The related guard (`when_related`) is included. The held-state sibling is excluded, for the reason in Fit review point 4.
- **Mutant.** The check "a mutant swapping the two branches is killed" is replaced: a precedence swap of a disjoint pair is `equivalent` (#501). The mutant to kill is guard-negate.
- **Docs.** `docs/design/closed-enum-outcome-coverage.md` (lines 14, 43-55), `website/docs/guides/specify/fields-and-invariants.md` (lines 82-99), and a CHANGELOG "Changed" entry: specifications previously refused now validate.
- **Reply to the adopter.** "Accepted with no format change: a `defined` / `not defined` pair over one Optional path, on stored fields or input, is proved exhaustive without a default."

## Acceptance

In `crates/specify/ess-domain/tests/subject_guard_input.rs` (`cargo test -p ess-domain --locked --test subject_guard_input <fn>`):

- `defined_and_not_defined_over_one_stored_optional_field_are_exhaustive` (first red test): the stored-pair shape validates at `ess/32` with no ESS-COMMAND-005.
- `a_defined_pair_over_an_optional_input_is_exhaustive`: the input-pair shape validates, at shape time and when typed.
- `a_defined_pair_over_a_related_optional_field_is_exhaustive`: `when_related: {via: …, predicate: defined(x)}` and the `not defined(x)` sibling validate.
- `defined_beside_a_value_comparison_still_declines`: `defined(token)` beside `token == "a"` keeps ESS-COMMAND-005 with the Unsupported text.
- `defined_alone_without_a_default_names_the_absent_case`: ESS-COMMAND-005 names an uncovered case with `token` absent.
- `a_default_bearing_command_keeps_its_enum_only_proof`: two `defined(token)` branches plus a default stay valid.
- `a_held_state_partition_keeps_its_decline`: a `when_subject_state:` command with a `when: defined(input.x)` refusal answers as it does on 0.74.0.

In `crates/verify/ess-conformance/tests/`:

- `finite_enum.rs`, `analyze_keeps_declining_presence`: `finite::analyze` over `Defined` of an Optional input is `None`. `cargo test -p ess-conformance --locked --test finite_enum analyze_keeps_declining_presence`
- `when_subject_witness.rs`, `a_defined_pair_witnesses_both_presence_states`: the stored and input pairs synthesize one scenario per branch, all passing on the interpreted target. `cargo test -p ess-conformance --locked --test when_subject_witness a_defined_pair_witnesses_both_presence_states`
- `negated_defined_beside_when_subject.rs`, `the_guard_negate_mutant_of_a_defined_pair_is_killed`: `cargo test -p ess-conformance --locked --test negated_defined_beside_when_subject the_guard_negate_mutant_of_a_defined_pair_is_killed`

In `crates/generate/ess-synth/tests/finite_presence.rs` (new file):

- `presence_partition_reaches_generated_targets` (mirrors `finite_boolean.rs`): generated Rust selects each branch. `cargo test -p ess-synth --locked --test finite_presence presence_partition_reaches_generated_targets`

Package gates: `cargo test -p ess-cli --locked` and `task test-xtask` (CHANGELOG, website).

## Scope

- `crates/specify/ess-domain/src/command/finite.rs`: presence domain in `input_paths` and `analyze_inputs`, kept apart from `analyze`; `Case` / `FieldCase` absence; the Decline text. Cited.
- `crates/specify/ess-domain/src/command.rs`: the deferral at about lines 4119-4134. Cited.
- `crates/specify/ess-domain/src/command/subject_fact.rs`, `related_guard.rs`, `subject_state.rs`: expected unchanged, since the mode is chosen by `default.is_none()`. Inferred.
- `crates/verify/ess-conformance/src/witness.rs`: expected unchanged (line 595 uses `analyze`). Inferred.
- `docs/design/closed-enum-outcome-coverage.md`, `website/docs/guides/specify/fields-and-invariants.md`, `CHANGELOG.md`: cited.
- Tests listed under Acceptance: inferred (`finite_presence.rs` is a new file).
- Confidence: medium. The prover site is cited. Whether synthesis witnesses a pair with both sides guarded is shown only for the one-default variants; 0.74.0 refuses the both-guarded pair, so that case is checked by the first red test.
- Shared with `story:when-subject-reads-the-addressed-identity` (both touch `crates/specify/ess-domain/tests/subject_guard_input.rs`, and possibly `subject_fact.rs`). Other units: unknown.
- Safety fact: every no-default closed-mode caller refuses when the proof declines (subject_fact.rs:607-626, command.rs:4067-4078, related_guard.rs:1899-1912 and :2019-2024), and any `Defined` node made the proof decline (finite.rs:152). So no document valid today has a guard those provers read through `defined`. Enabling presence there cannot change a valid document, and the one caller whose decline admits (subject_state.rs:348) is excluded. Level 2-3, unproven.
- Not established: whether a pair with both sides guarded is witnessed by synthesis and handled by the generated targets (refused by 0.74.0, so unrun); that the reason-less ESS-COMMAND-005 on an input pair comes from the shape-time pass at command.rs:4119-4134 (read from the code, not traced); and whether adding presence to held state could newly refuse a valid document (a reading of subject_state.rs:340-350, not a reproduction).
