---
format: aep.planning-md/3
id: story:feature-request-294
kind: story
status: active
title: 'mutate: no way to declare a known-failing baseline scenario; one failure refuses the whole audit (ESS-MUTATE-001)'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#294
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T15:00:14Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-04T15:00:14Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Audit eligible witnesses with explicit known-failing baseline declarations.

## Origin

beyond10x/ess#294. Original issue reproduction is retained in the local triage record; current source and CLI were inspected on the accepted bundle in 2026-10-03.

## Fit review

1. Need: A single known-failing scenario currently refuses the complete mutation audit as ESS-MUTATE-001. The accepted need is to retain that default while explicitly excluding exact known failures from eligible witnesses.
2. Classification: gap. This is audit/report tooling, with no new authored specification surface.
3. Existing idiom: mutate currently excludes skipped/unsupported baseline scenarios but refuses Failed/Error; no known-failing option exists. EmitDrop compiles source edits normally and fails when it removes the only observable event. Existing primitives do not supply the requested capability.
4. Fit: docs/design/mutation-scope-and-known-failures.md specifies shared native/collected scoring, existing synthesize_for component selection, original-byte admission and ordinary conformance truth. It coordinates manifest/report4 with #236 and accepted #212 operators, preserving older reader behavior. Native and generated Go/TypeScript evidence are required, not count-only claims.
5. Second adopters: an order service with a tracked cancellation defect and a catalog component audited independently from billing.
6. Cost: explicit declaration/accounting1 documents, mutation manifest/report4, CLI flags and strict parsers; no source, ordinary suite or ordinary conformance report version change. Full implementation identity is needed for direct mutation audits; legacy name-only reports stay readable under their old contract.
7. Alternatives: change nothing leaves the gap; weakening expectations or declaring intended behavior optional hides the obligation; a strict-mode waiver changes conformance truth. The proposed design keeps explicit mutation eligibility separate from the ordinary failed result.

## Decisions

Adopt redesigned external ess-known-failures/1 declarations and shared direct/collect eligibility. A matching declaration never makes conformance pass. Manifest-bound declarations prevent changing policy after mutant results are observed.

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

## Final design review correction

Final independent design pass2of2 returned needs-revision with one compatibility blocker: generated runners produce five-category report2, while report --results admits four-category results1. The three earlier findings were verified closed. The exact final report is retained in review-result:mutation-audit-design-20261003-r2; its finding is not rewritten as an approval.

Coordinator correction binds a separate accounting-only report mode taking original report2, execution1, exact suite and declaration. It invokes the existing CountReport original-byte reader, preserves GoV2 Skipped and every original status/count/coverage field, writes only accounting1, and refuses conflicting caller provenance/output flags. The existing external results1 path retains its original profile and meaning. This changes the design because of that finding; no third full design pass is requested. The normal implementation review must independently prove the reported seam using actual generated Go/TypeScript Failed/Skipped/Unsupported output. No accounting implementation or source acceptance is claimed by this correction.
