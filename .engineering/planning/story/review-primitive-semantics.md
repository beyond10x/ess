---
format: aep.planning-md/1
id: story:review-primitive-semantics
kind: story
status: implemented
title: Align primitive admission and exact numeric semantics
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-format-catalog
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/initial-baseline.json
- confidence: cited
  path: crates/generate/ess-gen
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: cited
  path: crates/generate/ess-synth
- confidence: cited
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/node.rs
- confidence: cited
  path: crates/specify/ess-primitives/tests/vectors/primitive-semantics.json
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: crates/verify/ess-conformance/assets/coverage-admission.js
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: docs/design/review-primitive-semantics.md
revision: 47
---
## Finding and source

F08 (P1) from `docs/reviews/2026-09-05-architecture-review.md:326`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/specify/ess-primitives/src/facts.rs:33`, `crates/verify/ess-conformance/src/input.rs:561`, `crates/generate/ess-gen/src/types.rs:145`, `crates/generate/ess-synth/src/rust/wire.rs:1`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

The normative primitive corpus produces equivalent admission and comparison results across facts, conformance, generated codecs and schemas without losing promised integer or decimal precision.

## Implementation boundary

Write the abstract-value/predicate-value/wire-encoding matrix first. Decide exact integer/decimal representations and canonical serialization, then migrate named readers/writers with versioned compatibility fixtures. Include UUID and other constrained primitives rather than retaining arbitrary text admission. The identity catalog supports inventory but does not choose new bytes.

## Validation

Cover 2^53 and 2^53+1, i64 extrema, decimals, invalid UUIDs and supported encodings using shared vectors in Rust, generated Go, schemas and browser adapters. Test old/current readers and explicit refusals for unsupported domains.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

This is a semantic migration, not a first-wave refactor; cross-repository adapters are inventory/ADR prerequisites to release.

## Scope

Derived 2026-09-09 by `aep-drive:story-scoper` 0.8.1 at ESS main 51dd8a7; file-level split against
story:review-typed-diagnostics applied by the coordinator. Every line is **cited** (read from the
story or the tree) or **inferred**. Supersedes the 2026-09-05 coordinator scope: the four cited
packages still own the defect, and 12 commits have touched them since (git log fd06a4d..51dd8a7).

- **Primary surface:** `crates/specify/ess-primitives` — cited; `Number(f64)` at `src/facts.rs:33`, `From<i64>` via `as f64` at :70-72, `is_integral` bound 2^53 at :64-66, `Node::from_value` via `as_f64()` at `src/node.rs:45-51`, numeric compare via `Number::cmp` at `src/predicate.rs:552`.
- **Files, ess-primitives (this unit):** `src/facts.rs`, `src/node.rs`, `src/predicate.rs` — cited. **Not this unit:** `src/error.rs` (story:review-typed-diagnostics) and `src/lib.rs` (coordinator).
- **Files, ess-domain (this unit):** `src/expression.rs:21-43` — cited; `ScalarKind::Number` collapses Integer/Decimal/Binary64 for typechecking. `src/primitive_admission.rs` — inferred. **Not this unit:** every other `ess-domain` file, including `src/types.rs` (the `Primitive` enum stays as declared).
- **Files (Rust admission):** `crates/verify/ess-conformance/src/input.rs:583-603` — cited; `primitive_value` admits Decimal as `Number` and Uuid/Timestamp/Duration/Bytes as bare text; test table :760-788 fixes `Uuid` = `FactValue::text("x")`.
- **Files (witness):** `crates/verify/ess-conformance/src/witness.rs:47-52,304-317,504-505` — cited; an Integer witness is written as `1.0` because the value type is `f64`.
- **Files (schemas):** `crates/generate/ess-gen/src/types.rs:47-63,480-518` — cited; `DECIMAL_PATTERN`, `UUID_PATTERN`, `primitive()` maps Integer→`integer`, Decimal→string+pattern, Uuid→string+pattern.
- **Files (generated Rust):** `crates/generate/ess-synth/src/rust/wire.rs:575-596,683-720`, `rust/mod.rs:327-339` — cited; `Decimal(pub String)`, `Uuid(pub String)` with text-only decode and no UUID grammar check.
- **Files (generated Go):** `crates/generate/ess-synth/src/go/layout.rs:864-872`, `go/http.rs:535-552,700-747`, `go/mod.rs:196-206` — cited; Integer→`int64`, Decimal/Uuid via `NewDecimal`/`NewUuid` string wrappers.
- **Files (conformance Go runtime):** `crates/verify/ess-conformance/src/go/runtime.go:988,2435-2445,2532-2545` — cited; `Node` numbers are `float64`, `asNumber` collapses `int64`/`json.Number` to `float64`, `primitive()` admits `"uuid"` as any string.
- **Files (browser adapter):** `crates/verify/ess-conformance/assets/coverage-admission.js:162,188,294` — cited; `NumberToken` → JS `Number`, primitive kinds admitted by name only. `player.js` has no number/UUID handling — cited. `ess-synth/src/web/bridge.rs` encodes only `occurrence` integers (:202,:393,:691), not model primitives — cited.
- **Symbols:** `Number`, `Number::is_integral`, `From<i64> for Number`, `Node::from_value`, `primitive_value`, `asNumber`, `primitive` (Go), `DECIMAL_PATTERN`, `UUID_PATTERN` — cited.
- **Also likely:** `crates/generate/schema-contract/src/realize/normalize/numeric.rs` — inferred; the Binary64 normalizer already owns a 2^53+1 vector at :128, but it is Binary64 territory (`docs/design/model-binary64.md`), not this story's. `Cargo.toml`, `Cargo.lock` — inferred; only if an exact-decimal crate is added under `[workspace.dependencies]`.
- **Documents:** `docs/design/review-primitive-semantics.md` — inferred; does not exist at 51dd8a7, and the story's "matrix first" boundary makes it the first deliverable. `.engineering/planning/obligation/review-contract-rollout-coordination.md:25` lists F08 as a migration candidate — cited.
- **Confidence:** high for the four packages and the exact defect sites (review F08 and tree reads agree); medium for where the shared vectors live, which the story leaves undecided.
- **Would collide with:** any unit touching `ess-primitives` `facts.rs`/`node.rs`/`predicate.rs`; `ess-domain` `expression.rs`/`primitive_admission.rs`; `ess-conformance` `input.rs`, `witness.rs`, `go/runtime.go`, `assets/coverage-admission.js`; `ess-gen` `types.rs`; `ess-synth` `rust/wire.rs`, `rust/mod.rs`, `go/layout.rs`, `go/http.rs`; `Cargo.lock`; and any unit that changes conformance report, witness or suite bytes. Within this wave: story:review-typed-diagnostics shares the `ess-primitives` and `ess-domain` crates at file-disjoint reservations, and story:review-execution-recovery-implementation shares `Cargo.lock`.

## Implementation pointers — 2026-09-09

Written 2026-09-09 by the wave coordinator from the scoper report and the tree at 51dd8a7. Each
line is cited (a `file:line` or an artifact) or marked inferred.

**The constraint that shapes the unit.** F08 is a listed migration candidate of
obligation:review-contract-rollout-coordination (its body, "Known scope"), and
`docs/design/review-conformance-coverage.md:172-176` freezes suite/report numeric spelling to
serde_json 1.0.151 ("do not coerce to integer or string") — cited. So the exact representation
lands in two stages, and only the first is this wave's:

1. **Byte-preserving stage (this story, this wave).** Make `Number` exact internally, keep every
   persisted spelling. `Number(f64)` at `ess-primitives/src/facts.rs:33` becomes an exact
   representation (an `Integer(i64) | Decimal(…) | Binary64(f64)`-shaped enum or an i128/decimal
   pair — the design page decides, inferred) whose `Serialize` writes the bytes today's readers
   receive for today's values (`1.0` for an integral witness, `witness.rs:47-52`) and whose
   comparison no longer collapses 2^53+1 onto 2^53. `From<i64>` (`facts.rs:69-73`) and
   `is_integral` (`:64-66`) stop lying. Admission tightens where the story asks: `primitive_value`
   (`ess-conformance/src/input.rs:583-603`) refuses non-UUID text for `Uuid`, Go `primitive()`
   (`go/runtime.go:2532-2545`) and `coverage-admission.js:294` refuse the same, using one shared
   vector corpus checked from Rust, the Go runtime and the browser adapter. Old-reader
   compatibility fixtures pin the unchanged bytes.
2. **Canonical serialization stage (not this wave).** The new spelling (`1` for an integer, exact
   decimal strings) ships behind a format version, coordinated through the obligation's ADR and
   relying-reader inventory (AEP's `aep-ess-evidence::adapt_json` reads
   `ess-conformance-report/1`, obligation body). Do not enable it here.

**First-change symbols (cited):** `pub struct Number(f64)` `facts.rs:33`; `From<i64>` `:69-73`;
`is_integral` `:64-66`; `Ord` via `total_cmp` `:103-106`; `Display` `:109-118`;
`Node::from_value` `as_f64()` `node.rs:45-51`; `Number::cmp` use `predicate.rs:552`;
`primitive_value` `input.rs:583-603`; `asNumber` `runtime.go:2435`, `primitive` `:2532`;
`primitive()`/`UUID_PATTERN`/`DECIMAL_PATTERN` `ess-gen/src/types.rs:480-518,47-63`;
`decode_primitive`/`decode_key` `ess-synth/src/rust/wire.rs:683-720`; `Uuid(pub String)`
`rust/mod.rs:339`.

**Byte surfaces that must not move in this wave (cited):** witness artifacts
(`witness.rs:47-52`); report prose via `quote(&Node)` (`ess-conformance/src/report.rs:417-422`);
JSON-schema / OpenAPI / AsyncAPI primitive nodes (`ess-gen/src/types.rs:480-518` — any `format`
or `pattern` change is a published-contract change); generated Rust and Go source text
(`rust/mod.rs:322-339`, `rust/wire.rs:575-596`, `go/layout.rs:864-872`, `go/http.rs:700-747`).
`cargo xtask generate --check` and `task consumer-check` are the gates that notice. The
consumer-coverage baseline names `facts::Number/field/0`
(`ess-xtask/src/consumer_coverage/initial-baseline.json:769-774`): a changed `Number` shape
needs a reviewed classification, not a silent baseline edit. docs-ir renders no numbers
(`ess-gen/src/document.rs`, grep clean) — cited.

**Design page first.** `docs/design/review-primitive-semantics.md` does not exist — cited. The
first commit of the unit is that page: the abstract-value / predicate-value / wire-encoding
matrix per `Primitive` variant (`ess-domain/src/types.rs:42-61`), the chosen exact
representation, the preserved spellings, and the corpus location. The adversary attacks the page
and the code together; no separate binding round is scheduled (coordinator decision, inferred).

**Where the vectors live (inferred).** Three unrelated 2^53+1 fixtures exist today
(`ess-cli/tests/support/coverage_cases.rs:147-148,357`, `ess-cli/tests/go_conformance.rs:942`,
`schema-contract/src/realize/normalize/numeric.rs:128`); none is a cross-language corpus. One
JSON corpus under `crates/specify/ess-primitives/tests/vectors/` read by the Rust tests, the Go
runtime test and the browser-adapter test is the smallest shape that satisfies "shared vectors".

**Typechecking (inferred).** `ess-domain/src/expression.rs:31` typechecks Integer/Decimal/Binary64
as one `ScalarKind::Number`. The matrix must say whether predicate typing distinguishes them; a
change there is inside this unit's reservation, a change to `Primitive` itself is not.

**Not this unit:** `ess-primitives/src/error.rs`, every `ess-domain` validator file, and
`ess-compiler` belong to story:review-typed-diagnostics in the same wave; `ess-primitives/src/lib.rs`
is the coordinator's (patch to scratch, name it in the report).

## Scope confirmation — wave 21

Written by the wave-21 coordinator from the implementor's confirmation tables (rounds 0-2) and
the unit's diff against 2900f62 (commits d107b53, 33f4568, 12ec9f1, 70ba641). The 2026-09-09
Scope above stays as the hypothesis; this is what landed.

- **Landed (cited, from the diff):** `crates/specify/ess-primitives/src/facts.rs`, `src/node.rs`,
  `tests/primitive_corpus.rs`, `tests/vectors/primitive-semantics.json`;
  `crates/verify/ess-conformance/src/input.rs`, `src/admission.rs` (new), `src/go/runtime.go`,
  `assets/coverage-admission.js`, `tests/witness.rs`, `tests/primitive_corpus.rs`,
  `tests/primitive_divergence.rs`, `tests/adversary_integer_bound_pass2.rs`;
  `crates/specify/ess-primitives/tests/adversary_round_trip_law_pass2.rs`;
  `crates/generate/ess-gen/src/types.rs`; `docs/design/review-primitive-semantics.md`;
  `crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json`,
  `initial-baseline.json`, `mod.rs` (coordinator re-freeze); `docs/design/review-consumer-coverage.md`
  (coordinator clause).
- **Inferred lines that were wrong:** `crates/specify/ess-domain/src/primitive_admission.rs` is
  Binary64 format-version admission, not this story's — untouched. `Cargo.toml`/`Cargo.lock` —
  no dependency was needed (scaled `i128`). `crates/generate/schema-contract/.../numeric.rs` —
  Binary64 territory, untouched. `crates/specify/ess-domain/src/expression.rs` — the matrix keeps
  Integer/Decimal/Binary64 as one `ScalarKind::Number`, untouched.
- **Cited lines not reached:** `crates/specify/ess-primitives/src/predicate.rs` and the four
  `crates/generate/ess-synth` files — no edit was needed to keep emitted bytes; `cargo xtask
  generate --check` is the proof.
- **Not this wave, filed:** story:primitive-canonical-serialization (the decimal half of the
  wire, behind a format version, under obligation:review-contract-rollout-coordination).

## Wave 21 closure — 2026-09-09

Source on ESS main through wave/review-boundaries-21: unit commits d107b53 (exact `Number`,
one admission grammar, the corpus), 33f4568 (the round-trip law, base64 and integer bounds
answered), 12ec9f1 (one `Integer` range, `Repr::exact` as the sole constructor of an exact
value), 70ba641 (coordinator: consumer-coverage baseline re-freeze under the new
representation-only clause of `docs/design/review-consumer-coverage.md`), 154d269 (the
coverage lineage no longer pins the binary64 collapse; the browser adapter compares integer
tokens by digits). Design page `docs/design/review-primitive-semantics.md`, written first.

Reviews: review-result:primitive-semantics-adversary-wave21-pass1 (5 findings, 4 red cases,
all introduced) and -pass2 (5 findings, 4 red cases; 1 pre-existing filed as
story:primitive-canonical-serialization); ledger between the passes carried 0, new 5,
resolved 5; both answered, review_outcome fixed recorded for each. The coordinator verified
each correction's diff for dropped assertions and relaxed pins.

What this stage delivers against the acceptance: fact numbers are exact where binary64 is not
(2^53 and 2^53+1 are two values in every constructor and through a document), `Integer` is one
range in every lane, admission is one grammar answered from one corpus
(`crates/specify/ess-primitives/tests/vectors/primitive-semantics.json`, 47 admission vectors)
by Rust, the Go runtime and the browser adapter, and every persisted spelling binary64 carries
is unchanged (witness `1.0` pin, `cargo xtask generate --check`, `schema --check`). A value
binary64 never carried is now written as its exact integer token instead of a rounded float;
the page argues this moves no byte any reader has received. The decimal half of the wire is
story:primitive-canonical-serialization, held by obligation:review-contract-rollout-coordination.

Whole gate run 4 at e448671e (integration branch wave/review-boundaries-21, every lane run individually, exit codes in the wave page's gate table): fmt-check 0, clippy 0, test 0 (217 lanes, 2561 passed, 0 failed, 0 ignored), doc-check 0, example-check 0, projection-check 0, support-check 0, consumer-check 0 (BaselineUnknown 157677, Supported 54, Refused 0), fuzz-check 0 (25 replay cases), release-check 0 (0.20.0 consistent), action-check 0. Runs 1-3 at d81245f, 144aa95 and f346f27 were red on lanes the wave page records; each defect was answered before run 4.
