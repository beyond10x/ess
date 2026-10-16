---
format: aep.planning-md/3
id: story:released-chart-equals-its-runtime-projection
kind: story
status: draft
title: Release verification refuses a chart that is not its runtime's projection
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Outcome

Release verification refuses a `helm_chart` artifact that is not exactly the projection of the
release's runtime IR, so a stale or hand-edited chart is caught when the release is verified, not
first at `deployment reconcile`.

## Evidence

Reported by an adopter on ess 0.59.0 (as "a chart output cannot name its runtime IR") and
reproduced on 0.60.0. A link from the build to the runtime is impossible (the runtime carries
`build_digest`), but `verify_release` (`crates/generate/ess-deployment/src/release.rs`) compares
digests and artifact kinds and never the chart's contents. The equality check exists only at
reconcile (`crates/edge/ess-cli/src/recovery/chart.rs`: every file must equal `project_helm` of
the pinned runtime).

## Acceptance

- `release_with_projected_chart_verifies`
- `release_with_hand_edited_chart_refused`
- `release_with_chart_from_other_runtime_refused`
