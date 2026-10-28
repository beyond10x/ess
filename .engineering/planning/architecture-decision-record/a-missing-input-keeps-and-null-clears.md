---
format: aep.planning-md/3
id: architecture-decision-record:a-missing-input-keeps-and-null-clears
kind: architecture-decision-record
status: accepted
title: A missing input keeps the stored value and null clears it
summary: 'Proposes a three-valued input under the G-U30 presence, an explicit missing: unchanged copy and a sent() guard; target 0.70.0 (ess#678)'
refs:
- provider: github
  reference: beyond10x/ess#678
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "proposed", to: "accepted", at: "2026-10-23T19:36:15Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"approval":1}}, correlation: "ess-0690"}
---
Status: accepted 2026-10-23 by the operator, after reviewing all 15 proposed ADRs in full: "I accept all of them" (see this record's transitions). Target: 0.70.0.

## Context

A PATCH-style write body has a member that is not required and may be `null`, and the two spellings mean different things: a missing member keeps the stored value, `null` clears it, a value sets it (beyond10x/ess#678, reproduction in its first comment: `demo.notes.EditNote.summary`).

- The wire half ships in 0.69.0 as unit G-U30: an `Optional<T>` input may declare that it admits both a missing member and `null`, projected not required and `oneOf [null, T]`. The model still reads both as one absent value.
- `sets: {summary: input.summary}` copies an absent input as an absent field (`docs/design/unread-optional-inputs.md:44-46`), so a missing member clears the field.
- `{input: x, else: …}` falls back to `{generated: true}` or a literal (`website/docs/guides/specify/values-and-views.md:37-38`); no fallback keeps the field's stored value.
- `input_absent: true` (`docs/design/outcome-shapes.md:152`) is about a missing body, not a missing member.
- Guards have `defined(input.x)`, which cannot tell `null` from missing, because the model holds no difference.

So the adopter's row 1 needs a second outcome guarded on `defined(input.summary)`, and rows 2 and 3 cannot be told apart.

## Options

| | Option | Cost |
|---|---|---|
| A | **Explicit mapping.** Under the input presence from G-U30, an input holds one of three values: missing, `null`, a value. A copy states what missing does: `sets: {summary: {input: summary, missing: unchanged}}`. `null` clears the field and a value sets it. A guard reads the difference through `sent(input.x)` (missing → false) beside `defined(input.x)` (a value → true). A plain `input.x` copy keeps today's meaning. | New `ess/N` (meaning of an input value changes for inputs that declare the presence). Synthesis witnesses three scenarios per such copy; the interpreter, the three runtimes and `ess verify diff` learn the third value. |
| B | **Implicit under the presence.** Declaring the presence on the input changes a plain `sets: {summary: input.summary}` to missing → unchanged, `null` → cleared. | Same format and synthesis cost as A. One spelling means two things depending on a declaration elsewhere in the command, and moving a presence silently changes stored effects. |
| C | **Keep it out.** The adopter models "clear" as a separate input (`clear_summary: Boolean`) or a separate command. | Nothing to build. The specification no longer describes the served API's body, and its conformance suite cannot hold an implementation to the PATCH semantics. |

## Recommendation

A. It states the effect where it happens, keeps every existing copy's meaning, and gives guards a name for the difference the wire now carries. B makes the same bytes mean different things depending on a declaration elsewhere; C leaves the served contract unspecified.

## Effect (A)

- **Format:** `ess/N` admits `missing: unchanged` on `{input: …}` and the guard function `sent(…)`, both only on an input that declares the G-U30 presence; below `ess/N`, refused with `unsupported_format_version`. `cargo xtask schema` regenerates the published schema.
- **Validation:** `missing: unchanged` on an input without the presence, or on a creating branch (no stored value to keep), is refused with a named code.
- **Synthesis and conformance:** for each such copy, one scenario per value (missing, `null`, a value) against a stored row holding a value, each asserting the stored field after the step. A target that clears on missing, or keeps on `null`, fails.
- **Interpretation and generated code:** the interpreter and the Rust, Go and TypeScript runtimes carry the third value; generated input types decode missing and `null` apart for those inputs.
- **Diff:** adding `missing: unchanged` or the presence to an input is classified as a breaking change of the command.

## Acceptance evidence

The reproduction in ess#678 as a fixture: three scenarios, `{}` keeps "old", `{"summary": null}` clears, `{"summary": "new"}` sets, each green on the interpreted target and red on a target mutated to clear on missing.

## Correction (2026-10-23)

The G-U30 projection in Context is not required and `anyOf [<T>, null]`, the encoding `null_when_absent` already uses, not `oneOf [null, T]`. It applies at every field position that takes `presence:`, struct fields included.
