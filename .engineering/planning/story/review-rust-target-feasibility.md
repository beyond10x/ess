---
format: aep.planning-md/1
id: story:review-rust-target-feasibility
kind: story
status: draft
title: Check Rust target feasibility before claiming generated output
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-synth
revision: 2
---
## Finding and source

F07 (P1) from `docs/reviews/2026-09-05-architecture-review.md:306`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/generate/ess-synth/src/rust/name.rs:20`, `crates/generate/ess-synth/src/rust/layout.rs:170`, `crates/generate/ess-synth/src/plan.rs:540`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Valid models with colliding Rust symbols or recursive layouts yield compilable generated crates or explicit pre-write refusals.

## Implementation boundary

Allocate target symbols and output paths as one feasibility pass, checking normalized names, reserved words, field/wire-name collisions and recursive layout. Use indirection where it preserves the declared semantics; retain source-language freedom and report target limitations.

## Validation

Compile emitted adversarial valid models offline, including FooBar/Foo_Bar, optional self-recursion, mutual recursion, namespace and keyword cases; assert deterministic output/refusals and no partially successful write for rejected plans.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No new target and no general source-model restrictions to satisfy Rust; TypeScript root collisions have a separate owner.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/generate/ess-synth` — cited; owning implementation or documented surface.
- Confidence: high — cited; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

