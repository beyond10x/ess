---
format: aep.planning-md/3
id: story:a-refusal-records-the-document-it-was-read-from
kind: story
status: draft
title: A refusal records the document it was read from
scope:
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/locator_citations.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/entity.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/spec.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/error.rs
- confidence: inferred
  path: docs/design/review-typed-diagnostics.md
revision: 9
---
# A refusal records the document it was read from

`ValidationError` (`crates/specify/ess-primitives/src/error.rs:663`) carries a code, a location
string, a message, a hint and an optional site. It does **not** carry the document the refusal was
read from. `Locator` (`crates/specify/ess-compiler/src/resolve.rs:427`) recovers the file by
searching the whole specification for the declaration's name, and reports `<document>` — the whole
specification, not a file — whenever that search is not unique.

`story:enum-variant-in-an-entity-invariant`'s acceptance clause 1 asks for "a stable refusal code,
the file it was read from, and the variants that are declared". The first and third are structural.
The second is a name search, and the search fails on the plainest case the compiler is built to
reach: one entity declared in two files, which ESS itself refuses with `DuplicateDeclaration`.
`whole_name` does not help — both occurrences are whole names.

Measured by the wave-24 adversary at `adversary_locator_pass1.rs:252`, exit 101: all three refusals
in that run cited `("<document>", None)`.

## Acceptance

A refusal produced while reading a multi-file specification names the file its declaration was read
from, without searching for the name, including when the same name is declared in two files.

## Scope

Re-derived 2026-09-28 by `story-scoper` on `46e367ab2`. Each line **cited** or **inferred**.

- **Status:** open — ignored acceptance test `crates/specify/ess-compiler/tests/locator_citations.rs:251` fails at `:267` (`<document>` ≠ `dup_a.yaml`), run 2026-09-28 — cited
- **Files:** `crates/specify/ess-primitives/src/error.rs:679` `SyntaxSpan`, `:693` `Site.span`, `:705` `ValidationError`, `:782` `at_span` — cited (story's `:663` stale)
- **Files:** `crates/specify/ess-compiler/src/resolve.rs:463` `Locator` (`<document>` fallback), `:667` `bridge`, `:704` `span_of_site` — cited (story's `:427` stale)
- **Tests:** `crates/specify/ess-compiler/tests/locator_citations.rs:187-270` — un-ignore the acceptance test — cited
- **Symbols:** `SyntaxSpan::source`, `Site`, `ValidationError::at_span` (0 production callers), `Locator::span`, `bridge` — cited
- **Also likely:** `crates/specify/ess-domain/src/system.rs:301` `SpecPart.source`, `:823` `Assembly.claims` — inferred
- **Also likely:** `crates/specify/ess-domain/src/spec.rs:244` `Specification::assemble`; `entity.rs:863` invariant refusal location string — inferred
- **Not required:** rewriting ~262 `ValidationError` construction sites; a declaration→source map consulted by the bridge may suffice — inferred design
- **Documents:** `docs/design/review-typed-diagnostics.md` if the design is recorded — inferred
- **Confidence:** medium — defect and test cited; plumbing design unmade
- **Would collide with:** units touching `resolve.rs` bridge, `error.rs` `Site`, or `system.rs` `Assembly` — inferred
- **Safety fact:** `Site` is `#[serde(skip)]` (`error.rs:723-731`), so a source on it changes no serialized bytes — inferred
