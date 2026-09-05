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
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/generate/ess-synth
- confidence: inferred
  path: docs/design/review-rust-target-feasibility.md
revision: 6
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

Derived 2026-09-05 by independent story-scoper at e0ea44383f00b05917bc24c828d3120d86afb3de. Relevant synth/CLI implementation sources remain unchanged through 0d267e25739ca495ad1a229393181ad1b75182f3. Every entry is cited or inferred.

- **Primary surfaces:** cited — crates/generate/ess-synth and crates/edge/ess-cli (including src/main.rs). Inferred new binding design docs/design/review-rust-target-feasibility.md. Current story has no dependency supplying additional scope.
- **Target/neutral boundary:** cited — synth lib.rs:117/129/166/271 defines Synthesis, TargetReport, TargetRefusal and synthesize_for; Rust currently returns target=None. Go/Web carry separate target reports. plan.rs:540 marks domain types generated under the neutral contract; Rust restrictions must not change domain validity or PLAN.md/plan.json.
- **Public emitter:** cited — rust/mod.rs:95 exposes workspace -> Vec<Artifact>; only synthesize_for calls it inside this repo, but it is public. Checking only the facade leaves it unchecked. Current workspace coverage assertions and lib.rs:332 duplicate-artifact assertion are not explicit target refusals.
- **Checked seam:** inferred — settle a checked workspace result using existing TargetReport before rendering/inserting code artifacts, including direct public entrypoint behavior. External callers are not yet inventoried.
- **Naming/layout:** cited — rust/name.rs owns Pascal/snake/fragments/keyword escape; layout.rs owns packages/modules/paths/crate identifiers/reference/type rendering. Final fallback and suffix repair can merge names; validate final names in actual Rust scopes instead of a global helper blacklist.
- **Declarations:** cited — items.rs emits tuple-newtypes, structs, enums, unions, commands plus Outcome types, events/errors/views; no Rust alias renderer exists. entity.rs expands Data, Snapshot, Any<Entity>, states/modules/Marker/sealed, fixed new/state/data/into_data/refine/snapshot methods and normalized transition methods.
- **Events and ports:** cited — items.rs:204 numbers repeated event-field bases, mod.rs:161 falls back to full names for variants; port.rs emits component types, PublishedEvent, handlers/queries/new/drain_outbox. Final numbering/full-name collisions and fixed outcome error fields need coverage.
- **System and obligations:** cited — system.rs emits System, SystemEvent, BindingInvocation, traits/functions/generics/component fields beside fixed obligations/invocations/published/cursor/retries. obligation.rs derives conversion/behavior/query traits plus UnmetObligation, Unimplemented and obligations modules.
- **Helpers and paths:** cited — layout emits bare String/Option/Vec/primitives and crate::primitives; codecs/obligations use Result and core/std. http.rs fixed lib/http/json/wire modules coexist with normalized component modules; wire.rs:54 derives global codec names. Inferred checks cover post-repair collisions, keyword module filenames, lib/helper paths, package-to-crate normalization and duplicate output paths. Check rustc filename behavior for raw identifiers, not merely filesystem acceptance.
- **Wire identity:** cited — wire uses existing ess-gen schema::wire_field_name and union_content_key. Inferred policy: check normalized Rust members separately from wire overrides and adjacent-tag keys; reuse shared helpers without renaming authority or editing ess-gen. An ambiguous wire shape requires target refusal even if Rust compiles.
- **Recursive layout:** cited — Optional emits Option, List emits Vec, Map emits BTreeMap; newtypes/structs/union payloads retain by-value references. Inferred graph covers all generated representation dependencies: Optional preserves size edges, List/Map break them. Detect self/mutual size cycles, keep legal collection recursion and acyclic controls.
- **Indirection choice:** inferred — refusal is the smallest compatible answer for infeasible representations. Boxing requires explicit treatment of generated signatures/construction/conversions/codecs and must not silently change valid APIs.
- **CLI:** cited — main.rs:2384 writes artifacts before target inspection, :2386 prints only neutral counts, then returns success. Inferred change: visible source/cause refusal and failure before writes for Rust; metadata-only TARGET artifacts do not prove this. Decide Go/Web/Clap behavior separately, preserve output containment.
- **Shared consumers:** cited — Web uses Rust Layout/name/event/wire/JSON helpers; Clap is a sibling target. Preserve neutral-plan parity and admitted cross-target output; Rust helper changes are not isolated from Web.
- **Existing package tests:** cited — synthesis.rs source fixtures/determinism/event/package/module cases, go.rs target=None valid billing assertion, http.rs/web.rs cross-target contracts, relations.rs committed Rust bytes.
- **Actual gate:** cited — current ess-xtask has only Generate/Schema/Release. Test commentary naming cargo xtask synth is stale. Workspace gates compile committed billing/gatepass generated code via realization path dependencies, not fresh adversarial emission.
- **Offline compiler lane:** inferred — package-local integration cases compile fresh isolated generated workspaces under the unit target. Generate lockfiles offline then cargo check --locked --offline --workspace --all-targets with fixture manifest and each fixture's own target, never shared CARGO_TARGET_DIR. Missing tooling fails, never skips. No root tooling edit established.
- **Compiler matrix:** inferred — fresh red FooBar/Foo_Bar and optional/mutual recursion, normalized members/helpers/synthesized names/path repairs/keywords/wire collisions. Positive admitted collection recursion, separate namespaces and old valid fixtures must compile. Compare historical generated bytes and neutral plans, not only new self-round-trips.
- **CLI matrix:** inferred — text/JSON/YAML and no-output invocation communicate refusal, fail, create no output tree and preserve an existing sentinel tree; keep valid controls.
- **Binding design:** inferred — choose feasibility/public API ownership, symbol/path scopes, cycle semantics, refusal propagation, whole-workspace versus partial emission, CLI write/exit policy and successful-byte compatibility before code. Reuse Capability/TargetReport/TargetRefusal; no new persisted version established as necessary.
- **Compatibility:** inferred — preserve target=None and no TARGET artifacts for prior admitted Rust models. New valid API changes or external-reader effects require the coordinated-migration obligation; external synthesis-library consumers were not inspected by this pass.
- **Exclusions:** inferred — no domain/compiler restriction, TypeScript, ess-gen/realization, root manifest/Taskfile/workflow edit established. Evidence-only references are not write scope.
- **Confidence:** high for actual ownership/public seams/CLI ordering/tooling/shared consumers; allocation/refusal choices remain binding-design decisions.
- **Collisions:** cited — any ess-synth or ess-cli unit, specifically the InfraIr query migration in main.rs:2807. Distant hunks do not make common tokens disjoint.

