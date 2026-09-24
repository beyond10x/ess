---
format: aep.planning-md/2
id: review-result:adversary-wave23-unit1-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: the Structured silence leaves a shape nobody refuses'
relations:
- reviews: story:structured-ring-is-refused-by-two-passes
revision: 1
---
## Pass

`aep-drive:adversary`, pass 1, against `wt-dcb4b24b4ed8` over base `46c7281e`.
Verdict **NEEDS-CHANGE**. Cases executed 433 → 438, red 4.
Origin: introduced 3, pre-existing 0, undecided 1 (four findings).

Five cases added in `crates/specify/ess-domain/tests/literal_representation_adversary_pass3.rs`.

## The blocker

The unit's silence defers to `check_inhabitation`, and `check_inhabitation` **declines this exact
shape**. `inhabited_names` cannot mark a struct or union whose field or variant names an undeclared
type, so the walk answers `Uninhabited`; `check_inhabitation` then skips that same declaration at
`system.rs:609` (`names_something_undeclared`), because the missing declaration is reported
elsewhere. The literal is refused by **nobody** — `OwnedByNobody`, the failure the unit's own
2744-shape matrix has a name for.

Whole-compiler output for the struct document is one line,
`[undeclared_reference] types.notifications.core.Pick: …`, with nothing at
`binding.reject-on-refusal.mapping.reason`. The base emits two.

**Origin measured, not inferred.** A scratch copy with `binding.rs`, `command.rs` and `system.rs`
restored to `git show 46c7281e:` content runs the same five cases byte-identical: 5 passed, 0
failed. Five green at base, four red at HEAD.

## `check_inhabitation` did not move

A differential of eight hand-built shapes through the whole compiler, base sources against HEAD
sources, `diff -u`: every `self_reference` message is byte-identical. The only two differences in
the entire differential are the intended narrowing (3 errors → 2 for the story's own shape) and the
blocker above (2 → 1).

## Attacked and could not break

A type inhabited only through a `Map` of itself; a union with one inhabited and one uninhabited
variant (two mistakes, two diagnostics); a struct with a primitive field; a newtype of a primitive;
a zero-field struct and an empty union, both refused earlier by `empty_declaration`; a struct field
of `Optional<Undeclared>`, where `Optional` is a base case so the struct stays inhabited and the
literal is still refused — the hole needs a bare `Named`. Registry divergence between the two
fixpoint calls: none found. `Established(Structured)` held for every inhabited struct and union.

```findings
- file: crates/specify/ess-domain/src/binding.rs
  line: 1800
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: a struct or union whose field names an undeclared type is now answered Uninhabited and left to check_inhabitation, which skips that exact declaration at system.rs:609, so the binding or payload literal written into it is refused by no pass at all.
- file: crates/specify/ess-domain/src/binding.rs
  line: 1680
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: the new doc on Representation::Structured, on Resolution::Uninhabited and in command.rs:2088 all promise that check_inhabitation refuses the shape the walk goes silent about, and for a type naming something undeclared it does not.
- file: crates/specify/ess-domain/src/binding.rs
  line: 1800
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: calling the O(N-squared) inhabitation fixpoint inside the walk once per literal takes a document from O(L*N) to O(L*N*N), measured 33.4ms to 515.4ms at 400 declarations against the base, though no specification in this repository exceeds 32.
- file: crates/specify/ess-domain/src/system.rs
  line: 574
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the lift into system.rs is outside the story's cited scope and is what enables the first finding, because the walk inherited the fixpoint's answer without the two skips check_inhabitation wraps around it.
```
