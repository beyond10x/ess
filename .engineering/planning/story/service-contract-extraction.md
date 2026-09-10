---
format: aep.planning-md/1
id: story:service-contract-extraction
kind: story
status: active
title: Extract reusable service contract resolution from the SDK into ESS
relations:
- informed_by: initiative:ess-evolution
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: crates/specify/ess-service-contract
- confidence: cited
  path: docs/design/service-contract.md
revision: 6
---
## Outcome

Implement the existing ESS evolution dependency-policy contract for ess-service-contract. Selected ESS component surfaces retain complete compiler-owned commands/outcomes, events/errors, owned entities and views. Reuse existing EssIr types and handles; no parallel model of predicates, state, payloads or identity. Service SDK consumes extracted synthesis-disposition resolution through its existing public entrypoints, retaining service-runtime-ir/3 bytes and SDK-owned authentication, authorization, content, execution and hosting policy.

## Design and scope

Binding design: docs/design/service-contract.md, under docs/design/ess-evolution/migration.md section 3. The source extraction starts from Service SDK c70c954da43c063143b34601ef8af7a1c511e5aa. Add crates/specify/ess-service-contract and wire workspace dependencies; semantic selection is an in-memory borrowed view of existing compiler entities, not a new persisted entity format. Optional synthesis integration owns the former SDK RequiredDisposition and exact required-capability lookup. Keep the existing runtime format and re-export the type from its former SDK path. Provider publication precedes consumer pin changes.

## Verification

Independent synthetic component fixtures distinguish accepted commands from unrelated ones, preserve per-outcome zero/multiple-event meaning, entity ownership and full query shape, and retain external outcomes as unresolved binding work. The synthesis boundary refuses missing, duplicate and explicitly refused capabilities and retains obligation reason/contract unchanged. Use SDK runtime compilation tests and exact existing runtime fixture comparisons to verify extraction without rewriting authorization or execution. Run affected package tests, strict Clippy and compiler compatibility only. No full or expensive remote persistence gate is authorized. ER execution and complete application adoption remain separate unfinished requirements.

## Provider implementation and observed evidence

The default library selects one component and borrows the original compiler model. Commands retain all outcomes and per-branch event order; events/errors are stable deduplicated surface inventories; entities and views follow declared domain ownership. Full view parameters, predicates, ordering and consistency remain in their existing resolved types. External outcome causes remain explicit. No SDK, transport, storage or runtime policy dependency was added. Optional synthesis integration extracts RequiredDisposition, exact capability cardinality/refusal handling and command/view field summaries from the SDK.

The synthetic fixture validates through ess specify validate (sample v1, three files) and compiles to retained IR. It distinguishes selected and unrelated commands/state, multiple emitted events, zero-event refusals, external conditions and full query shape. The first synthetic draft attempted accepted empty outcomes; ESS correctly refused them as empty_change and they were replaced with explicit outcomes. No acceptance rule was weakened to make the fixture compile.

Two focused integration tests pass, with multiple independent assertions for component isolation, per-outcome semantics, vocabulary, valid wire values and missing/duplicate/refused capabilities. Replacing accepted-command selection with all model commands caused the expected extra sample.other.Hidden command and failed the selection assertion; restoration passed. The extracted serde unit disposition initially swallowed extra fields despite deny_unknown_fields. A strict empty-struct wire helper now refuses them while preserving the original public variant and valid JSON bytes. The SDK before-extraction runtime fixture was captured independently against c70c954da43c063143b34601ef8af7a1c511e5aa for the consumer comparison.

Strict all-feature/all-target Clippy, Rustdoc for both default and synthesis features, and Rust 1.85 all-feature/all-target compilation passed. The compiler retains two existing const-helper warnings under Rust 1.85. Evidence: local-evidence:ess-evolution-20260910/ess-service-contract-final.log; ess-service-contract-selection-mutation.log; ess-service-contract-clippy.log; ess-service-contract-rustdoc.log; ess-service-contract-default-rustdoc.log; ess-service-contract-msrv.log; ess-service-contract-fixture-ir.json. No full or remote persistence gate ran. Publish this provider before the SDK switches to its exact revision; SDK compatibility and main integration remain pending.

## Consumer publication

Service SDK 03026ec181b6d343672a3863dc1c9874398c6105 is published on refactor/ess-service-contract and consumes exact ESS provider d453c50769f08d1c348daad3c2e8b721be571078. It removes the local disposition type/resolver and field-summary implementations, preserves the SDK public type path and error mapping, and keeps all runtime policy in the SDK. Its six targeted runtime tests pass, including exact equality with 8,951 pre-extraction canonical bytes and both original standalone/factory runtime documents. Strict affected Clippy and all workspace target compilation passed, including generated host crates. The lockfile changes only the ESS packages and new dependency. Evidence: local-evidence:ess-evolution-20260910/sdk-service-contract-tests.log; sdk-service-contract-host-fixtures.log; sdk-service-contract-clippy.log; sdk-service-contract-workspace-build.log.

The provider and consumer extraction is implemented and published on feature branches. Main integration, catalog observation of the added component edges, ER lowering/service-engine delegation and real consumer adoption remain unfinished. No release, deployment or full gate is claimed.
