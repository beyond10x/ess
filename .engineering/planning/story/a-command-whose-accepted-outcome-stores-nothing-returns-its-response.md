---
format: aep.planning-md/3
id: story:a-command-whose-accepted-outcome-stores-nothing-returns-its-response
kind: story
status: draft
title: 'A command whose accepted outcome stores nothing may return its response: accepts: nothing with returns: true'
summary: A downstream read tool over a computed answer is refused ESS-COMMAND-004 on 0.99.2 and 0.99.9; the nearest admitted form addresses a stored row
tags:
- gap
- gap-reports-2026-11-28
relations:
- serves: vision:O2
- informed_by: story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view
revision: 1
---
## Need

A downstream specification of reusable building blocks (ess 0.99.2, checked on 0.99.9 and on
`main` at `298790bd0`) has read tools: commands that take a few inputs, change nothing, emit
nothing and answer a list. One outcome per command, declared as the model's own words say it:
`accepts: nothing` (no immediate view changes) and `returns: true` (the outcome answers the
declared `response:`). `ess specify validate` refuses it:

```text
[conflicting_declaration] command.shelf.books.ListTitles.outcomes.listed.returns: returns requires a nonempty response and no error, accepts: nothing or replays
error[ESS-COMMAND-004]: returns requires a nonempty response and no error, accepts: nothing or replays
```

Neutral reproduction on 0.99.9:
`.engineering/repro/a-command-whose-accepted-outcome-stores-nothing-returns-its-response/system.yaml`
(`ess/37`, a command `ListTitles` with `response: [{name: titles, type: List<String>}]` and one
outcome `{name: listed, accepts: nothing, returns: true}`). `ess specify validate --path system.yaml`
exits 1 with the refusal above. The same refusal holds on 0.99.2 and on `main`.

What an author writes instead is the nearest form the validator admits,
`preserves: <Entity>` with `instance:` read from input plus `returns: true`, which addresses a
stored row and needs an immediate identity view of its entity
(`story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view`). That form says a row
exists; a read tool over a computed answer (a directory listing, a search over a store the model
does not hold) has no row to address, so the author declares what the model cannot state.

A command whose answer is computed from a state the model does not hold is a common shape of a
tool or a query service; the same refusal stops any specification of one.

## Outcome

An accepted outcome that stores nothing and emits nothing may declare `returns: true` over a
nonempty `response:`: it validates, the interpreter answers a value of the response's type, the
generated suite checks the answer's shape and that nothing changed, and generated Rust and Go keep
the command an obligation naming the answer. Where the pairing stays refused, the refusal names
the form that works, so an author is not left with a conflict between two keys that each read as
right.

## Acceptance

- `ess specify validate --path system.yaml` on the reproduction exits 0 and
  `ess verify conform synthesize` writes a scenario for `ListTitles/outcome/listed` that
  `ess verify conform run --target interpreted` passes (decision RD1 A); or, if B is taken, the
  refusal's help line names `preserves:` with `instance:` and an identity view.
- A command with `returns: true` over an empty `response:` is still refused with the existing
  message.
- No model that validates today changes its IR bytes; the format key is decided with the fix
  (below the key the pairing keeps its refusal, naming the key).

## Fit review

1. Need: a read tool over a computed answer is declarable. Reproduction above, one shape.
2. Class: gap in the language (a refusal of two keys that each hold alone), not a defect of
   synthesis.
3. Already expressible: `preserves:` + `instance:` + `returns: true` for an answer read from a
   stored row; nothing for a computed one.
4. Fit: `returns:` and `accepts: nothing` are existing outcome shapes
   (`docs/design/outcome-shapes.md`); the pairing is the unit's missing cell.
5. Second adopter: any specification of a query or tool service whose answers are computed.
6. Cost: a format key, a validator check lifted, an interpreter answer for a value no rule
   determines, and a suite scenario that can check only shape and the absence of change.
7. Considered: keep the refusal and improve its message (B); admit the pairing (A); admit it only
   with a declared `answer:` rule over rows the command reads (C, larger; ties to the row-set
   reads).

## Decisions

- RD1 (open): A, admit `accepts: nothing` with `returns: true`, the answer checked for shape only;
  B, keep the refusal and name the working form in its help; C, admit it with a declared answer
  rule. Recommended: A. No rule of the model states the answer's contents; A leaves them to the
  implementation, as the story above does for a read the model cannot state.

## Scope

- inferred: crates/specify/ess-domain/src/command.rs
- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/specify/ess-domain/src/format.rs
- inferred: docs/design/outcome-shapes.md
