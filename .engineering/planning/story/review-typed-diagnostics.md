---
format: aep.planning-md/1
id: story:review-typed-diagnostics
kind: story
status: draft
title: Carry diagnostic identity independently of rendered wording
tags:
- P2
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: inferred
  path: crates/specify/ess-domain/src/binding.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/component.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/domain.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/entity.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/locate.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/spec.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/topology.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/types.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/view.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/wire.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/error.rs
- confidence: inferred
  path: docs/design/review-typed-diagnostics.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
revision: 21
---
## Finding and source

F14 (P2) from `docs/reviews/2026-09-05-architecture-review.md:481`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/specify/ess-compiler/src/resolve.rs:565`, `crates/specify/ess-compiler/src/source.rs:1`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Changing human diagnostic wording no longer changes machine rule identity or the cited construct/source location for the migrated validation paths.

## Implementation boundary

Design typed rule identity, construct reference and source path in the existing diagnostic model, then migrate the heuristic bridge incrementally with an explicit remaining-path inventory. Preserve domain-specific codes and accumulated hints. Establish syntax spans for repeated-name and multi-file cases without inventing a generic semantic error registry.

## Validation

Compare diagnostics before/after wording-only changes; repeated names, nesting and cross-file references retain correct codes/locations. Require a fixture for every migrated rule and list any legacy heuristic path still unsupported.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No blanket claim that all parser spans are exact until every inventoried path is migrated; the story closes only when F14's current family-bridge paths have a typed replacement or explicit unsupported result.

## Scope

Derived 2026-09-09 by `aep-drive:story-scoper` 0.8.1 at ESS main 51dd8a7; file-level split against
story:review-primitive-semantics applied by the coordinator. Every line is **cited** (read from
the story or the tree) or **inferred**. Supersedes the 2026-09-05 coordinator scope, which named
`ess-domain` as the second crate: the type the compiler parses (`ValidationError` with
`location: String`) is declared in `ess-primitives`, and `ess-domain` is where 145 construction
sites write those strings.

- **Primary surface:** `crates/specify/ess-compiler` — cited; F14 evidence sites `src/resolve.rs:565`, `src/source.rs:1`.
- **Files:** `crates/specify/ess-compiler/src/resolve.rs:565` (`family_of`), `:591` (`class_of`), `:637` (`STRUCTURAL`), `:696` (`needles_for`), `:452` (`Locator::span`), `:543` (`bridge`), `:529` (`diagnose`), `:534` (`diagnose_locating`) — cited.
- **Files:** `crates/specify/ess-compiler/src/source.rs:53` (`Span`, `located: Option<Location>`) — cited.
- **Files:** `crates/specify/ess-compiler/src/diagnostic.rs:34` (`Code { family, number }`) — cited; the code the bridge synthesises.
- **Files, ess-primitives (this unit):** `src/error.rs:393` (`ValidationError.location: String`), `:188` (`ValidationCode`), `:78` (`ParseError::Shape { location: String }`) — cited; `resolve.rs:528` says "closing that gap means giving ValidationError structured fields". **Not this unit:** `src/facts.rs`, `src/node.rs`, `src/predicate.rs` (story:review-primitive-semantics) and `src/lib.rs` (coordinator).
- **Files, ess-domain (this unit):** `src/command.rs` (39 `ValidationError::new` sites), `src/entity.rs` (20), `src/binding.rs` (17), `src/component.rs` (15), `src/view.rs` (11), `src/topology.rs` (10), `src/system.rs` (10, incl. `Source(String)` :182 and `Source::DOCUMENT` :186), `src/spec.rs` (7), `src/types.rs` (6), `src/domain.rs` (5), `src/wire.rs` (1), `src/locate.rs` — inferred; migrating a rule means touching its constructor. **Not this unit:** `src/expression.rs` and `src/primitive_admission.rs` (story:review-primitive-semantics).
- **Symbols:** `family_of`, `class_of`, `needles_for`, `STRUCTURAL`, `Locator`, `codes::family::*`, `codes::class::*`, `ValidationError`, `ValidationCode`, `Span::located` — cited.
- **Tests that pin today's behaviour:** `crates/specify/ess-compiler/tests/billing.rs:401-443` pins `ESS-TYPE-008` at `types.yaml:6` — cited; `resolve.rs:2842-3085` in-crate tests pin `family_of` and `Span::located` — cited; `crates/specify/ess-domain/tests/billing.rs`, `expression.rs` and 48 in-crate `.location` assertions (`binding.rs` 11, `spec.rs` 10) — inferred; break if location strings become structured.
- **Consumer:** `crates/edge/ess-cli/src/load.rs:56` calls `diagnose_locating` and renders `error[ESS-…]` — cited. No CLI test pins a compiler-family code (grep across `ess-cli`, `verify/`, `generate/` returns only `ESS-AUTHOR-*`, a conformance envelope) — cited absence.
- **Documents:** `docs/design/review-typed-diagnostics.md` — inferred; does not exist at 51dd8a7. `website/docs/guides/write-a-specification.md:134-137` prints `[undeclared_reference] command.…` location strings and `error[ESS-COMMAND-001]` — cited; drifts if wording changes.
- **Confidence:** medium — the compiler files are cited by the review; the `ess-primitives` edit is the tree's own stated remedy but the story never named that crate.
- **Would collide with:** any unit touching `ess-compiler`'s `resolve.rs`/`source.rs`/`diagnostic.rs`; any unit changing `ess-primitives::error::ValidationError`/`ValidationCode` (9 crates depend on `ess-primitives`); any unit editing `ess-domain` validate paths or their location strings. Within this wave: story:review-primitive-semantics shares the `ess-primitives` and `ess-domain` crates at file-disjoint reservations.

## Implementation pointers — 2026-09-09

Written 2026-09-09 by the wave coordinator from the scoper report and the tree at 51dd8a7. Each
line is cited (a `file:line` or an artifact) or marked inferred.

**The heuristic bridge, inventoried (all `crates/specify/ess-compiler/src/resolve.rs`, cited):**

| fn | line | derives |
|---|---|---|
| `family_of(&str)` | 565 | family from the first token of `location` split on `. [`, `s` trimmed; unknown head → `SPEC` |
| `class_of(ValidationCode)` | 591 | class number from the enum; `_ => OTHER` — typed, but it collapses e.g. `UndeclaredReference` and `UnknownWorkflow` to one class, so a rule id cannot be recovered from the emitted `Code` (inferred consequence) |
| `bridge` | 543 | assembles `Code::new(family_of, class_of)` + `locator.span(location, needles_for)` |
| `STRUCTURAL` | 637 | 50-key stop-list deciding where a path stops naming a declaration |
| `needles_for(&str)` | 696 | text needles `"{last}:"`, `"name: X"`, `"id: X"`, `"component: X"` from path tokens |
| `Locator::span` / `scan` / `location_of` | 452 / 485 / 505 | substring search; a line only when the needle occurs exactly once across all files |
| `Span.located: Option<Location>` | `source.rs:60` | the honest `None` the story must preserve |

**Types that would carry the typed identity (cited):** `ValidationError { code, location: String,
message, hint }` at `ess-primitives/src/error.rs:393` — `serde` + `schemars::JsonSchema` with
`deny_unknown_fields`, so a new field is wire-visible wherever the struct is serialized;
`ValidationCode` `:188` (`#[non_exhaustive]`, ~35 variants); `ParseError::Shape { location: String }`
`:78`, the same pattern on the parse side; `Source(String)` `ess-domain/src/system.rs:182`;
`Code { family: &'static str, number: u16 }` `ess-compiler/src/diagnostic.rs:34` — already typed,
no source path.

**Persistence check (cited).** `ValidationError` is not in `schemas/generated/ess.schema.json`
(grep 0) and no `ess-cli` path serializes it (grep 0), so a structural field added to it is not a
published-format change today. Confirm with `cargo xtask schema --check` and `cargo xtask generate
--check` before relying on that; if either moves, the field travels in-process instead (a
`Site`/`ConstructRef` struct beside the string, consumed by `bridge`) and the persisted shape stays
— inferred.

**Shape of the change (inferred, for the design page to decide):**
1. Add a typed site to the domain error: rule identity (the `ValidationCode` variant is already
   that — keep it), a construct reference (kind + qualified name + member path as segments, not a
   dotted string) and an optional syntax span. Keep `location: String` rendered from the site so
   the 48 `.location` assertions and the guide sample keep passing during the migration.
2. Migrate `bridge` to read the site when present and fall through to `family_of`/`needles_for`
   when absent; keep the fallback until the inventory is empty, and list the remaining string-only
   sites in the design page ("explicit remaining-path inventory", story boundary).
3. Migrate producers by family, `command.rs` (39 sites) first because it is the largest and
   `ESS-COMMAND-001` is the code the guide prints; each migrated rule gets a fixture under
   `crates/specify/ess-compiler/tests/fixtures/` (today: one file, `adversary_expression.yaml`)
   covering repeated names, nesting and a cross-file reference, asserting code and `Location` are
   unchanged under a wording-only edit.
4. Syntax spans for repeated-name and multi-file cases come from the parser's node positions, not
   from `Locator`'s substring scan; where no position is available the span stays `None`.

**Design page first.** `docs/design/review-typed-diagnostics.md` does not exist — cited. The
unit's first commit is that page: the site type, the rendering rule for `location`, the
migration order, the inventory of paths left on the heuristic, and what the closing fixture set
asserts. The adversary attacks the page and the code together (coordinator decision, inferred).

**Out of this story, same finding:** `crates/infra/infra-domain/src/code.rs:190` has a second
`ValidationError { location: String }` — the "multiple envelopes" F14 names; file it, do not fix
it here — cited.

**Not this unit:** `ess-primitives/src/facts.rs`, `node.rs`, `predicate.rs`, `ess-domain/src/expression.rs`
and `primitive_admission.rs` belong to story:review-primitive-semantics in the same wave;
`ess-primitives/src/lib.rs` is the coordinator's (patch to scratch, name it in the report).
