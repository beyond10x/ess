---
format: aep.planning-md/3
id: story:a-synthesis-may-be-split-into-parts-that-are-kept-by-the-digest-of-their-inputs-and-merged-into-the-suite-one
kind: story
status: draft
title: A synthesis may be split into parts that are kept by the digest of their inputs and merged into the suite one synthesis writes
summary: The runs of all --component parts do not cover the whole suite and nothing merges or reuses parts; waits on decision-blocker:synthesis-parts-and-reuse (gap report, 2026-11-24)
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-committed-suite-is-checked-against-the-inputs-of-synthesis-without-synthesizing
- informed_by: story:the-outcome-loop-of-a-whole-caller-reading-runs-on-several-threads-and-writes-the-suite-one-thread-writes
- informed_by: story:a-run-with-component-says-what-it-covers-in-text-and-in-json
revision: 2
---
## Summary

A synthesis can be written as parts that together hold every scenario of the whole system, each part kept under the digest of its
inputs, and a merge gives the suite one synthesis writes, so independent processes or a later run reuse parts a change did not touch.

## Need

A downstream application reported on 2026-11-24 (eleventh gap report; reporter name kept out). A system of five components and about
800 scenarios took six to nine minutes to synthesize on one core of a 64-CPU node. It asked for any of: several cores in one run (the
parallel-readings story and `the-outcome-loop-...` story), a split into parts that hold every scenario plus a merge equal to the single
synthesis, and an incremental synthesis that reuses unchanged parts.

Status: architecture decision pending (ST-D5). Nothing here is built on the default.

What exists:

