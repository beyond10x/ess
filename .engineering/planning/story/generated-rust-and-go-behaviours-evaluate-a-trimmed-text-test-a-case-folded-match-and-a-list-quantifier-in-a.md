---
format: aep.planning-md/3
id: story:generated-rust-and-go-behaviours-evaluate-a-trimmed-text-test-a-case-folded-match-and-a-list-quantifier-in-a
kind: story
status: draft
title: Generated Rust and Go behaviours evaluate a trimmed text test, a case-folded match and a list quantifier in a command guard
summary: Rust synthesis leaves a trimmed text test, a case-folded match and a list quantifier in a command guard as obligations (gap report, 2026-11-24)
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 2
---
## Need

Three facts a specification states in a command guard that Rust synthesis leaves as obligations, so a
service has hand-written code for each:

- `trimmed(kind in_ignore_case [web, phone])`: a text input is compared after whitespace is trimmed,
  and without regard to case.
- `exists s in list: (s.kind == web)` and `forall s in list: ...`: a test over the members of a list
  input or a stored list.

Reproduction: a command guarded by `kind: {trimmed: {in_ignore_case: [web, phone]}}` (the spelling of
`docs/design/value-expressions.md:398-415`) and one guarded by an `exists` over a list input (the G2
quantifier of `docs/design/generic-language-features.md:227`); `ess generate synthesize --target rust`
keeps the first command owed by "a `trimmed` text test no generated guard evaluates" and the second by
"the guard `...`" (read from `determined.rs`, not run here).

What exists (read at `origin/main`):

- The check that decides whether a guard is generated, `determined::supported`
  (`crates/generate/ess-synth/src/determined.rs:1260-1360`), has arms for presence, truthiness,
  comparison, a text match against a literal or an input (1342), and refuses by name a calendar window,
  `distinct` (1347) and `trimmed` (1357, "owed by name, never rendered as the operator it wraps over the
  untrimmed text"). `Forall`, `Exists` and the fold-match operators (`equals_ignore_case`,
  `in_ignore_case`) have no arm and reach the catch-all `the guard ...`.
- The generated invariant checker already renders them: `iv::fold_match` for
  `FoldMatch` (`crates/generate/ess-synth/src/rust/invariant.rs:527-540`) and `forall`/`exists` over a
  list or map (`invariant.rs:542-543, 640-665`). It refuses `trimmed` there (555-561): validation admits
  `trimmed` in a command guard only.
- The command behaviour renderer (`rust/behaviour.rs:3178`) renders a text match and no fold-match or
  quantifier.
- The interpreter decides all three; the suite synthesizes cases for them, so a target without them
  fails the suite (that is how the obligation is found).

## Outcome

Generated Rust and Go behaviours evaluate a fold-match, a `trimmed(...)` text test and a list quantifier
in a command guard with the evaluator's three-valued semantics: an absent operand is unknown, the
text is trimmed of the whitespace the interpreter trims, and case is folded as the interpreter folds it.

## Acceptance

- `ess generate synthesize --target rust` and `--target go` on models that use each of the three report
  `0 obligation(s)` for those commands (before: one obligation each), and the synthesized suite passes
  against each generated server.
- A text of `"  Web "` takes the branch `trimmed(kind in_ignore_case [web])` selects; `"we b"` and an
  absent kind do not; the third case is unknown and takes the branch the interpreter takes for an
  unknown guard.
- `exists` over an empty list is false and `forall` is true; one member satisfying selects the branch;
  an absent list is unknown. A mutant that treats an empty `forall` as false fails the suite.
- Whitespace and case follow the documented rule, ASCII only (`value-expressions.md:398-415`,
  `website/docs/reference/entity-runtime-lowering.md:77-78`): a test with `"É"` against `"é"` does not
  fold in the interpreter, the Rust server or the Go server.
- Calendar windows and `distinct` stay owed with their text; a model with none of the three generates
  byte-identically.

## Scope

- cited: `crates/generate/ess-synth/src/determined.rs` (`supported`, 1260-1360), `rust/behaviour.rs`
  (3178), `rust/invariant.rs` (527-543, 640-665), `go/` behaviour renderer,
  `crates/specify/ess-primitives/src/predicate.rs` (`FoldOp` 502-522, `Quantified` 1633).
- Size: M, about 500 changed lines; Rust and Go side by side.

## Fit review

1. **Need.** Generate the guards a specification states. Reporter: two generator limits.
2. **Class: gap** in generator coverage (grid O, generated guards).
3. **Already expressible?** The facts, yes; the code, no.
4. **Fit.** One row of the effect by target grid: the three guard shapes against the Rust and Go
   behaviours. The invariant checker is the existing renderer; the command renderer reuses its helpers.
5. **Second adopter.** A channel allow-list compared after trim; "some attachment is an image".
6. **Cost.** No format step. Two renderers, one shared helper set.
7. **Alternatives.** Change nothing; generate only the literal-text forms.

## Decisions

AC9-D9: where the guards render. A: command guards, the three shapes in Rust and Go together. B: Rust first. Decided: A, 2026-11-24 (the coordinator, under the owner's delegation); accepted as proposed (no authored surface).
