---
format: aep.planning-md/3
id: story:outcome-shapes-beyond-ess-14
kind: story
status: implemented
title: Create into a state, delete a subject, answer an unknown id, accept with no subject, seed the explorer
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-20260927
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/generate/ess-gen/src/unknown_instance.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/explore.ts
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: schemas/generated/ess.schema.json
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T14:48:52Z", actor: "human:timo", revision: 6, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T14:49:26Z", actor: "human:timo", revision: 7, imported: true}
- {from: "active", to: "implemented", at: "2026-09-27T15:01:47Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1}}, imported: true}
---
## Scope

- #145: an `unknown_instance:` answer that is not an external refusal (0.35.1 covers the external
  case).
- #150: `creates:` into a declared non-initial state.
- #151: an outcome that deletes its subject; synthesis asserts absence from the entity's views.
- #144: an accepted no-op on a command with no subject (`preserves:` covers the subject case).
- #152: an ambient precondition (declared seed state) the explorer and synthesis set up first.

## Acceptance

Each construct validates under a new source format, is refused below it, and has a synthesized
scenario the issue's repro passes against a correct implementation.

## Derived scope

Derived 2026-09-27 by `aep:story-scoper`. Every line is **cited** (read from the story or the tree)
or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/specify/ess-domain` (outcome validation) and `crates/verify/ess-conformance` (synthesis) — inferred, from the issues' diagnostics and the ess/14 commit's footprint
- **Files:** `crates/specify/ess-domain/src/command.rs:4421` (creates+moves refusal, ESS-COMMAND-004, #150) — cited
- **Files:** `crates/specify/ess-domain/src/command.rs:1992` (neither emits nor names an error, ESS-COMMAND-007, #144) — cited
- **Files:** `crates/specify/ess-domain/src/command.rs:619` (`Effect`) — inferred, where a delete effect (#151) and create-into-state (#150) would be declared
- **Files:** `crates/specify/ess-domain/src/system.rs:53` (`SUPPORTED_FORMATS`, `FormatVersion`) — cited, the acceptance needs a new source format
- **Files:** `crates/specify/ess-compiler/src/ir.rs:653` (`ResolvedEffect`), `crates/specify/ess-compiler/src/resolve.rs` — inferred
- **Files:** `crates/verify/ess-conformance/src/synthesize.rs:2264` (creation lands in `lifecycle.initial`, #150), `:4860` (`unknown_instances`, #145/#151), `:247` — cited
- **Files:** `crates/generate/ess-gen/src/unknown_instance.rs:30` (`unknown_instance_answer`, #145) — cited
- **Files:** `crates/verify/ess-conformance/src/go/explore.go:1088`, `src/ts/explore.ts:786` (explorer rows start in `lifecycle.initial`; seed state, #152) — cited
- **Also likely:** every file matching on `ResolvedEffect::` (16 files incl. `ess-entity-runtime/src/lib.rs`, `ess-gen/src/{openapi,asyncapi,docs}.rs`, `ess-synth/src/plan.rs`, `ess-synth/src/web/catalog.rs`, `ess-conformance/src/synthesize/{aggregate,subject_fact}.rs`, `ess-conformance/src/web.rs`, `ess-diff/src/diff.rs`) — inferred
- **Also likely:** `crates/generate/ess-synth/src/{go,rust}/*` unknown-instance seams (#145) — inferred
- **Documents:** schema, CHANGELOG, spec-versions, formats, guide, `docs/design/typed-literals-and-unknown-instances.md`, `docs/design/unknown-instance-seams.md`, a new design page — inferred
- **Confidence:** medium — refusal and synthesis/explorer sites cited; the `ResolvedEffect` ripple inferred; no construct designed yet
- **Would collide with:** any unit touching `command.rs`, `system.rs`, the compiler IR, `synthesize.rs`, the Go/TS explorers, or the generated schema and format docs
- **Not established:** which of #152's three alternatives; where synthesis reads views for #151's absence; whether #145 changes the generated seams; #144's construct; the format number. Five issues is wide; #151 and #145 are a possible split line.
