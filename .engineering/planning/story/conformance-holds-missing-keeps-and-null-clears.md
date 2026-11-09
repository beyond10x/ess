---
format: aep.planning-md/3
id: story:conformance-holds-missing-keeps-and-null-clears
kind: story
status: draft
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
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/command.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/
- confidence: inferred
  path: crates/verify/ess-conformance/tests/keep_on_missing.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs
revision: 4
---
## Need

Once `missing: unchanged` and `sent()` validate (`story:an-input-keeps-missing-and-null-apart`), the
interpreted target must carry the third input state and the suite must hold an implementation to
it: a target that clears on missing, or keeps on `null`, fails. Today the interpreter reads `null`
as absent (`crates/verify/ess-conformance/src/input.rs:1789`), and a suite never carries an
`omitted_or_null` member's value (`ess/31` entry of `ess specify formats`,
`triage/678/repro/run.txt`).

## Acceptance

- `ess verify conform synthesize` on the reproduction (`EditNote` with
  `sets: {summary: {input: summary, missing: unchanged}}`) emits three scenarios against a note
  created with summary "old": member missing, then `NoteDetails` shows "old"; `null`, then
  `summary` is absent (`null` in the row, `null_when_absent`); "new", then "new".
- A `sent(summary)` guard gets a scenario per side: sent (`null` and a value) and not sent.
- `ess verify conform run --target interpreted --report-format 2` passes all of them; interpreter
  mutants that clear on missing, or keep on `null`, each fail one (ADR, Acceptance evidence).
- The Go and TypeScript conformance runtimes send a missing member and a `null` member as written.
- A specification without either construct keeps its suite bytes (`synthesis_suite_bytes_table`
  unchanged); if a step must newly distinguish a missing member from `null`, the suite format pair
  moves as the design page decides.
- Tests in `crates/verify/ess-conformance/tests/keep_on_missing.rs` (new).

## Scope

- cited: crates/verify/ess-conformance/src/input.rs (null read as absent, 1789-1803)
- cited: crates/verify/ess-conformance/src/interpret/command.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/go/runtime.go
- inferred: crates/verify/ess-conformance/src/ts/ (request encoding)
- cited: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs (unchanged rows)
- inferred: crates/verify/ess-conformance/tests/keep_on_missing.rs (new)
confidence: medium

## Fit review

1. **Need.** The suite checks the three-way effect and the guard.
2. **Class: gap** (follows from the construct; ADR Effect, "Synthesis and conformance").
3. **Already expressible?** No: no scenario sends `null` apart from missing today.
4. **Fit.** Reuses the view read-back after a step and the per-branch witnesses; three values per
   copy, as the ADR states.
5. **Second adopter.** The JSON Merge Patch endpoint of the parent story.
6. **Cost.** New scenarios only for specifications using the constructs; possibly a suite format
   pair.
7. **Alternatives.** (a) Leave it unchecked: the construct would be documentation. (b) Chosen.

## Decisions

**accept, redesigned**, as the parent story (ADR `a-missing-input-keeps-and-null-clears`, option A).

## Source

github:beyond10x/ess#678
