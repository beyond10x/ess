---
format: aep.planning-md/3
id: story:a-null-newtype-over-json-member-is-judged-alike-in-go-and-typescript
kind: story
status: draft
title: A null newtype-over-Json response member is judged alike in Go and TypeScript
summary: The Go suite refuses a null newtype-over-Json response member before resolving the newtype; the TypeScript suite resolves it and accepts null (turn 1 null-member adversary, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:zi-w1-null-adv-1
revision: 1
---
## Need

Found by review-result:zi-w1-null-adv-1 while attacking
story:a-required-response-member-answered-as-null-fails-exploration. For a REQUIRED response member
whose type is a newtype over `Json`, answered as JSON null, the Go suite checks for nil before it
resolves the newtype and refuses the answer (`crates/verify/ess-conformance/src/go/runtime.go:8741`),
while the TypeScript suite resolves the newtype first and accepts it
(`crates/verify/ess-conformance/src/ts/direct_response.ts:201-203`). The same target passes in one
language and fails in the other. The explorers (turn 1) keep the null test for a newtype over Json,
matching Go.

## Acceptance

- The Go and TypeScript suites give the same verdict for a REQUIRED newtype-over-Json member
  answered as null, and the verdict matches what the explorers decide.
- A parity test runs both languages on that answer.

## Scope

- `crates/verify/ess-conformance/src/go/runtime.go` (cited by the review)
- `crates/verify/ess-conformance/src/ts/direct_response.ts` (cited by the review)
