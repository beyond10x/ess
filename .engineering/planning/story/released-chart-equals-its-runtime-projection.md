---
format: aep.planning-md/3
id: story:released-chart-equals-its-runtime-projection
kind: story
status: active
title: Release verification refuses a chart that is not its runtime's projection
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-delivery
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/recovery/chart.rs
- confidence: cited
  path: crates/generate/ess-deployment/src/component.rs
- confidence: cited
  path: crates/generate/ess-deployment/src/environment.rs
- confidence: cited
  path: crates/generate/ess-deployment/src/release.rs
- confidence: inferred
  path: crates/generate/ess-deployment/tests/deployment.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:46Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:47Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Release verification refuses a `helm_chart` artifact that is not exactly the projection of the release's runtime IR,
so a stale or hand-edited chart is caught when the release is verified, not first at `deployment reconcile`.

## Evidence

Reported by an adopter on ess 0.59.0 (as "a chart output cannot name its runtime IR"), reproduced on 0.60.0, and
unchanged on main at 0.92.0 (code reading). A link from the build to the runtime is impossible (the runtime carries
`build_digest`), but `verify_release` (`crates/generate/ess-deployment/src/release.rs:170-245`) compares the release's
build, runtime and semantic digests and each artifact's kind and build-output identity, and never the chart's
contents. `ess generate release verify` takes only `--path`, `--build-ir` and `--runtime-ir`
(`crates/edge/ess-cli/src/main.rs:2165-2187`; 0.92.0 `--help`: "Check release metadata consistency against exact build
and runtime IR"), so it has no chart bytes to compare. `verify_release` is also what `release bundle` runs per release
(`crates/generate/ess-deployment/src/component.rs:368`).

The equality check exists only at reconcile: `crates/edge/ess-cli/src/recovery/chart.rs` requires every file to equal
`ess_deployment::project_helm` (`crates/generate/ess-deployment/src/environment.rs:651`) of the pinned runtime
(`recovery/chart.rs:91`).

## Acceptance

`ess generate release verify` (and `release bundle`, through `verify_release`) takes the chart's files where the
release carries a `helm_chart` artifact, and compares each with `project_helm(runtime, name, version)`:

- `release_with_projected_chart_verifies`: a release whose chart directory is `project_helm` of the supplied runtime
  IR verifies, exit 0, output unchanged.
- `release_with_hand_edited_chart_refused`: the same release with one byte changed in `templates/` is refused, naming
  the file, exit non-zero.
- `release_with_chart_from_other_runtime_refused`: a chart projected from another runtime IR is refused, naming the
  first differing file.
- A `helm_chart` release verified without the chart's files is refused by name (no silent metadata-only pass); an
  `oci_image` release needs none.

Tests in `crates/generate/ess-deployment/tests/deployment.rs` (library) and an `ess-cli` command test.

## Scope

- cited: crates/generate/ess-deployment/src/release.rs
- cited: crates/edge/ess-cli/src/main.rs
- cited: crates/generate/ess-deployment/src/component.rs
- cited: crates/generate/ess-deployment/src/environment.rs
- inferred: crates/edge/ess-cli/src/recovery/chart.rs (share its five-file comparison)
- inferred: crates/generate/ess-deployment/tests/deployment.rs

confidence: medium
