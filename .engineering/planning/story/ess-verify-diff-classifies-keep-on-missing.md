---
format: aep.planning-md/3
id: story:ess-verify-diff-classifies-keep-on-missing
kind: story
status: draft
title: ess verify diff classifies keep on missing
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#678
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-input-keeps-missing-and-null-apart
revision: 2
---
## Need

The accepted ADR `a-missing-input-keeps-and-null-clears` (Effect, "Diff") classifies adding
`missing: unchanged` to a copy, or `omitted_or_null` to an input, as a breaking change of the
command: what a missing member does to the stored row changes. `ess verify diff` must say so
(`story:an-input-keeps-missing-and-null-apart`).

## Acceptance

- `ess verify diff` between the reproduction with `sets: {summary: input.summary}` and the same with
  `{input: summary, missing: unchanged}` reports the command's outcome as breaking, naming the field;
  removing it is breaking too.
- Adding or removing `presence: omitted_or_null` on a command input is reported as breaking; adding
  or removing a `sent()` guard is reported as the existing `outcome-condition-changed`.
- Two specifications without these differences diff as before (existing tests unchanged).
- Tests in `crates/verify/ess-diff/tests/keep_on_missing.rs` (new).

## Scope

- cited: crates/verify/ess-diff/src/diff.rs (presence comparison, near 353)
- cited: crates/verify/ess-diff/src/change.rs
- cited: crates/verify/ess-diff/src/compatibility.rs
- inferred: crates/verify/ess-diff/tests/keep_on_missing.rs (new)
confidence: medium

## Fit review

1. **Need.** A changed keep-on-missing meaning is caught by the diff gate.
2. **Class: gap** (follows from the construct).
3. **Already expressible?** No.
4. **Fit.** Reuses the breaking classification and `outcome-condition-changed`
   (`docs/design/generic-language-features.md`, § Migration and format consequences).
5. **Second adopter.** Any adopter gating a release with `ess verify diff`.
6. **Cost.** A change kind, possibly; a diff format only if a new kind needs one.
7. **Alternatives.** (a) Leave the diff blind: a red flag of the assessing skill. (b) Chosen.

## Decisions

**accept as proposed** (ADR `a-missing-input-keeps-and-null-clears`, Effect, "Diff").

## Source

github:beyond10x/ess#678
