---
format: aep.planning-md/1
id: story:conformance-compact-json-output
kind: story
status: implemented
title: Write fresh conformance suites as deterministic compact JSON
tags:
- priority-high
relations:
- decomposes: task:ess-gaps-measured-in-a-consumer-specification
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/coverage.rs
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/coverage_cli.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/coverage_lineage.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: website/docs
- confidence: cited
  path: website/docs/guides/verify-conformance.md
revision: 6
---
## Outcome and priority

Next small delivery: add an explicit compact output option to newly synthesized ordinary and coverage IR suites, preserving the current pretty default and every existing obligation. This addresses the formatting portion of gap 8; it does not claim to deduplicate repeated negative obligations.

## Established boundaries

The CLI currently writes ConformanceSuite::to_canonical_json (ess-cli/src/main.rs:425-442,2888-2934; ess-conformance/src/scenario.rs:163-194). Fresh coverage output writes admitted bytes (ess-cli/src/coverage.rs:51-59). AdmittedSuite hashes the original UTF-8 bytes (ess-conformance/src/admission.rs:117-134), and report/2 binds that exact version/digest (counts.rs:245-251). Therefore compact output preserves model/contract digests and meaning, but MUST receive a different exact suite digest. The archived requirement for all digests to remain unchanged is corrected.

## Acceptance

This story is complete when the two measured adopter fixtures can be regenerated through the explicit compact IR option with fewer bytes and identical admitted obligations and execution outcomes, with reports bound to those new exact bytes, unchanged pretty defaults, and the measurements and compatibility checks below recorded.

Required verification:

- An explicit output option applies to fresh --target ir ordinary and coverage generation, with unambiguous applicability and newline rules. Legacy default bytes and to_canonical_json stay unchanged.
- Compact output preserves ordered keys/arrays, scalar/string values, scenario IDs and steps, coverage/refusals, source/model/contract identity and execution outcomes. Repeated generation is byte-stable; old Rust and Go readers admit the same suite version.
- Admission and reports bind the actual compact bytes. Coverage selection and parent lineage remain exact; pairing a compact run with stale pretty-file evidence is refused.
- Measure before/after bytes and scenario/step counts for the same two adopter fixtures. Do not promise the reported 40 percent until its per-file denominator is verified.
- Document opt-in regeneration and renewed exact-artifact evidence. No new suite/source/report version merely for whitespace; Go embedding and rewriting already admitted inputs are outside this first delivery.

A separate measured follow-up may evaluate repeated expect_no_event sets. Changing their representation, order or timing requires its own design and compatibility evidence; formatting alone does not fulfill that broader storage claim.

## Historical source

Archived argument: story:a-synthesized-suite-has-a-compact-form. Its original acceptance is evidence to assess, not a legal lifecycle path to reactivate.


## Provenance and delivery

Decomposes task:ess-gaps-measured-in-a-consumer-specification under initiative:ess-evolution. The operator prioritized these gaps on 2026-09-11. Source-only scoping at dcdc3343 is retained in local-evidence:ess-evolution-20260910/priority-wave/gap-scoping/time-size.md; original archived argument bytes and hashes are in gap-scoping/source/manifest.json. Original archived stories remain terminal and untouched; this is an explicitly distinct implementation/design unit, not an illegal unarchive. Planning does not claim implementation or a rerun of consumer measurements. Integrate overlapping CLI/compiler/conformance changes after the active accessor candidate; source overlap does not depend on finishing its separate downstream adoption proof. No full local workspace/ownership gate or unchanged reruns. Consumer-only PR pipeline failures remain accepted, with core feature checks retained.

## Scope

- Cited: crates/edge/ess-cli/src/main.rs.
- Cited: crates/verify/ess-conformance/src/scenario.rs.
- Cited: crates/edge/ess-cli/src/coverage.rs.
- Inferred: crates/edge/ess-cli/tests/coverage_cli.rs.
- Inferred: crates/edge/ess-cli/tests/coverage_lineage.rs.
- Inferred: website/docs.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 e2c842f374fbc5fc374bd24887d281419705971283dd1707f35d2df1cb549dd0, retained as local-evidence:runtime-gaps/publication-replay/snapshots/e2c842f374fbc5fc374bd24887d281419705971283dd1707f35d2df1cb549dd0.md. Source creation recorded at 2026-09-11T00:24:34Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
