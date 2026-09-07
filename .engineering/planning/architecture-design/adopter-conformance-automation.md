---
format: aep.planning-md/1
id: architecture-design:adopter-conformance-automation
kind: architecture-design
status: draft
title: Automate adopter qualification through reusable ESS capabilities
relations:
- serves: vision:O2
- informed_by: story:source-pinned-data-normalization
- informed_by: story:review-conformance-coverage
revision: 1
---
## Intent

Working with ESS should reduce the hand-built machinery an adopter maintains. Reuse existing ESS capabilities first. A missing reusable capability should be designed and implemented in ESS as Rust, with project-specific semantics and minimal observation adapters supplied by adopters. This draft records the automation boundary and the questions required before implementation; it does not select a new wire format, command vocabulary, process transport or runtime change.

## Verified existing capabilities

The inspected source baseline is e5e4fdf899b8afb59c59b6182a8f1e47115569a6; the released CLI checked separately is 0.20.0.

- `crates/verify/ess-conformance/src/target.rs` defines `ConformanceTarget`: semantic commands, views, events and related observations. Targets report observations; the runner determines outcomes.
- `crates/verify/ess-conformance/src/runner.rs`, `src/admission.rs`, `src/counts.rs` and `src/report.rs` own execution, admitted-input identity and reports. Reuse their defined behavior rather than minting parallel success or coverage meanings.
- `website/docs/guides/verify-conformance.md` documents synthesis, execution and adding a Rust target. The 0.20.0 `ess verify conform run --help` lists only the built-in billing and oracle-fixture CLI targets; this is a CLI integration limitation, not absence of a Rust extension boundary.
- `crates/edge/ess-cli/src/normalize.rs` and `crates/generate/schema-contract/src/realize/normalize/` own normalization checking, reference execution and generation. The 0.20.0 `ess schema --help` exposes normalize-check, normalize-run and normalize-generate. Adopters should not reimplement those transformations.
- `docs/design/review-conformance-coverage.md` and `docs/design/review-conformance-coverage-transport.md` already govern coverage and input transport. This draft neither supersedes those bindings nor treats an accepted design as a released implementation.

## Automation boundary

| Responsibility | Ownership and next action |
| --- | --- |
| Domain meaning, supported compatibility policy and independently justified expected observations | Adopter-owned authored input; never invented from a successful run. |
| Calling an existing implementation and exposing its actual observations | Small adopter adapter using the existing Rust target seam where its semantics fit. Generate mechanical bridge code only from an explicit supported contract. |
| Suite generation, execution accounting, admitted-input identity and conformance reports | Reuse existing ESS owners. Identify a concrete missing capability before extending them. |
| Comparing literal inputs across reference evaluation, generated targets and an original implementation | Evaluate a reusable ESS qualification facility. Raw decoder observations are not automatically semantic command conformance. |
| Tool/process execution, retained command output and source/fixture integrity | Candidate reusable Rust edge support; keep filesystem, compiler and process authority outside pure semantic libraries. Determine the smallest owner rather than moving an adopter's entire runner wholesale. |
| Corpus-specific field exposure and canonical/native comparison policy | Adopter contracts and minimal mappings. A mapping must preserve relevant observations and declare omissions, rather than conceal mismatches. |

## Required design evidence before implementation

1. Map each proposed operation to an existing ESS type/API and enumerate the exact residual gap. Separate semantic conformance from literal decoder/normalizer compatibility; do not invent domain commands to disguise raw parser tests.
2. Define what can be generated or derived safely, what requires authored semantics, and which unsupported cases must remain explicit. Independently authored expected results must not be replaced by observations from the implementation under test.
3. Demonstrate the proposed reusable API with at least two independently authored, unrelated example implementations. Both should share runner/comparison/reporting machinery and differ only in typed inputs, fixtures and minimal adapters.
4. Preserve raw input bytes, exact numeric observations where the selected profile requires them, null/empty distinctions and declared error profiles. Define comparison policies explicitly; do not silently normalize a mismatch away or widen canonical admission to match one legacy implementation.
5. Prove missing, duplicate, skipped and failed executions cannot become passing evidence. Reuse ESS's coverage and report semantics where admitted; matching a finite corpus must not imply complete system conformance.
6. Keep source acquisition, process authority, tool identity, timeouts and retained evidence at explicit edges. A target cannot attest to its own success or fabricate source identity. Decide resource and cancellation behavior through the design before promising guarantees.
7. Preserve existing formats and defaults. Any actual new persisted meaning requires a separately reviewed format decision; no parallel report format is selected here.
8. Provide an adopter migration path that removes duplicated machinery after equivalent results are demonstrated. Release and pin reusable ESS support before retiring an adopter's working checks. No service runtime change is necessary merely to integrate test tooling.

## Scope and status

This is a design-only planning record. Existing typed seams cited above are read evidence, not an implementation write allowance. No new business entity, model semantics, format, dependency, executable code, release or deployment is introduced. Keep the artifact draft until the unresolved integration and compatibility questions are answered. No decomposition was made, so there is no multi-story panel or implementation wave to dispatch.
