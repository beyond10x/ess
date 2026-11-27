---
format: aep.planning-md/3
id: story:a-component-declares-level-and-time-steps-that-validate-checks-and-the-interpreter-runs
kind: story
status: draft
title: A component declares level and time steps that validate checks and the interpreter runs
summary: A component that acts on its own (a command run per row a view answers, a command run every period) has no declaration; the time form is partly a periodic binding, the level form has none
tags:
- downstream-report
- gap
- issues-2026-11-26
relations:
- serves: vision:O2
- informed_by: story:periodic-binding-trigger-contract
- informed_by: story:system-issued-command-is-awaited-not-driven
revision: 2
---
## Need

A downstream product has components that act on their own, in two forms, and the specification cannot say so. A level step: while a view answers rows, an actor runs a command (for each answered row). A time step: an actor runs a command every period. Wanted by the reporter, declared on the component:

```yaml
steps:
  - {form: level, view: <view>, command: <command>, actor: <actor>, every: <period>}
  - {form: time, command: <command>, actor: <actor>, every: <period>}
```

`validate` checks that the view and the command exist and that the actor may run the command and read the view, and the interpreter runs the steps. Today the product keeps the steps in a comment table and runs hand-written controllers in its simulator, so nothing checks the table against the model and the suite never exercises the loop.

The reporter's spelling above is theirs, one option below, not the outcome.

## What main does today (origin/main 14248d409e)

- A component declares what it owns, accepts and publishes, who reaches it, its command-line surface, naming, summary, settings and refs, and refuses any other key: `crates/specify/ess-domain/src/component.rs:119-165` (`RawComponentSpec`, `deny_unknown_fields`). It has no key for work it starts itself.
- A time step has a part of its meaning today as a periodic binding cause: `when: {periodic: {every: PT2S, anchor, first, cadence, overlap, missed, lifetime, cancellation, host: {owner, authority, eligibility, context_fields, read_fields}}}` with `invoke: {command}` (`docs/design/periodic-binding-triggers.md`, `website/docs/guides/specify/binding-context.md:50-85`, `crates/specify/ess-domain/src/binding.rs:166-243`). It names a component as `host.owner` and a command, takes the whole-second `Period` (`PT<seconds>S`), and is validated against the declared command and component. It names no actor; the interpreter does not run it: a periodic host's inputs "have not been supplied" and the answer is `unsupported` (`crates/verify/ess-conformance/src/interpret/bindings.rs:399`, `await_outcome.rs:213-217`).
- A command only the system issues is declared by grants: an actor with `system: true` and a command granted to no other actor (`docs/design/system-issued-commands.md`, ess/27, implemented). The design refuses a second key that names who issues a command: "grants stay the one place that says who issues a command, and no second key can disagree with them". A binding whose `invoke:` names such a command is refused as `conflicting_declaration`. The runtime awaits the system-issued command's outcome on an instance within `await_within:`; it does not run it.
- An actor reads a view where its `may:` names the view (`crates/specify/ess-domain/src/actor.rs:175-180`, `may_read`, from ess/22), so "the actor may read the view" is a check main already answers.
- No level form exists: nothing in `docs/design/` or the format starts a command from the rows a view answers. A search of the design pages and the store for `controller`, `reconcile`, `level`, `trigger`, `schedule`, `every`, `step` finds, for the time form, `story:a-binding-may-be-triggered-by-a-period` (archived, delivered as the periodic cause), `story:periodic-binding-trigger-contract` (implemented) and `story:a-periodic-binding-may-run-on-a-wall-clock-schedule` (draft, a different need: a wall-clock schedule); for the level form, nothing. The `controller` hits of the store are other things (`story:a-view-answers-a-request-not-only-a-model`, `story:review-execution-recovery-design`).

## Decisions

Open, lettered. The form is open. Recommended: C.

