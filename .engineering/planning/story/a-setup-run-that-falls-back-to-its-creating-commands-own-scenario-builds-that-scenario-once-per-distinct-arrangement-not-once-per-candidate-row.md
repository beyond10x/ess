---
format: aep.planning-md/3
id: story:a-setup-run-that-falls-back-to-its-creating-commands-own-scenario-builds-that-scenario-once-per-distinct-arrangement-not-once-per-candidate-row
kind: story
status: draft
title: A setup run that falls back to its creating command's own scenario builds that scenario once per distinct arrangement, not once per candidate row
summary: 'Regression 5cb9ae3ebc (0.99.0 to 0.99.2): synthesis of a 201-scenario model takes 195.9 s against 6.35 s on 0.98.2, suite bytes unchanged; fix bba4a140ad takes it to 58.6 s, partial'
tags:
- conformance
- defect
- external-request
- gap-reports-2026-11-24
- regression
relations:
- serves: vision:O2
- informed_by: story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose
- informed_by: story:the-own-scenario-arrangement-of-a-setup-run-finds-an-equal-arrangement-again-instead-of-building-it-once-per-path
revision: 10
---
## Summary

`ess verify conform synthesize` of a downstream application's model (201 scenarios, 41 refusals)
takes 195.9 s on release 0.99.2 where the same model took about 1 s on 0.97.4 and 6.3 s on 0.98.2.
The time is one regression, released in 0.99.0 and carried by 0.99.0, 0.99.1 and 0.99.2: a setup
run whose creating command reads several related rows builds that command's whole own scenario
again for every candidate row it tries, and nothing remembers the result. The suite's bytes do not
change; only the time does. `--jobs` does not shorten it: CPU time equals wall time at every job
count.

## Evidence

Release builds, `--jobs 1` unless named, one build node; the suite is byte-identical in every row:

| release | wall | scenarios / refusals |
|---|---|---|
| 0.97.4 (spec copy requiring ess 0.97.4) | 0.99 s | 124 / 123 |
| 0.98.0 | 6.27 s | 191 / 67 |
| 0.98.2 | 6.35 s | 191 / 67 |
| 0.99.0 | 197.35 s (user 195.58 s) | 201 / 41 |
| 0.99.1 | 199.66 s | 201 / 41 |
| 0.99.2 | 195.90 s (`--jobs 8`: 194.60 s) | 201 / 41 |

Peak resident memory 3.57 GiB on 0.99.2. The reporter measured 282 s (`--jobs 1`), 220 s (`--jobs 4`)
and 185 s (`--jobs 32`) on a 32-CPU node, 3,645 MiB in every run; the step that runs it in the
reporter's CI has 120 s.

Bisect: the first bad commit is `5cb9ae3ebc`, "fix(conformance): a setup run of a creating branch
reading several rows is sent the chosen field values" (released in 0.99.0); its parent `22ec3c2363`
synthesizes the same model, with 191 scenarios and 67 refusals, in 6.16 s. The commit belongs to
`story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose`,
whose Where section records that no memo of the own-scenario arrangement exists.

Where the time goes (0.99.2, first 40 s of the run, 7,986 samples, `perf` with LBR call graphs):
86 % inclusive in the `with_row_where` closures of `crates/verify/ess-conformance/src/synthesize/related_guard.rs`,
of which 63 % in `subject_fact::evaluate_row_flat`, 48 % in `related_guard::leaves_to`, 32 % in
`related_guard::selects` and `chooses`, 19 % in `input::bind`. A counting probe on 0.99.2 source:
5,470 own-scenario rebuilds, 186 of the 194 s inside them, 1,306 distinct by (command, outcome,
distinction, bound values, chosen field values, actors, open arrangements).

Cause: `5cb9ae3ebc` opens the own-scenario fallback also when a caller chose field values (before:
only when none were chosen). Each such fallback re-arranges the creating command's whole scenario,
and `with_row_where` runs it once per candidate row. Nothing keys the result.

Not the same cause as the unreleased synthesis-time fix of the witness search
(`witness::Directed` is not among the top frames): applied to the 0.99.2 head it moves 195.9 s to
191.0 s and nothing else.

## Red test

