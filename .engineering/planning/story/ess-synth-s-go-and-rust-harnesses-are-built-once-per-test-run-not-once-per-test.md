---
format: aep.planning-md/3
id: story:ess-synth-s-go-and-rust-harnesses-are-built-once-per-test-run-not-once-per-test
kind: story
status: draft
title: ess-synth's Go and Rust harnesses are built once per test run, not once per test
summary: 'In http_binding_rust each go:: test builds the Go harness and, where it compares both servers, a Rust harness; 78 go:: tests took 4,060 test-seconds, 31 over 60 s'
tags:
- defect
- from-finding
- test-budget
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- informed_by: story:the-ess-synth-tests-nested-response-wasm-one-time-wasm-and-emit-swap-served-run-within-60-s-because-each-builds-its-wasm-host-or-served-service-once-per-run-not-inside-the-test
revision: 1
---
## Need

In `ess-synth`'s `http_binding_rust` test target, each `go::` test builds and runs a Go harness, and each of those that compares both servers also builds a Rust harness with `cargo run`. The `OnceLock` that caches answers in that target shares nothing under nextest, which runs each test in its own process, so every test makes the builds again.

Measured on pull request 846's first CI run under the limits (run 46063361909, 2026-11-27):

- 78 `go::` tests took 4,060 test-seconds in that run.
- 31 of them ran over 60 s.
- One was killed at its 82 s override; five more ran 70.8-80.6 s.

This is a defect of the test budget: an expensive product is made once, never per test (`AGENTS.md` § Tests stay fast), and no single test runs longer than 60 s. It is one of the per-crate follow-ups that design step 4 of `story:the-test-step-meets-the-test-budget` calls for (`ess-synth`).

## Cause

The harness build sits inside the test body, and the in-process cache that was meant to share it is per process under the runner. The per-test overrides in `.config/nextest.toml` raise the kill above 60 s for the tests that run longest; they hide the cost and are not a fix.

`story:tests-that-build-into-one-shared-scratch-directory-collide-when-each-test-runs-in-its-own-process-one-fixture-is-built-once-per-run-keyed-by-its-sources` names the shared-fixture form the repository accepts, and `story:the-ess-synth-tests-nested-response-wasm-one-time-wasm-and-emit-swap-served-run-within-60-s-because-each-builds-its-wasm-host-or-served-service-once-per-run-not-inside-the-test` applies it to three other `ess-synth` targets. This story applies it to the Go and Rust harnesses of `http_binding_rust`.

## Outcome

The Go harness and the Rust harness of `http_binding_rust` are built once per test run, and every `go::` test reads them. No `go::` test of the target builds a harness itself.

## Acceptance

1. The harnesses are built once per test run: either a cache on disk under `target/` keyed by the digest of their sources, or a setup task before the tests, as `AGENTS.md` § Tests stay fast accepts. A `OnceLock` alone does not meet this.
2. Every `go::` test of `http_binding_rust` runs under 60 s on CI, read from the ten slowest tests the step prints, and none is killed at 60 s.
3. The per-test overrides for these tests in `.config/nextest.toml` are removed in the same change, and the 60 s kill holds for the target.
4. The measure is taken before and after on the build node: the count of `go::` tests over 60 s and their total test-seconds, written beside the run id.
5. Where a `go::` test needs a compile of its own, it moves to the slow lane with its reason written beside it and keeps its assertions.

## Fit review

1. Need: the test step must end within its budget; a test over 60 s is killed and counts as failed.
2. Class: a defect of the test budget, not a feature.
3. Already expressible: yes, by the two accepted forms of the rule (a committed generated file with a digest, or one shared fixture built once per run).
4. Fit: the same shape as the other shared-fixture stories; this story names one more target and its two harnesses.
5. A second adopter: any test that builds a harness in its body.
6. Cost: one cache on disk or one setup task for two harnesses.
7. Considered: raising the per-test overrides (rejected by the rule), moving the whole target to the slow lane without a shared build (kept as the fallback of acceptance item 5).

## Decisions

- None open. One story for both harnesses of one target, because one cause is plain: a build inside the test body, with a cache that is per process.

## Scope

- `crates/generate/ess-synth/tests/http_binding_rust*` and its harness helpers
- `.config/nextest.toml`
- `Taskfile.yml`, if a setup task is added
