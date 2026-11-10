---
format: aep.planning-md/3
id: story:the-sequential-and-concurrent-explorers-exclude-alike
kind: story
status: draft
title: The sequential and concurrent explorers exclude alike
tags:
- defect
- severity-low
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/explore.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/explore.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/explore_stored_rows.rs
revision: 8
---
## Outcome

The generated sequential and concurrent explorers exclude a command for the same reason, and report
the same first reason, except where the concurrent design needs a different answer and names it
(`preserves`; a delete, whose deleted row a workload drawn before any call runs can still name).
Measured over the 39 explorer fixtures at 0.60.0, about 25 commands differ: the concurrent
explorer excludes every condition read from the stored row; only the sequential one excludes "no
actor may invoke it", `affects:` and `replays`; and the two name a different first reason for the
same command.

## Problem

Still so on main (0.92.0), in both generated runtimes; each split is a `concurrent` flag in one
plan function:

- Stored-row conditions (`existing_instance` and the row kinds) are decided only when sequential:
  `stored := !concurrent && …` in `crates/verify/ess-conformance/src/go/explore.go`
  (`explorePlanAs`) and `crates/verify/ess-conformance/src/ts/explore.ts` (`plan`), so the
  concurrent explorer excludes such a command as "outcome `x` has a `<kind>` condition".
- `affects:` and `instances:` are excluded only when sequential (`exploreEffectRefusal` /
  `effectRefusal`: `… && !concurrent`).
- `replays` is excluded only when sequential (`!concurrent && outcome["replays"] != nil`).
- "no actor may invoke it" only when sequential (Go `later == "" && !ok && !concurrent`; TypeScript
  `actorFor.get(…) ?? (concurrent ? '' : undefined)`).
- A newtype with invariants or an alphabet is undrawable only when sequential
  (`exploreResolveAs` / `resolveKind`, `!concurrent && …`).

The concurrent explorer is judged by `ess verify conform check-history`, not by the embedded model,
which is why it can draw what the sequential model does not apply (`affects:`, `replays`); the
stored-row exclusion has no such reason and only loses coverage. A command excluded by one explorer
and drawn by the other is therefore covered in one mode only, and an adopter reading the two
exclusion lists sees two first reasons for one command.

## Acceptance

- `cargo test -p ess-conformance --locked --test explore` passes
  `sequential_and_concurrent_explorers_exclude_alike`: for every fixture model the explorers
  explore, in Go and in TypeScript, the sequential and the concurrent explorer report the same
  excluded commands with the same first reason, except the differences a list in the test names,
  each with its reason in the runtime source (at least `preserves` and `deletes` on a supplied
  subject).
- The same test asserts that Go and TypeScript give the same exclusion list, reason for reason, for
  each fixture in each mode.
- `cargo test -p ess-conformance --locked --test explore_stored_rows` passes a case in which a
  command with an `existing_instance` outcome and one with a `subject_state` outcome are in neither
  mode's exclusion list, and every concurrent history recorded against the correct target is
  accepted by `ess verify conform check-history` (0 violations, 0 unknown), in Go and in
  TypeScript.
- No file under `crates/verify/ess-conformance/tests/` is added with a `.rs` extension
  (`git diff --stat --diff-filter=A` against the base lists none): the cases are added to the
  existing `explore` and `explore_stored_rows` binaries.

## Decisions

- The concurrent explorer draws a stored-row condition rather than both explorers excluding it,
  because `check-history` runs every linearization through the interpreter, which decides the
  condition against the row that order holds (`crates/verify/ess-conformance/src/linearize.rs`),
  so the exclusion only loses coverage.

## Scope

- cited: crates/verify/ess-conformance/src/go/explore.go
- cited: crates/verify/ess-conformance/src/ts/explore.ts
- inferred: crates/verify/ess-conformance/tests/explore.rs
- inferred: crates/verify/ess-conformance/tests/explore_stored_rows.rs

confidence: medium

Shared surfaces: no format row and no byte-pin row; it proposes its `CHANGELOG.md` sentence.