`own_scenario_rebuilt_once_work` in `crates/verify/ess-conformance/tests/own_scenario_rebuilt_once_work.rs`,
commit `95de3d821f` on the branch `fix/n89-a-setup-run-rebuilds-its-creating-scenario-once`: a
creating command read by one reader builds its own scenario 108 times, by four readers 432 times;
the test counts the builds (3.7 s in a debug build) and is red until the count no longer grows with
the readers.

## Fix

Commit `bba4a140ad` on the same branch (pushed): the own-scenario arrangement of a setup run is
built once per what it reads. The memo lives on the thread, as the memo of the strict search does,
and its key is (model, command, outcome, distinction, bound values, chosen field values, actors,
open arrangements, held rows projected to the entities the command can reach). Earlier-run captures
and an under-test scope that copies no `Optional` are not read by the rebuild and stay out of the
key. Measured on the 0.99.2 head, release, `--jobs 1`: 195.9 s before, 58.6 s after (`--jobs 8`:
58.1 s); on the same session 0.99.2 itself ran 231.1 s (`--jobs 1`) and 199.7 s (`--jobs 8`);
peak memory 3.74 GB before, 2.21 GB after. The four suites (`--jobs 1` and `--jobs 8`, before and
after) are byte-identical (`cmp`). The gate of the branch ran green: 14 steps exit 0, test steps
5 to 33 s, slow byte lanes 33, 10 and 119 s, clippy 54 s, `task fmt-check` 28 s.

## Acceptance

- `own_scenario_rebuilt_once_work` is green: the count of own-scenario builds is the same with one
  reader as with four.
- The synthesis of the reporter's model class (a creating command that reads several related rows,
  201 scenarios, 41 refusals) writes the same bytes as 0.99.2 at `--jobs 1` and `--jobs 8`; every
  suite byte pin of the repository is unmoved.
- Synthesis time of that model on the build node, release, `--jobs 1`, is recorded in the change
  beside 0.99.2's 195.9 s and 0.98.2's 6.35 s. The reporter's 120 s CI step is the figure the
  reporter reads it against; whether 58.6 s on this node meets it on the reporter's node is the
  reporter's measurement, not a claim of this story.
- The change keeps `story:the-own-scenario-arrangement-of-a-setup-run-finds-an-equal-arrangement-again-instead-of-building-it-once-per-path`
  open: that story raises `OWN_NESTING` and measures a chain of creators; this one removes the
  repetition per candidate row at depth 1 and moves no bound.

## Fit review

1. **Class.** A defect, a time regression of a released synthesis: a verb that ended in about 6 s
   ends in about 196 s on the same model, with the same bytes.
2. **Release.** The fix is a patch release, 0.99.3, by the repository's Hotfix route (the operator
   session decided this on 2026-11-25): branch from the release tag 0.99.2, red test first, then
   the fix, the version bump with its `CHANGELOG.md` section and `release-notes/0.99.3.md`, the
   regression test and the touched crate's tests and Clippy, the tag, and the pull request to
   `main` in the same minute. The bytes of every suite are unchanged, so no pin moves and the
   changelog names no moved suite.
3. **Borders.** The memo story above (a chain of creators, a nesting bound) and the story of
   `5cb9ae3ebc` (the behaviour this regression came with, which stays).
4. **Evidence form.** Times are measured, release builds, one node, `--jobs 1` unless named; the
   reporter's own times are quoted as the reporter's.

## Decisions

- None open. The route (patch release 0.99.3 by the Hotfix route) was decided by the operator
  session on 2026-11-25.

## Left open

The fix is partial. The same model takes about 6 s on 0.98.2, about 196 s on 0.99.0 to 0.99.2 and
58.6 s with the fix: synthesis of this model is still about nine times slower than on 0.98.2. The
remaining rebuilds are distinct by held rows: 3,661 of 5,470 rebuilds are distinct with the full
held key (the counting probe on 0.99.2 source), so a memo keyed that way cannot remove them.
Whether the rebuild can read fewer held rows, or the fallback can run once per distinct candidate,
is not decided by this story.

The slow byte lanes of the fix's gate ran one test for 156 s in the first gate run and 119 s in the
re-run (the lane of the 1, 2 and 7 job byte comparison), both over the 60 s bound of a single
test. That test is an item for `story:tests-that-run-over-60-s-on-main-without-a-branch-of-the-week-run-within-60-s-or-move-to-the-slow-lane`.

