---
format: aep.planning-md/3
id: story:ess-verify-conform-run-suite-suite-jobs-n-accepts-a-job-count-that-reaches-nothing-the-flag-is-ignored-where
kind: story
status: draft
title: '`ess verify conform run --suite <suite> --jobs N` accepts a job count that reaches nothing: the flag is ignored where no synthesis runs, and the command does not say so'
summary: ConformCommand::Run takes --jobs beside --suite or --suite-input, validates its range and drops it; the help says so, the command line does not
tags:
- issues-2026-11-24
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:ess-verify-conform-synthesize-and-run-take-jobs-and-the-bytes-are-the-same-at-every-job-count
revision: 2
---
## Outcome

`ess verify conform run --suite <suite> --jobs 4` does not silently drop the job count: a caller who
names a count where no synthesis runs learns that from the command, not from reading the flag's
help.

## Need

**Reading.** Verified at the wave head `b2cdf564b` (`origin/integration/wave-2026-11-23`); the
code is not on `main` yet, so the story is for the wave's merge, not for a released `ess`.

- `crates/edge/ess-cli/src/main.rs:839` declares `jobs: Option<u16>` on `ConformCommand::Run`, with
  `value_parser` 1 to 256 and no `conflicts_with` (compare `suite_input` and `suite_format`, which
  name their conflicts).
- Its help (`main.rs:832-837`) says the count applies "where the suite is synthesized rather than
  named by `--suite`". The story that added the flag says the same
  (`story:ess-verify-conform-synthesize-and-run-take-jobs-and-the-bytes-are-the-same-at-every-job-count`).
- The dispatch applies it only to a run with neither `--suite` nor `--suite-input`
  (`main.rs:3858-3863`: `Run { suite: None, suite_input: None, jobs, .. }` goes through
  `with_job_count`); every other run takes the arm at `main.rs:3864`, and `conform_run` binds
  `jobs: _` (`main.rs:4371`).
- So `--suite X --jobs 4` runs, and the 4 reaches nothing: no error, no note, no effect, and an
  out-of-range count is refused by `clap` while a valid one is ignored. The documented behaviour
  holds; the defect is that the command cannot tell the caller.

**Not the same as the variable.** At this head the CLI does not read `ESS_SYNTH_JOBS` on the
`--suite` path (the adversary's finding F9 of the wave close, that an invalid variable was refused
for a run that synthesizes nothing, no longer holds at `b2cdf564b`: `with_job_count`, which reads
it, is not called there).

## Acceptance

`ess verify conform run --suite <a committed suite> --jobs 4` exits non-zero with a message naming
`--jobs` and `--suite` (decision D1, default A), and the same command without `--jobs`, and
`conform run` with no `--suite` and `--jobs 4`, behave as they do today; a test in `ess-cli`
runs the three commands and asserts the status and the message.

## Decisions

- **D1 (open): what a count beside a named suite does.** A (default): refuse it with `clap`'s
  `conflicts_with_all = ["suite", "suite_input"]` on `jobs`, as `suite_input` and `suite_format`
  already do; the refusal names both flags and costs a caller who scripts one command line for both
  modes a branch. B: accept it and print one note on stderr that no synthesis runs. C: leave it and
  state it in the help only (today's behaviour). A is the default because the other flags of the
  command refuse such a combination and a silently ignored number looks like a speed-up that did
  not happen.

## Scope

- `crates/edge/ess-cli/src/main.rs` (`ConformCommand::Run`, `jobs`)
- `crates/edge/ess-cli/tests/generated_docs_synthesize_help.rs` (or a new `ess-cli` test module) for the cases
