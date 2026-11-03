---
format: aep.planning-md/3
id: story:a-view-read-checks-every-rows-shape
kind: story
status: draft
title: A view read checks every returned row against the view fields
summary: 'Row shape check on synthesized and authored view reads, split from #680 for 0.70.0; design page first.'
tags:
- adopter-report
- ess-0.70.0
refs:
- provider: github
  reference: beyond10x/ess#680
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
revision: 10
---
## Outcome

A synthesized or authored view read checks every row the target returns against the view's declared fields: a missing required field or a mistyped field fails the scenario, not only the fields the scenario claims.

## Evidence

Split from `story:a-view-read-checks-its-rows-shape` (beyond10x/ess#680) for 0.70.0. Traced in 0.69.0: `ExpectView` and `EventuallyView` (`crates/verify/ess-conformance/src/scenario.rs`) carry no shape; the event payload shape is the model. A shape on view reads needs a new suite format pair and a row check in the Rust runner and the Go and TypeScript runtimes.

## Acceptance

- A design page under `docs/design/` settles which view reads carry a shape and the format consequence, before code.
- A view read refuses a row missing a required field, and a row with a mistyped field.
- A suite whose view reads carry no shape keeps its bytes.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance`, the view-step vocabulary and its three runners (`src/runner.rs`, `src/go/runtime.go`, `src/ts/runtime.ts`) — cited.
- **Files:** `src/scenario.rs:2748` (`ExpectView`) and `:2776` (`EventuallyView`), each gaining a `shape: PayloadShape` field — cited.
- **Also likely:** `src/admission.rs` (`step_keys` rows at :656-657; the major gate and coverage list ending at :283), `src/go/mod.rs:429` (`NEWEST_ADMITTED_SUITE_MAJOR`), `src/go/runtime.go:104` (`newestSuiteMajor`), `src/ts/runtime.ts:7173` (`'ess-conformance/63'` table) — cited; a new `src/view_row_shape.rs` format module, `src/synthesize.rs` and `src/authored.rs` if synthesis emits shapes — inferred.
- **Documents:** a new `docs/design/` page, which the acceptance requires before any code — cited.
- **Confidence:** high for the surfaces and the format bump; medium for the synthesis footprint (the design page decides which view reads carry a shape).
- **Would collide with:** any unit adding a field to `ExpectView`/`EventuallyView` (about 100 struct-literal sites in 83 test files, grep); any unit adding a suite format pair (`scenario.rs`, `admission.rs`, `go/mod.rs`, `go/runtime.go`, `ts/runtime.ts`); the view constructors in `synthesize*` and `authored.rs`; the committed `suites/generated/{billing,gatepass,oracle-fixture}/suite.json` and `website/data/billing.presentation.json` if synthesis emits shapes — inferred.
- **Safety fact:** a suite with no view shape keeps its bytes when the new field is skipped when empty (`skip_serializing_if = "PayloadShape::is_empty"`, e.g. `scenario.rs:2767`) — cited.
- **Overlap:** `story:view-and-error-assertions-carry-their-value-shape` adds the same `shape` field to the same two variants (and `ExpectError`), the same admission rows and the same format bump; recommended: one shared pair `/64` + `/65` with two independent floors, delivered as one unit, with one `null` rule settled first (`LeafShape::admits` already folds `null` for any optional leaf) — inferred.
- **Store note:** `story:a-view-read-checks-its-rows-shape` is `implemented`, but the tree has only its item 1 (the AUTHOR-042 hint, `tests/author_generated_value.rs:255`); none of the four row scenarios exist — cited.
- **Size:** large, about 900-1,100 lines plus about 100 mechanical literal edits; a new suite format pair `ess-conformance/64` and `/65` (newest pair is `member_paths` 62/63, `src/member_paths.rs:40`) — cited.
- **ess-conformance targets:** `leaf_payloads`, `leaf_payloads_go`, `member_paths_runtimes`, `runtime_suite_admission`, `runtime_parity_go_28_35`, `runtime_parity_typescript_28_35`, `author_generated_value` — inferred.
