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
  path: crates/verify/ess-conformance/tests/
revision: 5
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

- `sequential_and_concurrent_explorers_exclude_alike`: for every fixture model the explorers
  explore, Go and TypeScript, the two report the same exclusions with the same first reason, except
  a named list of concurrent-only and sequential-only differences, each with its reason in the
  runtime source (at least `preserves` and `deletes` on a supplied subject).
- A stored-row condition is decided by the concurrent explorer against the row each linearization
  holds, or excluded by both with one reason.
- Go and TypeScript give the same exclusion list for each fixture in each mode.
- Tests are modules of existing test binaries; no new `tests/*.rs` file.

## Scope

- cited: crates/verify/ess-conformance/src/go/explore.go
- cited: crates/verify/ess-conformance/src/ts/explore.ts
- inferred: crates/verify/ess-conformance/tests/
- inferred: CHANGELOG.md

confidence: medium
