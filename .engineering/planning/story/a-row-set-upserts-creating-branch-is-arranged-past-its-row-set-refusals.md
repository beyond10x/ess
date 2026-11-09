---
format: aep.planning-md/3
id: story:a-row-set-upserts-creating-branch-is-arranged-past-its-row-set-refusals
kind: story
status: draft
title: A row-set upsert's creating branch is arranged past its row-set refusals
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#642
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command/precedence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/precedence.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/row_set_upsert_arrangement.rs
revision: 4
---
## Need

On a create-or-update guarded by a row-set refusal (a "row-set upsert"), the creating
`unknown_instance:` branch answers after the row-set refusals. The precedence plan says so
(`crates/specify/ess-domain/src/command/precedence.rs:515-524`: `PresentRelated`, `Trail` when the
row set is read first and the command upserts), and the interpreter follows it. Synthesis does not.
`consistent` accepts an `unknown_instance:` branch without reading any row set
(`crates/verify/ess-conformance/src/synthesize/row_set.rs:1128-1145`), so the scenario for the
creating branch sends the command with no row arranged for the row-set refusal to pass over. The
synthesized suite then fails its own model on the interpreted target. Where the arranged row's
compared field is a minted identity, the existing-row branch gets no scenario at all.

Minimal reproduction (0.92.0, `ess/36`): the `example.g.Mint` command of
`story:when-subject-beside-a-row-set-validates-under-the-g2-format` without `again`
(`no-parent` / `first` / `already`), the `Parent` created by `example.g.Open` with a generated id,
and `example.g.Disable` pausing a grant:

```text
$ ess verify conform synthesize --path without-again
refused: refusal[ESS-SYNTH-001]: outcome example.g.Mint/already has no scenario `example.g.Mint/outcome/already`
  no witness: `example.g.Mint/already: `no-parent` is not decidedly passed over: its row set is True` is `row set`, ...
5 scenario(s) (0 authored), 1 refusal(s), nothing written
$ ess verify conform run --target interpreted --report-format 2 --path without-again
  failed example.g.Disable/outcome/disabled
  failed example.g.Mint/outcome/first
  failed example.g.Mint/outcome/no-parent
  5 scenarios: 2 passed, 3 failed, 0 error, 0 unsupported
```

The failing check is `expected: outcome = first`, `observed: outcome = no-parent`: the scenario sends
`example.g.Mint(parent_id = …)` with no `Parent` arranged. With the parent id from the input
(`without-again-parent-id-input`), or the row set over a plain `code` field
(`without-again-field`), synthesis refuses nothing, but 4 of 6 scenarios fail the same way. The
identity-addressed sibling (`via-sibling`) synthesizes 7 scenarios and passes all 7.

## Acceptance

- On `without-again`, `without-again-parent-id-input` and `without-again-field`,
  `ess verify conform synthesize` reports 0 refusals, and `ess verify conform run --target interpreted`
  passes every scenario.
- `example.g.Mint/outcome/first` arranges a `Parent` through `example.g.Open` and sends its captured
  identity. `example.g.Mint/outcome/no-parent` sends an id no `Parent` carries.
  `example.g.Mint/outcome/already` arranges the `Parent`, creates the `Grant` through `first`, and
  sends `Mint` again.
- With `again` admitted (`story:when-subject-beside-a-row-set-validates-under-the-g2-format`) and
  declared first, the reproduction synthesizes `again` through `example.g.Disable` and passes on the
  interpreted target.
- A mutant target answering `first` before `no-parent` fails `no-parent`.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/row_set.rs
- cited: crates/specify/ess-domain/src/command/precedence.rs
- inferred: crates/verify/ess-conformance/src/synthesize/precedence.rs
- inferred: crates/verify/ess-conformance/src/synthesize/existence.rs
- inferred: crates/verify/ess-conformance/tests/row_set_upsert_arrangement.rs
confidence: medium

## Fit review

1. **Need.** Every branch of a validating create-or-update under a row-set refusal must have a
   scenario that a correct implementation passes. Reproduction above (triage run `without-again`,
   `without-again-parent-id-input`, `without-again-field`).
2. **Class.** Defect. The synthesized suite fails the interpreted target on a model that validates,
   and the interpreter follows the precedence plan (`precedence.rs:515-524`) while synthesis does not
   (`synthesize/row_set.rs:1135-1145`).
3. **Already expressible?** The model validates today. Only the suite is wrong.
4. **Fit.** No language change. Synthesis reads the same precedence plan as validation and the
   interpreter (`docs/design/generic-language-features.md:452-458`: validation, the interpreter and
   synthesis read one plan). The arrangement reuses the related-row capture that `via-sibling`
   already uses.
5. **Second adopter.** The issue notes a second adopter reported the same refusal. The seat-reservation
   example of the companion story has the same shape.
6. **Cost.** No format. Suites of models with a row-set upsert move: their creating and existing-row
   scenarios gain the arranging steps. The change names the committed example suites that move, if
   any.
7. **Alternatives.** (a) Refuse such a command in synthesis by name: rejected, because the model is
   valid and the interpreter answers it. (b) Read the row sets for a row-set upsert's
   `unknown_instance:` branch as the plan orders them: chosen.

## Decisions

accept as proposed: the request's "synthesizes every branch" is the plan's own order applied in
synthesis.

## Source

github:beyond10x/ess#642
