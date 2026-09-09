---
format: aep.planning-md/1
id: story:review-typed-diagnostics
kind: story
status: implemented
title: Carry diagnostic identity independently of rendered wording
tags:
- P2
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/typed_diagnostics.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/error.rs
- confidence: cited
  path: docs/design/review-typed-diagnostics.md
revision: 44
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

## Scope confirmation — wave 21

Written by the wave-21 coordinator from the implementor's confirmation tables (rounds 0-2) and
the unit's diff against 2900f62 (commits 36fa1df, 1b0368b, ba6cb7b). The 2026-09-09 Scope above
stays as the hypothesis; this is what landed.

- **Landed (cited, from the diff):** `crates/specify/ess-primitives/src/error.rs`;
  `crates/specify/ess-compiler/src/resolve.rs`, `tests/typed_diagnostics.rs`,
  `tests/fixtures/typed_diagnostics/*.yaml`, `tests/adversary_typed_diagnostics_pass1.rs`,
  `tests/adversary_typed_diagnostics_pass2.rs`; `crates/specify/ess-domain/src/command.rs`,
  `src/entity.rs`, `src/types.rs`; `docs/design/review-typed-diagnostics.md`;
  `crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json`.
- **Inferred lines that were wrong:** `crates/specify/ess-domain/src/locate.rs` — 0
  `ValidationError` references; `actor.rs` has 1 site the scope did not list. The migration is
  by location head, not by file: `entity.rs` writes `command.…` heads and was migrated for them.
- **Inferred lines not reached this wave:** `binding.rs`, `component.rs`, `view.rs`,
  `topology.rs`, `system.rs`, `spec.rs`, `domain.rs`, `wire.rs`, `refs.rs`, `actor.rs` — the
  remaining families, inventoried by head and file in the design page's machine-checked block
  with a stated result each; `website/docs/guides/write-a-specification.md` — unchanged and pinned
  by a test.
- **Left for the next unit:** `crates/specify/ess-domain/src/primitive_admission.rs:96` writes one
  `command.…` location (unit U1's file this wave; patch retained in the unit's scratch, pinned at
  1 by the head census); `crates/infra/infra-domain/src/code.rs:190` is the second
  `ValidationError { location: String }` envelope F14 names.

## Wave 21 closure — 2026-09-09

Source on ESS main through wave/review-boundaries-21: unit commits 36fa1df (the typed site,
the bridge reading it, 29 command-family sites migrated), 1b0368b (one needle derivation, a
machine-checked inventory, no wildcard kind), ba6cb7b (every command-head writer sited, a
separator grammar per kind, `rebase` as the one door to `location`, `at_span` instead of a
discarding `with_span`). Design page `docs/design/review-typed-diagnostics.md`, written
first, with two machine-readable blocks a test greps against the tree.

Reviews: review-result:typed-diagnostics-adversary-wave21-pass1 (6 findings, 3 red cases) and
-pass2 (8 findings, 5 red cases), all introduced; ledger between the passes carried 0, new 8,
resolved 6; both answered, review_outcome fixed recorded for each. The coordinator verified
each correction's diff: no assertion removed, one pin tightened, the adversary files untouched.

What this delivers against the acceptance: for the `command` head, every writer carries a
typed construct reference, rule identity and optional syntax span; rewording a migrated rule
changes neither its code nor its cited location, pinned by fixtures for repeated names (both
the located and the honestly unlocated case), nesting and a cross-file reference; the rendered
`location` string is byte-identical everywhere. The remaining heads are inventoried per file
and per head with a stated result each: `entity ` and `component ` (space form, renderable
through `ConstructKind::separator`), the plural `types.`/`domain.` heads no kind renders, one
`command.…` writer in `primitive_admission.rs:96` (patch retained, pinned at 1) and one in
`wire.rs:30` (shared helper), and `crates/infra/infra-domain/src/code.rs:190`, the second
envelope F14 names. Those are the next unit of this migration, not this story's close
condition, which the story's compatibility clause defines as a typed replacement or an
explicit unsupported result per path.

Whole gate run 4 at e448671e (integration branch wave/review-boundaries-21, every lane run individually, exit codes in the wave page's gate table): fmt-check 0, clippy 0, test 0 (217 lanes, 2561 passed, 0 failed, 0 ignored), doc-check 0, example-check 0, projection-check 0, support-check 0, consumer-check 0 (BaselineUnknown 157677, Supported 54, Refused 0), fuzz-check 0 (25 replay cases), release-check 0 (0.20.0 consistent), action-check 0. Runs 1-3 at d81245f, 144aa95 and f346f27 were red on lanes the wave page records; each defect was answered before run 4.
