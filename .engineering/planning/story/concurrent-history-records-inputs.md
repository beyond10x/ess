---
format: aep.planning-md/3
id: story:concurrent-history-records-inputs
kind: story
status: active
title: A concurrent history records each command's input
summary: ess-history records no inputs, so a Linearizable verdict can be wrong for an input-dependent fault
relations:
- serves: vision:O2
- informed_by: review-result:linearizability-checker-over-the-interpreter-adversary-pass-2
- depends_on: story:concurrent-history-format
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/check_history.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T20:19:29Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-01T20:19:29Z", actor: "human:timo", revision: 9}
---
# Story: a concurrent history records each command's input

## Outcome

`ess-history/1` records the outcome of each operation but not the input the client sent. The
linearizability checker (`crates/verify/ess-conformance/src/linearize.rs`) therefore explains a step
with any input `witness::candidates` builds for the command. Two consequences, from adversary pass 2
of `story:linearizability-checker-over-the-interpreter` (2026-09-28,
`review-result:linearizability-checker-over-the-interpreter-adversary-pass-2`, finding 4, marked
pre-existing):

- a `Linearizable` verdict can be wrong for a fault that depends on the input — for example a target
  that answers `accepted` to an invalid amount is explained by some valid candidate amount
  (inferred from the code, not run);
- a `Violation` can be wrong for a history that only an input outside the candidates explains.

A format revision that records the input per operation (optional, so existing histories still read)
lets the checker replay the exact input.

## Acceptance

- `ess-history` gains an optional per-operation input; histories without it still read.
- A faulty target that accepts an input the model refuses produces a history the checker reports as
  `Violation` when inputs are recorded, and the same history without inputs still reads.
- The Go and TypeScript writers record the input, and their bytes stay equal for one seed.

## Scope

Derived 2026-11-01 by `story-scoper` against `integration/ess-d357` at `78d58fb47` (ESS 0.81.0).
Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).
This replaces the 2026-09-28 scope: that scope's surfaces were changed by beyond10x/ess#576.

- **Status of the outcome:** delivered by `c85a3be56` "feat(history): ess-history/3 records each call's input and events, and check-history holds them" and `c5ef2ede2` (beyond10x/ess#576, 2026-10-15) — cited
- **Acceptance 1 (optional input; histories without it still read):** `crates/verify/ess-conformance/src/history.rs:330-342` (`Operation::input: Option<BTreeMap<String, Node>>`, `ess-history/3`); lower formats read and write back unchanged, pinned by `crates/edge/ess-cli/tests/explore_concurrent.rs:1805` `an_older_history_reads_and_writes_back_unchanged_and_the_format1_reader_refuses_later_formats` and `:1972` `an_ess_history_3_operation_records_its_input_and_events_and_the_reader_holds_them_to_the_format` — cited
- **Acceptance 2 (checker uses the recorded input):** `crates/verify/ess-conformance/src/linearize.rs:590-592` steps with exactly the recorded input; candidates only when it is absent (`:95-115`). Pinned for a wrong declared branch by `crates/verify/ess-conformance/tests/linearizability_adversary_pass2.rs:581` `a_recorded_input_that_takes_small_does_not_explain_big` and `explore_concurrent.rs:1909` (`wrong-outcome` mutant) — cited
- **Acceptance 2, gap:** no test pins the story's exact case: a target accepts an input the model refuses (an `error:` branch), it is a `Violation` with the input, and the same history without the input still reads. Checked with `git grep` over `ess-history/3` tests — inferred
- **Acceptance 3 (Go/TS writers record the input; bytes equal for one seed):** `crates/verify/ess-conformance/src/go/explore.go:3961,4363`, `crates/verify/ess-conformance/src/ts/explore.ts:3382`; equal bytes asserted at `explore_concurrent.rs:1913-1918` and `:2232` `every_command_call_of_every_lane_records_its_input_and_every_answered_one_its_events` — cited
- **Primary surface (the remaining unit):** `crates/edge/ess-cli/tests/check_history.rs` — inferred: it drives `ess verify conform check-history` on handwritten histories, the cheapest home for the missing regression case
- **Files:** `crates/edge/ess-cli/tests/check_history.rs` (append one test, with a one-command spec inline that refuses `amount < 1`, and three histories: format 3 with input, format 3 with input and events, format 1 without input) — inferred
- **Also likely:** none. History format, checker, writers, schema and model need no change — cited (behaviour probe below)
- **Documents:** none. `ess verify conform check-history --help` already describes stepping with the recorded input — cited
- **Confidence:** high — the tree's commits, tests and a run of the checker show the behaviour; only where the remaining test goes is a choice
- **Would collide with:** only units that append tests to `crates/edge/ess-cli/tests/check_history.rs`. The format, checker and writer surfaces are not touched
- **Safety fact:** the checker already gives `Violation` for an `accepted` answer to `amount: -5` with the input recorded, and `Linearizable` without it. The installed `ess` 0.80.0 was run on probe histories (exit 1 and exit 0). `git diff --stat 0.80.0 HEAD` is empty for `history.rs`, `linearize.rs` and `interpret/`, so HEAD should behave the same; not run on HEAD — inferred for HEAD
