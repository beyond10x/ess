---
format: aep.planning-md/1
id: story:create-only-command-cannot-refuse
kind: story
status: draft
title: A command that only creates cannot declare a refusal
summary: 'wrong_state: is the only refusal form and ESS-COMMAND-012 refuses it where nothing moves, so the claim survives only in a hand-written scenario'
revision: 1
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
