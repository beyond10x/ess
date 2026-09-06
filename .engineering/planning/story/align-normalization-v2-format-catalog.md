---
format: aep.planning-md/1
id: story:align-normalization-v2-format-catalog
kind: story
status: implemented
title: Align the internal normalization format catalog
relations:
- serves: vision:O2
- informed_by: story:review-format-catalog
- informed_by: story:source-pinned-data-normalization
scope:
- confidence: cited
  path: docs/design/review-format-catalog.md
revision: 6
---
## Context

At exact ESS source `f0cbf56a1e3985a08effffc88a3e7f5b17893a9b`,
`website/docs/reference/formats.md:87` records unreleased `ess-normalization/2`, while
`docs/design/review-format-catalog.md:46–79` still inventories only normalization /1 and
normalization-target /1. The public entry agrees with the source. Synchronize only this internal
inventory through a companion to the completed format-catalog work; retain its historical review,
closure and publication evidence.

`FORMAT` and `FORMAT_V2` (constants exported from
`crates/generate/schema-contract/src/realize/normalize/recipe.rs:7–10`) and
`normalize::Plan::{read,check,to_json}` (`normalize.rs:49–155`) establish the authored-format
versions, checked-bundle admission and typed pretty JSON plus LF. The new operations and numeric
input declarations require /2 (`normalize/check.rs:221–247`; `normalize.rs:80–97`);
omitted declarations preserve /1 serialization and numeric routing
(`normalize/recipe.rs:26–50`; `normalize/input.rs:64–69,134–165`).
`normalize/target.rs:127–145` still emits the independent normalization-target /1 report.

## Acceptance

- The internal catalog records `ess-normalization/2` at the exact source pin and as unreleased
  there, describing its ordered construction/selection and explicit binary64 additions without
  changing any producer, reader, format, canonical bytes or numeric semantics.
- It distinguishes closed Recipe DTO parsing from `Plan::read`/`check` admission against supplied
  checked bundles. The /1 refusal includes new operations and an explicitly present
  `binary64_inputs` map even when empty; explicit null is a syntax refusal. Omitted declarations
  retain /1 canonical field layout and its existing numeric-admission path.
- It identifies the checked recipe writer as P: typed pretty JSON plus one LF. Existing
  recipe-digest, schema/file-digest and report self-exclusion statements remain accurate.
  `ess-normalization-target/1` is the independent Serialize-only Rust report produced from either
  checked recipe version; it is neither an authored recipe reader nor a full-synthesis report.
- All historical baseline, review, closure and publication records remain intact. The already
  matching public row is cited as evidence. No release, external consumer migration, Go/TypeScript
  implementation or newly executed compatibility result is claimed.

## Validation

Review the minimal internal-document diff against the pinned constants, Serde attributes, version
checks, input routes and writers. Inspect the existing /1 fixture digest, /2 nested-operation
refusals, numeric declaration refusals/round trip and target report identity assertions only as
source evidence; distinguish those assertions from tests actually run by a later implementor or
coordinator. This proposal ran no tests or builds. No new prose-only executable test is required.
The integration coordinator retains the repository-required `task check` and `task site-build`
evidence on the final candidate, as required by `AGENTS.md` Gate.

## Boundaries

Only `docs/design/review-format-catalog.md` is an intended implementation edit. Source owners,
existing tests, the matching public reference and historical planning/review artifacts are evidence,
not edit reservations. This companion does not reopen either predecessor, migrate formats, implement
normalization, refresh public delivery, add new formats or expand the empty-scenarios remediation.

## Scope

Derived 2026-09-06 by `story-scoper` from the coordinator's bounded companion request and exact
ESS `f0cbf56a1e3985a08effffc88a3e7f5b17893a9b`; every scope entry below is cited or inferred — cited.

- **Primary surface:** documentation only; reconcile the existing internal format inventory with the already present normalization /2 source and public row — cited, coordinator assignment and `docs/design/review-format-catalog.md:46–79`.
- **Files:** `docs/design/review-format-catalog.md` — cited, the sole amendment path named in the assignment.
- **Symbols:** no production symbol edits; `FORMAT`, `FORMAT_V2`, `Recipe`, `Plan::{read,check,to_json,rust}` and `Report` are read-only evidence — cited, `normalize/recipe.rs:7–40`, `normalize.rs:43–155` and `normalize/target.rs:21–30,127–145`.
- **Also likely:** none; this is a single-document amendment — inferred from the matching public row and bounded acceptance.
- **Documents:** only the internal catalog; existing public, design, source and test citations do not reserve their paths for editing — cited, coordinator assignment.
- **Confidence:** high because the assignment names the exact document, and the pinned internal/public rows plus source establish the missing entry — cited.
- **Would collide with:** another edit to `docs/design/review-format-catalog.md`, especially its Source-driven integration additions section — inferred; no crate or public-document reservation is justified by this amendment.

## Implemented result and evidence

The single internal-document amendment was compared with exact incoming f0cbf56 constants, closed DTO and version checks, input routing, writers and the already matching public row. The source correspondence report ran no tests and makes no deployed-reader or release claim. The full combined repository gate below is separate executable evidence.

Combined integration 37db6228da231e5d80a889d9fc9f344b5e2c5126 passed all ten declared lanes at 2026-09-06T00:13:18Z: fmt-check, clippy, test, doc-check, example-check, projection-check, release-check, action-check, site-build and planning. Rust executed1760 passing cases, zero failed and zero ignored across133 summaries. Site-lab compiled WASM, checked21 browser claims and28 steps over64 rows, then the pinned site build succeeded. Raw lane commands, exits and times are target/review-boundaries-6/integration/results.json and adjacent logs; the durable integration record is docs/plan/2026-09-06-review-boundaries-6.md. Existing npm dependency advisories and twelve planning prose-findings advisories remain visible in raw output. No release/tag or recovery-runtime completion is inferred.
