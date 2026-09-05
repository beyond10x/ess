---
format: aep.planning-md/1
id: story:review-conformance-format-design
kind: story
status: active
title: Design the conformance suite and report migration
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: docs/design/review-conformance-coverage.md
revision: 6
---
## Finding and source

F03 at `docs/reviews/2026-09-05-architecture-review.md:217`; the current v1 report is `crates/verify/ess-conformance/src/evidence.rs:18`. The scoper independently found different Rust error/unsupported and Go skipped vocabulary.

## Acceptance

A binding migration design specifies unambiguous old/current suite and report semantics for every current producer state and coverage category before a new writer is implemented.

## Required decisions

Name failure, skipped, error and unsupported counts and list semantics without inventing equivalence; preserve aggregate execution verdicts. Define exact-suite digest canonicalization and generated/authored/outside/refused coverage accounting, version-specific validation and strict-mode behavior. Inventory every relying reader and write the staged migration order, including the independently owned AEP adapter. Existing v1 meanings and valid bytes remain frozen.

## Validation

Review a matrix of Rust/Go producer states, complete/partial/empty suites, old/new readers and mismatched digests; every row has one explicit expected result. Identify the Atlas ADR needed for cross-repository byte changes before a new default is released. This is design evidence, not implementation evidence.

## Scope

Derived 2026-09-05 by `story-scoper` from the resolved story and repository sources — cited.

- **Primary surface:** `docs/design/review-conformance-coverage.md` — inferred; create the binding migration design at the story's reserved location.
- **Implementation surface:** none — cited; acceptance requires a reviewed design and migration matrix before implementing new writers.
- **Required content:** version-specific execution counts, outcome-list semantics, exact-suite digest canonicalization, generated/authored/outside/refused coverage, strict-mode behavior, relying-reader inventory and staged rollout — cited; these are the story's required decisions.
- **Evidence references:** Rust and Go producers, suite/report readers, CLI output paths, browser player, semantic-impact consumer and independently owned AEP adapter — cited; inspected sources inform the design without becoming edit scope.
- **Confidence:** high — cited; the acceptance is wholly a document and the producer/consumer boundaries are established in source.
- **Would collide with:** changes to `docs/design/review-conformance-coverage.md` — inferred; no implementation files need changing for this story.

## Ownership

This story owns the common design. The existing skipped-count story owns producer count migration; review-conformance-coverage owns durable coverage and exact-suite binding. The narrow v1 reader validation needs neither design nor a format change.


## Exact AEP reader inspection

The coordinator fetched AEP main and inspected Git object cc321f31fa0120b32a5b9f5e7b8c8fdfa55f69f9 (not its dirty primary files). At crates/observe/aep-ess-evidence/src/lib.rs:15 the adapter accepts only ess-conformance-report/1, with deny_unknown_fields at :20 and count consistency at :144; it transfers scenarios_failed into AEP's EssConformanceResult at :162. A second reader, crates/edge/aep-cli/src/planning.rs:5907 recorded_from_report, independently parses JSON Value, checks only the report format at :5914, refuses zero total at :5929 and composes planning evidence using scenarios_failed at :5956. Both reader paths need inventory and governed compatibility work before any new report default. This inspection is code evidence; no AEP adapter test or change was executed.

## Independent scoping evidence for the design

The read-only scoper returned the following code findings on 2026-09-05. These are design inputs, not executed compatibility tests.

- Rust report.rs:62 combines checks within one scenario as failed > error > unsupported > passed, while report.rs:556 combines scenario outcomes with any failed/unsupported dominating error. Go runtime.go:560/599 uses passed/failed/skipped; ErrUnsupported and unknown runner vocabulary skip, ordinary command errors fail, and EndScenario can replace a skip with failed (:695). Freeze the producer-specific aggregate semantics rather than equate skipped and unsupported.
- ConformanceSuite at scenario.rs:101 persists provenance and scenarios; Synthesis at synthesize.rs:197 separately holds refusals/outside. CLI conform run --path main.rs:2210 discards those fields. Component/authored-only selections can be intentionally partial. Coverage needs an explicit selection boundary, not a nonzero scenario count.
- SuiteFormat deserialization at scenario.rs:423 parses syntax without enforcing supported membership; Runner::run at runner.rs:286 does not check it. Generated Go runtime.go:383/499 uses a reduced JSON struct, ignores unknown fields and never checks SuiteVersion. Browser assets/player.js:28/96/273 directly consumes suite JSON without version or digest admission. A version bump alone cannot guarantee old runtimes refuse new suites; reader-first rollout and regeneration are necessary coordinator inferences.
- CLI detailed JSON/YAML output (main.rs:2238) differs from the standalone --report-out contract. Inventory both. Impact reads suites at ess-diff/src/impact.rs:775/893 using model/contract provenance and scenario dependencies; exact-suite and coverage migration need an explicit disposition there.
- ess-realization/src/lib.rs:378 contains separate ConformanceEvidence suite/report digest references; the scoper found no parser verifying those referenced bytes. Do not claim it is an established report reader or silently equate its digest with a new profile.
- Design a complete matrix: v1 valid reports, new reports, old/new/future suites, unknown fields and vocabulary, empty/partial/component/authored selections, exact-suite mismatches, and actual old generated Go runtimes. New category counts partition the total; list agreement and duplicate scenario IDs must be explicit. New Go digest handling must not hash a lossy decoded struct.

The scoper's local AEP inspection was stale and did not establish the CLI route. The exact remote inspection above supersedes that uncertainty: both the optional adapter and recorded_from_report CLI reader exist at the cited current commit. AEP's closed EssConformanceResult and predicates interpreting scenarios_failed also need a governed disposition, independently of ESS wire parsing. Atlas AGENTS.md:297 at authority 6035d6e1209686ca474a3f43975fde7d8621ba48 requires the ADR, relying-party order, contract version and shipped log.

Final wire shapes, digest canonicalization and incomplete-coverage policy remain decisions for this design story; no new writer or compatibility outcome is claimed by scoping.
