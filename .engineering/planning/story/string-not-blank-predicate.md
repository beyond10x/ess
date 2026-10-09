---
format: aep.planning-md/3
id: story:string-not-blank-predicate
kind: story
status: draft
title: A guard or constraint requires a string to be not blank
tags:
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A guard or a type constraint can require that a string is not blank (holds at least one
non-whitespace character), and synthesis witnesses both sides.

## Evidence

An adopter on ess 0.56.0 wrote `matches` for a non-whitespace rule and got "unknown operator". The
string tests are `starts_with`, `ends_with`, `contains` and equality; `alphabet:` on a newtype is
a whitelist of characters and cannot say "contains a non-whitespace character" over Unicode text.

## Acceptance

- A closed predicate (for example `blank` / `not blank`, Unicode `White_Space`) validates on a
  `String` input or field; no general regular-expression operator is added.
- Synthesis builds a blank and a non-blank witness, including a whitespace-only one.
- The interpreted, Rust, Go and TypeScript targets agree on the whitespace set; a vector file
  pins it.
- It adds an authored key, so it ships with a source format.
