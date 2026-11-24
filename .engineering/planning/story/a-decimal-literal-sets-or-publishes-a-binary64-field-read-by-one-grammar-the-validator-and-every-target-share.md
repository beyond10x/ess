---
format: aep.planning-md/3
id: story:a-decimal-literal-sets-or-publishes-a-binary64-field-read-by-one-grammar-the-validator-and-every-target-share
kind: story
status: draft
title: A decimal literal sets or publishes a Binary64 field, read by one grammar the validator and every target share
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A creating branch starts a `Binary64` measurement at zero, `sets: {load: 0.0}`, and is refused as
`type_mismatch`. The reason reads "`<Entity>.load` is `Binary64` underneath, and a literal in a
sets is text, `true` or `false`, a whole number or a decimal". The hint reads "a literal may be
text, a variant of an enum, `true` or `false`, a whole number or a decimal; …". The value was a
decimal, so the message contradicts the refusal it explains.

- `primitive_literal` admits a literal for `Boolean`, `Integer` and `Decimal` only, and answers
  `Err(None)`, no literal spelling at all, for `Binary64` among others
  (`crates/specify/ess-domain/src/command.rs:5896-5934`, `Binary64` at 5926).
- Its comment gives the reason. A primitive is admitted here only together with the conformance
  reader that sends it, because "a `Timestamp` or a `Uuid` admitted in this function and not in the
  reader that sends it is exactly how the two come apart". `Decimal` joined that way in
  beyond10x/ess#135 (`:5885-5895`).
- The refusal and its hint come from `literal_representation` (`command.rs:6006-6018`,
  `:6094-6097`). The same hint is in the binding's literal check
  (`crates/specify/ess-domain/src/binding.rs:2260`).
- The guide agrees with the code: "A decimal such as `1.5` is a literal only over a `Decimal`
  target" (`website/docs/guides/specify/fields-and-invariants.md:81-82`). An authored scenario
  already writes a `Binary64` literal below 2^53 in magnitude, and a larger one is `ESS-AUTHOR-015`
  (`fields-and-invariants.md:368-372`). Normalization has a token-preserving `binary64_literal`
  (`website/docs/reference/formats.md:303`).

So this is not a defect. The documented rule refuses it, and the code follows the rule. The
message's list of allowed forms is misleading for a `Binary64` target, and this story corrects it.

Minimal reproduction (fresh, not run here): an entity field `{name: load, type: Binary64}` at
`format: ess/37`, and a creating outcome with `sets: {load: 0.0}`. `ess specify validate` reports
`type_mismatch` at the outcome's `sets.load`.

The requester's proposal, restated: a decimal literal sets a `Binary64` field.

## Outcome

A literal over a `Binary64` target in `sets:`, `payload:` and a binding's `mapping:` is admitted
when it is a whole number or a decimal in `Number::decimal_literal`'s grammar, finite, and below
2^53 in magnitude, the bound an authored scenario's `Binary64` literal already has. One function
decides it, and that function is called by the validator, the conformance reader and the
interpreter. The value is the binary64 nearest the decimal. Every target that writes a
`Binary64` writes that value or refuses the branch by name. The refusal over a target with no
literal names what the target takes rather than listing "a decimal".

## Acceptance

- Red first, in `crates/specify/ess-domain/tests/sets_literal.rs`: `sets: {load: 0.0}`, `{load:
  2.5}`, `{load: -3}` and `{load: '0.1'}` over a `Binary64` field validate. Each is refused before
  the change.
- `{load: 1e3}`, `{load: 9007199254740993}`, `{load: '+1.0'}` and `{load: NaN}` stay refused, each
  naming the grammar and the bound.
- Over a `Timestamp` field, `{at: 0}` is refused with a reason naming `Timestamp` and a hint that
  does not offer "a decimal".
- `ess verify conform synthesize` over a model whose creating branch sets `load: 0.0` writes a
  scenario expecting `0` at `load`. The interpreted target passes it, and the Rust runner passes it
  against a generated Rust server.
- Generated Rust and Go behaviours set the field to the literal's value, or refuse the branch by
  name as an obligation. Neither silently drops it.
- `fields-and-invariants.md` § literals and `values-and-views.md` (the `Decimal` sentence at 200)
  state the `Binary64` rule. `CHANGELOG.md` names it.

## Fit review

Tree `bcfc51555`, read-only. Second report, G2.

1. **Need.** A floating-point measurement has a known starting value that a creating branch
   writes. Reproduction above.
2. **Class.** *Gap*. The rule is documented and kept. The hint wording is a small defect folded in.
3. **Already expressible?** Only by taking the value from an input, which changes what the command
   accepts. That is the reason `Decimal` joined in #135 (`command.rs:5916-5917`).
4. **Fit.** Grid O (`docs/design/language-grids.md:63-84`) has no literal row. Its comparison rows
   give `Binary64` every comparison with a numeric literal, and authored scenarios and normalization
   write a `Binary64` literal. `sets:`, `payload:` and `mapping:` are the places left without one,
   so all three open together (the siblings rule), not `sets:` alone. One grammar is shared with the
   reader, as the comment at `command.rs:5885-5895` requires.
5. **Second adopter.** A sensor registry starts each sensor's `calibration_offset: Binary64` at
   `0.0` when it is registered.
6. **Cost.** New authored surface: a document refused before now validates. Under the `Decimal`
   precedent ("a literal over a `Decimal` target is admitted in every format",
   `values-and-views.md:200`) no format version is needed. Gating it behind a format is decision
   SL-D8. No suite major moves: a suite carries `Binary64` from `ess-conformance/62`.
7. **Considered.** (a) Change nothing, and correct the hint only. (b) Any finite decimal, without
   the 2^53 bound: a view row compares an integer exactly, which is the reason for
   `ESS-AUTHOR-015`. (c) Chosen: the authored scenario's grammar and bound.

## Decisions

**Accept, redesigned.** The bound and the grammar are the existing authored `Binary64` literal's.

- **SL-D8. Which specification formats admit it?** (A) Every format that admits `Binary64` (major 2
  on), by the `Decimal` precedent. (B) From `ess/39` only, opened by the first report's
  `g15-ess39-opens`, refused below it naming `ess/39`. It does not join `ess/38`: that format opens
  the member address of a struct input, and by policy 2 a construct off its row waits.
  **Recommended: A.**

## Scope

- cited: `crates/specify/ess-domain/src/command.rs` (`primitive_literal` 5896-5934,
  `literal_representation` 6006-6104)
- cited: `crates/specify/ess-domain/src/binding.rs` (2260)
- cited: `crates/specify/ess-domain/tests/sets_literal.rs`
- inferred: `crates/verify/ess-conformance/src/authored.rs` (the shared grammar), the interpreter's
  literal reader, `crates/generate/ess-synth/src/rust/` and `go/` behaviour writers,
  `website/docs/guides/specify/fields-and-invariants.md`, `values-and-views.md`
- Byte pins: a new hint wording moves `crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json`
  only if a `ValidationCode` changes. None does. Run `task test-xtask` and
  `cargo test -p ess-compiler --test typed_diagnostics` (`AGENTS.md` § Gate).
- Size: M, about 450 changed lines.

## Depends on

Nothing under A. Under B, `g15-ess39-opens` (the first report's P2).

## Source

A downstream application, 2026-11-24, second report, G2.
