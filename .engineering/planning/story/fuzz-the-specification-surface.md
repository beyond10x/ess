---
format: aep.planning-md/1
id: story:fuzz-the-specification-surface
kind: story
status: draft
title: Fuzz the specification surface
summary: Anything validate accepts, every projection and every synthesis target survives — asserted rather than hoped.
owner: ess
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: inferred
  path: crates/edge/ess-cli/tests/target_failure.rs
- confidence: cited
  path: crates/generate/ess-synth
- confidence: inferred
  path: docs/design/review-specification-fuzzing.md
- confidence: cited
  path: fuzz
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 12
---
# Fuzz the specification surface

## Outcome

`ess validate` accepting a document means every projection and every synthesis target survives it.
Today that is a hope, and three defects in one afternoon were all the same shape.

## Why this, rather than fuzzing inputs

**Input generation already exists and is deliberately not random.**
`crates/verify/ess-conformance/src/witness.rs` states the rule it is built under: "Never generate an
arbitrary value and claim it satisfies an outcome predicate unless the generator can prove or
evaluate that it does." Candidates come from the guard's own literals, one either side, in a bounded
deterministic order. Randomising that would weaken it — an arbitrary value that happens to satisfy a
guard proves nothing about the guard.

The unguarded surface is the **document**, not the input. Three defects found while writing one
consumer specification, and every one of them is `validate` admitting something a later stage cannot
survive:

| defect | validate | downstream |
|---|---|---|
| a system-level `types:` entry | accepts | all three synthesis targets panic — `"…" is not a declaration this layout knows` |
| a binding mapping `T` into `Optional<T>` | accepts | emitted Rust does not compile, `E0308`, three occurrences |
| a specification with no `on_failure: retry` binding | accepts | the web bridge calls a `redeliver` the rust target does not emit |

The first was reproduced from a two-line addition to a valid document. A fuzzer over documents would
have found it in minutes, and none of the three needed a clever input.

## Acceptance

For every generated ess/1 document accepted by validation, the fuzz target verifies that compilation and every current generation and synthesis target terminate without panicking.

## Validation and corpus

Create the document-generation harness under `fuzz/` and retain the three defects above as mandatory regression seeds. Exercise all six current generate kinds (including the separate docs-ir path) and all four synthesis targets enumerated in the Scope section. A crash is a defect in the stage that crashed, not an invalid generator result; compilation of emitted programs is owned by the target-feasibility stories.

A `fuzz/` target that generates `ess/1` documents and asserts one property: **for every document
`ess validate` accepts, `compile`, all six `generate --kind`s and all four `synthesize --target`s
terminate without panicking.** A crash is a defect in whichever stage crashed, never in the
generator.

The three defects above become regression seeds, so the corpus starts with cases known to have been
wrong.

## What is already here

- `proptest 1.11.0` is a dev-dependency of `ess-compiler`, so property testing is not a new tool in
  this workspace.
- There is no `fuzz/` directory. `cargo-fuzz` and `arbitrary` are not dependencies anywhere.
- `sipx` carries a `fuzz/` directory, which is worth reading for the shape a b10x repository uses
  before inventing one here.

## What this is not

Not a replacement for the gate. `task check` proves the committed examples work; a fuzzer proves the
*unwritten* documents do not crash, which is the half no example can cover. And not a test of
whether a specification is any good — a document can be meaningless, survive every stage, and that
is a pass.

## Scope

Derived 2026-09-06 by aep-drive:story-scoper from draft revision 8, the retained 33-command mandatory-seed baseline and current coordinator production corresponding to published 239996d846460aee342ce42514378c25b2be5152 — cited.

