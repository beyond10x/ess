---
format: aep.planning-md/3
id: story:a-row-set-command-whose-branches-address-different-rows-passes-its-own-suite
kind: story
status: implemented
title: A row-set command whose accepting branches address different rows passes its own suite against the reference interpreter
summary: The interpreter addresses the first declared accepting branch's row; a later branch's synthesized scenario fails (ESS-MUTATE-001).
tags:
- defect
- ess-issues-3
- severity-high
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_l5_pass1.rs
- confidence: inferred
  path: docs/design/filtered-related-reads.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:41:33Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:41:34Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-12T14:55:49Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Problem

On a row-set command (one outcome reads a row set through the input) whose accepting branches
address different rows, the reference interpreter's `addressed_row`
(`crates/verify/ess-conformance/src/interpret/execute.rs:1446-1490`) collects every accepting
branch's supplied subject and checks only the first declared one (`acting.first()`), whatever
branch the request selects. The synthesized scenario of a later accepting branch arranges that
branch's own row, so the unmutated suite fails against the interpreter, and `mutate::audit`
refuses with `ESS-MUTATE-001`.

Reproduction on 0.92.0: the `SHELF` model of
`crates/verify/ess-conformance/tests/adversary_l5_pass1.rs`, `book-paused` declared first, with
`shelf-paused` made reachable through a new Boolean input `halt`:
`when: {any: ['rush == true', {all: ['pause == false', 'halt == true']}]}`. `Act` declares
`unknown` (`unknown_instance`), `refused` (row set over `Library`), `rushed`, `hurried`,
`book-paused` (`updates: Book`, `instance: book`), `shelf-paused` (`updates: Shelf`,
`instance: shelf`) and `idle`.

```text
$ ess verify conform run --path . --target interpreted --report-format 2
  failed demo.shelf.Act/outcome/shelf-paused
  10 scenarios: 9 passed, 1 failed
  ESS-CF-OUTCOME expected outcome = shelf-paused, observed outcome = unknown
    input: Act(book = "book", shelf = <built shelf>, halt = true, pause = false, rush = false)
$ ess verify conform mutate --path . --target interpreted --class precedence-swap
refusal[ESS-MUTATE-001]: the unmutated suite did not pass against `interpreted` …
  demo.shelf.Act/outcome/shelf-paused                                      (exit 3)
```

The scenario builds the shelf and sends a book identity that no row carries. The interpreter reads
the book (first acting branch), finds none and answers `unknown`.

Validate still accepts a dead branch kept dead by two jointly covering refusals, so the existing
adversary keeps `shelf-paused` dead, and the addressed-row swap stays exercised there.

## Decision needed

Which row an `unknown_instance` / held-state check reads when accepting branches address different
rows: the row of the branch the request selects (synthesis's reading), or the first declared
(the interpreter's). `docs/design/filtered-related-reads.md` "Subject borrowing and precedence"
puts addressed-row existence before the row-set phase but does not settle multi-subject commands.
The fix is in whichever side the ruling makes wrong.

## Acceptance

- `a_row_set_command_whose_branches_address_different_rows_passes_its_own_suite`: the reachable
  `SHELF` model above synthesizes a suite that passes `--target interpreted` in full, and
  `mutate::audit` with `precedence-swap` runs without `ESS-MUTATE-001`. Before the fix
  `Act/outcome/shelf-paused` fails as shown.
- The two addressed-row cases of `adversary_l5_pass1.rs`
  (`a_swap_that_moves_the_addressed_row_is_not_scored_equivalent`,
  `a_killed_swap_of_branches_addressing_different_rows_names_no_overlap_taken`) hold on the
  reachable model.

## Scope

- cited: `crates/verify/ess-conformance/src/interpret/execute.rs` (`addressed_row` :1446-1490)
- inferred: `crates/verify/ess-conformance/src/synthesize.rs` (`addressed_row` :5589, if the ruling puts the fix in synthesis)
- cited: `crates/verify/ess-conformance/tests/adversary_l5_pass1.rs`
- inferred: `docs/design/filtered-related-reads.md` (the ruling)

confidence: medium
