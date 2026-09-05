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
  path: fuzz
revision: 6
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

- **Derivation:** cited — refreshed 2026-09-05 from story revision 5, F16, and production `acb7859e3202ffdc1ca840dde67f7ca4da33c746`, published in `0d08e219a61cd1911b913d3da20c8e4b87993b94`. This refresh performed source and installed-tool inspection only; there is no newly measured fuzz/test baseline or completion claim.
- **Primary edit token:** cited — `fuzz`; the acceptance explicitly requires a document-generation harness and three regression seeds there. No `fuzz/` directory, fuzz manifest, target or corpus currently exists in this checkout. The root manifest and lock contain no `libfuzzer-sys` or `arbitrary` dependency.
- **Existing property coverage:** cited — `crates/specify/ess-compiler/tests/adversarial.rs` already generates small YAML documents with `proptest 1.11.0`, a fixed `0x5eed_0002` seed, limited names/types/fields, and both accepting/refusing controls. Its `run` parses with `RawSpecFile::parse`, assembles, compiles and renders canonical IR; the property compares two runs byte for byte. It invokes no projection or synthesis API. The dependency is local to `ess-compiler` dev-dependencies; that existing test is an evidence/reuse reference, not an edit reservation.
- **Actual admission predicate:** cited — `RawSpecFile::parse` in `ess-domain/src/spec.rs:151` performs the duplicate-key-preserving YAML admission path. `Specification::assemble` at `:219` already calls `validate()` and roster validation at `:260`. CLI `ess validate` in `ess-cli/src/main.rs:1779` calls `resolved`, and `src/load.rs:105`/`:117` both assembles and compiles before reporting success. An assembly-valid document whose resolution returns diagnostics is not CLI-accepted input.
- **No-panic property:** inferred — drive the public parse → assemble/validate → compile pipeline from generated document text and retain its refusal/acceptance distinction. Parse, validation and resolution refusals are ordinary data outcomes, not harness panics. On compiled inputs, invoke every required generation and synthesis stage; a panic must escape as a crash, never become a discarded generator case. Do not claim `Specification::validate` alone is equivalent to CLI acceptance or use direct Serde deserialization that skips `RawSpecFile::parse`.
- **Five registered generation paths:** cited — `ess_gen::generators()` in `crates/generate/ess-gen/src/lib.rs` returns docs, site, schema, openapi and asyncapi. `artifact::run` at `src/artifact.rs:160` calls each generator and asserts readable/correct slice provenance before collecting outputs; it returns typed duplicate-path refusals. Portable destination validation is a separate `artifact::validate_paths` API used at output boundaries; `run` does not establish that every destination set is writable. Calling only `Generator::generate` would omit the production provenance boundary.
- **Sixth generation path:** cited — docs-ir is absent from that registry. `ess-cli/src/main.rs:2323` constructs a `ProvenanceMint`, calls `ess_gen::docs::document` (`src/docs.rs:106`) and serializes the document as pretty JSON with a newline. The harness must separately execute that construction and serialization. The basic site generator is an in-memory path; CLI-authored includes and front-page filesystem discovery are not part of generated specification content.
- **Synthesis paths:** cited — `crates/generate/ess-synth/src/lib.rs:271` exposes `synthesize_for(&EssIr, Target)` and enumerates Rust, Go, Web and Clap. It produces plan renderings, artifacts and optional target refusal/weakening reports and asserts emitter/planner consistency. All four variants are required; Rust's convenience `synthesize()` is not the complete surface. These production crates are exercised dependencies, not automatically edit scopes.
- **Stage accounting:** inferred — guard against a vacuous run by replaying accepted seeds and demonstrating that the six generation kinds and four synthesis targets are each reached on accepted inputs. Keep parse/validation/resolution rejection counts distinct from compiled cases and target invocations. A typed generation refusal or synthesis target refusal is not itself a panic under this story's acceptance; do not `unwrap` a normal Result to manufacture a crash, and do not stop all later targets merely because one returned a refusal. Minimal controls should detect skipped docs-ir/target dispatch and a swallowed panic.
- **Document generation boundary:** inferred — generate or mutate actual `ess/1` text, with bounded size/depth and a route to meaningful accepted specifications, including the named seed constructs. The existing tiny type strategy or a byte stream that only yields parse errors cannot demonstrate downstream coverage. Choose byte mutation, structured generation or a shared combination explicitly; if a structured encoding is used, retain an unambiguous translation between its input bytes and committed readable seed documents. Parser robustness is useful additional coverage but does not replace accepted-document downstream execution.
- **Mandatory corpus identities:** cited — retain (1) a system-level `types:` declaration, (2) a binding mapping `T` into `Optional<T>`, and (3) a specification with no `on_failure: retry` binding. Revision 5 permits reconstruction of minimal local fixtures when original external consumer files are unavailable. These historical reports are not fresh reproductions at the frozen production commit.
- **Current source qualification for the historical seeds:** cited — Rust layout ownership is collected from domain rosters in `ess-synth/src/rust/layout.rs:52`, and `owner` at `:131` still panics on a missing declaration. This identifies a crash candidate, not an executed seed. Rust `src/rust/system.rs:757`/`:775` emits `redeliver` when deliveries are nonempty, while `src/web/bridge.rs:338` emits a call to it. Absence of a retry policy alone is therefore not the current source predicate for absence of that method; preserve the historical seed identity and measure its actual current shape/outcome.
- **Seed retention and corpus ownership:** inferred — reserve all harness seeds, replay assertions and corpus documentation inside `fuzz`. Keep the three readable mandatory regressions tracked and replayed even if live coverage minimization would remove them. Store mutable discovered corpus entries, crash artifacts, coverage and compiled targets under ignored local paths; copy/reconstruct only minimal public/synthetic specifications, with no dependency on unavailable consumer checkouts. A fresh fuzz input that crashes remains evidence to minimize and route to the responsible stage.
- **Default corpus trap:** cited — the locally installed cargo-fuzz 0.13.2 template ignores `target`, `corpus`, `artifacts` and `coverage` wholesale. Following it verbatim would omit mandatory corpus files from publication. Use deliberate harness-local ignore rules or a separate tracked regression directory; this is covered by `fuzz`, with no root `.gitignore` change required.
- **Smallest harness layout:** inferred — a standalone `fuzz/Cargo.toml` with its own `[workspace]`, `fuzz/Cargo.lock`, local `.gitignore`, shared pipeline/replay code, a `fuzz_targets/specification.rs` entry, committed seed inputs and a README can satisfy this story without becoming a root workspace member. The proposed filenames/package/target are implementation choices, not existing files. Path dependencies can reach the public `ess-domain`, `ess-compiler`, `ess-gen` and `ess-synth` APIs; no production API change is needed simply to call them.
- **Local tooling evidence:** cited — inspection found cargo-fuzz 0.13.2, stable Rust, nightly `1.100.0-nightly (908501772 2026-08-30)`, installed dated nightly `1.99.0-nightly (09ee43b2d 2026-07-27)` under `nightly-2026-07-28`, plus `clang` and `clang++`. Local Cargo archives include `libfuzzer-sys 0.4.13`, `arbitrary 1.4.2`, `derive_arbitrary 1.4.2`, `proptest 1.11.0`, and C++ build dependencies. Archive presence is not a completed dependency-resolution/build test.
- **Offline execution lane:** inferred — use a locked, independent harness manifest for deterministic regression replay and local formatting/strict Clippy, followed by a finite instrumented cargo-fuzz run using the installed nightly, AddressSanitizer, explicit per-input timeout and bounded input/run/RSS limits. Keep input corpus, crash artifacts and target directories inside the assigned tree. Record actual processed, compiled and per-stage case counts, not only the fuzzer's process exit. Bounded fuzzing supplies sampled evidence; it does not establish universal termination.
- **Tool constraints:** cited — cargo-fuzz 0.13.2's inspected build/run help has no `--locked` option; use offline Cargo configuration and separately verify the harness lock did not change rather than claiming its build was locked. `rust-src` is absent on the inspected nightlies. The installed tool's archived source has `build_std: false` and only passes `-Z build-std` for an explicit request, careful mode or MemorySanitizer; ordinary AddressSanitizer does not require that missing component. No component installation or fetch was attempted.
- **Root reservation decision:** inferred — recommend no root `Cargo.toml`, root `Cargo.lock`, `Taskfile.yml` or workflow reservation for the standalone local harness. If stable seed replay must join `task check`, add only the explicit `Taskfile.yml` gate hook and its validation to the scope first; current CI already delegates to that task. If the harness becomes a root workspace member instead, reserve root `Cargo.toml`, root `Cargo.lock` and the explicit formatter/task list. Nightly workflow provisioning/scheduled fuzzing is an additional requested feature, not required by the present acceptance.
- **Gate limitation:** cited — root `Cargo.toml` enumerates members, `Taskfile.yml:4` explicitly enumerates packages for formatting, and `:34`/`:38` run workspace Clippy/tests. A standalone fuzz workspace is not automatically included in those commands. `.github/workflows/ci.yml` provisions stable Rust and calls `task check`; there is no existing fuzz workflow to update. Harness-local evidence must not be described as automatic CI coverage unless an integration change is actually made.
- **No generated-program compilation claim:** cited — revision 5 explicitly assigns emitted-code compilability to target-feasibility stories. Optional binding `E0308` and missing emitted `redeliver` are generated-program defects that can survive a no-panic fuzz run. This harness invokes Rust/Go/Web/Clap generators in memory; it does not need Go execution, a WASM target, browser tests, generated Cargo workspaces or an external compilation corpus to assert its own property. It does not fuzz command-value witnesses or change `ess-conformance/src/witness.rs`'s deterministic semantic rule.
- **Defect routing:** inferred — if a mandatory accepted seed or newly generated input panics in a production stage, retain the input and red evidence and route repair to that stage's existing owner with an explicit scope/dependency decision. The current `fuzz` token does not authorize edits to domain/compiler/generator/synthesizer source, discarding the crashing seed, accepting the panic as an expected success, or claiming the harness story green. Measure the mandatory seeds before relying on this candidate's disjointness or readiness.
- **Documents and compatibility:** inferred — harness-local operating instructions and dependency/toolchain/corpus conventions are sufficient; no public-doc, generated-product, persisted ESS-format or conformance-format migration is implied. A root manifest/lock/workflow change must follow the explicit integration choice rather than being added for convenience.
- **Would collide with:** cited — any edits to the new `fuzz` manifest, lock, target, pipeline/replay code, seed corpus or local instructions. Production API reads create dependency sensitivity but do not reserve those crates for writes.
- **Potential collisions requiring a new reservation:** inferred — root workspace/lock and formatter/gate edits if integration changes, CI tooling if nightly automation is requested, or the concrete production stage of a reproduced crash if repair is assigned here. These are conditional decisions, not hidden reservations in the current single-token scope.
- **Confidence:** inferred — medium: acceptance and callable stage owners are explicit and the standalone layout avoids root edits, but concrete seed acceptance/crashes, complete offline dependency resolution, generator strategy and root-gate policy remain unmeasured or undecided.

## Remediation ownership and limits

Owns the F16 general document-fuzzing harness; no-panic is its property, not emitted-code compilability. Target-feasibility stories own concrete compile checks and review-consumer-coverage owns the consumer matrix. Preserve the three named regression seeds, reconstructing minimal local fixtures if their external originals are unavailable. Reassess root Cargo/Taskfile scope when choosing harness integration; this item is not in the first wave.
