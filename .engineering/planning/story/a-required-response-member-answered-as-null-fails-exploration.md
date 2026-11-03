---
format: aep.planning-md/3
id: story:a-required-response-member-answered-as-null-fails-exploration
kind: story
status: draft
title: A required response member answered as null fails exploration
summary: The explorers count a null required response member as present; the suite refuses it (W2 EXP1 review finding, pre-existing, backlog)
tags:
- backlog
- from-finding
- review-finding
relations:
- informed_by: review-result:adversary-w2-exp1-pass-1
revision: 1
---
## Need

The generated explorer (Go and TypeScript) checks that a `returns: true` outcome's answer carries
every required member of the command's `response` (wave W2,
story:go-explorer-compares-direct-response-members). A required member answered as JSON `null`
counts as present: `explore.go:2690` and `explore.ts:2314` test the key only. The suite itself
refuses that answer as `invalid_input` (`crates/verify/ess-conformance/src/go/runtime.go:8738`), so
exploration passes a target the suite fails. Found by the W2 EXP1 adversary pass
(review-result:adversary-w2-exp1-pass-1, F2); pre-existing at 4a0d58c611, where the explorers read
no response at all.

## Acceptance

- Exploring a target that answers a required (non-`Optional`) direct-response member as `null`
  fails, naming the outcome and the member, in the Go and the TypeScript explorer alike.
- An `Optional` member answered as `null` or left out still passes.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.

## Repro

The adversary's red case `a_required_response_member_answered_as_null_fails_exploration_naming_it`
(Go lane, fixture `tests/fixtures/explore-direct-response.yaml`, a target answering `token: null`)
was left out of the committed test file because the finding is outside W2's scope.
