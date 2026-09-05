---
format: aep.planning-md/1
id: story:review-persisted-delivery-validation
kind: story
status: active
title: Restore delivery IR invariants at persisted read boundaries
tags:
- P0
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/generate/ess-deployment
revision: 6
---
## Finding and source

F02 (P0) from `docs/reviews/2026-09-05-architecture-review.md:188`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/generate/ess-deployment/src/build.rs:267`, `crates/generate/ess-deployment/src/runtime.rs:230`, `crates/generate/ess-deployment/src/component.rs:70`, `crates/generate/ess-deployment/src/environment.rs:130`, `crates/edge/ess-cli/src/main.rs:1613`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Malformed persisted delivery documents are refused before analysis or external execution while valid compiler-produced documents retain their canonical bytes.

## Implementation boundary

Inventory BuildIr, RuntimeIr, component descriptors, stack locks, deployment plans and nested bundle models. Validate format discriminators, map-key identities, references, cycles, order membership/completeness and domain constraints at DTO-to-validated conversion. Test supported deserialization routes, not only from_json. Route the CLI through the validated entrypoint; prevalidate the complete plan before the first executor call.

## Validation

Mutate each inventoried envelope and nested model, including future/99 plus missing order entries, duplicate/order/cycle/reference defects and a consistently rehashed invalid bundle. Use an injected fake executor to prove zero calls on rejection; valid fixtures round-trip byte-for-byte. Package gates: ess-deployment and ess-cli.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Release evidence authenticity, cache origin and recovery belong to F11 stories; no live Docker/Helm/registry operations are required.

## Scope

Derived 2026-09-05 by `story-scoper` from the story and inspected source — cited.

- **Primary surface:** `crates/generate/ess-deployment` — cited; owns all persisted delivery models, their Serde implementations, compiler checks, bundle verification and projections.
- **Owning files and symbols:** `src/build.rs` (`BuildIr`, `compile_build`), `src/runtime.rs` (`RuntimeIr`, `compile_runtime`), `src/component.rs` (`ComponentIr`, `ReleaseBundle`, `verify_release_bundle`), `src/release.rs` (`ReleaseManifest`, `verify_release`), `src/stack.rs` (`ReleaseCatalog`, `StackLock`, `resolve_stack`), `src/environment.rs` (`DeploymentIr`, `compile_deployment`) within the primary surface — cited.
- **Shared validation support:** `src/identity.rs`, `src/diagnostic.rs`, `src/lib.rs` within the primary surface — inferred; reusable checked decoding or public validation APIs may require these existing package-local surfaces.
- **CLI surface:** `crates/edge/ess-cli` — cited; generic JSON/YAML `read_document`, release-bundle transport, projection readers, deployment diff and reconcile consume the persisted models.
- **Executor boundary:** `src/main.rs:1614`, `src/main.rs:1682`, `src/main.rs:2797` within the CLI surface — cited; desired/current plans are read before reconcile, but chart checks currently happen per release during execution.
- **Tests:** `crates/generate/ess-deployment/tests/deployment.rs` — cited; existing compiler-produced fixtures and round-trip/bundle tests provide the package-local regression base.
- **Additional tests:** package-local reader-route mutations and a CLI fake-executor regression — inferred; no executor abstraction currently exists, so its precise fixture location is an implementation choice within the two package tokens.
- **Documents:** none established — cited; the story requires restoring existing invariant ownership without introducing a new persisted contract.
- **Confidence:** high — cited; every owning model and production CLI reader was found in these two packages.
- **Would collide with:** changes within `crates/generate/ess-deployment` or `crates/edge/ess-cli` — inferred; these directory tokens cover implementation, projections and package-local tests.

## Independent scoping constraints

Read-only scoper findings on 2026-09-05:

- All compiled models derive Deserialize; convenience JSON/YAML wrappers and CLI read_document use Serde directly. Validate all public routes, including nested models and duplicate JSON map keys before BTreeMap loses them. Authored DTOs normally pass through compilers; keep their parsing contract distinct from returning invariant-bearing IR.
- Reuse existing byte-local compiler rules for exact format, map identity, references, dependency cycles/orders, output kinds, declared secrets, path/argv/mount rules, process/container/workload/endpoint relationships, replica positivity, environment-variable/name uniqueness, volume/probe checks, release evidence attachments and source-commit syntax. Do not invent full Kubernetes/URL/quantity validation that compilation never established.
- StackLock systems map keys identify composition-local services; LockedSystem.system is an ESS system identity and need not equal the key (stack.rs:158/202/430). External-system keys do identify their system. Preserve these distinct meanings.
- verify_release_bundle at component.rs:326 discards releases map keys via into_values before rebuilding. Validate original keys first. Consistently rehashed invalid nested IR must still fail. Included bundle bytes suffice for runtime-to-build digest and process-image output checks; hashes alone establish no graph validity.
- Desired and current deployment plans both require validation before affected/removed analysis. Chart checks currently occur inside reconcile_release, after the loop can already apply an earlier release. The first possible external call is ORAS (:2866), followed by Helm (:2832); test zero calls to both with a valid first/invalid later release and invalid current removal state. No executor abstraction exists; a fake process fixture or narrow injected runner is within CLI scope.
- Standalone bytes cannot prove original ESS semantic digest, realization digest, semantic component existence/completeness, semantic replica bounds or statefulness; those require omitted compiler inputs. Omitted manifests/stack requirements likewise prevent re-proving full lock selection or required deployment bindings. Do not turn digest presence into those claims. Authentication, registry/cache origin and conformance truth remain F11.

Coordinator inference for the future brief: preserve compiler-produced canonical bytes and refuse invalid input without normalizing it. Where a persisted IR explicitly owns a deterministic canonical order, compare against its compiler algorithm; otherwise decide whether all valid topological orders are admitted before coding. Several models expose mutable public fields; establish whether entrypoint validation or controlled mutation is needed to prevent corrupted plans reaching execution, without treating read validation as a universal in-memory invariant proof. No implementation decision has yet been tested.
