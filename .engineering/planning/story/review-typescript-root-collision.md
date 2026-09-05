---
format: aep.planning-md/1
id: story:review-typescript-root-collision
kind: story
status: active
title: Allocate TypeScript roots and definitions in one symbol space
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/schema-contract
- confidence: cited
  path: crates/generate/schema-contract/Cargo.toml
- confidence: cited
  path: crates/generate/schema-contract/src/typescript.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/registry.ts
- confidence: cited
  path: crates/generate/schema-contract/tests/typescript_typecheck.rs
revision: 9
---
## Finding and source

F07 (P1) from `docs/reviews/2026-09-05-architecture-review.md:306`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/generate/schema-contract/src/typescript.rs:66`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A requested TypeScript root that collides with a referenced definition produces valid uniquely named output or a deterministic refusal.

## Implementation boundary

Include caller root names, definition names and reserved names in one target allocation/feasibility check without changing JSON Schema authority or wire-property names.

## Validation

Add colliding root/definition and normalized-name vectors plus a noncollision control; typecheck generated output using the repository's pinned/local TypeScript toolchain in the target gate.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No schema $id namespace decision or Rust synthesis change is required.

## Scope

Derived 2026-09-05 by `story-scoper`. Every line is cited or inferred.

- **Primary surface:** `crates/generate/schema-contract` — cited; TypeScript projection and tests.
- **Implementation and existing tests:** `crates/generate/schema-contract/src/typescript.rs` — cited; project:65 checks the root separately from definition_names:262, then emits it at :91.
- **Symbols:** `ProjectionError`, `project`, `definition_names`, `reference_type`, `typescript_identifier`, `pascal_case` and existing tests — cited.
- **Allocation mechanism:** reserve the requested root alongside normalized definitions or deterministically refuse collision before rendering — inferred; the story permits refusal while preserving noncolliding bytes.
- **Reserved binding mechanism:** account for generated Array references, language keywords and forbidden type-alias names using compiler-backed vectors — inferred; render_kind:170 emits Array and type-name validation currently checks spelling only.
- **Wire boundary:** render_object:213 uses typescript_identifier for property quoting — cited; binding feasibility must be separate to preserve wire-property spelling and bytes.
- **Compiler tests:** `crates/generate/schema-contract/tests/typescript_typecheck.rs` — inferred; a new Rust integration lane can keep generated inputs/configuration under its own target.
- **Test feature:** `crates/generate/schema-contract/Cargo.toml` — inferred; an explicit feature may keep external compiler requirements out of the Rust-only default gate while making the selected target lane mandatory.
- **Local tool evidence:** coordinator website TypeScript and /usr/lib/node_modules/typescript both report 6.0.3, matching website/package-lock.json; node and tsc exist — cited; no toolchain file edit is in this unit scope.
- **Confidence:** high — cited; allocator, renderer, tests and refusal-propagating CLI caller were inspected.
- **Would collide with:** this package's TypeScript code, tests and manifest — inferred; any shared gate provisioning change must be reserved separately before dispatch.

## Scoping decisions for dispatch

The scoper found no dependency and no nested AGENTS. CLI schema.rs:140 propagates projection refusal before output, so no CLI edit is established. Keep noncolliding projection bytes, structural vocabulary, insertion-order determinism and reference resolution controls.

The current default CI gate provisions Rust and Go, without explicit TypeScript installation. Coordinator inference: use an explicitly selected compiler test lane that fails when its required compiler is missing, then record and execute that lane with the locally available pinned 6.0.3 compiler. Do not silently skip a selected compiler test or claim default task check runs it. A separate feature is allowed within manifest scope if needed; a shared Taskfile/CI change must be proposed as a coordinator patch before expanding scope. Generated compiler configuration should use noEmit, strict, types: [] and explicit fixture inputs.

The installed compiler source rejects alias names any, unknown, never, number, bigint, boolean, string, symbol, void, object and undefined; this is evidence for vectors, not an exhaustive keyword policy. Also protect emitted Array references. Exact discovery configuration and complete feasibility rules must be measured before implementation. The scoper ran no compiler probes or builds.
