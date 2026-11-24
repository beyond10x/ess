---
format: aep.planning-md/3
id: story:generated-behaviours-fill-the-members-of-a-typed-response-that-the-model-sources-from-input-a-stored-field-or
kind: story
status: draft
title: Generated behaviours fill the members of a typed response that the model sources from input, a stored field or a literal, so only generated members stay owed
summary: 'determined::command owes every command with a typed response: whatever the members hold, the whole command stays hand-written (gap report, 2026-11-24)'
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:response-field-declares-its-value-source
revision: 2
---
## Need

Every command with a typed `response:` is an obligation in Rust synthesis, whatever the response
holds. The reporter counts 12 of them in one specification. A response whose members are an echo of an
input, a stored field or a literal is decided by the model; only `{generated: true}` members belong to
the implementation. Today the whole command is owed because it has a response at all.

Evidence (read at `origin/main`):

- `determined::command` returns `Err("a typed response (`response:`)")` first thing when
  `command.response` is not empty (`crates/generate/ess-synth/src/determined.rs:55-56`), before any
  outcome, guard or effect is looked at. The command keeps that one reason, so its decided guards and
  its set effects are not generated either.
- The design page gives the split: the generated code emits a command-specific typed response record,
  `{generated: true}` "means the implementation supplies the value", and "implementations still own the
  command behavior and return both its response and emitted facts"
  (`docs/design/typed-response-outcome-payloads.md:23, 29`). It does not say that a decided source
  (`input.<f>`, a stored field, a literal) must stay the implementation's.
- The same source table is generated elsewhere: event payloads and `sets:` entries render
  `InputField`, `Literal`, `SubjectField` (`determined.rs:446, 1603, 1977`).
- The draft `story:response-field-declares-its-value-source` (beyond10x/ess#506) adds sources to the
  response field in the model. This story is the generator half for the sources that exist today and,
  once that story lands, for its sources too.

## Outcome

A command with a typed `response:` is generated when its response members are decided sources: the
generated behaviour decides the outcome and fills those members. A member sourced `{generated: true}`
or `{response: …}` stays the implementation's and keeps the command owed, naming that member, not
"a typed response".

## Acceptance

- `ess generate synthesize --target rust` and `--target go` on a command whose response members are
  `input.<f>`, a stored field and a literal report `0 obligation(s)` for it (before: "a typed response
  (`response:`)"), and the suite, including `ExpectResponsePayload`, passes against each generated
  server.
- A response member sourced `{generated: true}` keeps the command owed and the obligation names that
  member and outcome.
- An optional source that is absent is observable as absent in the response (the rule at
  `typed-response-outcome-payloads.md:34`); a target that answers `null` fails the suite.
- A command with a response, a guard and a set effect has all three generated together (the guards and
  effects that were hidden behind the one reason).
- A specification with no typed response generates byte-identically.

## Scope

- cited: `crates/generate/ess-synth/src/determined.rs` (`command`, 55-56; the payload-value arms
  446, 1603, 1977), `rust/behaviour.rs` (response record, outcome helpers), `go/` behaviour renderer,
  `docs/design/typed-response-outcome-payloads.md`.
- Size: M, about 500 changed lines; Rust and Go side by side. The affected obligation text is pinned in
  tests (`served`/`determined` pins) and moves with it.

## Fit review

1. **Need.** Typed-response commands should not all be hand-written. Reporter's marker: 12 commands.
2. **Class: gap** in generator coverage.
3. **Already expressible?** The facts: yes. Splitting each command into one with the response and one
   without would change the model to suit the generator.
4. **Fit.** Effect by target grid, row `response`, columns Rust and Go. Reuses the payload-value arms.
   The interpreter fills the same members already.
5. **Second adopter.** Any command that answers the created row's echoed input and a status literal.
6. **Cost.** No format step, no key. The obligation text for the commands that stay owed changes.
7. **Alternatives.** Change nothing; generate the response record only and leave the behaviour owed.

## Decisions

AC9-D10: how much of a response the generator may fill. A: input, stored field, literal. B: only literals and input echoes. C: decline; typed responses stay the implementation's. Decided: A, 2026-11-24 (the coordinator, under the owner's delegation); accepted as redesigned (the split is per member, not per command).
