---
format: aep.planning-md/1
id: story:provision-wasm-for-gate
kind: story
status: active
title: Provision the mandatory WASM compiler target in CI
relations:
- serves: vision:O2
- informed_by: story:review-rust-target-feasibility
scope:
- confidence: cited
  path: .github/workflows/ci.yml
revision: 4
---
## Evidence

The mandatory feasibility tests now compile generated workspaces for
wasm32-unknown-unknown (crates/generate/ess-synth/tests/feasibility.rs:383 and
crates/generate/ess-synth/tests/feasibility_adversary.rs:106).
The shared CI job provisions rustfmt and clippy but declares no WASM target
(.github/workflows/ci.yml). The documentation job already provisions that target
with the same pinned Rust setup action (.github/workflows/release.yml).

Local gates pass with the target installed. At discovery the published Wave 5 CI
run was still executing; this record does not claim an observed CI failure.

## Outcome

The shared CI and release gate explicitly provision the compiler target required
by the existing unconditional generated-WASM tests, without depending on an
incidental hosted-runner installation or skipping those tests.

## Acceptance

The shared gate's pinned Rust setup action installs wasm32-unknown-unknown before
task check, and the actual CI run executes the unchanged compiler tests successfully.

## Scope

Only .github/workflows/ci.yml target provisioning. The separate optional Go and
TypeScript compiler lane remains owned by story:native-realization-ci.
