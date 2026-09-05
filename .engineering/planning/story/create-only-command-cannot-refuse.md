---
format: aep.planning-md/1
id: story:create-only-command-cannot-refuse
kind: story
status: draft
title: A command that only creates cannot declare a refusal
summary: 'wrong_state: is the only refusal form and ESS-COMMAND-012 refuses it where nothing moves, so the claim survives only in a hand-written scenario'
scope:
- confidence: inferred
  path: crates/generate/ess-gen
- confidence: inferred
  path: crates/generate/ess-synth
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/specify/ess-domain
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: crates/verify/ess-diff
- confidence: inferred
  path: docs/design
revision: 10
---
## The defect

A command whose every outcome only `creates:` cannot declare a refusal. `wrong_state: true` is the
only way the format offers, and `ESS-COMMAND-012` refuses it — correctly, on its own terms, because
there is no subject to be in a wrong state:

```console
$ ess specify validate --path model
model was refused:
  - [unreachable_branch] command.acd.routing.SubmitTask.outcomes.already-queued: outcome
    `already-queued` is taken when the subject is in a state no move starts from, and no outcome of
    `acd.routing.SubmitTask` moves an entity at all, so there is no subject and no state for it to
    be about (hint: drop the branch, or give the command a `moves:` whose transitions leave some
    declared state out)
```

`validate_wrong_state_is_reachable` (`crates/specify/ess-domain/src/entity.rs:1117-1174`) computes
the wrong-states as the entity's states less the `from` sets of the command's transitions, and
`creates:` is not a transition (`crates/specify/ess-compiler/src/ir.rs:566-570`). So the set is empty
and every `wrong_state:` branch on such a command is unreachable by construction.

## Why the hint does not answer it

The refusal suggests giving the command a `moves:`. Reproduced against a real model — `acd/specs`
`acd.routing.SubmitTask`, which creates a call — that is not available: the implementation refuses a
duplicate submit from **two** of the entity's four states, and the other two fail its own state table
with a different error entirely. A `moves:` written to satisfy the checker would claim transitions
the system does not make.

## What it costs

The claim does not disappear; it moves somewhere weaker.

- An **authored** scenario can still assert it, because the authored compiler checks `error:`
  against the domain's declared errors rather than against the command's branches
  (`crates/verify/ess-conformance/src/authored.rs:1820-1834`). So the refusal is expressible by
  hand.
- **Nothing generated covers it.** `crates/generate/…/synthesize.rs:2734-2738` skips the family for
  the same structural reason, so the specification derives no obligation from it.
- `ess verify impact` traces such a scenario to the **error**, not to an outcome of the command that
  raised it, so the command's own surface does not show that it can refuse.

Net: a real refusal that two implementations disagree about is checkable only if somebody remembers
to hand-write the scenario, and the model cannot say the command refuses at all.

## Where it was found

Writing a parity scenario for ACD: Go answers 409 to a duplicate `POST /submit` from a typed
sentinel; `acd-rs` answers 400 from an untyped string. The scenario discriminates — Go passes, Rust
fails on `ESS-CF-ERROR`, *"no declared error was carried"* — but the model records the refusal only
in a comment, because it cannot record it anywhere else.

## What would close it

Not proposed as a design, only as the shape of the gap: a create-only command needs some way to say
*"this input is refused, and here is the declared error"* without borrowing a lifecycle it does not
have. A branch keyed on the creation conflicting with an existing instance would fit what both ACD
implementations actually do, and would let the synthesiser derive the obligation rather than waiting
for an author.

## Scope

Derived 2026-09-05 by `aep-drive:story-scoper` from the story and repository tree — cited.

- **Primary surface:** `crates/specify/ess-domain` — cited; `src/entity.rs:1117` validates wrong-state reachability, while `src/command.rs:318` defines `OutcomeCondition` and `src/command.rs:384` defines its test strategy.
- **Compiler surface:** `crates/specify/ess-compiler` — cited; `src/ir.rs:566` excludes creation from transitions, and `src/resolve.rs:1459` lowers each outcome's condition, subject, test strategy and error into the IR.
- **Conformance surface:** `crates/verify/ess-conformance` — cited; `src/synthesize.rs:2723` implements `refused_here`, whose mover filter excludes creation; `src/authored.rs:1820` resolves authored error claims independently of command outcomes. The story's abbreviated synthesis citation resolves to this crate.
- **Also likely:** `crates/generate/ess-gen` — inferred; a distinct refusal condition would require handling in the exhaustive HTTP-status and generated-document mappings at `src/http.rs:84`, `src/openapi.rs:852` and `src/docs.rs:1534`.
- **Also likely:** `crates/generate/ess-synth` — inferred; a distinct condition would require updating the exhaustive `condition_phrase` match at `src/plan.rs:640`.
- **Also likely:** `crates/verify/ess-diff` — inferred; a distinct condition would require updating the exhaustive `written_condition` match at `src/diff.rs:829` and checking semantic impact attribution.
- **Documents:** `docs/design` — inferred; the story leaves its proposed conflict branch undesigned, and repository guidance requires a binding design before introducing a construct. The exact design page is unresolved.
- **Tests:** domain parsing/validation, compiler lowering and conformance synthesis coverage belong within the corresponding crate surfaces above; downstream compatibility tests depend on the selected representation — inferred.
- **Confidence:** medium — inferred; the existing limitation and core locations are explicit, but the story proposes a gap rather than selecting the semantics or persisted representation that would close it.
- **Would collide with:** any work touching the exact directory tokens `crates/specify/ess-domain`, `crates/specify/ess-compiler`, `crates/verify/ess-conformance`, `crates/generate/ess-gen`, `crates/generate/ess-synth`, `crates/verify/ess-diff`, or `docs/design`; reassess conditional surfaces after the design is selected — inferred.

## Current-source qualification

The scoper found that `Outcome::is_refusal` is `error.is_some()` at `crates/specify/ess-domain/src/command.rs:954`; the historical claim that wrong_state is the only refusal form is broader than current source supports. The remaining gap is an existing-instance conflict for lifecycle-free creation; its lookup/identity semantics remain undecided. This item is excluded from review remediation scheduling.