# Review boundaries 21 — opening selection evidence

Captured 2026-09-09 from `aep artifact waves --format json` (aep protocol 0.54.0) at the store state after the three scope refreshes (revisions: primitive-semantics 8, typed-diagnostics 6, execution-recovery-implementation 5), before the opening commit. Rows are the verb's own; nothing is summarised.

## Placement of the selected units

```
wave 4	story:review-typed-diagnostics	inferred=true	entries=16
wave 11	story:review-execution-recovery-implementation	inferred=true	entries=34
wave 13	story:review-primitive-semantics	inferred=true	entries=21
```

## Collisions between selected units

```
{"a":"story:review-execution-recovery-implementation","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
```

## Collisions between a selected unit and an unselected draft

116 rows of 1536 in the store. None of the other side is selected.

```
{"a":"story:a-skipped-scenario-is-not-a-failed-one","b":"story:review-execution-recovery-implementation","path":"Cargo.lock","confidence":"cited"}
{"a":"story:a-skipped-scenario-is-not-a-failed-one","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:a-skipped-scenario-is-not-a-failed-one","b":"story:review-primitive-semantics","path":"crates/verify/ess-conformance","confidence":"cited"}
{"a":"story:authored-site-link-resolution","b":"story:review-execution-recovery-implementation","path":"Cargo.lock","confidence":"cited"}
{"a":"story:authored-site-link-resolution","b":"story:review-execution-recovery-implementation","path":"crates/edge/ess-cli/Cargo.toml","confidence":"cited"}
{"a":"story:authored-site-link-resolution","b":"story:review-execution-recovery-implementation","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:authored-site-link-resolution","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:canonical-build-ir-roundtrip","b":"story:review-execution-recovery-implementation","path":"Cargo.lock","confidence":"cited"}
{"a":"story:canonical-build-ir-roundtrip","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:canonical-build-ir-roundtrip","b":"story:review-primitive-semantics","path":"Cargo.toml","confidence":"inferred"}
{"a":"story:cli-first-level-is-the-four-areas","b":"story:review-execution-recovery-implementation","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:command-line-surface","b":"story:review-execution-recovery-implementation","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:command-line-surface","b":"story:review-primitive-semantics","path":"Cargo.toml","confidence":"inferred"}
{"a":"story:command-line-surface","b":"story:review-typed-diagnostics","path":"crates/specify/ess-domain/src/component.rs","confidence":"inferred"}
{"a":"story:component-declares-its-settings","b":"story:review-typed-diagnostics","path":"crates/specify/ess-domain/src/component.rs","confidence":"inferred"}
{"a":"story:component-release-check-toolchains","b":"story:review-execution-recovery-implementation","path":"Cargo.lock","confidence":"cited"}
{"a":"story:component-release-check-toolchains","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:component-release-check-toolchains","b":"story:review-primitive-semantics","path":"Cargo.toml","confidence":"inferred"}
{"a":"story:composite-action-shell","b":"story:review-execution-recovery-implementation","path":"Cargo.lock","confidence":"cited"}
{"a":"story:composite-action-shell","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:composite-action-shell","b":"story:review-primitive-semantics","path":"Cargo.toml","confidence":"inferred"}
{"a":"story:crates-under-area-directories","b":"story:review-primitive-semantics","path":"Cargo.toml","confidence":"inferred"}
{"a":"story:create-only-command-cannot-refuse","b":"story:review-primitive-semantics","path":"crates/generate/ess-gen","confidence":"inferred"}
{"a":"story:create-only-command-cannot-refuse","b":"story:review-primitive-semantics","path":"crates/generate/ess-synth","confidence":"inferred"}
{"a":"story:create-only-command-cannot-refuse","b":"story:review-primitive-semantics","path":"crates/verify/ess-conformance","confidence":"cited"}
{"a":"story:create-only-command-cannot-refuse","b":"story:review-typed-diagnostics","path":"crates/specify/ess-compiler","confidence":"cited"}
{"a":"story:early-stop-assertion","b":"story:review-primitive-semantics","path":"crates/verify/ess-conformance/src/go/runtime.go","confidence":"cited"}
{"a":"story:elapsed-time-claims","b":"story:review-primitive-semantics","path":"crates/verify/ess-conformance/src/go/runtime.go","confidence":"cited"}
{"a":"story:enum-variant-in-an-entity-invariant","b":"story:review-typed-diagnostics","path":"crates/specify/ess-domain/src/entity.rs","confidence":"inferred"}
{"a":"story:enum-variant-in-an-entity-invariant","b":"story:review-typed-diagnostics","path":"crates/specify/ess-domain/src/view.rs","confidence":"inferred"}
{"a":"story:fuzz-the-specification-surface","b":"story:review-primitive-semantics","path":"crates/generate/ess-synth","confidence":"cited"}
{"a":"story:helm-defaults-satisfy-schema","b":"story:review-primitive-semantics","path":"Cargo.toml","confidence":"inferred"}
{"a":"story:helm-secret-slot-defaults","b":"story:review-execution-recovery-implementation","path":"Cargo.lock","confidence":"cited"}
{"a":"story:helm-secret-slot-defaults","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:helm-secret-slot-defaults","b":"story:review-primitive-semantics","path":"Cargo.toml","confidence":"inferred"}
{"a":"story:integrate-source-driven-realizations","b":"story:review-execution-recovery-implementation","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:java-conformance-target","b":"story:review-primitive-semantics","path":"crates/verify/ess-conformance","confidence":"cited"}
{"a":"story:model-binary64-fields","b":"story:review-execution-recovery-implementation","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:model-binary64-fields","b":"story:review-primitive-semantics","path":"crates/generate/ess-gen/src/types.rs","confidence":"cited"}
{"a":"story:model-binary64-fields","b":"story:review-primitive-semantics","path":"crates/generate/ess-synth/src/go/http.rs","confidence":"cited"}
{"a":"story:model-binary64-fields","b":"story:review-primitive-semantics","path":"crates/generate/ess-synth/src/go/layout.rs","confidence":"cited"}
{"a":"story:model-binary64-fields","b":"story:review-primitive-semantics","path":"crates/generate/ess-synth/src/rust/mod.rs","confidence":"cited"}
{"a":"story:model-binary64-fields","b":"story:review-primitive-semantics","path":"crates/generate/ess-synth/src/rust/wire.rs","confidence":"cited"}
{"a":"story:model-binary64-fields","b":"story:review-primitive-semantics","path":"crates/generate/schema-contract/src/realize/normalize/numeric.rs","confidence":"inferred"}
{"a":"story:model-binary64-fields","b":"story:review-primitive-semantics","path":"crates/specify/ess-domain/src/expression.rs","confidence":"cited"}
{"a":"story:model-binary64-fields","b":"story:review-primitive-semantics","path":"crates/specify/ess-domain/src/primitive_admission.rs","confidence":"inferred"}
{"a":"story:model-binary64-fields","b":"story:review-primitive-semantics","path":"crates/verify/ess-conformance/src/input.rs","confidence":"cited"}
{"a":"story:model-binary64-fields","b":"story:review-primitive-semantics","path":"crates/verify/ess-conformance/src/witness.rs","confidence":"cited"}
{"a":"story:model-binary64-fields","b":"story:review-typed-diagnostics","path":"crates/specify/ess-domain/src/spec.rs","confidence":"inferred"}
{"a":"story:model-binary64-fields","b":"story:review-typed-diagnostics","path":"crates/specify/ess-domain/src/system.rs","confidence":"inferred"}
{"a":"story:model-binary64-fields","b":"story:review-typed-diagnostics","path":"crates/specify/ess-domain/src/types.rs","confidence":"inferred"}
{"a":"story:model-binary64-fields","b":"story:review-typed-diagnostics","path":"website/docs/guides/write-a-specification.md","confidence":"cited"}
{"a":"story:normalize-model-owned-records","b":"story:review-execution-recovery-implementation","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:oci-component-release","b":"story:review-execution-recovery-implementation","path":"Cargo.lock","confidence":"cited"}
{"a":"story:oci-component-release","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:oci-component-release","b":"story:review-primitive-semantics","path":"Cargo.toml","confidence":"inferred"}
{"a":"story:raw-json-normalization-provenance","b":"story:review-execution-recovery-implementation","path":"Cargo.lock","confidence":"cited"}
{"a":"story:raw-json-normalization-provenance","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:review-authored-discovery","b":"story:review-execution-recovery-implementation","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:review-authored-discovery","b":"story:review-typed-diagnostics","path":"website/docs/guides/write-a-specification.md","confidence":"cited"}
{"a":"story:review-browser-replay-fidelity","b":"story:review-execution-recovery-implementation","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:review-cache-origin","b":"story:review-execution-recovery-implementation","path":"website/docs/concepts/component-delivery.md","confidence":"cited"}
{"a":"story:review-composition-contract","b":"story:review-execution-recovery-implementation","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:review-conformance-coverage","b":"story:review-primitive-semantics","path":"crates/verify/ess-conformance","confidence":"cited"}
{"a":"story:review-consumer-coverage","b":"story:review-execution-recovery-implementation","path":"Cargo.lock","confidence":"cited"}
{"a":"story:review-consumer-coverage","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:review-delivery-trust-contract","b":"story:review-execution-recovery-implementation","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:review-delivery-trust-contract","b":"story:review-execution-recovery-implementation","path":"crates/edge/ess-cli/tests/command_surface.rs","confidence":"cited"}
{"a":"story:review-delivery-trust-contract","b":"story:review-execution-recovery-implementation","path":"website/docs/concepts/component-delivery.md","confidence":"cited"}
{"a":"story:review-delivery-trust-contract","b":"story:review-execution-recovery-implementation","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:review-execution-recovery-design","b":"story:review-execution-recovery-implementation","path":"docs/design/review-execution-recovery.md","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-infra-ir-invariants","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-kubectl-diagnostic-sanitization","path":"crates/infra/ess-kubernetes/src/lib.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-openapi-semantic-accounting","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-openapi-semantic-accounting","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-output-containment","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-output-ownership","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-output-ownership","path":"website/docs/reference/cli.md","confidence":"inferred"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-persisted-delivery-validation","path":"crates/edge/ess-cli/tests/persisted_delivery.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-persisted-delivery-validation","path":"crates/edge/ess-cli/tests/support/fake_delivery.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-public-support-claims","path":"crates/edge/ess-xtask/src/support.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-public-support-claims","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-public-support-claims","path":"website/docs/status/where-this-stands.md","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:review-rust-target-feasibility","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:scenarios-directory-compiles-nothing","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:schema-bundle-import","path":"Cargo.lock","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:schema-bundle-import","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:schema-bundle-import","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:schema-document-root-import","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:schema-document-root-import","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:schema-unique-items-signed-zero","path":"Cargo.lock","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:source-pinned-data-normalization","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:source-pinned-data-normalization","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:types-only-realizations","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:review-execution-recovery-implementation","b":"story:types-only-realizations","path":"crates/edge/ess-cli/src/main.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:types-only-realizations","path":"crates/edge/ess-cli/tests/command_surface.rs","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:types-only-realizations","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:review-execution-recovery-implementation","b":"story:typescript-normalization-target","path":"website/docs/reference/cli.md","confidence":"cited"}
{"a":"story:review-expression-typechecking","b":"story:review-primitive-semantics","path":"crates/verify/ess-conformance","confidence":"cited"}
{"a":"story:review-expression-typechecking","b":"story:review-typed-diagnostics","path":"crates/specify/ess-compiler","confidence":"cited"}
{"a":"story:review-glossary-boundaries","b":"story:review-typed-diagnostics","path":"website/docs/guides/write-a-specification.md","confidence":"cited"}
{"a":"story:review-openapi-semantic-accounting","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:review-output-containment","b":"story:review-primitive-semantics","path":"crates/generate/ess-gen","confidence":"cited"}
{"a":"story:review-output-ownership","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:review-primitive-semantics","b":"story:review-report-reader-validation","path":"crates/verify/ess-conformance","confidence":"cited"}
{"a":"story:review-primitive-semantics","b":"story:review-rust-target-feasibility","path":"crates/generate/ess-synth","confidence":"cited"}
{"a":"story:review-primitive-semantics","b":"story:review-semantic-diff-coverage","path":"crates/generate/ess-gen","confidence":"cited"}
{"a":"story:review-primitive-semantics","b":"story:review-semantic-diff-coverage","path":"crates/generate/ess-synth","confidence":"cited"}
{"a":"story:review-primitive-semantics","b":"story:schema-bundle-import","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:review-primitive-semantics","b":"story:schema-unique-items-signed-zero","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:review-primitive-semantics","b":"story:schema-unique-items-signed-zero","path":"Cargo.toml","confidence":"inferred"}
{"a":"story:review-primitive-semantics","b":"story:source-pinned-data-normalization","path":"crates/generate/schema-contract/src/realize/normalize/numeric.rs","confidence":"inferred"}
{"a":"story:review-primitive-semantics","b":"story:the-generated-go-runtime-is-gofmt-clean","path":"crates/verify/ess-conformance","confidence":"cited"}
{"a":"story:review-primitive-semantics","b":"story:types-only-realizations","path":"Cargo.lock","confidence":"inferred"}
{"a":"story:review-semantic-diff-coverage","b":"story:review-typed-diagnostics","path":"crates/specify/ess-compiler","confidence":"cited"}
```

## Unassessed

```
["story:authored-scenarios","story:own-planning-store","story:relations-design-page","story:relations-in-the-billing-example","story:relations-in-the-domain-model","story:relations-projected","story:unique-wire-field-identity"]
```

## Cycles

```
[]
```
