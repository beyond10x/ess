---
format: aep.planning-md/3
id: story:the-wasm-host-an-adversary-test-builds-regenerates-its-lock-file-when-the-workspace-version-moves-not-only-when-the-lock-is-missing
kind: story
status: draft
title: The wasm host an adversary test builds regenerates its lock file when the workspace version moves, not only when the lock is missing
summary: support_initial_state/mod.rs:268 keeps a Cargo.lock in target/tmp across a version bump; three adversary_287_pass1 cases then fail with the --locked refusal until the directory is deleted
tags:
- defect
- gap-reports-2026-11-24
- test-step
relations:
- serves: vision:O2
revision: 1
---
## Summary

A conformance test that builds a WebAssembly host keeps the host's `Cargo.lock` in
`target/tmp/initial-state-wasm-host/` and regenerates it only when it is missing
(`crates/verify/ess-conformance/tests/support_initial_state/mod.rs:268`). After a workspace version
bump the kept lock names the old version of every `ess-*` crate, and the next build
(`cargo build --offline --locked`, the same function) refuses it: the cases of
`adversary_287_pass1` that use the host fail on every run until someone deletes the directory.

## Evidence

Found on 2026-11-25 in the whole check of a branch at workspace version 0.99.1 on a build node whose
`target/tmp/initial-state-wasm-host/Cargo.lock` had been written at 0.99.0:

- `adversary_287_pass1`: 23 of 26 pass; the 3 host cases (among them `wasm_executes_the_same_shared_row_and_isolation_fixtures`)
  fail on every run with `cannot update the lock file target/tmp/initial-state-wasm-host/Cargo.lock because --locked`.
- Removing the cached directory made the binary pass 26 of 26 at once.
- The branch changes no file under that test; an earlier batch of the same branch passed it on the
  same node.
- CI starts with no `target/`, so CI never meets it; a developer machine or a build node that keeps
  `target/` across a version bump does.

## Red test

A test in `crates/verify/ess-conformance/tests/` that writes a host `Cargo.lock` naming a version
other than the workspace's into the host directory, runs the host build step, and expects the build
to succeed: red today with the `--locked` refusal above, green when the lock is regenerated whenever
it does not match the sources it is built for. It does not run `cargo` for the whole host: the unit
is the function at `mod.rs:268`, or the smallest build of a host crate that shows the refusal.

## Acceptance

- The red test is green: the lock is regenerated when the workspace version, or any `ess-*`
  dependency version the host names, differs from what the kept lock records; a lock that matches
  is kept (the build stays offline and does not regenerate a good one).
- `adversary_287_pass1` passes on a node whose host directory was written by the previous version,
  without deleting it by hand.
- The fix reads the version from the sources the host is built from; it is not a rule that deletes
  `target/tmp` on every run.

## Fit review

1. **Class.** A defect of a test fixture: a cache keyed on existence, not on its sources.
2. **Border.** The shared-scratch races of
   `story:tests-that-build-into-one-shared-scratch-directory-collide-when-each-test-runs-in-its-own-process-one-fixture-is-built-once-per-run-keyed-by-its-sources`
   (the same directory, a different failure: a second process reads the lock while the first
   generates it).
3. **Rule.** The test-budget section of `AGENTS.md` (an expensive product is made once, keyed by the
   digest of its sources): a lock kept without that key is the failing half of it.

## Decisions

- None open.
