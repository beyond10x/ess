---
format: aep.planning-md/3
id: review-result:zi-w1-swap-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:a-swapped-caller-run-renames-only-the-names-it-binds
relations:
- reviews: story:a-swapped-caller-run-renames-only-the-names-it-binds
revision: 2
---
unit: story:a-swapped-caller-run-renames-only-the-names-it-binds
verdict: broken
new failing cases: 1 (crates/verify/ess-conformance/tests/caller_swap_value_invariant_witness.rs::the_swapped_run_of_a_value_invariant_scenario_selects_its_own_row)
mutants survived: none

- **What I touched:** `git --no-pager diff --stat` is empty. `git status` shows one untracked file, `crates/verify/ess-conformance/tests/caller_swap_value_invariant_witness.rs` (a test). No implementation file was edited; the tree is HEAD 5a0d1d3cd3 plus that file.
- **Finding (NEEDS-CHANGE, introduced, blocker):** `values_mut` skips the whole value-invariant view expectation (`caller.rs:1950`, `ViewExpectation::ValueInvariants(_) => {}`). That expectation's witness holds a `ScenarioValue` (`Selector::ProjectedIdentity/IdentityQuery.expected`). The removed JSON walk renamed it, and recaptured it when it was an `observed` reference; the new walk does neither.
  - Effect: on any model with caller attributes that has a wrapped value-invariant site, the swapped run's witness still reads `observed(BoardOpened, board_id)`. That is the first run's row, so the swapped run never checks its own row. A supplied-identity `Instance` there is not suffixed either.
- **Red case, run alone:** `build.sh adv/r2.txt <tree> test -p ess-conformance --locked --test caller_swap_value_invariant_witness` → exit 101, at `caller_swap_value_invariant_witness.rs:108`: "the swapped run's witness reads the swapped run's captured row, not the first run's observed reference: Observed { event: EventRef(QualifiedName(demo.tags.BoardOpened)), field: "board_id" }".
  - What reaches it: `caller::synthesize` → `append`, for scenario id `demo.tags.Tag/invariant-wrapped/...`. The dump shows two `execute_command` steps with owners `owner-262144` and `owner-262145`, so the swapped run is appended.
- **Disclosure:** the first run of this case (`adv/r1.txt`) was red on a precondition I wrote wrongly. I asserted that the text contains `-swapped`, but on the new code this run binds no name at all, which is the same defect showing. I dropped that line and re-ran; r2 is red for the reason claimed above.
- **Origin:** introduced, read from the diff (the old `observed`/`replace_observed`/`rename` matched any JSON object). I did not run it against the base.
- **Scope:** the story's acceptance bullets don't name this case. It contradicts the design (caller-values.md:138, "every instance and instant it binds") and the story's Fit claim that bytes move only for models with an authored `instance`/`instant` field. The byte pins stay green only because no committed model reaches this path.
- **Suggested fix:** walk `ValueInvariants` witness selectors' `expected` in `values_mut`. `singleton::rename` inherits the fix.
- **Suite run, after the case existed:** `build.sh adv/r3.txt <tree> test -p ess-conformance --locked --no-fail-fast --test caller_swap_authored_field_names --test interpreted_caller --test synthesis_suite_bytes_table --test caller_swap_value_invariant_witness` → exit 101.
  - Counts: executed 7→8, red 1. The "before" count comes from the implementor's g.1/g.4/g.8 runs, not a pre-run of mine.
  - Byte pins: 2 passed. `caller_swap_authored_field_names`: 2 passed. `interpreted_caller`: 3 passed.
- **Note (INFEASIBLE, introduced):** `names_mut`/`values_mut` skip `CaptureCommandResult`/`ExpectReplayResult` (`caller.rs:2000,2058`). Their `replay::Observation.instance` is an `InstanceName` that the old JSON walk renamed. The caller swap excludes these steps (`once`). `singleton::rename` (`singleton.rs:401`) could reach them in a merged tail, but I found nothing that builds that state.
- **Could not break:** `observed-N-swapped` colliding with an authored name (no synthesized name uses `observed-`). An instant name in `ExpectNotBefore/Within/Quiet`, which is walked. Lists and members, which are walked recursively. `redraw` after rename, unchanged and keyed by identity keys. `InstanceName` suffixing, where `-swapped` is always valid lower-kebab.
- **Paths written outside the worktree:** `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/a-swapped-caller-run-renames-only-the-names-it-binds/adv/` (r1.txt, r1.txt.touch, r2.txt, r3.txt). The builds used the shared slots `.../ess/targets/A` and `B` through build.sh.

```findings
- file: crates/verify/ess-conformance/src/synthesize/caller.rs
  line: 1950
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: values_mut skips ViewExpectation::ValueInvariants, so the swapped run's value-invariant witness keeps the first run's observed reference (or unsuffixed instance) and re-checks the first run's row instead of its own
- file: crates/verify/ess-conformance/src/synthesize/singleton.rs
  line: 401
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the typed walk no longer renames replay::Observation.instance in CaptureCommandResult/ExpectReplayResult, which the old JSON walk did; no reachable singleton-merge tail holding one was found
```
