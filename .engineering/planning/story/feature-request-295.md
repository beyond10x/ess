---
format: aep.planning-md/3
id: story:feature-request-295
kind: story
status: implemented
title: 'mutate: emit-drop is stillborn on every outcome that emits one event, so single-event emission is never audited'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#295
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:43:16Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-06T09:43:16Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:17Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Audit single-event substitution with an honest unavailable-site result.

## Origin

beyond10x/ess#295. Original issue reproduction is retained in the local triage record; current source and CLI were inspected on the accepted bundle in 2026-10-03.

## Fit review

1. Need: The retained combined CLI reproduces five stillborn emit-drop mutants against a green 32-scenario billing baseline (exit 3). Removing the only event violates ESS-COMMAND-007, so these sites have not been audited.
2. Classification: gap. This is audit/report tooling, with no new authored specification surface.
3. Existing idiom: mutate currently excludes skipped/unsupported baseline scenarios but refuses Failed/Error; no known-failing option exists. EmitDrop compiles source edits normally and fails when it removes the only observable event. Existing primitives do not supply the requested capability.
4. Fit: docs/design/mutation-scope-and-known-failures.md specifies shared native/collected scoring, existing synthesize_for component selection, original-byte admission and ordinary conformance truth. It coordinates manifest/report4 with #236 and accepted #212 operators, preserving older reader behavior. Native and generated Go/TypeScript evidence are required, not count-only claims.
5. Second adopters: an order service with a tracked cancellation defect and a catalog component audited independently from billing.
6. Cost: explicit declaration/accounting1 documents, mutation manifest/report4, CLI flags and strict parsers; no source, ordinary suite or ordinary conformance report version change. Full implementation identity is needed for direct mutation audits; legacy name-only reports stay readable under their old contract.
7. Alternatives: change nothing leaves the gap; weakening expectations or declaring intended behavior optional hides the obligation; a strict-mode waiver changes conformance truth. The proposed design keeps explicit mutation eligibility separate from the ordinary failed result.

## Decisions

Adopt redesigned separate emit-swap operator using an existing compatible event published by all accepting components and ordinary source compilation. Preserve emit-drop. No admissible alternative is an explicit incomplete site, not a kill. Reject dropping suite assertions.

Design is proposed for independent review, not yet approved for implementation. The accepted bundle includes this issue; the binding contract and any review correction must be recorded before dispatch.

## Acceptance

Every applicable named control in docs/design/mutation-scope-and-known-failures.md is required, including actual direct and external emit/collect equivalence, planted faulty scorer/runner controls, strict failed conformance preservation, stale/mismatched declarations, component-crossing commands, and old-reader refusals. Source-only tests and unsupported execution do not satisfy target coverage. For #295 specifically creating/updating/moving swaps, invalid and absent alternatives, wrong-event survivors and discarded-event assertions must execute. For #294/#296, default refusal, excluded-only inconclusive, stale passing entries, unlisted failures and exact suite/spec/build matching must execute.

## Scope

- crates/verify/ess-conformance/src/mutate.rs and report/admission support (cited shared scorer and manifest reader).
- crates/edge/ess-cli/ CLI definitions, mutation/run/report wiring and actual CLI tests (inferred entry points; locate exact modules before edits).
- crates/verify/ess-conformance/tests/ and generated Go/TypeScript acceptance (cited runtime/report contracts).
- docs/design/mutation-scope-and-known-failures.md and affected website guides/references (cited design).

## Design review revisions

The first independent review required three corrections, now specified in docs/design/mutation-scope-and-known-failures.md. Component mutant selection remains changed-scenario based; whole-system refusal deltas cannot attribute a mutant. Selected commands without executable scenarios are explicit unavailable obligations outside the mutant denominator. Manifest4 collects exact-suite-bound report2 only; legacy report1 remains confined to legacy manifests. Known-failure declarations bind a separate public build digest, supplied by the execution host before target calls, using closed ess-conformance-execution/1 sidecars for external collected reports. Protected one-time target identity stays redacted. This supersedes the earlier name/version-only assumption and adds the explicit execution-context format/cost. Native executable identity and generated/external host provenance require changed-build and private-sentinel controls. The proposal still requires its final independent design review before implementation dispatch.
