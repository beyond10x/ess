---
format: aep.planning-md/1
id: verification-report:coverage-go-execution-adaptation
kind: verification-report
status: draft
title: Coverage Go execution adapter and future-version decisions
relations:
- verifies: story:review-conformance-coverage
revision: 1
---
# Coverage Go execution adaptation and future-version test decision

Story: story:review-conformance-coverage. Coordinator decision on 2026-09-06 during the
authorized wave 11 implementation; not an executed test or completed source review.

The accepted coverage numeric rule preserves exact unsigned u64 metadata. The transport rule
requires complete original lineage admission before target activity. The inherited Go
Step.After and ScanRequest.StopAfter are int, and selected count/position execution fields also
have narrower host representations. A valid Rust suite/5 containing expect_halt.after=u64::MAX
on the current 64-bit host exposed an early whole-parent Go unmarshal refusal. The timestamp
binding's narrower-adapter clause alone did not explicitly decide this scenario metadata case.

Root recorded D7 in docs/design/review-conformance-coverage-transport.md: preserve exact wire and
lineage admission for every parent, then check conversion of only the selected execution view
against the actual Go int width. A field in an omitted parent scenario must not cause execution
adaptation refusal for a representable child. A selected unrepresentable value refuses explicitly
before target construction, identity callbacks, state or report/output creation, naming its
scenario and field. Do not wrap, clamp, use binary64, or fabricate an unsupported result.
Preserve the existing positive u64 wire vector and add distinct selected-adaptation-negative and
parent-only-positive controls. Original bytes and their digest remain unchanged. The implementor
accepted this decision; the final source and recorded checks still require independent review.

The inherited test a_suite_format_from_a_later_build_is_refused_rather_than_guessed uses /5 as
its unsupported future literal. Root authorized changing only that literal to /6 now that /5 is
the explicitly selected admitted format. Preserve its refusal assertion and every old-version,
report/1 and default assertion. Retain exact before/after test bytes with the final handoff.
This is a concrete expectation adaptation, not permission to weaken unrelated assertions.

The coordinator read the relevant wire/digest, timestamp and transport clauses and inherited
Go types. Initial source searches guessed an absent crates/generate/ess-conformance path; the
actual crates/verify/ess-conformance path was then read. Those read-only search exits are not
product/test failures. No root producer/helper execution was performed for this decision.
