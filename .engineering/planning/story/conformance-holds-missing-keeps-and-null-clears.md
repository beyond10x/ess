---
format: aep.planning-md/3
id: story:conformance-holds-missing-keeps-and-null-clears
kind: story
status: active
title: Conformance holds missing keeps and null clears
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#678
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-input-keeps-missing-and-null-apart
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/command.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/keep_on_missing.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:43:25Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:43:28Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

Once `missing: unchanged` and `sent()` validate (`story:an-input-keeps-missing-and-null-apart`), the
interpreted target must carry the third input state and the suite must hold an implementation to
it: a target that clears on missing, or keeps on `null`, fails. Today the interpreter reads `null`
as absent (`crates/verify/ess-conformance/src/input.rs:1789`), and a suite never carries an
`omitted_or_null` member's value (`ess/31` entry of `ess specify formats`,
`triage/678/repro/run.txt`).

## Acceptance

All tests below are in `cargo test -p ess-conformance --locked --test keep_on_missing` (new file),
each on the reproduction (`EditNote` with `sets: {summary: {input: summary, missing: unchanged}}`,
the note created with summary "old") unless it names another model.

- `ess verify conform synthesize` emits three scenarios for the copy: member missing, then
  `NoteDetails` shows "old"; member `null`, then `summary` is absent (`null` in the row,
  `null_when_absent`); member "new", then "new".
- A `sent(input.summary)` guard gets three witnesses: sent as `null`, sent with a value, and not
  sent.
- `ess verify conform run --target interpreted --report-format 2` passes every scenario of the
  reproduction's suite.
- An interpreter mutant that clears on a missing member fails the missing-member scenario, and one
  that keeps on `null` fails the `null` scenario (ADR `a-missing-input-keeps-and-null-clears`,
  Acceptance evidence).
- The Go and TypeScript conformance runtimes send a missing member left out and a `null` member as
  `null`: a test target that records the request body it received sees `{}` and
  `{"summary": null}` for the two scenarios.
- A suite holding a step whose input leaves out or nulls a member read by `missing: unchanged` or
  `sent()` is written as `ess-conformance/64` (ordinary) and `ess-conformance/65` (coverage); a
  runner of the formats before them refuses it by format name.
- A specification without either construct keeps its suite format and suite bytes:
  `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes with no row
  moved.

## Scope

- cited: crates/verify/ess-conformance/src/input.rs
- cited: crates/verify/ess-conformance/src/interpret/command.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/go/runtime.go
- cited: crates/verify/ess-conformance/src/ts/runtime.ts
- inferred: crates/verify/ess-conformance/tests/keep_on_missing.rs
confidence: medium

Landing points: `input.rs` reads `null` as absent at 1789-1803; `go/runtime.go` sends the input at
`ExecuteCommand` (:2512); `ts/runtime.ts` at `executeCommand` (:3933).

Shared surfaces: proposes its `CHANGELOG.md` sentence and its rows of the `ess-conformance/64` and
`/65` format entries; moves no suite-bytes row.

## Fit review

1. **Need.** The suite checks the three-way effect and the guard.
2. **Class: gap** (follows from the construct; ADR Effect, "Synthesis and conformance").
3. **Already expressible?** No: no scenario sends `null` apart from missing today.
4. **Fit.** Reuses the view read-back after a step and the per-branch witnesses; three values per
   copy, as the ADR states.
5. **Second adopter.** The JSON Merge Patch endpoint of the parent story.
6. **Cost.** New scenarios only for specifications using the constructs; the suite format pair
   `ess-conformance/64` / `65` for the suites that hold them.
7. **Alternatives.** (a) Leave it unchecked: the construct would be documentation. (b) Chosen.

## Decisions

**accept, redesigned**, as the parent story (ADR `a-missing-input-keeps-and-null-clears`, option A).

- A step that tells a missing member from `null` moves the suite to `ess-conformance/64` / `65`,
  because the `ess/31` entry says a suite "never carries the value" and a runner of the earlier
  formats that drops a `null` member would pass a target that keeps on `null`; refusing such a suite
  by format is the only way that runner cannot pass it silently.

## Source

github:beyond10x/ess#678
