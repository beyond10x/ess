---
format: aep.planning-md/3
id: story:go-explorer-generates-alphabet-constrained-inputs
kind: story
status: draft
title: The Go explorer generates inputs whose type declares an alphabet
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The generated Go explorer draws random sequences for a command whose input type constrains its
values with `alphabet:`, generating values the alphabet admits, instead of excluding the command.

## Evidence

An adopter on ess 0.56.0: the explorer excludes a command with "input `code_verifier` is
`demo.CodeVerifier`, which constrains its values". https://github.com/beyond10x/ess/issues/512
covers List inputs and related guards only; this is a third exclusion reason. Effect: a whole
endpoint gets no random sequences.

## Acceptance

- A test generates the Go explorer for a command whose input newtype has `alphabet:` and a
  length bound; the command is in the explored set and every generated value satisfies the
  alphabet and bounds.
- An input constraint the explorer cannot generate for still excludes the command, naming the
  constraint.
