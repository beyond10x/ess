---
format: aep.planning-md/3
id: story:the-typescript-runtime-checks-a-view-rows-shape
kind: story
status: active
title: The TypeScript runtime checks a view row's declared shape
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#680
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-view-read-checks-every-rows-shape
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/mod.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.test.ts
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/view_row_shape_typescript.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:48:37Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:48:39Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

`story:a-view-read-checks-every-rows-shape` puts the view's declared shape on `expect_view` and
`eventually_view` steps under `ess-conformance/64` and `/65` and checks it in the Rust runner. The
TypeScript runtime that `ess verify conform synthesize --target typescript` writes must admit those
majors and apply the same check. Today its version table ends at `'ess-conformance/63'`
(`ts/runtime.ts:7177`), it compares only the named row fields (`ts/runtime.ts:6058` `rowMatches`),
and it already checks an event payload against its shape (`ts/runtime.ts:6426` `holds`).

## Acceptance

- The TypeScript runtime admits `ess-conformance/64` and `/65`, and admits `shape` on
  `expect_view` and `eventually_view` only from /64.
- Run on the suite/64 fixture of `story:a-view-read-checks-every-rows-shape`, it fails the view step
  for a row missing `placed_at` and for `placed_at: 7`, and passes a conforming row, with the same
  verdicts as the Rust runner (a `node --test` case in `ts/runtime.test.ts` and a parity test under
  `crates/verify/ess-conformance/tests/`).

## Scope

- cited: crates/verify/ess-conformance/src/ts/runtime.ts
- inferred: crates/verify/ess-conformance/src/ts/runtime.test.ts
- inferred: crates/verify/ess-conformance/src/ts/mod.rs
- inferred: crates/verify/ess-conformance/tests/view_row_shape_typescript.rs
confidence: medium

## Fit review

1. **Need.** Same as `story:a-view-read-checks-every-rows-shape`, for the TypeScript runtime.
   Requester's proposal (theirs): fix the Go and TypeScript runtimes alike.
2. **Class.** Gap; a runtime ignoring the shape is a silent target (assessing skill, red flags).
3. **Already expressible?** No; see the parent story and `run.txt` § 2.
4. **Fit.** Reuses `holds` (`ts/runtime.ts:6426`) and `admitShape` (`ts/runtime.ts:7922`); no new
   vocabulary.
5. **Second adopter.** Any TypeScript service held to a model with a view, for example
   `example.catalog.Products`.
6. **Cost.** No format of its own; admits the parent's pair.
7. **Alternatives.** Keep TypeScript at /63: refused for the same reason as for Go. Chosen: parity.

## Decisions

Accept, redesigned (the design is the parent story's; this is its TypeScript target).

## Source

github:beyond10x/ess#680
