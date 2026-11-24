---
format: aep.planning-md/3
id: story:a-second-input-refusal-whose-guard-equals-an-earlier-one-is-refused-as-unreachable
kind: story
status: active
title: A second input refusal whose guard equals an earlier one's is refused as unreachable, where the finite prover cannot read the guard
tags:
- external-request
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:24:43Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T03:24:45Z", actor: "human:timo", revision: 3}
---
## Need

beyond10x/ess#816 item 3. A copy of a `when:` refusal under a new name, with an identical guard,
validates, though the second can never answer: the first always answers before it. Validation
already refuses a branch whose guard an earlier input refusal implies, as `unreachable_branch`
(`Command::validate_implied_input_guards`, `crates/specify/ess-domain/src/command.rs:4152`), but it
decides implication with the finite prover only (`implies(later_guard, earlier_guard) ==
Some(true)`, 4188 before the change), and the prover declines an `Integer` comparison such as
`input.amount > 10`. So the reported shape passed while the same copy over a `Boolean` was refused.

## Reproduction

On 0.98.0, `triage/final/run/816/m/system.yaml` (`dup.a.Add`, `too-large` and `too-large-copy`,
both `when: input.amount > 10`): `dup v1 — 1 file(s), valid`. The same copy over a `Boolean`
(`run/816/b/`, `when: flag == true` twice) is refused:
"[unreachable_branch] command.dup.a.Add.outcomes.too-large-copy: `too-large-copy` is unreachable:
every input satisfying `flag == true` is answered first by `too-large` (flag == true), an
input-guarded refusal declared before it ... ESS-COMMAND-012".

## Outcome

Two equal guards count as implied without asking the prover: the later refusal (or accepting
branch) is refused with the existing `unreachable_branch` message, naming it and the earlier
refusal that shadows it. Equality is the guard expressions' own equality, the value the analysis
already holds (`later_guard == earlier_guard`); no second notion of guard equality.

## Acceptance

1. Red before the change: in `crates/specify/ess-domain/tests/implied_input_guard.rs`,
   `an_equal_guard_the_prover_declines_is_unreachable`: a command with two refusals `when: count >
   5` is refused as `unreachable_branch` at the second, naming the first. Red on the base
   ("validates, no unreachable_branch"), green after.
2. Controls stay valid (`guards_the_prover_declines_that_differ_stay_valid`): two refusals with
   different literals; the same guard on two different commands; a first refusal narrowed by a
   second conjunct the other lacks.
3. The existing cases of the target stay green (`an_equal_guard_declared_later_is_unreachable`,
   `a_broader_later_branch_stays_valid`, `an_undecidable_implication_is_not_refused`).
4. No repository example or fixture is refused (else the step would have been `LEFT: needs` a
   format step).

## Scope

- `crates/specify/ess-domain/src/command.rs`: `validate_implied_input_guards` (4152; the rustdoc
  at 4145-4151 and the condition at 4186-4190).

## Owned files

- `crates/specify/ess-domain/src/command.rs`
- `crates/specify/ess-domain/tests/implied_input_guard.rs` (new `#[test]`s only)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (`ess-domain/src` is
  hashed by the stamp)
- `CHANGELOG.md` (one bullet)

## Gate

- `task fmt-check`
- `cargo clippy -p ess-domain --all-targets --locked -- -D warnings`
- `cargo test -p ess-domain --locked --test implied_input_guard`
- `cargo test -p ess-cli --locked --test <one validation target>` (the table: `ess-domain` also
  gates `ess-cli`)
- `task test-xtask` (`CHANGELOG.md` changed; the refusal text is the existing one)
- the sources stamp:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`

**Open on the commit:** `3214e53c6` changes `ess-domain/src/command.rs` and does not touch
`synthesis-sources.digest`, whose last change is `9d75b6524`. The stamp hashes every `src/` file of
`ess-domain` (`triage/defects/check.md`, C1), so
`the_synthesis_sources_digest_matches_the_committed_stamp` is red on the branch from this commit
until the stamp is rewritten by the command above, in a follow-up commit of batch 1.

## Depends on

Nothing.

## Changelog

As committed, under `Fixed` with a **Breaking** lead: "`ess specify validate` refuses a `when:`
branch whose guard is the same expression as that of an input-guarded refusal declared before it,
as `unreachable_branch`, where the finite prover cannot read the guard, such as two refusals both
`when: input.amount > 10`; the copy validated although it can never answer (beyond10x/ess#816)."

## Fit review

1. Need: a copied refusal that can never answer is a mistake an author wants to hear about.
2. Class: defect. The rule refuses a branch an earlier input refusal implies
   (`command.rs:4152`, rustdoc); it missed equal guards the prover declines.
3. Already expressible: not applicable (a refusal of a mistake).
4. Fit: the existing code and message; no new code.
5. A second adopter: any specification grown by copy and paste.
6. Cost: breaking for a specification that holds such a copy (it is refused); none known in the
   repository.
7. Considered: a structural normal form of guards (rejected: a second notion of equality); expression
   equality where the prover declines (chosen).

## Decisions

- Decision: accept as proposed. Built as `3214e53c6`.
