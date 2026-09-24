---
format: aep.planning-md/2
id: story:the-metadata-guard-rejects-every-build-but-one
kind: story
status: draft
title: The metadata guard rejects every build but one
scope:
- confidence: inferred
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/metadata_tests.rs
revision: 3
---
# The metadata guard rejects every build but one

`crates/edge/ess-xtask/src/consumer_coverage/metadata_tests.rs:253`,
`current_compiled_provider_executes_one_guard_and_binds_its_opaque_proof_to_this_run`, fails under
any build environment but the exact `env` line in `Taskfile.yml`'s `test:` task.

Reproduced twice, deterministically, by the wave-24 pass-2 adversary:

| Invocation | Result |
|---|---|
| `cargo test -p ess-xtask --locked` | `128 passed; 1 failed` — "unsupported measured compiled profile DEBUG: Some(String(\"true\"))" |
| profile knobs pinned only | fails on "NUM_JOBS: Some(String(\"20\"))" |
| the Taskfile's full `env` line | `129 passed; 0 failed` |

The base file is byte-identical to `bd722fa9`, so this is not a wave-24 change.

The cost is not the gate — `task check` uses the right environment. It is that the per-crate command
every implementor brief prescribes runs this lane red, so an implementor either reports a red it did
not cause or learns to ignore a red. Wave 24 saw both: unit 2 reported it as "pre-existing, left
alone" without naming the case or the environment, and the coordinator then read two green runs
under the pinned profile as evidence the red was imaginary.

## Acceptance

The guard states which build environment it requires and refuses with that sentence, or it tolerates
the environments the repository's own briefs prescribe. A reader of the failure can tell in one line
whether their invocation or their change is at fault.

## Scope

- `crates/edge/ess-xtask/src/consumer_coverage/metadata_tests.rs` — `cited`
- `Taskfile.yml` — `inferred`, if the requirement is to be stated in one place
