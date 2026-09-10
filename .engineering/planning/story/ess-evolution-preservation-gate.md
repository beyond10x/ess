---
format: aep.planning-md/1
id: story:ess-evolution-preservation-gate
kind: story
status: implemented
title: Enforce the ESS feature-preservation mapping during consumer checks
relations:
- decomposes: initiative:ess-evolution
- informed_by: architecture-decision-record:ess-evolution-01-evolve-without-feature-loss
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/mod.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/preservation.rs
- confidence: cited
  path: docs/design/ess-evolution/feature-preservation.md
revision: 7
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

## Implementation progress

Implemented the early preservation validator in consumer_coverage/preservation.rs, invoked before extraction in check_at. Six regression tests mutate the real mapping and source authorities; all passed, as did strict Clippy. A real task consumer-check with the first mapping row removed exited 1 with missing consumer cli-binding-resolution, before creating any extraction or cases directory. The original mapping was restored byte-for-byte. The final task check exited 0 on staged tree fb5a6b8b9e732af6cf3c84e0412ae479624f54da, with TMPDIR=/tmp. All 152 exact consumer cases and the metadata guard passed; the 4,305 Supported, 1,182 Refused, 157,677 BaselineUnknown and six SchemaDocumentMetadata cells remain unchanged. task site-build also passed. Evidence is retained under local-evidence:ess-evolution-20260910/: ess-preservation-unit.log, ess-preservation-command-mutation.log, ess-preservation-gate.log and ess-preservation-site.log. Final evidence and lifecycle receipts are appended after verification; they do not change the validated implementation.

## Finite helper review

The full gate passed every workspace test and refused during finite entry classification, before consumer execution. The extracted new production entries are exactly preservation::module, preservation::fn::check and preservation::fn::validate. Review of their complete source establishes inventory JSON reading and consistency validation only: no RawSpecFile/EssIr interpretation or semantic-support claim. Classify these three exact entries as OwnedHelper in entry-classifications.json, consistent with adjacent gate accounting helpers. This adds no consumer profile and changes no existing classification or eligibility. The earlier run is not passing full-gate evidence.

## Performance finding

The existing consumer qualification runner has excessive provenance overhead. executor::execute_case invokes authority four times per case. On this run, each case hashes at least 818,160,888 bytes across repository sources and shared tools, excluding its retained test executable; 152 cases therefore hash at least 124,360,454,976 bytes. A concrete case executed in 0.003514210 seconds. An isolated SHA-256 measurement on the 39,800,184-byte checker executable took 0.930802 seconds unoptimized and 0.131752 seconds with only sha2 optimized, producing identical digests. This machine lacks the SHA hardware extension. These observations are in local-evidence:ess-evolution-20260910/ess-hash-cost.json. The earlier /tmp ownership result was 39 tests in 47.31 seconds, a distinct suite. No hashing policy or build optimization was changed in this story; improving this excessive cost remains open and must preserve trustworthy result attribution. The measurements do not claim a whole-gate optimized timing.