- **Harness:** fuzz; the story explicitly requires the document harness and three permanent regression seeds here. Own standalone manifest/lock boundaries, shared in-memory pipeline, stable replay, engine entry points, bounded generators, focused controls, corpus translation and local instructions — cited.
- **Concrete prerequisite repair:** crates/generate/ess-synth; the actual Go panic is in Layout::owner during Layout::of name allocation. Own a Go-specific missing-type-owner prerequisite before layout construction, its source-addressed checked failure and focused Go/direct-facade/compatibility tests. Existing shared failure vocabulary and Rust behavior remain read-only authorities — cited.
- **Default gate:** Taskfile.yml; current check does not include a standalone fuzz workspace. Add explicit stable harness formatting, strict lint, focused tests and deterministic regression replay while retaining every existing gate lane — cited.
- **CLI boundary regression:** crates/edge/ess-cli/tests/target_failure.rs; add a separate system-level-type Go case using the existing executable harness, preserving its Rust/Web empty-domain fixture and assertions. Cover both CLI spellings, text/JSON/YAML, Go failure/2, missing-type-owner and destination preservation; CLI production remains read-only — inferred.
- **Binding:** docs/design/review-specification-fuzzing.md, currently absent; bind the Go prerequisite, corpus encoding and limits, accepted-document route, exact stage accounting, drift detection, stable gate and bounded local ASan policy before code — inferred.
- **Public correction:** website/docs/reference/formats.md; update only the existing Go/Clap complete-failure row to acknowledge Go missing-owner refusal under its current envelope, preserving distinct Binary64, Rust/Web and normalization claims — inferred.
- **Ownership boundary:** the measured prerequisite can remain with this harness because it is necessary for a mandatory seed and has one concrete existing producer owner. Any additional discovered panic requires retained evidence and a fresh exact repair assignment; this is not blanket production-remediation scope — inferred.
- **Compatibility:** preserve all valid owned Go artifact bytes, neutral plans, Rust/Web behavior, the earlier Binary64 guard and existing format envelopes. No root workspace membership, root Cargo manifest/lock, conformance witness generation or generated-program compilation change is required by the standalone option — inferred.
- **Collision and sequencing:** actual containment matters: the synth package contains the Go owner; the exact CLI test may be contained by a broader CLI reservation; the formats file is currently written by the active coverage unit. A literal-token nonmatch does not establish disjoint files. Wait for coverage closure and refresh integrated source, CLI and format-row bytes before later selection; Taskfile also overlaps the prepared consumer-gate scope — cited.
- **Delivery:** a public format-row change requires the normal ESS source publication and Website/Atlas delivery under the coordinator's standing authorization. No broader public directory or external repository is an implementation reservation here — inferred.
- **Confidence:** medium, because the current failure, repair timing, checked API, gate owner and reusable CLI harness are established, but the exact input/generation limits, standalone dependency layout, stage-drift mechanism and offline instrumented build remain binding/setup decisions — inferred.

## Remediation ownership and limits

Owns the F16 general document-fuzzing harness; no-panic is its property, not emitted-code compilability. Target-feasibility stories own concrete compile checks and review-consumer-coverage owns the consumer matrix. Preserve the three named regression seeds, reconstructing minimal local fixtures if their external originals are unavailable. Reassess root Cargo/Taskfile scope when choosing harness integration; this item is not in the first wave.

## Current mandatory seed observation — 2026-09-06

verification-report:fuzz-seed-baseline-go-panic retains 33 actual CLI invocations over three reconstructed mandatory seeds at the unchanged published 239996d production source. All three validate and reach all six generation kinds and four synthesis targets. Go synthesis of the system-level type exits 101 with an actual panic at crates/generate/ess-synth/src/go/layout.rs:290. Rust/Web issue ordinary target refusals for that seed and the optional binding; the remaining commands exit zero. This is preparation, not a fuzz harness, generated-program compilation or completed story. The old fuzz-only scope must be refreshed with the measured Go repair owner before selection; retaining the crashing seed is mandatory. The existing passing target-feasibility records remain historical evidence of their tested subset.

Companion verification-report:fuzz-seed-baseline-refusal-stdout corrects the baseline report's empty diagnostic excerpts: those four CLI commands printed their textual target failures on stdout, while the original excerpt builder read empty stderr. The companion verifies the retained stdout hashes and quotes the actual bytes without rerunning commands. The Go panic, seed inputs and 33 original execution receipts are unchanged.

## Candidate binding review

The coordinator accepted the six refreshed write reservations on 2026-09-06 and applied only
this Scope replacement; the story remains draft. The independent scope report is
target/review-boundaries-11/next-scope/fuzz-refresh/report.md, SHA256
e310b7f6c7c1e3b19165548a82db32f10219e10cbfc0e927d7f532801972d823; root verified all 69
inputs and four archive members. The earlier panic and refusal-stdout reports remain unchanged.
Document reviews review-result:fuzz-specification-binding-pass1 and
review-result:fuzz-specification-binding-pass2 close three contract findings: checked production
generator dispatch, actual compiled live callbacks and complete observation/watchdog behavior.
Root independently verified 27 and 30 review inputs. The current unselected candidate is
target/review-boundaries-11/fuzz-preparation/binding-draft-v3.md, SHA256
225d2561dc92a8884180fee2e828ad1b9f2b211a72b5d75844e9066240a80f43. It selects the finite
2048-run/120-second budget per engine entry; the earlier scope's suggested larger budget was
not executed or accepted. Offline graph resolution, dated ASan setup and positive Go artifact
capture remain first-stage prerequisites. Refresh coverage-integrated source/CLI/formats and
accept the binding before later story selection; no source gate or fuzz execution is claimed.
