---
format: aep.planning-md/3
id: story:the-own-scenario-arrangement-of-a-setup-run-finds-an-equal-arrangement-again-instead-of-building-it-once-per-path
kind: story
status: draft
title: The own-scenario arrangement of a setup run finds an equal arrangement again instead of building it once per path
tags:
- from-finding
relations:
- serves: vision:O2
- informed_by: review-result:wave-2026-11-22-whole-diff-adversary
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/own_scenario_nesting_bounded_work.rs
revision: 6
---
## Outcome

- The nesting bound of the own-scenario fallback (`OWN_NESTING`, 1 in the wave of 2026-11-22) can rise to 2, then 3, at a synthesis cost that grows with the size of the suite and not faster.
- A chain of creators that each read two related rows keeps the scenarios the unbounded fallback wrote, instead of the subset by id that the bound keeps from depth 2 on.
- Every run the strict search finds, and every refusal text, stays as it is.

## Need

Found by the whole-diff adversary review of the wave of 2026-11-22 (`review-result:wave-2026-11-22-whole-diff-adversary`, finding 1, a blocker), and bounded in the wave by the commit `7ea3fb0196` (the unit story `a-creator-reading-several-related-rows-arranges-its-row-for-other-scenarios`). The bound is a stop, not a fix: it is the smallest value that loses no scenario on the specifications measured, and it refuses at depth 2 and deeper what the unbounded fallback witnesses.

Measured on a generic chain fixture, in which each level's own scenario creates the rows of the level below (user CPU in seconds, scenarios / refusals, release build):

- Unbounded fallback, depth 1 to 4: 0.035, 0.236, 2.09, 33.2 s; 9, 15, 21, 27 scenarios; depth 5 did not end in 540 s. The suite grows about 3 times per level (67,951, 249,046, 782,531, 2,360,036 bytes) and the work about 15 times.
- Release 0.97.4, depth 1 to 5: under 0.08 s at every depth, and 1, 6, 11, 16, 21 branches refused.
- Bound 1, depth 1 to 8: 0.04 to 1.1 s, 9 to 20 scenarios, 0 to 31 refusals. Bound 2: 0.04 to 10.7 s. Bound 3: 14.6 s at depth 4, 64.0 s at depth 8.
- One real downstream specification with the bound at 1: 14.1 s against 2.0 s on 0.97.4 (7 times the CPU), 180 against 113 scenarios (1.6 times) and a suite of 4.0 MB against 1.6 MB. Part of the extra CPU is the larger suite; the rest was not profiled. A second real specification is unchanged at 3.7 s.

Why no memory finds an arrangement again: every arrangement is made at a distinction number of its own, derived from the caller's, so two arrangements that read the same rows carry different numbers. A trace of the chain at depth 2 shows 37 arrangements under 37 distinct keys: a memo keyed on what the arrangement reads has 0 hits.
Evidence, relative to the session scratch root: `wave-1122/close/s3fix/REPORT.md` (the tables and the work counts), `wave-1122/close/s3fix/measure.log`, `wave-1122/close/adv/REPORT-2.md` (finding 1 and its scale and chain scripts), the work test `crates/verify/ess-conformance/tests/own_scenario_nesting_bounded_work.rs`.

## Design

Open question, left to the implementor and to a design note beside the unit (D1):

- Number an arrangement by its content: the distinction of an own-scenario arrangement derives from what the arrangement reads and creates, not from the path of the caller that asked, so two equal arrangements carry one number and the existing memos find the second.
- Or share the sub-arrangement: the arrangement is made once per (command, outcome, rows read) and its result is reused by every path that asks for it.

Either must keep the bytes: a suite written with the bound at 1 is byte-identical to the suite written before, and a suite at depth 2 or more is a superset of it by scenario id.

## Acceptance

- The work test of the wave, `own_scenario_nesting_bounded_work`, gains a case at bound 2 and one at bound 3: the arrangements counted for a chain of depth `n` grow at most with the number of distinct arrangements (a number the test names for the fixture), not with the number of paths to them.
- The chain fixture at bound 3 and depth 4 synthesizes in a release build in at most 2 s, against 14.6 s with the memo missing, and the test itself stays under 5 s in a debug build.
- The specification that cost 14.1 s at bound 1 costs at most 4 s at bound 3 (twice the release before), with 180 scenarios or more.
- Every suite the bound-1 build writes on the repository's fixtures is byte-identical at bound 3, except where the chain is deeper than 1 level, and those rows are named in the change with their added scenarios.
- `OWN_NESTING` is raised to the value the measurement supports and the changelog entry's "nests at most 1 level" says that value.

## Decisions

- D1. How equal arrangements are found.
  - A (recommended): number an arrangement by its content, so the existing memos hit.
  - B: share the sub-arrangement in a new memo keyed on (command, outcome, rows read).
- D2. The bound after the change.
  - A (recommended): 3, if its cost is within the acceptance figures; else the largest value that is.
  - B: remove the bound once the cost is linear in the suite.
