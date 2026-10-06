---
format: aep.planning-md/3
id: story:specification-declares-its-ess-release
kind: story
status: implemented
title: A specification declares the ess release it is maintained with
refs:
- provider: github
  reference: beyond10x/ess#106
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:42:42Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:43:26Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:45Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A specification declares the ess release it is maintained with.

## Why

GitHub issue beyond10x/ess#106; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#106 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.

## Reconciliation 2026-10-02

The release requirement shipped and evolved before published0.51.0. The adopted design uses ess-inputs/2 for requires: ess X.Y[.Z], and output-state/2 for producer attribution. Tests requires_release.rs:110/:133/:177/:260/:296 cover older refusal with upgrade guidance, newer once-only warning/strict refusal, no-requirement compatibility, producer changes and old output-state reading. No-op regeneration intentionally remains silent and retains its previous producer, as the design/test state.

Subsequent issue147 changes exact pins discovered in the working directory/ancestors to install/delegate to the pinned release before ordinary CLI processing (toolchain.rs:128, tests:217/:237/:311; design section Any ess runs the pinned release). Minor-line pins retain the earlier check. The explicit --path tests put manifests below the working directory to exercise enforcement without nearest-manifest delegation. This evolution supersedes the original exact-pin warning/refusal path where delegation applies; it is not missing implementation.

Keep the current documented versioning, no-op behavior and exact-pin delegation. Do not restore an obsolete behavior to close an old literal acceptance. Original red-first run evidence was not located; lifecycle remains active with this process-evidence qualification, while implementation/release status is recorded as delivered.
