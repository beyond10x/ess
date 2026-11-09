---
format: aep.planning-md/3
id: story:the-go-and-typescript-runtimes-follow-continuations-and-check-malformed-answers
kind: story
status: draft
title: The Go and TypeScript runtimes follow continuations and check malformed answers
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#750
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:conformance-follows-a-views-continuation-to-its-last-page
- depends_on: story:a-value-outside-its-declared-type-is-answered-malformed
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/mod.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.test.ts
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/go_runtime.rs
revision: 4
---
## Need

The generated Go and TypeScript conformance runtimes execute the `page` expectation at the suite
majors that carry it (`docs/design/view-paging.md`, "The Go and TypeScript runtimes execute the
expectation at `/26` and `/27`"). The new suite pair that
`story:conformance-follows-a-views-continuation-to-its-last-page` and
`story:a-value-outside-its-declared-type-is-answered-malformed` open carries a continuation on a
page and a `malformed` answer. A runtime that does not know them refuses the suite by version, so a
Go or TypeScript implementation cannot be held to continuation paging or to malformed refusals.

## Acceptance

- `ess verify conform synthesize --target go` and `--target typescript` on the reproduction model of
  `story:a-view-answer-takes-its-protocols-list-shape` write packages whose `go test` and
  `npm test` admit the new suite pair.
- The Go target interface's view answer carries `Next *string` and the TypeScript one `next?:
  string`; a target answer of `Malformed{Path, Constraint}` (Go) and `{malformed: {path,
  constraint}}` (TypeScript) is the command's or read's result, as `NotGranted` is.
- Each runtime judges the continuation, last-page and malformed scenarios with the Rust runner's
  verdicts: the package tests run the defective in-memory targets of the two parent stories (first
  page again, continuation always, continuation never, `limit: 201` accepted) and each fails the
  scenario the Rust runner fails.

## Scope

- cited: crates/verify/ess-conformance/src/go/runtime.go
- cited: crates/verify/ess-conformance/src/ts/runtime.ts
- inferred: crates/verify/ess-conformance/src/go/mod.rs
- inferred: crates/verify/ess-conformance/src/ts/mod.rs
- inferred: crates/verify/ess-conformance/src/ts/runtime.test.ts
- inferred: crates/verify/ess-conformance/tests/go_runtime.rs
confidence: low

## Fit review

1. **Need.** A Go or TypeScript implementation is held to the same continuation and malformed
   scenarios as a Rust one. Requester's syntax: none; the request asks for "conformance synthesis
   for the above", which every runner must execute.
2. **Class.** Gap, following the two parent stories.
3. **Already expressible?** Not applicable: the vocabulary does not exist yet.
4. **Fit.** Every target handles a new suite construct or refuses it by name
   (`.agents/skills/assessing-external-requests/SKILL.md`, question 4); the runtimes follow the Rust
   runner's verdicts as they did for `page` (beyond10x/ess#188, `view-paging.md`).
5. **Second adopter.** Any Go service with a paged list read, the protocol list operation of the
   first parent story among them.
6. **Cost.** Generated runtime code; a Go or TypeScript test adapter gains one answer field and one
   refusal variant (additive). No format beyond the parents'.
7. **Alternatives.** (a) Leave the runtimes refusing the new pair by version: a named refusal, but
   it shuts every Go and TypeScript suite out of the pair, paging or not. (b) Execute it: chosen.

## Decisions

accept, redesigned, as the two parent stories.

## Source

github:beyond10x/ess#750
