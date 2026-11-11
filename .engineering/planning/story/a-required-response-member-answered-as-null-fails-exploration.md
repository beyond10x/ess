---
format: aep.planning-md/3
id: story:a-required-response-member-answered-as-null-fails-exploration
kind: story
status: implemented
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
  path: crates/verify/ess-conformance/tests/explore_direct_response_json.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/explore-direct-response-json-target.mjs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/explore-direct-response-target.mjs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/explore_direct_response_json_target.go
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/explore_direct_response_target.go
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:41:26Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:41:27Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-10T17:53:23Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
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

Delivered in turn 1 of the zero-open-issues plan (merge ba749746cd, correction 55e3856e5b). The paths below are the ones
its commits changed, read from the diff, so every entry is cited.

- cited, confirmed: `crates/verify/ess-conformance/src/go/explore.go`
- cited, confirmed: `crates/verify/ess-conformance/src/ts/explore.ts`
- cited, confirmed: `crates/verify/ess-conformance/tests/explore_direct_response.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/explore_direct_response_json.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/fixtures/explore-direct-response-json-target.mjs`
- cited, confirmed: `crates/verify/ess-conformance/tests/fixtures/explore-direct-response-target.mjs`
- cited, confirmed: `crates/verify/ess-conformance/tests/fixtures/explore_direct_response_json_target.go`
- cited, confirmed: `crates/verify/ess-conformance/tests/fixtures/explore_direct_response_target.go`

Shared surface written by the coordinator, not a scope entry: `CHANGELOG.md`.

## Schedule

Waved under epic:zero-open-issues-2026-11 (accepted 2026-11-10).

