---
format: aep.planning-md/3
id: story:an-input-keeps-missing-and-null-apart
kind: story
status: draft
title: A command input keeps a missing member and an explicit null apart
summary: 'Three-state input: missing, null and a value, readable by guards and sets:'
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#678
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/keep_on_missing.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: cited
  path: docs/design/language-grids.md
- confidence: cited
  path: docs/design/omitted-or-null-presence.md
revision: 8
---
## Need

A PATCH-style write body has a member that is not required and may be `null`, and the three
spellings mean three things against a stored row whose `summary` is "old": `{}` keeps "old",
`{"summary": null}` clears it, `{"summary": "new"}` sets it. The wire half shipped in `ess/31`
(0.69.0): `presence: omitted_or_null` admits both a missing member and `null`, projected not
required and `anyOf [T, null]`. The model still reads both as one absent value, so no `sets:` and no
guard can tell them apart. The accepted ADR `a-missing-input-keeps-and-null-clears` (option A)
decides the rest: `sets: {summary: {input: summary, missing: unchanged}}` and a guard function
`sent(input.x)`.

Minimal reproduction on 0.92.0 (`triage/678/repro/run.txt`; a `Note` with
`summary: Optional<String>`, `EditNote` with `summary: Optional<String>, presence: omitted_or_null`):

```text
$ ess specify validate --path plain-copy            # sets: {summary: input.summary}
example v1 — 1 file(s), valid                       # projected anyOf [string, null], not required
$ ess specify validate --path missing-unchanged     # sets: {summary: {input: summary, missing: unchanged}}
  - error[ESS-COMMAND-002]: `summary` is `Optional<String>`, and a nested mapping fills a struct's fields
exit 1
$ ess specify validate --path sent-guard            # when: "sent(summary)"
  - error[ESS-SPEC-012]: cannot parse predicate "sent(summary)": ...
exit 1
$ ess specify validate --path coalesce-idiom        # {expr: "coalesce(input.summary, subject.summary)"}
example v1 — 1 file(s), valid                       # keeps on missing, but also keeps on null
```

## Acceptance

- `docs/design/omitted-or-null-presence.md` gains § "Keep on missing, clear on null", status
  accepted, stating the three-valued input, both constructs, their positions on the grids, every
  refusal, the format, and each target's treatment (built, or refused by name).
- Under the next source format (`ess/37` if still open when this lands), the reproduction with
  `sets: {summary: {input: summary, missing: unchanged}}` validates and compiles, the IR carrying
  the new source variant; `when: "sent(summary)"` beside `defined(summary)` validates on an input
  declaring `omitted_or_null`, at every input-reading predicate position where `defined()` over
  that input is admitted (`cargo test -p ess-domain --locked --test keep_on_missing`, new).
- Refused by name, each with its code and path: `missing: unchanged` on an input without
  `omitted_or_null`; on a creating branch (no stored value to keep); in an event's or an error's
  `payload:`; beside `else:`. `sent(…)` over anything but an input path whose presence is
  `omitted_or_null` (a stored field or a view field has one absent spelling, so the function would
  be constant), with a hint naming `defined(…)`.
- At `format: ess/36` both are refused as `unsupported_format_version` naming the new format; a
  plain `input.summary` copy keeps its meaning (missing and `null` both clear) at every format, and a
  model without either keeps its IR bytes.
- Until their stories land, the interpreter and synthesis, generated Rust, Go and TypeScript, and
  Entity Runtime lowering refuse either construct by name; none ignores it.
- `FORMAT_HISTORY` gains the row; `cargo xtask schema` regenerates the published schema.

## Scope

- cited: docs/design/omitted-or-null-presence.md
- cited: docs/design/language-grids.md (grid O, `defined()` row)
- cited: crates/specify/ess-domain/src/command.rs (`PayloadSource::InputOrGenerated`, 1276-1288; input source parsing near 1708 and 2061-2215)
- cited: crates/specify/ess-domain/src/command/value_expression.rs
- cited: crates/specify/ess-domain/src/expression.rs (`defined()` handling, 3431, 3821)
- cited: crates/specify/ess-primitives/src/predicate.rs (`Predicate::Defined`, 1498)
- cited: crates/specify/ess-domain/src/primitive_admission.rs (`Presence`)
- cited: crates/specify/ess-domain/src/system.rs (`FORMAT_HISTORY`)
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/specify/ess-domain/tests/keep_on_missing.rs (new)
- inferred: schemas/generated/ess.schema.json (regenerated)
confidence: medium

## Fit review

1. **Need.** An input member keeps three states (missing, `null`, a value) that a stored effect and
   a guard can each tell apart. Reproduction above. Requester's proposal (issue comment): a
   nullable presence (shipped as `omitted_or_null`) and either a plain `sets:` meaning
   missing-keeps under that presence or an explicit `unless_missing:`.
2. **Class: gap.** The model holds one absent value for both spellings (`ess specify formats`,
   `ess/31` entry: "both read as one absent value"; `crates/verify/ess-conformance/src/input.rs:1789`,
   "`null` is absent").
3. **Already expressible?** No. `coalesce(input.summary, subject.summary)` validates (run 4) and
   gets row 1, but `null` is also absent, so row 2 keeps instead of clearing; two commands or a
   `clear_summary: Boolean` input no longer describe the served body, so the suite cannot hold the
   PATCH meaning (ADR option C; policy 1, `docs/design/generic-language-features.md:588-594`).
4. **Fit.** Grid O: `sent()` is a sibling row of `defined()` (`docs/design/language-grids.md`,
   `defined()` row), opened over the whole input column of grid R for predicates, and refused for
   every other root with the reason (no other root has two absent spellings). `missing: unchanged`
   extends the existing `{input: …}` source beside `else:` (`command.rs:1276-1288`) in every
   `sets:` that writes an existing row, refused on creation and in payloads with the reason. It
   reuses `omitted_or_null` rather than adding a presence. Targets: split into
   `story:conformance-holds-missing-keeps-and-null-clears`,
   `story:generated-code-decodes-a-missing-member-apart-from-null` and
   `story:ess-verify-diff-classifies-keep-on-missing`.
5. **Second adopter.** A user-profile API where `PATCH {"nickname": null}` removes the nickname and
   omitting it leaves it; a JSON Merge Patch (RFC 7396) endpoint has exactly these semantics.
6. **Cost.** One source format, one source variant, one predicate function, about six refusals, a
   diff classification, a schema regeneration; the suite format moves only if a step cannot carry a
   missing member apart from `null` (decided on the design page). No migration: a plain copy keeps
   its meaning.
7. **Alternatives** (ADR options): A, explicit `missing: unchanged` and `sent()` (chosen); B,
   implicit under the presence, the requester's first form: one spelling would mean two things
   depending on a declaration elsewhere; C, keep it out: the body is no longer described. The
   requester's `unless_missing:` is the same idea as A under the name the ADR fixed.

## Decisions

**accept, redesigned.** Option A of the accepted ADR `a-missing-input-keeps-and-null-clears`
(operator, 2026-10-23). The wire half the earlier body covered shipped in `ess/31` (0.69.0, part of
beyond10x/ess#678), so this story now holds the keep-versus-clear half only. Refined here: `sent()` is
opened over the whole input column and refused elsewhere by reason, and `missing: unchanged` over
every `sets:` writing an existing row. The ADR's 0.70.0 target is past, so this takes the next
format. The work is split by target.

## Source

github:beyond10x/ess#678
