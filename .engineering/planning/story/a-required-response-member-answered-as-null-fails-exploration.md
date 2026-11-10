---
format: aep.planning-md/3
id: story:a-required-response-member-answered-as-null-fails-exploration
kind: story
status: active
title: A required response member answered as null fails exploration
summary: The explorers count a null required response member as present; the suite refuses it (W2 EXP1 review finding, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-high
- zero-open-issues
relations:
- informed_by: review-result:adversary-w2-exp1-pass-1
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/explore.ts
- confidence: cited
  path: crates/verify/ess-conformance/tests/explore_direct_response.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/explore-direct-response-target.mjs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/explore_direct_response_target.go
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:41:26Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:41:27Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

The generated explorer (Go and TypeScript) checks that a `returns: true` outcome's answer carries
every required member of the command's `response` (wave W2,
story:go-explorer-compares-direct-response-members). A required member answered as JSON `null`
counts as present: `crates/verify/ess-conformance/src/go/explore.go:2690`
(`_, present := result.Response[member]`) and `crates/verify/ess-conformance/src/ts/explore.ts:2314`
(`Object.prototype.hasOwnProperty.call(response, field.name)`) test the key only. The suite itself
refuses that answer as `invalid_input` (`crates/verify/ess-conformance/src/go/runtime.go:8741-8743`,
a non-`Optional` value that is `nil`), so exploration passes a target the suite fails. Found by the
W2 EXP1 adversary pass (review-result:adversary-w2-exp1-pass-1, F2); still present on 0.92.0.

Minimal reproduction: the fixture `crates/verify/ess-conformance/tests/fixtures/explore-direct-response.yaml`
with its target (`explore_direct_response_target.go`, `explore-direct-response-target.mjs`) answering
the required member `token` as `null`; both explorers report no disagreement.

## Acceptance

- `crates/verify/ess-conformance/tests/explore_direct_response.rs`, case
  `a_required_response_member_answered_as_null_fails_exploration_naming_it`: a target answering a
  required (non-`Optional`) direct-response member as `null` makes the Go and the TypeScript
  explorer each report a `response` disagreement whose detail names the outcome and the member.
  Failing before the change on the current tree.
- Same file, case `an_optional_response_member_answered_as_null_or_left_out_passes`: an `Optional`
  member answered as `null` or left out reports no disagreement in either language.
- `both_languages_report_the_same_result_for_every_case` keeps passing with the new cases.

## Scope

- cited: crates/verify/ess-conformance/src/go/explore.go (the direct-response check at 2690 tests the key only; block around 2679-2694)
- cited: crates/verify/ess-conformance/src/ts/explore.ts (the same at 2314; block around 2306-2320)
- cited: crates/verify/ess-conformance/tests/explore_direct_response.rs
- cited: crates/verify/ess-conformance/tests/fixtures/explore_direct_response_target.go (modes switch at 50-57; gains the null modes)
- cited: crates/verify/ess-conformance/tests/fixtures/explore-direct-response-target.mjs (modes switch at 39-49; gains the null modes)

Read only: `tests/fixtures/explore-direct-response.yaml` (already declares the `Optional` member `note`, lines 29-30; pinned at row 190 of the bytes table, so left as it is); `src/go/mod.rs:149` and `src/ts/mod.rs:149` embed the explorer sources with `include_str!`; `runtime.go:8741-8743` refuses a nil non-Optional value.

confidence: high — the story names both defect sites, both read as described, and the test and both targets are the existing harness for this exact check (story-scoper, 2026-11-10)

Correction to the earlier pass: the two target fixtures move from inferred to cited (the story's reproduction names both, and the acceptance needs new modes in them).

Shared surfaces this story moves (epic § Shared surfaces, not scope entries): `CHANGELOG.md`. No byte pin moves while the yaml fixture is unchanged.

## Schedule

Waved under epic:zero-open-issues-2026-11 (accepted 2026-11-10).

