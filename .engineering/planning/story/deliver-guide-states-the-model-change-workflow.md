---
format: aep.planning-md/3
id: story:deliver-guide-states-the-model-change-workflow
kind: story
status: draft
title: The deliver guide states what a model change costs the deployment chain
tags:
- adopter-report
- deployment-chain
- docs
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The deliver guide says what a model change costs the deployment chain: the runtime's semantic and
realization digests are updated in `runtime.yaml`, the runtime is recompiled and the chart
re-projected, with the commands in order.

## Evidence

An adopter on ess 0.55.0 found it by trial. On main `RuntimeSpec` binds `semantic_digest`,
`realization_digest` and `build_digest`
(`crates/generate/ess-deployment/src/runtime.rs:201-206`) and compile refuses a mismatch
(`runtime.rs:500-504`). `website/docs/guides/deliver/build-and-chart.md:152-158` says where each
digest comes from and `website/docs/guides/record-realization.md` that stale digests are refused;
neither states the workflow after a model change.

## Acceptance

- The guide has a section with the command sequence after a model change, and the refusal an
  adopter sees when a digest is stale.
- Its commands are exercised by the tutorial or guide test that runs documented commands, if one
  covers this page.
