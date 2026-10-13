---
format: aep.planning-md/3
id: story:system-issued-command-is-awaited-not-driven
kind: story
status: draft
title: A system-issued command is awaited, not driven
revision: 2
---
## Outcome

An actor can be declared the system itself (`system: true`); a command granted only to system
actors is system-issued. Conformance does not ask a target to invoke it: the scenario awaits the
command's outcome on an instance within a declared bound, compares inputs the system decides,
checks minted values by type, and reads a refusal of a system-issued command as "does not happen
within the bound" (no events, subject unchanged), never as a typed error.

Closes https://github.com/beyond10x/ess/issues/542.

## Fit review

1. **Need.** Every transition needs a causing outcome (ESS-ENTITY-005), so a lifecycle the system
   moves itself (a scheduler choosing an agent, a deadline sweep) is modelled as commands of an
   internal actor. The synthesized suite then drives them, which a real system cannot honestly
   accept: one adopter has 67 of 129 scenarios unsupported on five such commands, its happy path
   included.
2. **Class: gap** (a new conformance step kind).
3. **Already expressible?** No. `bindings:` maps one event onto one command's input across
   components and cannot express "the system picks".
4. **Fit.** A flag on the actor (not on the command), so grants stay the one place that says who
   issues a command. A suite step gains an `await` form; the Target interface gains
   `AwaitOutcome(command, instance, bound)` answering the outcome reached, unsupported, or nothing
   within the bound. Runners that do not implement it answer unsupported, so existing targets keep
   their results.

## Decisions

- `actors: [{name, system: true}]`; a command whose every grant is a system actor is
  system-issued; mixed grants stay ordinary (driven).
- Bound: a per-command `await_within: <duration>` with a default the suite states; a refusal
  scenario of a system-issued command passes when no outcome is reached within the bound and the
  subject is unchanged.
- Inputs the system decides are compared as stated; `{generated: true}` inputs/payload fields are
  checked by type (#545's present mint applies).
- New suite step kind is a new major of `ess-conformance/*`; the source key is `ess/25`.
- Staged: stage 1 (this story) is declaration, validation, synthesis of await steps, the
  interpreter answering them, and the Rust runner. Stage 2 (story
  `system-issued-command-await-in-go-and-typescript-runners`) adds the Go and TypeScript runners.
  #542 closes when both ship.
- Reversible: a new optional actor flag and step kind; documents without them keep their bytes.

## Acceptance

Each line names the failing test written first.

- `crates/specify/ess-domain/tests/system_actor.rs` `a_system_actor_validates_at_ess_25`,
  `a_command_granted_only_to_system_actors_is_system_issued`, `mixed_grants_stay_driven`.
- `crates/verify/ess-conformance/tests/system_issued_command.rs`
  `synthesis_awaits_a_system_issued_command_instead_of_driving_it`,
  `a_refusal_of_a_system_issued_command_expects_nothing_within_the_bound`,
  `the_interpreter_answers_an_await_step`, `an_old_suite_keeps_its_bytes`.
- Rust runner (file inferred) `await_outcome_unsupported_by_default_keeps_old_targets_green`.

## Scope

- inferred: `crates/specify/ess-domain/src/{actor*,wire.rs,grants*}`,
  `crates/specify/ess-compiler/src/ir.rs`, `crates/verify/ess-conformance/src/{synthesize.rs,
  suite format, interpret/, rust runner}`, `docs/design/` conformance step page,
  `website/docs/reference/formats.md`, `spec-versions.md`.

## Plan

- depends_on: the unit that opens `ess/25`; #545's present mint for minted input checks (soft).
- du estimate: 10G.
- adversary: yes (a new suite major and step semantics).

## Design pass

Design pass (2026-10-13, unit commit e6c4b4c46, merged on `integrate/ess-acd-gaps-20261013` as fda971d48): `docs/design/system-issued-commands.md`. Its Fit review answers all seven questions of `.agents/skills/assessing-external-requests/SKILL.md` with citations. Decision: **accept, redesigned**.

Departures from the decisions above, accepted by the coordinator:

- Refusals decided by the record (`wrong_state:`, `when_subject*`, `unknown_instance:`) read as "not issued". An `external:` refusal reads as an outcome the system reached. A refusal decided by the input or by `when_related:` gets no synthesized scenario (`ESS-SYNTH-024`).
- With no `await_within:`, the bound is the runner's deadline.
- Inputs the system mints use the existing `{generated: true}` value source.
- A target that implements await holds the step until the scenario awaits it.
- Stage 1 (Rust) and stage 2 (Go, TypeScript) ship in one release.
- Suite majors and code numbers follow merge order.
