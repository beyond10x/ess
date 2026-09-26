---
format: aep.planning-md/2
id: story:optional-guards-mean-what-they-say
kind: story
status: active
title: An Optional guard is refused or witnessed as its author meant
relations:
- serves: vision:O2
revision: 3
---
## Outcome

A guard that reads an `Optional` field means what its author meant: `x == null` is refused at
validate with a hint, and both sides of a presence guard (`defined(x)` / `not defined(x)`) get a
synthesized scenario.

## Why

GitHub issue beyond10x/ess#93 (`ess` 0.32.0). `when: note == null` on `note: Optional<String>`
validates and synthesizes an input carrying the four-character string `"null"`. `not defined(x)`
validates, but synthesize refuses with `ESS-SYNTH-003`, because witness rule 1 fills every optional
(`crates/verify/ess-conformance/src/witness.rs:17-19`).

## Decision (operator default, 2026-09-25)

`== null` / `!= null` against an unquoted `null` are **refused** at validate with a hint naming
`defined(x)` / `not defined(x)`. A quoted `"null"` stays a text literal. No absence semantics are
added to `==`.

## Acceptance

- Red first: validate refuses `note == null` and `note != null` with a stable code and a hint naming
  `defined(note)`; a quoted `"null"` still validates as text.
- For each `Optional` path a guard reads through `defined`/`exists`, synthesis tries one candidate
  with that path omitted, bounded like rule 3. `defined(note)` / `not defined(note)` both get a
  scenario; a case asserts both.
- Witness rule 1's doc comment states the new exception.
- "Absent" vs "present but null" on the wire: the implementation states which the omitted
  candidate is, in the witness module doc.

## Out of Scope

Absence semantics for `==`; `when_subject` over Optional fields (#75).
