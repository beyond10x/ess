---
format: aep.planning-md/1
id: story:fuzz-the-specification-surface
kind: story
status: active
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
revision: 16
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

Derived 2026-09-07 by aep-drive:story-scoper 0.8.0 from draft revision 12, reviewed candidate binding v3 and coverage-integrated source d1fe6755e842c8ef5486a90493530a39050dae48 — cited.

- **Harness:** `fuzz` — cited. Own the standalone stable workspace and lock, nested separately locked engine workspace, shared public-API pipeline, ordered multi-document carrier, two bounded engine entries, three mandatory regression seeds, deterministic replay, complete stage observations, watchdogs, mutation controls and local instructions. Preserve candidate v3’s bounds and require actual compiled callbacks and all ten downstream calls; an engine exit or replay count alone is insufficient.
- **Mandatory prerequisite:** `crates/generate/ess-synth` — cited. Add only the Go missing-type-owner refusal before layout construction and after the existing Binary64 guard. Check every compiled type against actual domain ownership, including unreferenced types. Use the existing MissingTypeOwner code and Go failure/2 envelope, deterministic source-addressed causes and the reviewed exact detail. Own focused direct-workspace, facade, precedence and compatibility regressions; preserve neutral plans and valid owned Go artifact bytes.
- **Stable gate:** `Taskfile.yml` — cited. Add explicit standalone stable formatting, strict Clippy, tests and deterministic replay to the existing check flow. Preserve every current lane. Keep engine/nightly dependencies outside the root workspace and ordinary stable graph.
- **CLI regression:** `crates/edge/ess-cli/tests/target_failure.rs` — inferred. Add a separate Go system-level-type case while preserving the existing Rust/Web empty-domain fixture and assertions. Cover both command spellings, text/JSON/YAML, exact typed failure/2 causes, stdout/exit behavior and absent/existing destination preservation. Also establish actual CLI generation/synthesis choice correspondence. CLI production remains a read-only authority.
- **Binding:** `docs/design/review-specification-fuzzing.md` — inferred. Promote the reviewed candidate after acceptance, refreshing its source/status references. Preserve its closed carrier, budgets, complete stage accounting, production dispatch, bounded live ASan campaign and setup prerequisites.
- **Public correction:** `website/docs/reference/formats.md` — inferred. Update the Go/Clap complete-failure row narrowly to include Go missing-owner refusal under the existing envelope. Preserve Binary64 precedence, Rust/Web distinctions, normalization claims and the newly integrated coverage documentation.
- **Repair boundary:** retain the known crashing Go seed and its historical evidence; any additional production defect needs its own concrete repair assignment rather than silently enlarging this prerequisite — inferred.
- **Compatibility:** preserve existing tests, valid artifact maps and neutral plans. Root workspace membership, root manifest/lock changes, conformance witness generation and emitted-program compilation are outside these reservations — inferred.
- **Would collide with:** changes within the synth package or fuzz subtree, the exact CLI regression file or a containing CLI reservation, Taskfile gate wiring, the binding page or formats catalog. Compare actual path containment; distinct scope tokens alone do not establish disjointness — inferred.
- **Delivery:** public catalog publication and Website/Atlas delivery remain coordinator-owned; no external repository becomes an implementation reservation — inferred.
- **Confidence:** high for these six write reservations because the reviewed binding and current source identify their owners. Offline resolution, instrumented setup and actual observations remain first-stage evidence requirements — cited.

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

## Coverage-integrated scope refresh

The independent scope report is retained at
`target/review-boundaries-12/preparation/scopers/fuzz-report.md`, SHA256
`e25facc246c3cb91b4f2d44eb75f7b278bc35a1526c863f460e2903a84beef8d`. Root read back every inspected input hash before applying this section
(45 inputs). The report was captured from the original final message without
rewriting it. The machine reservations remain unchanged. This read-only refresh ran no source
tests or build and did not itself select an implementation.

## Wave20 acceptance and execution — 2026-09-08

The preceding observations and acceptance are historical and remain unchanged. The coordinator
accepts docs/design/review-specification-fuzzing.md at source 9ab8a1b16dec7c08ba2dd2ac3b507f48254514f5
under the standing implementation/publication records and selects this story alone for wave20.
The same six typed reservations remain authoritative. The fresh aep-drive:story-scoper 0.8.1 report
and binding-read addendum are retained in the wave20 opening receipt; root verified 24 source inputs
and 237 recovered preparation files. This is source/setup refresh, not a third binding attack.

Stage one must resolve both standalone lock graphs offline, verify the exact stable and dated
nightly ordinary-ASan payloads with a real compile/link/run probe, and capture complete positive
Go artifact maps/plans before any production repair. Minimal owned readiness scaffolding is allowed.
Stage two establishes observed failing verifiers, repairs only Go's MissingTypeOwner prerequisite,
and completes the accepted harness, original seeds, structured diversity, both bounded live lanes,
complete observations, mutation controls, watchdogs, CLI matrix and package checks. Stage-one
success alone does not satisfy acceptance. The accepted design supplies all exact finite bounds.

Current scope confirmation: 3 cited and 3 inferred reservations, all unchanged. The inferred CLI test,
binding page and public formats page are confirmed by the current source and binding. Any newly
found production panic needs its own concrete repair assignment; it does not silently widen this
unit. Preserve root Cargo files, all previous assertions, valid owned Go bytes, Binary64 precedence,
neutral plans and every current Taskfile lane including support-check and consumer-check.

The wave page is docs/plan/2026-09-08-review-boundaries-20.md. Root owns AEP, bot Git operations,
independent source review, complete task check and site-build, verified main publication and exact
managed cleanup. Current user scope excludes downstream Website/Atlas delivery, superseding the
historical Delivery scope line. No new product format, release tag or version bump is selected.
