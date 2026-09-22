---
format: aep.planning-md/1
id: story:pre-execution-fixture-values
kind: story
status: active
title: Resolve typed fixture values before target execution
relations:
- informed_by: design:pre-execution-fixture-values
- decomposes: epic:independently-provisioned-conformance
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/reviewed-schema-metadata.json
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/generate/ess-cli-project/tests/consumer_cli_model_ingress.rs
- confidence: inferred
  path: crates/specify/ess-compiler
- confidence: inferred
  path: crates/specify/ess-domain
- confidence: inferred
  path: crates/verify/ess-conformance
- confidence: cited
  path: docs/design/cli-schema-metadata-accounting.md
- confidence: inferred
  path: docs/design/pre-execution-fixture-values.md
- confidence: cited
  path: docs/design/review-typed-diagnostics.md
- confidence: cited
  path: models/toolchain/domains/specify.yaml
- confidence: inferred
  path: schemas/generated/ess.schema.json
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 17
---
## Goal

Resolve declared, typed fixture values before scenario execution so tests against real implementations use independently provisioned identities and inputs. Expected values remain independent of observed post-state.

## Acceptance

- Implement the source, suite and runtime contract in docs/design/pre-execution-fixture-values.md.
- Preserve every existing fixture-free canonical suite and passing runtime test.
- Demonstrate failing-first behavior for an independently provisioned identity and a wrong observed value.
- Rust, emitted Go and emitted TypeScript agree on typed resolution, missing capabilities, invalid data, frozen values and scenario isolation.
- Old readers refuse the new specification/authored/suite formats before target activity. Regenerate the published schema if RawSpecFile changes.
- Default and CI-profile validation results are recorded separately; an existing consumer-accounting refusal is not reported as a passing default gate.

## Scope

Source admission, typed compiler metadata and schema projection; conformance authored/synthesized inputs, format admission, runtime fixture resolution and dynamic event equality; native runtime tests and source-owned design documentation. Includes the source-derived toolchain format enum and exactly three reviewed root-definitions metadata hashes. No consumer classification, profile, guard or baseline exception changes. No stimulus builder, integration credential, consumer identifier or application backend change belongs here.

## Authorization and delivery

Interactive implementation and local verification are complete for this checkpoint. On 2026-09-22 the operator explicitly authorized committing the verified candidate and opening a draft pull request. Publish it for review; no merge, release or downstream pin integration is authorized. The full CI-profile gate passes, while default consumer accounting remains refused as recorded below. The story remains active.

## Verification checkpoint — 2026-09-22

The isolated candidate is uncommitted on 16aa8c76, still equal to origin/main at the last fetch. Source ess/7, authored ess-scenario/3 and suite majors 12/13 carry the new meaning; no release version or downstream pin has changed.

The complete `task check SKIP_CONSUMER_CHECKS=true` exits zero. This includes formatting, strict Clippy, every workspace/tooling test, rustdoc, examples, projections, support, fuzz replay, release consistency and workflow checks. Node declarations were supplied for actual TypeScript package typechecking. The 42 publication/recovery tests and 30 browser-replay tests pass, as do all 147 tooling unit tests and the toolchain model's format-enum check. The public site build passed separately. Earlier retries encountered temporary-filesystem quota and Git-marker placement refusals; the successful run used a verified temporary directory under /var/tmp without weakening either check.

Thirteen focused fixture tests pass in the full gate. They execute emitted Go and TypeScript packages for ordinary suites and inventory-bearing suites with exact parent lineage: valid values pass, wrong outputs fail, invalid/missing/unknown provider data stops before session startup, and unsupported capability skips explicitly. The event assertion selects the first matching direct occurrence before comparing all literal, fixture and shape claims; a failing-first regression prevents a later correct event concealing the wrong first value. The TypeScript runtime's 206 cases and its external FixtureContract import/typecheck pass. Fixture-free canonical compatibility passes with the existing workspace tests.

Released 0.28.0, verified at binary SHA256 3c003e662ce6c2b28af16ada748b2c707abf546d12c05516294217c582afe09c, refuses the new source and authored markers. Its Rust, emitted Go and emitted TypeScript readers refuse both suite majors before target activity.

The schema was regenerated. Exactly three RootDefinitionsContainer shape pins were refreshed using the existing Rust wire extractor, with the bounded ess/7 review in docs/design/cli-schema-metadata-accounting.md. No profile, relationship, metadata guard, classification or baseline exception changed. Concurrent evolution worktrees touch the same whole-schema hash lines: recompute them from the combined generated schema at integration. Those worktrees were inspected read-only and remain untouched. The new toolchain model enum entry follows SUPPORTED_FORMATS.

Default consumer accounting was measured separately in its prescribed native profile and default directory. After the schema metadata refresh it still refuses the unclassified concrete entry ess_cli::bin(ess)::enum::SuiteTarget/variant/Typescript and the unknown claimed model wire:RawSpecFile#/definitions/NamedType/oneOf/2/properties/variants/items/type. Clean main independently refuses those same first blockers. This does not qualify the new fixture obligations or establish that all possible later refusals predate the change. The default gate is not green, and no release-ready or publication claim is made. Full local logs remain operator-retained under their evidence references. The story stays active until review and delivery.