About 1,300 to 1,500 distinct own-scenario arrangements with chosen values are still built once
each, at about 34 ms each (about 44 to 50 s); 0.98.2 refused these runs and did not build them.
The profile puts the time in `with_row_where` (86 % inclusive) and `evaluate_row_flat` (63 %).

## Delivery

Red test `95de3d821f`, fix `bba4a140ad` on `fix/n89-a-setup-run-rebuilds-its-creating-scenario-once`,
pushed; the gate of the branch green. Tagged 0.99.3 on `e0f8d1d3b2` (pull request 831). The tag's
release run 45596324248 was cancelled on 2026-11-25 by the 60-minute limit of its targeted test job
and published nothing. The release workflow was dispatched again for the same tag with the full
gate (run 45610533532, in progress when this was written); the tag did not move.

The second release run for tag 0.99.3 (45610533532) was cancelled on 2026-11-25 before its test
jobs started, so that 0.99.4, which holds everything 0.99.3 holds, is gated first; the full gate for
0.99.3 is dispatched again after 0.99.4 is published (the operator session, D144 part 2 = A). The
tag did not move.

0.99.5 is released (2026-11-25 23:23 CET): tag on `3cdf39f157` (branch `hotfix/0.99.5`, the release the second finding's fix was planned for); the tag's release run 45682187587 took the full gate; the release is public at https://github.com/beyond10x/ess/releases/tag/0.99.5. Pull request 837 brings 0.99.5 to `main`; it was open when this was written.

Pull request 837 brings 0.99.5 to `main`, open when written.

## Second finding

Base time, release build of the 0.99.4 release commit `78c4ba842c`, `--jobs 1`: 50.21 s wall on
the reported model class (178 scenarios, 66 refusals, 1.86 GB peak). Of a probed run of 79.6 s,
70.7 s lay inside the 1,276 own-scenario arrangements that are built with chosen values; the largest
costs inside one arrangement are `Around::admits` into `leaves_to` (about 43 % inclusive), `chooses`
into `selects` (30 %), and `subject_fact::evaluate_row_flat` (59 %) beneath both, all under the
input trial of `with_row_where` (75 %).

Verdict: the repetition is inside one row search, not across arrangements. With chosen (fixed)
input fields, `searched_inputs` (`crates/verify/ess-conformance/src/synthesize/related_guard.rs`)
overwrites the candidates' fields with the chosen values, so most inputs of a search become equal
copies of one another and each copy was tried again on the row. Counts of the probe: lists with
chosen fields held 1,239,300 inputs of which 52,530 are distinct (23.6 times); `leaves_to` ran
1,133,087 calls for 19,580 distinct ones per search, `chooses` 976,814 calls for 57,117, and
`evaluate_row_flat` 4,430,122 calls for 118,265.

Red test: `an_input_the_chosen_values_make_equal_is_tried_once` in
`crates/verify/ess-conformance/tests/fixed_input_tried_once_work.rs`, commit `bfcf715890` (it
replaces a first shape, `389608fcab`, whose count grew with the rows and not with the copies). It
fails with the inputs tried per row, without the guard on the chosen name (648 on 324 rows) and with
it (1,152 on 216 rows); 1.6 s.

Change: a repeated input is dropped after the chosen values are applied; the first of each keeps its
place, so the order the row search tries inputs in is the same.

Times, same session, release build, before and after: `--jobs 1` 52.78 s to 18.21 s, `--jobs 8`
57.25 s to 17.44 s. The four suites (`--jobs 1` and `--jobs 8`, before and after) are byte-identical
(`cmp`). After the change the row search `with_row_where` holds 37 % inclusive as a whole and no
single cost is over a third of what is left, so no second round was made.

The red test is committed locally and the fix is written but not committed: the build node ended on
2026-11-25 before its gate ran; nothing is pushed.

The second finding's fix is committed and pushed (2026-11-25): branch
`fix/n89-an-own-scenario-arrangement-is-cheap-to-build`, red test `bfcf715890`
(`an_input_the_chosen_values_make_equal_is_tried_once`), fix `04bbe7fa6e` (a row search with chosen
values tries each input once). One reported model: 52.78 s before, 18.21 s after with `--jobs 1`;
57.25 s and 17.44 s with `--jobs 8`; its suites are byte-identical. It is planned for 0.99.5
(branch `hotfix/0.99.5`, in preparation when this was written). The full gate for tag 0.99.3 was
dispatched again as run 45653752956 after 0.99.4 was published; it was in progress when this was
written.