No files, scratch, tests, stores, Git state or lifecycle were changed and no builds ran. Fresh compiler outcomes, historical byte equality and external consumer compatibility remain unexecuted.

## Pre-dispatch inventory

Read-only coordinator preparation at wave 3 opening 45832cc885377b2d61845ee33af14f0293d99e67, not independent scoping or executed evidence.

- The existing public seam is synthesize_for -> Synthesis { plan, artifacts, target }. SynthesisPlan is language-neutral; all target constraints belong to a TargetReport. Existing Go/Web refusals are useful patterns. Do not put Rust-only restrictions into domain validity or change the shared plan to accommodate Rust.
- rust/name.rs normalizes Pascal and snake names; path keywords get a suffix, so self and self_ can collide. rust/layout.rs reserves module/package names and allocates some collisions, but final post-normalization uniqueness still needs checking. Include type aliases, entities, synthesized state types, command/outcome/event/error/view names, methods/fields/variants, helper imports (Option, Vec, String, Result, primitive module), module names and package/file paths.
- Review E0428 and E0072 are attributed reproduction evidence; run the real generated programs for fresh red-first evidence. Struct/alias/enum cycles through Optional preserve by-value size; List/Map introduce indirection and need positive controls. Refusal is permitted when representation cannot be preserved; do not silently rename source/wire properties or reject the source language.
- The CLI synthesize function at crates/edge/ess-cli/src/main.rs:2374 writes all synthesis artifacts then prints only language-neutral plan counts, and returns success. A new target refusal must be visible in the actual CLI workflow, not just a hidden field. Determine whether the existing report artifact suffices for the acceptance or whether the CLI requires a scoped change; do not invent a no-write proof from generator-only tests.
- The package's current tests mainly inspect emitted source. The existing generated-crate compile gate lives in the repository tooling. Add a real offline Rust compiler-backed lane with isolated generated fixtures, exact selected cases and no hidden missing-tool skips; reuse cached package dependencies and separate fixture targets within the owning unit. No shared CARGO_TARGET_DIR.
- Preserve valid prior generated bytes when possible. If choosing a change to previously valid generated APIs or persisted vocabulary, stop for the existing coordinated-migration obligation instead of treating it as an incidental collision repair. Refusing an infeasible target can keep the successful-byte compatibility surface narrow.
- A binding target-feasibility design and final edit scope require independent story-scoper confirmation before dispatch. Current CLI and new design paths remain inferred rather than reserved until that confirmation.
