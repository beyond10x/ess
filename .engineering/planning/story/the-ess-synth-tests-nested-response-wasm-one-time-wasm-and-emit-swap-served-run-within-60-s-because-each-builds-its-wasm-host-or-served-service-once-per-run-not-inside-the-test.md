---
format: aep.planning-md/3
id: story:the-ess-synth-tests-nested-response-wasm-one-time-wasm-and-emit-swap-served-run-within-60-s-because-each-builds-its-wasm-host-or-served-service-once-per-run-not-inside-the-test
kind: story
status: draft
title: The ess-synth tests nested_response_wasm, one_time_wasm and emit_swap_served run within 60 s, because the wasm host or served service each compiles is built once per run and not inside the test
summary: 'Three ess-synth tests measured 88.97 s, 97.07 s and 64.91 s, over the 60 s bound: each compiles a wasm host or a served Rust and Go service in its own body'
tags:
- defect
- issues-2026-11-26
- test-budget
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- informed_by: story:tests-that-build-into-one-shared-scratch-directory-collide-when-each-test-runs-in-its-own-process-one-fixture-is-built-once-per-run-keyed-by-its-sources
- informed_by: story:tests-that-run-over-60-s-on-main-without-a-branch-of-the-week-run-within-60-s-or-move-to-the-slow-lane
revision: 1
---
## Need

Three tests of `ess-synth` run over the 60 s bound of one test, measured in a package gate on 2026-11-25 and 2026-11-26:

| test | target | seconds |
|---|---|---|
| `actual_wasm_observes_independent_response_event_and_generated_sibling` | `nested_response_wasm` | 88.97 |
| `shared_disclosure_controls_execute_inside_wasm_through_browser_bridge` | `one_time_wasm` | 97.07 |
| `emit_swap_served` (the test of that name as reported; the target holds three tests that serve or run against the generated services) | `emit_swap_served` | 64.91 |

This is a defect of the test budget the test step of the gate keeps: no single test runs longer than 60 s, and an expensive product is made once, never inside each test. A step over the budget is fixed before the next feature; raising a timeout, adding shards or waiting longer is not a fix.

## Cause

One cause for all three: each test builds its product inside its own body, with a compiler run, and builds it again each time it runs.

- `nested_response_wasm` and `one_time_wasm` write a host crate into a scratch directory of their own and run `cargo build --target wasm32-unknown-unknown` on it, then run the module under `node`. The split between the build and the run was not measured; the first step is the measure.
- `emit_swap_served` builds the generated Rust service (and the Go service where `go` is on the path) in a `OnceLock`. A `OnceLock` is in-process: under the runner that starts one process per test, each of its tests that needs the build makes it again.

`story:tests-that-build-into-one-shared-scratch-directory-collide-when-each-test-runs-in-its-own-process-one-fixture-is-built-once-per-run-keyed-by-its-sources` names the shared-fixture form the repository accepts for this: one product built once per test run, in a cache on disk keyed by the digest of its sources, or in a setup step before the tests.

## Outcome

Each of the three tests runs within 60 s in the test step, and the three targets together add no build of a wasm host or a served service to the step beyond one per run. A test that cannot meet the bound moves to the slow lane with its reason written beside it, and keeps its assertions.

## Acceptance

1. The measure is taken on the build node, one target at a time, before and after, with the node's load written beside it; the times of every test of the three targets are recorded.
2. The wasm host of each of the two wasm targets, and the Rust (and Go) service of `emit_swap_served`, are built once per test run, in a cache keyed by the digest of their sources or in a setup step, and read by every test that needs them. A test builds none of them.
3. No test of the three targets is over 60 s in the step's ten slowest, and the runner's kill at 60 s does not end any of them.
4. A test moved to the slow lane runs the same body there; the assertions are the same.
5. Where the cause differs from the one named, the story is split per test and says so.

## Fit review

1. Need: the test step must end within its budget, and a test over 60 s is killed and counts as failed.
2. Class: a defect of the test budget, not a feature.
3. Already expressible: yes, by the two accepted forms of the rule (a committed generated file with a digest, or one shared fixture built once per run).
4. Fit: the same shape as the other shared-fixture stories; this story only names three more tests.
5. A second adopter: any test that compiles inside its body.
6. Cost: a setup step or a cache on disk for two kinds of build.
7. Considered: raising the limit (rejected by the rule), moving all three to the slow lane without a shared build (kept as the fallback of acceptance item 4).

## Decisions

- None open. One story for the three tests, because one cause is plain (a build inside the test body); acceptance item 5 splits it if a trace shows otherwise (the coordinator, 2026-11-26).

## Scope

Cited: `crates/generate/ess-synth/tests/nested_response_wasm.rs`, `crates/generate/ess-synth/tests/one_time_wasm.rs`, `crates/generate/ess-synth/tests/emit_swap_served.rs`, `.config/nextest.toml`.
