---
format: aep.planning-md/3
id: story:validate-bundle-enforces-a-string-equality-invariant
kind: story
status: draft
title: validate-bundle enforces a string-equality invariant
summary: 'spindle wave 12: format == "..." validates but validate-bundle does not enforce it (ess 0.88.0)'
tags:
- backlog
- from-consumer
- queue-0.91.0
- rank-7
relations:
- serves: vision:O2
revision: 1
---
## Need

Reported by spindle wave 12 against ess 0.88.0. An invariant that compares a string field with a
literal (`format == "…"`) passes specification validation, but `validate-bundle` does not enforce
it. A bundle whose value breaks the invariant validates. The report gives no line reference. The
first step is to reproduce it and name the code path.

## Acceptance

- A model with a string-equality invariant on a value object, and a bundle that breaks it:
  `validate-bundle` refuses the bundle and names the invariant.
- A bundle that satisfies the invariant validates as before.
- If `validate-bundle` deliberately enforces only some invariant shapes, specification validation
  refuses or warns on an invariant it will not enforce. The reference page says which.

## Scope

- inferred: the `validate-bundle` command in `crates/edge/ess-cli`, and invariant evaluation in
  `crates/specify/ess-domain` or the interpreter.
