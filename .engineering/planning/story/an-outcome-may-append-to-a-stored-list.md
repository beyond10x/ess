---
format: aep.planning-md/3
id: story:an-outcome-may-append-to-a-stored-list
kind: story
status: draft
title: An outcome may append one element to a stored list
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22). An outcome adds one element to a stored list: the adopter's artifact-append command stores `coalesce(subject.artifacts, []) + [input.artifact]` (adopter `ess/domains/tasks.yaml:1409-1415`). ESS refuses the expression, so the outcome cannot say the list grows by one, and the stored list is left unspecified.

## Reproduction

Minimal spec on 0.68.0: an entity with `items: Optional<List<String>>` and an updating outcome with `sets: {items: {expr: "coalesce(subject.items, []) + [input.item]"}}`. `ess specify validate`:

```text
error[ESS-SPEC-012]: `coalesce(subject.items, []) + [input.item]`: `[` is no part of an expression; it is written with paths, literals, `+ - * /`, parentheses and function calls
  `ess-domain` refuses this as `unparsable_predicate`
```

## Where it lives

- The expression lexer refuses `[`: `crates/specify/ess-primitives/src/expr.rs:1453-1457`, traced through the message.
- No `sets:` source appends to a list (searched `ess-domain` and `ess-compiler`: no append form), inferred from the search.

## Acceptance (provisional)

- An outcome may store a list that is the subject's list, absent read as empty, with one element added at its end, written either as an expression with a list literal or as a dedicated `sets:` source; the design picks one.
- Validation types the element against the list's element type and refuses a mismatch with a named code.
- Generated Rust and Go store the appended list; the interpreter and conformance check the row holds the earlier elements and the new one last.