- `--component` runs synthesize only a component's operations (`synthesize_for`, `scope_of`, `split` in `synthesize.rs`). Since 0.97.3
  their suites are written exactly as the whole suite writes the same scenarios (the entry's 122 scopes of 97 models), and a scenario
  that needs another component is listed `outside:` with its id and its needs, without its body, by the run of the component whose
  operation it is about. The 0.97.3 entry states that together the runs of all components do not cover the whole suite.
- No command unions suites. `ess verify conform diff` compares two suites; `--suite-input` carries a suite and its parent chain.
- Nothing is reused across processes: every memory of a synthesis is in-process and keyed by the address of a model.

## Findings covered

- Parts of a system as the CLI cuts them hold about four fifths of its scenarios; the rest are in no part's suite.
- There is no merge.
- The two largest parts took about as long as the whole system's synthesis (about nine minutes each, against six to nine). Whether a cut by component makes
  any part cheaper than the whole depends on `scoping_reproduces` (a model with a one-time-response marking or a read-granted view with
  parameters synthesizes the whole and reads it down, `synthesize.rs` 4240) and on the extension of a part's scope to commands that share
  a subject where the model has actors (`scope_of`). A per-part profile is missing.

## Fit review

1. Need: wall time of one synthesis, and a result kept so that a small change redoes a small amount.
2. Class: gap in the tool (cost). Nothing authored changes.
3. Expressible today: not as asked. The no-change case is: commit the suite, run it with `--suite`, check freshness with the digest of
   `a-committed-suite-is-checked-against-the-inputs-of-synthesis-without-synthesizing` and with `synthesize --check` in a slow lane.
4. Fit: it is built from `scope_of` and `split`; it needs a part that covers the cross-component remainder (those scenarios' bodies
   are computed by the part of their own component and dropped), a persisted part document, and a merge that re-runs the whole-suite
   passes (`synthesize_in`: cross-caller grants, singleton and repeated-identity withdrawals, precedence contradictions, self-admission) and
   re-selects the suite format of the merged content. It must not change the meaning of `--component`: a component's suite stays
   what one implementation can be held to. It meets the policy of `a-run-with-component-says-what-it-covers-...` (Out of scope:
   "making the runs of all components cover the whole suite") as a new selection, not a change to `--component`.
5. Second adopter: any large multi-component system whose parts are cheaper than the whole.
6. Cost: a persisted part document and its format version, a claim that the merge writes the bytes of the single synthesis (a
   determinism rule held by a byte table over the corpus), a cache keyed by the digest of a part's inputs, and a decision about which
   unit is stable under a change: a change to one command can move the scenarios of commands that arrange through it (the parent
   story rejected "one obligation" as a unit for that reason). An unknown: the saving. The report's own measurement is that the
   largest parts cost as much as the whole.
7. Alternatives: change nothing and use the slow-lane recipe with `--jobs` (ST-D5 option A); parts and merge without reuse (B); parts,
   merge and reuse (C); reduce single-thread work (`pf1` to `pf3`, the wider-parallelism story), which makes parts matter less.

## Decisions

Deferred. `decision-blocker:synthesis-parts-and-reuse` blocks this story and this story waits on it.

ST-D5 (ARCHITECTURE): parts of a synthesis, a merge, and reuse across runs. Open, with the owner; no default; sent to the owner at about 16:27 CET on 2026-11-24, no answer yet. A: do not build it now; close need 2 and need 3 with the committed-suite recipe (ST-D1), `--jobs` and ST-D4, and record the measurement and the cause that decides whether parts are cheaper than the whole (`scoping_reproduces`, the actor-scope extension) as the condition for reopening; the story is closed, not built. B: parts and a merge, no reuse (a remainder part for the scenarios that cross components, a part document, working name `ess-conformance-part/1`, a merge command whose output is `cmp`-identical to `synthesize` over the corpus; a persisted format and a determinism claim). C: B plus reuse (parts kept under the digest of their inputs; B's cost and a stable unit under change, since a change to one command can move scenarios of commands that arrange through it and the memories are shared across commands). The coordinator recommends A, because no saving is shown for the reporter's cut and B or C commit a format and a determinism rule before the cost of a part is known; the owner decides. The Acceptance below is written for C and shrinks for B; option A closes the story with the evidence recorded.

## Acceptance

Written for option C.

- parts_cover_every_scenario: for a corpus model with three components and a scenario that needs two of them, the parts' suites and the
  remainder part hold each scenario of the whole system once, and the part for the remainder holds the cross-component scenarios with
  their bodies.
- the_merge_writes_the_single_synthesis: `ess verify conform` merges the parts into a suite `cmp`-identical to
  `synthesize --target ir --compact` over every committed corpus model that has components (with and without actor attributes, with a
  one-time-response marking and with a read-granted view that takes parameters), and refuses parts of different specifications, of
  different `ess` versions, or a missing or duplicate part by name.
- a_part_is_kept_under_the_digest_of_its_inputs: a part document names the digest of the inputs that determine it; a second run with the
  same digest reads the part and synthesizes nothing (counter of whole syntheses started is 0 for it).
- a_change_to_one_command_redoes_the_parts_it_moves: after changing one command, only the parts whose digest moved are synthesized and the
  merge equals a full synthesis (corpus test over single-command edits, including a creator another command arranges through).
- the_merge_is_no_slower_than_one_part_by_much: the merge itself runs no synthesis and no reading (counter).
- component_keeps_its_meaning: `--component` suites and `outside:` lists are byte-identical to before.

## Scope

- crates/verify/ess-conformance/src/synthesize.rs  cited — `synthesize_for_in`, `scope_of`, `scoping_reproduces`, `split`, `synthesize_in`
- crates/verify/ess-conformance/src/scenario.rs  cited — `SuiteProvenance` (a part names its scope), `select_fresh_format_for`
- crates/edge/ess-cli/src/main.rs  cited — `ConformCommand::Synthesize`, a merge command or flag
- crates/verify/ess-conformance/tests/  inferred — a byte table of merged suites over the corpus
- docs/design/  inferred — a design page for parts, the digest of a part's inputs and the merge

## Format

Option B and C add a part document (`ess-conformance-part/1`, working name) and may add a field to the suite provenance: a persisted
format. Option A changes none.