- D1, the declaration.
  - A: the reporter's form: a component key `steps:` naming view, command, actor and `every`. Cost: a second place that names who runs a command, beside grants (the rule above), checked for consistency; a second spelling of a period cause beside the periodic binding; a format step.
  - B: no key on the component. The time step is the periodic binding that exists; add one new binding cause, a level cause naming a view, so that a binding says what starts a command (an event, a period or a view's rows); the actor is the system actor the command is granted to. Cost: the periodic binding stays a binding with its host contract, which is heavier than the reporter's line; no component-level listing.
  - C: the component lists its steps in the reporter's spelling (`form`, `view`, `command`, `every`), and the actor is the one actor the command is granted to as a system actor, not named a second time; the compiler lowers a time step to the existing periodic cause (reusing `Period` and its validation) and a level step to a new level cause, so there is one meaning and one projection set; a command granted to several actors needs the actor named, and then validate checks it is among the grants. Cost: a format step and the lowering; the checks the reporter asked for hold (view and command exist, the actor may run the command and read the view).
- D2, what a level step passes to its command. A: one run per answered row, each command input filled from the row's field of the same name and type, with the identity input the row's identity; refused at validate where an input has no such field. B: one run per evaluation, no inputs from the row, every input of the command supplied or `Optional`. Recommended: A.
- D3, how the interpreter runs steps. Steps run after each command of a scenario, in the declaration order, until no level step's view answers rows, or until a bound named in the format is reached, which is a named refusal and never a silent stop; a time step runs once per elapsed period of the scenario's clock (`story:elapsed-time-claims`), not on wall time. Recommended as written.

## Acceptance

- At the new source format `ess specify validate` exits 0 on a component with one level step and one time step, and refuses: a view or a command that does not exist (`undeclared_reference`, the declared names in the hint); an actor that may not run the command, or may not read the view (`conflicting_declaration`, naming the grant that is missing); a period that is not a valid `Period` (the `Period` parse refusal, unchanged); a level step over a command with an input no row field fills (`type_mismatch` or `missing_declaration`, naming the input).
- Below the new format each key is refused as `unsupported_format_version` naming it; the compiled IR of a model with no step keeps its bytes.
- The interpreter, on a generated model, runs a level step until its view answers no rows and the model's views and rows after it are those the declared steps give; a time step runs once per period of the scenario's clock; a level step that never empties its view ends in the named refusal of D3, under the 60 s bound of one test.
- Synthesis writes a scenario for each step or names a refusal for it; generated Rust and Go either run the step or refuse it by name; `ess verify diff` reports an added, removed or changed step.
- `docs/design/` has a page for the construct, and `website/docs/reference` lists the keys.

## Fit review

1. Need, apart from the syntax: a system that acts on its own (a loop that works the rows a view shows, a sweep that runs every period) is a domain fact the component cannot state, so the suite never exercises it and a hand-kept table drifts from the model. Reporter's proposal: `steps:` on a component, as above.
2. Class: gap. The time form is partly expressible (periodic binding); the level form is not.
3. Already expressible: only the time form, as a periodic binding with a host contract and a system actor; that leaves the actor unnamed in the binding, the interpreter unable to run it, and no level form. An idiom that moves the loop into a hand-written controller moves a checked figure out of the specification, so it is no answer (policy 1, `docs/design/generic-language-features.md`).
4. Fit: grid R (what can be read where) admits a view's rows as a source in a command's position only as input; a level step opens a new row, "a view's rows start a command", and the siblings (bindings, events) must say the same or refuse by name. It reuses `every` and `Period`, the `system: true` actor and the grants as the one place that says who issues a command. Every target handles it or refuses it by name: Rust, Go, TypeScript, the interpreter, `ess verify diff`, the entity runtime, generated docs, synthesis.
5. A second adopter, brand-free: a job queue whose worker runs a `Lease` command for each row of a `pending` view; a cache whose sweeper runs `Expire` every minute.
6. Cost: a source format (the next open one), a new binding cause or component key, new checks on existing codes, an IR carrying a step, a diff classification, and for the interpreter a loop with a bound.
7. Considered: change nothing (rejected: the table stays a comment); A, B, C above, and for the time form only, the periodic binding as it is.

## Scope

- `crates/specify/ess-domain/src/component.rs`, `binding.rs` and `binding/periodic.rs`, `actor.rs`.
- `crates/specify/ess-compiler/src/ir.rs`, `resolve.rs`.
- `crates/verify/ess-conformance/src/interpret/`, `synthesize/`.
- `docs/design/` (new page), `website/docs/reference/`.

## Status

Pull request 842 (https://github.com/beyond10x/ess/pull/842, "feat: component steps, and batch 10's units") is open and not delivered (2026-11-27): red test `e7df708f0e` and fix `b6991cb9de` for component steps; the pull request also carries the open-format batch of `ess/38` and the story of the comparison over an absent Optional field. Delivery form D162 A. Not in any release.
