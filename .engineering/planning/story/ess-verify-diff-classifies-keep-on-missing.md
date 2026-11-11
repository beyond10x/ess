---
format: aep.planning-md/3
id: story:ess-verify-diff-classifies-keep-on-missing
kind: story
status: active
title: ess verify diff classifies keep on missing
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#678
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-input-keeps-missing-and-null-apart
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: crates/verify/ess-diff/src/compatibility.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: crates/verify/ess-diff/tests/keep_on_missing.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:43:52Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:43:53Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

The accepted ADR `a-missing-input-keeps-and-null-clears` (Effect, "Diff") classifies adding
`missing: unchanged` to a copy, or `omitted_or_null` to an input, as a breaking change of the
command: what a missing member does to the stored row changes. `ess verify diff` must say so
(`story:an-input-keeps-missing-and-null-apart`).

## Acceptance

Every bullet runs in `cargo test -p ess-diff --locked --test keep_on_missing` (new), over the
reproduction of `story:an-input-keeps-missing-and-null-apart`.

- `ess verify diff` from `sets: {summary: input.summary}` to
  `sets: {summary: {input: summary, missing: unchanged}}` reports one `outcome-sets-changed` change
  on the command's outcome, naming `summary`, with callers compatibility `breaking`.
- The same diff in the other direction (removing `missing: unchanged`) reports the same kind,
  field and `breaking`.
- Adding `presence: omitted_or_null` to the command input `summary` reports one
  `input-type-changed` change naming `summary`, with callers compatibility `breaking`; removing it
  reports the same.
- Adding a `sent(summary)` guard to an outcome reports one `outcome-condition-changed` change on
  that outcome, and removing it reports the same; its compatibility is what that kind already
  answers for the guard.
- `cargo test -p ess-diff --locked` passes with no existing test file changed: two specifications
  without these differences diff as before.

## Scope

This story moves only the `CHANGELOG.md` shared surface.

- cited: crates/verify/ess-diff/src/diff.rs
- cited: crates/verify/ess-diff/src/change.rs
- cited: crates/verify/ess-diff/src/compatibility.rs
- inferred: crates/verify/ess-diff/tests/keep_on_missing.rs

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

- No new change kind: `missing: unchanged` moves the outcome's `sets` in the IR, which
  `outcome_state_changes` in `crates/verify/ess-diff/src/diff.rs` already reports as
  `outcome-sets-changed`, and input presence is reported as `input-type-changed` from
  `command_changes`, so the delta needs no new `ess-diff` format; the story's work is the
  `breaking` classification of both in `compatibility.rs`.

## Source

github:beyond10x/ess#678
