---
format: aep.planning-md/1
id: story:ess-evolution-preservation-gate
kind: story
status: draft
title: Enforce the ESS feature-preservation mapping during consumer checks
relations:
- decomposes: initiative:ess-evolution
- informed_by: architecture-decision-record:ess-evolution-01-evolve-without-feature-loss
revision: 2
---
## Context
The reviewed evolution candidate adds a 96-consumer feature-preservation mapping with 479 reviewed requirement references. crates/edge/ess-xtask/src/consumer_coverage/mod.rs::check_at currently never loads that mapping. Manual review proved equality with profiles.json and reviewed-candidates.json, but future drift would not fail the existing consumer gate.

## Acceptance
The existing consumer-check command refuses an omitted, duplicated or unknown consumer, changed destination entrypoint or execution/classification boundary, and any dropped or altered reviewed requirement/case/behavior reference in feature-preservation.json, while the unchanged mapping and full existing ESS gate pass.

## Scope
Cited: crates/edge/ess-xtask/src/consumer_coverage/mod.rs::check_at, profiles.json, reviewed-candidates.json, initial-baseline.json and feature-preservation.json. These existing files define the consumer identities and evidence; no new product entity or semantic-support claim is introduced.
Inferred addition: a focused preservation validator module and its regression tests, invoked before consumer extraction/execution. docs/design/ess-evolution/feature-preservation.md will state what is actually checked.

## Boundaries
Do not change supported/refused/unknown eligibility, requirement accounting, canonical generated formats, runtime source or package versions. The mapping remains an additional consistency constraint over the existing source authorities. This is a Phase 0 preservation follow-up; ER asynchronous execution and its Eventlog adapter remain the next runtime prerequisite.

## Verification

Run validator tests against the actual inventories with independent removals/changes, demonstrate that a deliberately removed mapping consumer makes the real consumer-check entrypoint fail before expensive execution, restore the mapping, then run task check. Use TMPDIR=/tmp; the integration fixes the Unix socket fixture to bind directly on its destination filesystem through a short temporary symlink. A source change requires its own final verification after the design integration.
