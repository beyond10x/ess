---
format: aep.planning-md/2
id: story:when-subject-witness-and-diagnostics
kind: story
status: draft
title: when_subject branches are witnessed without an immediate view, and ESS-SYNTH-008 names an author action
refs:
- provider: github
  reference: beyond10x/ess#172
- provider: github
  reference: beyond10x/ess#173
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

#172: subject-fact selection witnessed through eventual views (or the fallback branch at least), so reachable states after the branch stay reachable. #173: `StrategyWithoutGuard` in `crates/verify/ess-conformance/src/synthesize.rs` (~789) prints guidance, or the refusal scenario is written for a guard-less fallback.

## Acceptance

Both repros in #172 and #173 synthesize with no ESS-SYNTH-001/004/008 refusal, or the refusal names a change the author can make.
