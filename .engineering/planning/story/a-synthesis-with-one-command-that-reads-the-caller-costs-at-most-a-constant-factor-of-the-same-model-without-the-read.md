---
format: aep.planning-md/3
id: story:a-synthesis-with-one-command-that-reads-the-caller-costs-at-most-a-constant-factor-of-the-same-model-without-the-read
kind: story
status: draft
title: A synthesis with one command that reads the caller costs at most a constant factor of the same model without the read
summary: One caller read takes synthesis of a 188-command model from 444-1130 s to over 1800 s; 109 commands granted to an actor with attributes and no read add nothing
tags:
- defect
- downstream-report
- issues-2026-11-26
- performance
relations:
- serves: vision:O2
- informed_by: story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan
- informed_by: story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes
revision: 2
---
## Need

A downstream specification (188 commands, 109 of them granted to an actor that declares caller attributes, 54 of those reading the caller) ran `ess verify conform synthesize` past 30 minutes and was ended. The report inferred that synthesis repeats a full pass per command that reads the caller. The measurement below says otherwise: the same model synthesizes in 444 to 1130 s with no caller read, even with all 109 commands granted to an actor with attributes, and runs past 1800 s with a single command that reads the caller. The cost comes with the first read, not with each read.

## Outcome

One caller read costs at most a constant factor of the same model without it.

- Computed once per synthesis: everything a pass finds that does not depend on the caller's values (arrangements, witnesses, prepared rows, orderings), shared between the `first` and `second` readings and the per-command readings.
- Computed per assignment: only what the caller's values change, namely the outcomes whose guard or value reads the caller and the arrangement of the rows those outcomes read.

## What the code runs (origin/main 14248d409e; the cited files of the caller path are unchanged since c0b5f560cb)

- `crates/verify/ess-conformance/src/synthesize/caller/readings.rs:42-58` (`plan`): two `Reading::Whole`, then per mixed command two `Reading::About` and two `Reading::Cross`.
- `crates/verify/ess-conformance/src/synthesize/caller.rs:828-840` (`mixed`): a command is mixed when the credential values of its two callers differ, which holds for every command granted to an actor that declares attributes, whether it reads the caller or not.
- `caller.rs:365-386` (`Assigned::new`): with no command reading the caller, both whole readings share one model; with one, `first` and `second` become two models rewritten from the source.
- `crates/verify/ess-conformance/src/synthesize.rs:3956-3961` (`synthesize_invocations`): the witness, made, ordering, model, created and prepare memos are scoped to the model a pass reads, so every pass on the `second` model and every `About` pass on a model rewritten for one command (`caller.rs:400-411`) starts with them empty.
- `caller.rs:715-719` with `readings.rs:62-79`: a pair of whole passes for each mixed command whose focused runs send a command sent in turn by each granted actor. `caller.rs:987-1050` (`arranged_by_callers`): two focused passes per command left refused.
- `a_caller_reading_command_costs_no_whole_synthesis_of_its_own` (tests of `caller.rs`) counts whole passes only; nothing bounds the work inside a pass or the focused passes.

## Measured

Copy of the downstream specification, release 0.99.5 at c0b5f560cb, `--jobs 2`, one run each, on a shared host (the rows without a read spread 444 to 1130 s by host load). Every row has 188 commands. "Mixed" is the number of commands left granted to an actor with attributes (the others were granted to an actor without attributes). "Reading" is how many of those read the caller.

| mixed | reading | seconds | scenarios |
|---|---|---|---|
| 0 | 0 | 1130 | 7512 |
| 5 | 0 | 988 | 7521 |
| 40 | 0 | 497 | 7680 |
| 109 | 0 | 444 | 7953 |
| 1 | 1 (guard and value) | over 1800, ended | - |
| 1 | 1 (guard only) | over 1440, stopped | - |
| 2 | 2 | over 1800, ended | - |
| 5 | 5 | over 1800, ended | - |

Cause (inferred, not profiled: a stack sampler was refused by the host): with a first read, the two uniform readings become two distinct rewritten models (`caller.rs:378-386`), each with cold per-model memos, and the work that was done once for a shared model is done again on each. Which phase takes the time is not measured; the implementor measures it first, with a phase count on the generated model of acceptance 1.

## Acceptance

1. A test in `ess-conformance` builds a generated model with N commands granted to an actor that declares one attribute, of which K read it (`caller.<attribute>` in a guard), and counts the synthesis work (whole and focused passes and the outcome searches they run, through `#[cfg(test)]` counters). With K = 1 the work is at most a constant factor of the same model with the read removed. From K = 1 to K = 16 it grows by at most a constant per caller-reading outcome. At K = 0 it does not grow with N. Red on main for K = 1.
2. The test runs on a generated model only, under the 60 s bound of one test and inside the 120 s budget of the test step.
3. Suite bytes are unchanged: `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`, `adversary_related_values_all_bytes`, `external_beside_held_guard` and the tests of `caller.rs` (among them `a_run_written_for_one_command_gives_the_suite_the_whole_runs_gave`) pass without a re-pin.
4. `docs/design/caller-values.md` § Synthesis says which work is shared across readings and which is per assignment.

## Fit review

1. Need: a synthesis whose cost jumps from minutes to over half an hour with one caller read cannot be run in a gate. The reporter's proposal, to synthesize the `second` assignment only for the branches that read the caller, is evidence of the need, not the design.
2. Class: defect of cost. `docs/design/caller-values.md` § Synthesis promises the readings and their bytes, not their cost; the bytes are right, the cost is not bounded.
3. Already expressible: no idiom changes the cost; granting commands to an actor without attributes lowers it only by removing the caller from the model.
4. Fit: no authored surface, no format step, no new code. The suite and its bytes stay; what is recomputed changes.
5. A second adopter: any specification with an actor that declares attributes and one guard that reads one.
6. Cost: none to the format. Work in `caller.rs`, `readings.rs` and the memo scoping of `synthesize.rs`.
7. Considered: change nothing (rejected: the gate cannot run); synthesize the second assignment only for the branches that read the caller (the reporter's proposal; it changes which scenarios the suite holds unless the shared work is the unit, so the outcome is stated as shared work instead).

## Decisions

- None open. Taken ahead of other work on the owner's word, 2026-11-26.

## Design page

`docs/design/caller-values.md` § Synthesis.

## Not done

No change to which scenarios the suite holds or to the swapped second run.

## Scope

- `crates/verify/ess-conformance/src/synthesize/caller.rs` and `caller/readings.rs` (cited).
- `crates/verify/ess-conformance/src/synthesize.rs`, the memo scoping at `:3956-3961` (cited).
- `docs/design/caller-values.md`.

## Status

Pull request 840 (https://github.com/beyond10x/ess/pull/840) is open and not delivered (2026-11-27): red test `d0b9a0da72` and fix `402a0c2cd2`, with the fix of `story:a-hint-of-ess-command-005-names-only-a-form-validation-accepts-on-the-model-it-was-written-for` in the same pull request. Not in any release; no entry in CHANGELOG sections 0.99.3 to 0.99.8.
