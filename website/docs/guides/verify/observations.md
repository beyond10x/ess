---
title: Observations
sidebar_position: 7
description: "What a suite observes beyond an act's own answer: held-state outcomes, retries, selection, periodic activity and clocks, event context and binding accessors."
---

# Observations

What a suite checks beyond a command's answer and the events it publishes: outcomes selected by held
state, retries of an original result, selection, periodic activity and clock evidence, events
delivered with their context, and bounded binding accessors.

## Observe outcomes selected by held state

For a command declaring `when_subject_state`, synthesis establishes a real
reachable state, queries a declared immediate view exposing identity and state,
invokes the command, and checks the resulting row. The same input can therefore
prove different outcomes from different held states. An implementation that
returns the expected outcome while incorrectly changing the row fails.

The initial adapter requires an unfiltered immediate view without parameters.
Missing observation authority or an unreachable required state produces an
explicit synthesis refusal. This uses existing command and view assertions;
the state guard alone does not require a newer suite vocabulary. The source
declaration requires `ess/3`.

## Observe retries of the original result

Source `ess/7`, introduced in 0.29.0, lets an outcome declare `replays` with
the name of an earlier successful outcome in the same command. The generated
witness executes that original outcome, captures its actual result and subject
identity, then retries the same input with the same actor. The retry must return
the exact original typed response without an error, direct event or subject
change.

Synthesis must establish that the retry's own condition holds immediately after
the original command. An unreachable or unprovable immediate retry produces a
named refusal. A valid model can still require an authored witness for a retry
that becomes eligible only after later activity.

The target must expose the complete subject through declared immediate,
unfiltered views. Complete observations check required fields and their types
in both actual query results before comparing all returned fields. Returning
only the identity fails even when that partial row stays unchanged. Optional
absence is distinct from null; extra returned fields also participate in the
comparison. Declared Integer values must reach the adapter without rounding.
Recursive Decimal and Binary64 response or complete-row positions are outside
this observation profile and refuse.

Source `ess/7` uses the same complete observations for ordinary `wrong_state`
refusal witnesses. Existing independent snapshot steps remain available with
their original, weaker contract. A generic error assertion alone does not
establish subject preservation.

These observations select suite/12 or declared-coverage suite/13 and require
report/2 in Rust, generated Go and generated TypeScript. Browser replay refuses these
envelopes. An immediate retry witness does not prove
restart recovery, retries after a later head, or absence of physical writes;
those remain implementation-specific acceptance.

## Observe selection, periodic activity and clock evidence

Selection observations, introduced in 0.23.0, compare actual source occurrences and selected
indices, preserving optional absence and occurrence-based exclusion. A host
conversion remains an explicit obligation; declaring the conversion does not
execute or prove it.

Periodic checks exercise the named host under controlled target time: ready,
initially inactive, failed first read and slow first read. The runner examines
actual scoped timer, read and invocation facts, including the complete live
interval and a window after stop acknowledgement. Missing ticks, overlapping
work, stale mapped inputs and post-stop activity fail. Unsupported authority
produces a non-passing result. The bounded check observes at most five live
periods; it does not sleep in the runner.

Clock comparisons use the typed `ExpectReadingOrder` operation with references
to already observed event members. The adapter supplies occurrence-scoped
process, epoch, origin and formatter facts; the runner normalizes and compares.
Declared alternative origins are requirements, not evidence. Wrong occurrence,
unknown offset, differing process/epoch or mutated request requirements cannot
establish success. Authored YAML convenience syntax for this comparison is not
provided; construct and persist the typed operation through the admitted writer.

These operations use suite/6 or declared-coverage suite/7 and explicit report/2.
Previous readers refuse their vocabulary. Controlled adapter tests do not prove
that an unrelated production adapter implements these capabilities.

## Deliver an event with its context

A binding that declares a delivery context (`ess/18`) reacts to an event that nothing in the
specification publishes. No command in the suite can make that event happen, so the suite
delivers it with the `deliver_event` step. The step names the event, the external channel
(`authority`), the occurrence's fields and the context the channel binds. The suite chooses
all of these values, so every expected input is a value it put in.

| Scenario | What it requires |
|---|---|
| `mapping` | One event is delivered under two different contexts. Each invocation carries its own delivery's context. |
| `delivery` | Two occurrences are delivered under two contexts, and then the event is redelivered. Every invocation for the second occurrence (`expect_every_invocation`) carries the second context. |
| `flow`, `on-failure` | The same as for any binding, with the event delivered by the suite. |

A target implements `deliver_event` in Rust. It binds the context as a real channel would,
and the invocation must read the context from the delivery, never from the payload. A later
`redeliver_event` repeats the most recent delivered occurrence, with that occurrence's own
context.

If a target cannot deliver an event with its context, it answers unsupported, which is the
default. Its scenarios are then recorded `unsupported` with the target's reason, and are never
passed.

These steps use suite/30, or suite/31 with declared coverage. Rust, Go and TypeScript
execute them with report/2.

## Observe bounded binding accessors

The `ess/3` binding paths described in
[Write a specification](../specify/bindings-and-components.md#read-a-field-inside-an-event-envelope)
require new suite vocabulary, introduced alongside them in 0.23.0. An ordinary suite retaining an
accessor uses `ess-conformance/6`; a declared-coverage suite retaining the new accessor
vocabulary uses `ess-conformance/7`. Models that do not retain that vocabulary
keep their existing suite formats. Both new versions require explicit
`--report-format 2` for execution. Report format 1 is refused before the target
runs, even without `--report-out` or when incomplete execution is allowed.

The observation checks the declared path against the triggering event and the
resulting command input. It distinguishes an unavailable path from a terminal
value and rejects missing required members, invalid union discriminators and wrong
payload kinds along the traversal. Whole-value collection copies do not validate
their elements. A conversion declaration supplies a reason
for a type crossing, not an executable conversion algorithm: a mapping whose
expected input cannot be determined receives a capability refusal.

Native Rust and Go values can distinguish nested Optional states that serialize
to the same JSON `null`. When that distinction is necessary to decide whether a
mapping is correct, conformance refuses the ambiguous observation instead of
guessing. Copying the typed native value remains distinct from proving that copy
through a JSON observation.

Ordinary suite/6 retains unknown coverage. Suite/7 uses the same exact-input and
parent-lineage rules as suite/5; selecting a child preserves its version and
requires the original parent chain. A passing run proves only the admitted
selection and does not resolve any recorded refusal.
