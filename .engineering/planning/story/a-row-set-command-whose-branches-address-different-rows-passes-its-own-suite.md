---
format: aep.planning-md/3
id: story:a-row-set-command-whose-branches-address-different-rows-passes-its-own-suite
kind: story
status: draft
title: A row-set command whose accepting branches address different rows passes its own suite against the reference interpreter
summary: The interpreter addresses the first declared accepting branch's row; a later branch's synthesized scenario fails (ESS-MUTATE-001).
tags:
- ess-issues-3
revision: 1
---
## Problem

On a row-set command (one outcome reads a row set through the input) whose accepting branches
address different rows, the reference interpreter's `addressed_row` reads the row of the first
declared accepting branch, whatever branch the request selects. The synthesized scenario of a
later accepting branch arranges that branch's own row, so the unmutated suite fails against the
interpreter and `mutate::audit` refuses with `ESS-MUTATE-001`.

Observed 2026-10-29 while adapting `crates/verify/ess-conformance/tests/adversary_l5_pass1.rs` to
the 0.77.0 unreachable-branch refusal: the `SHELF` model with `shelf-paused` made reachable
(`when: {any: ['rush == true', {all: ['pause == false', 'halt == true']}]}`, a new Boolean input
`halt`) was refused by the audit with `ESS-MUTATE-001` on `demo.shelf.Act/outcome/shelf-paused`.
The mechanism (scenario arranges the shelf, interpreter addresses the book) is inferred from the
test's own description of `addressed_row`, not traced.

The 0.77.0 adversary keeps the branch dead through two jointly covering refusals, which validate
does not yet refuse, so the addressed-row swap stays exercised.

## Acceptance

- The `SHELF` model of `adversary_l5_pass1.rs` with `shelf-paused` reachable through `halt` audits
  without `ESS-MUTATE-001`: every synthesized scenario passes against the reference interpreter.
- The two addressed-row cases of that file hold on the reachable model.
