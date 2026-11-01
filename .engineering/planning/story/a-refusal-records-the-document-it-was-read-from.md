---
format: aep.planning-md/3
id: story:a-refusal-records-the-document-it-was-read-from
kind: story
status: active
title: A refusal records the document it was read from
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/load.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/locator_citations.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: inferred
  path: docs/design/review-typed-diagnostics.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T20:19:27Z", actor: "human:timo", revision: 13}
- {from: "proposed", to: "active", at: "2026-11-01T20:19:28Z", actor: "human:timo", revision: 14}
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

Re-derived 2026-11-01 by `story-scoper` on `78d58fb47` (ESS 0.81.0). Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Status:** open — the acceptance test `crates/specify/ess-compiler/tests/locator_citations.rs:251-254` is still `#[ignore]`d. The defect reproduces with the installed `ess` 0.80.0: the story's fixture loaded as a directory prints `--> <document> (entity shop.probe.Order.invariants[0])` and `"source": "<document>"` under `--format json` — cited
- **Primary surface:** `crates/specify/ess-compiler/src/resolve.rs`, the bridge and the `Locator` — cited
- **Files:** `crates/specify/ess-compiler/src/resolve.rs`: `Locator` `:724`, `Locator::span` `:773` with the `<document>` fallback at `:799-803`, `unique` `:809`, `scan` `:872`, `diagnose_locating` `:989`, `bridge` `:1006` (untyped arm `:1014-1017`), `span_of_site` `:1089`, `needles_from_tokens` `:1390` (declaration derived at `:1413-1424`) — cited
- **Files:** `crates/specify/ess-domain/src/spec.rs`: `Specification::assemble` `:331`, its per-file absorb loop `:384-386`, `declare` `:875`, `record` `:934`, `Collected` `:993` with its `declared` map `:1019`, and the 8 `record(...)` sites `:1292, :1313, :1384, :1407, :1435, :1471, :1522, :1550` — cited
- **Files:** `crates/edge/ess-cli/src/load.rs:141-144`: the CLI's refused path, `Specification::assemble` then `diagnose_locating` — cited
- **Tests:** `crates/specify/ess-compiler/tests/locator_citations.rs`: remove the ignore at `:252-254`, route the `refused` helper (`:88-115`) through the traced entry points, keep the assertion at `:267-272` unchanged — cited
- **Symbols:** the invariant refusal is built untyped (`ess-domain/src/expression.rs:468`, location from `entity.rs:1148`), so the bridge reaches it through `needles_for`, not a `Site` — cited. The kind labels `declare` keys on match `ConstructKind::as_str` (`ess-primitives/src/error.rs:565-577`), so a `(kind, name)` key is derivable from typed and untyped refusals — cited
- **Design (smallest):** a side record returned beside the errors and consulted by the bridge; no `ValidationError` field, no construction sites rewritten — inferred:
  1. `ess-domain` adds `Specification::assemble_traced(files) -> Result<Self, Traced>` (`Traced { errors, provenance }`); `assemble` forwards to it and drops the record.
  2. The provenance holds per file the range of error indices pushed while `Collected::absorb` read it (`:384-386`), and `kept: (kind, name) -> Source` written where `record` inserts.
  3. The bridge: a refusal pushed while a file was read gets that file; one produced after assembly gets the kept copy's file by `(kind, name)`; anything else falls back to today's search.
  4. `Locator` gains a search for the line inside one file only (that file with `located: None` when the needle is not unique there).
  5. A new public `diagnose_traced`; `diagnose_locating`, `advise_locating` and `compile_locating` keep their behaviour.
- **Not touched:** `ess-primitives/src/error.rs` (`SyntaxSpan` `:706`, `Site` `:720`, `ValidationError` `:732`): a field there changes derived `PartialEq` against ~50 hand-built refusals — inferred. `ess-domain/src/system.rs` `SpecPart`/`Assembly` — inferred. Compile and advise paths (`resolve.rs:1455`, `:1736`, `:1057`) — follow-up, inferred
- **Documents:** none required; `docs/design/review-typed-diagnostics.md:440-442` is quoted by `design_page_matches_the_fixture.rs:190-194` and stays — inferred
- **Serialized bytes, `ValidationCode`, design-page counts:** none move, so `task test-xtask` is not needed — inferred
- **Confidence:** high on the sites; medium on the design (the rule that a refusal pushed while a file is read is about that file is unproven; the implementor proves it with a cross-file case where the first copy fails its own conversion)
- **Would collide with:** units touching `ess-compiler` `resolve.rs` `Locator`/`bridge`/`needles_from_tokens`; `ess-domain` `spec.rs` `assemble`, the absorb loop or `record`/`declare`; or `ess-cli` `load.rs` — inferred
