---
format: aep.planning-md/3
id: story:go-explorer-compares-direct-response-members
kind: story
status: implemented
title: The Go explorer compares direct response members
tags:
- adopter-report
- defect
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/explore.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/explore.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/explore-target.mjs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/explore_target.go
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T05:57:21Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-11-03T05:57:21Z", actor: "human:timo", revision: 11}
- {from: "active", to: "implemented", at: "2026-11-03T10:46:42Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

The generated Go explorer compares a command's direct response members with the model's, so a
target that omits a REQUIRED member, or answers a different value the model determines, fails
exploration.

## Evidence

An adopter on ess 0.56.0: a server that omits a REQUIRED response member passes 12,000 explorer
steps (the synthesized suite still catches it). On `main`,
`crates/verify/ess-conformance/src/go/explore.go:2013-2053` compares only the outcome, the error,
the event names and the event payload; nothing in `explore.go` reads the command response.

## Acceptance

- A test runs the Go explorer against a target that drops a REQUIRED direct response member; the
  exploration fails naming the member.
- A member whose value the model does not determine (a `{generated: true}` field) is checked for
  presence only.

## Scope

Derived 2026-11-03 by `aep:story-scoper` for wave W2 on b89faaa41. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance` (the sequential Go explorer's per-step check, `explorePerform`) — cited
- **Files:** `crates/verify/ess-conformance/src/go/explore.go:2560-2614` — cited; the check compares outcome (2567), error (2571), event names (2578-2586) and determined payload fields (2588-2613), and no line in the file reads `CommandResult.Response`
- **Also likely:** `crates/verify/ess-conformance/src/ts/explore.ts` (`perform`, :2191) — inferred; `explore.go` is a port of it "function for function", and `tests/explore.rs:386` holds both languages to one result
- **Also likely:** `crates/verify/ess-conformance/tests/fixtures/explore_target.go`, `tests/fixtures/explore-target.mjs`, `tests/explore.rs` — inferred
- **Symbols:** `CommandResult.Response` (`src/go/runtime.go:1339`), read by the suite's `responseObservation.compare` (`src/go/response.go:345`), not by the explorer — cited
- **Documents:** none
- **Confidence:** high for the check site; the value clause ("answers a different value the model determines") has no route — `exploreValue` (`explore.go:1670-1692`) has no response case — inferred
- **Would collide with:** any unit editing `explorePerform` / `perform`, the explorer fixtures or `tests/explore.rs` — inferred
- **Safety fact:** a new check reads only the command's own response and fails only on a missing REQUIRED member, so runs with every member present are unchanged — inferred
- **Not established:** whether the story is presence-only; "REQUIRED" read as non-Optional (`validateValue`, `runtime.go:8725`); the story's cite `explore.go:2013-2053` is stale (the comparison is at 2560-2614); whether `ExploreConcurrent` (`explore.go:4558`) is in scope.
- **Size:** medium, about 250-350 hand-written lines; no format version (`ORDINARY = 28`, `src/direct_response.rs:12`) — inferred
- **Targeted run:** ess-conformance `--test explore` — inferred

## Delivered

W2 (D408), 2026-11-03: the generated explorer (Go and TypeScript) fails a target whose answer to a `returns: true` outcome leaves out a required member of the command's `response`, naming the outcome and the member; values are not compared. Unit ab6d56ac01, README 15ce4749c6; merged 3cc17ed596. Known limit: a required member answered as `null` passes, tracked as backlog draft story:a-required-response-member-answered-as-null-fails-exploration.
