---
format: aep.planning-md/3
id: story:an-input-keeps-missing-and-null-apart
kind: story
status: active
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
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/lowerable_subset.rs
- confidence: cited
  path: crates/generate/ess-synth/src/failure.rs
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
- confidence: inferred
  path: crates/specify/ess-domain/tests/keep_on_missing.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: docs/design/language-grids.md
- confidence: cited
  path: docs/design/omitted-or-null-presence.md
- confidence: inferred
  path: website/docs/reference/entity-runtime-lowering.md
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:39:53Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"review_outcome":2,"verification":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:39:54Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"review_outcome":2,"verification":1}}}
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
  refusal, the format `ess/37`, and for each target the story that builds it and the interim
  refusal below.
- At `format: ess/37`, the reproduction with
  `sets: {summary: {input: summary, missing: unchanged}}` validates and compiles, the IR carrying
  the new source variant; `when: "sent(summary)"` beside `defined(summary)` validates on an input
  declaring `omitted_or_null`, at every input-reading predicate position where `defined()` over
  that input is admitted (`cargo test -p ess-domain --locked --test keep_on_missing`, new).
- Refused by name, each with its code and path: `missing: unchanged` on an input without
  `omitted_or_null`; on a creating branch (no stored value to keep); in an event's or an error's
  `payload:`; beside `else:`. `sent(…)` over anything but an input path whose presence is
  `omitted_or_null` (a stored field or a view field has one absent spelling, so the function would
  be constant), with a hint naming `defined(…)`.
- At `format: ess/36` both are refused as `unsupported_format_version` naming `ess/37`; a plain
  `input.summary` copy keeps its meaning (missing and `null` both clear) at every format, and a
  model without either keeps its IR bytes.
- Interim refusal, synthesis and the interpreter: on the `missing-unchanged` and `sent-guard`
  reproductions at `ess/37`, `ess verify conform synthesize --path <dir> --out suite.json` and
  `ess verify conform run --target interpreted --path <dir>` each exit 1, write no suite, and print
  one `ess_conformance::admission::model` issue at the construct's path (ending `.sets.summary`
  for `missing: unchanged`, at the branch's `when` for `sent(summary)`) naming the construct.
- Interim refusal, generated Rust, Go and web: on the same two reproductions,
  `ess generate synthesize --target <rust|go|web> --path <dir> --out out` exits 1, writes no
  workspace, and prints `<target> target cannot emit this workspace` followed by a
  `MissingRepresentation` line at the same path naming the construct.
- Interim refusal, Entity Runtime: `cargo test -p ess-entity-runtime --locked --test lowerable_subset`
  passes a case in which lowering each reproduction returns one `LoweringDiagnostic` at the
  construct's path, with a new `LoweringCode` that `subset::CONSTRUCTS` lists as
  `Lowering::Refused`, and the generated `website/docs/reference/entity-runtime-lowering.md`
  matches the catalogue.
- After `cargo xtask schema`, `schemas/generated/ess.schema.json` admits `missing: unchanged` in
  the `{input: …}` source of `sets:`, and `task projection-check` passes.

## Scope

- cited: docs/design/omitted-or-null-presence.md
- cited: docs/design/language-grids.md (grid O, `defined()` row)
- cited: crates/specify/ess-domain/src/command.rs (`PayloadSource::InputOrGenerated`, 1276-1288; input source parsing near 1708 and 2061-2215)
- cited: crates/specify/ess-domain/src/command/value_expression.rs
- cited: crates/specify/ess-domain/src/expression.rs (`defined()` handling, 3431, 3821)
- cited: crates/specify/ess-primitives/src/predicate.rs (`Predicate::Defined`, 1498)
- cited: crates/specify/ess-domain/src/primitive_admission.rs (`Presence`)
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/specify/ess-domain/tests/keep_on_missing.rs (new)
- cited: crates/verify/ess-conformance/src/admission.rs (`model`, the interim refusal both `conform synthesize` and `conform run` call)
- cited: crates/generate/ess-synth/src/failure.rs (an interim gate beside `input_absent`)
- inferred: crates/generate/ess-entity-runtime/src/lib.rs (`LoweringCode`, the lowering refusal)
- cited: crates/generate/ess-entity-runtime/src/subset.rs (`CONSTRUCTS`)
- inferred: crates/generate/ess-entity-runtime/tests/lowerable_subset.rs
- inferred: website/docs/reference/entity-runtime-lowering.md (generated from `CONSTRUCTS`)

confidence: medium

Shared surfaces moved: proposes the `ess/37` `FORMAT_HISTORY` sentence and its `CHANGELOG.md`
sentence, and regenerates `schemas/generated/ess.schema.json` and
`website/docs/reference/spec-versions.md`; no byte-pin row moves.

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
every `sets:` writing an existing row. The work is split by target.

- Format: `ess/37`, the epic's one source format (epic § Shared surfaces); the ADR's 0.70.0 target
  is past.
- The interim refusals sit at each target's existing front gate (`admission::model`,
  `ess-synth`'s `failure.rs`, Entity Runtime's `CONSTRUCTS`), so a target never reads either
  construct as a plain copy; `story:conformance-holds-missing-keeps-and-null-clears` and
  `story:generated-code-decodes-a-missing-member-apart-from-null`, which depend on this story, lift
  the conformance and Rust/Go refusals, and the Entity Runtime refusal stays.
- `ess generate types --target typescript` writes structural declarations, not behaviour, and
  already spells a not-required nullable member apart from a missing one, so it reads neither
  construct and has no interim refusal.

## Source

github:beyond10x/ess#678
