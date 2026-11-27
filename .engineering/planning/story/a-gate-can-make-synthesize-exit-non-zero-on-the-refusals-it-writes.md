---
format: aep.planning-md/3
id: story:a-gate-can-make-synthesize-exit-non-zero-on-the-refusals-it-writes
kind: story
status: draft
title: A gate can make conformance synthesis exit non-zero on the refusals it writes
summary: ess verify conform synthesize exits 0 beside refusals; an opt-in exit status lets a gate tell a whole suite from a partial one (reproduced on 0.99.8)
tags:
- adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: website/docs/reference/cli.md
revision: 2
---
## Outcome

A gate that runs `ess verify conform synthesize` can fail on the refusals the run writes, without parsing
the output: an opt-in way to make a run that wrote one or more in-scope refusals exit non-zero, the suite
still written.

## Problem

Found while reproducing a downstream report on 0.99.8: `ess verify conform synthesize` exits 0 when it writes
refusals, so a gate cannot tell a whole suite from a partial one by the exit status. Reproduced on 0.99.8 with a minimal model (one `String` newtype with a length invariant used only
as a command input, one command, no view): `ess verify conform synthesize --path <dir> --component <api>
--target ir --out <file>` prints `refused: refusal[ESS-SYNTH-013] ...` and ends "1 scenario(s) for `<api>`
(0 authored), 1 refusal(s), written to <file>", and the process exits 0. The same holds for any ordinary
run: a refusal is data of the suite, and the exit status is 0.

This is documented, not a defect: `CHANGELOG.md` (the `ESS-SYNTH-025` entry) says "`ess specify validate` and
`ess verify conform synthesize` keep their exit status (0)" and names the one exception, a declared-coverage
run (`--suite-format 5`), which exits 1 "as for any in-scope refusal no authored scenario covers". So there
are two behaviours, by flag, and no way to ask for the second on an ordinary suite. Today a gate reads
`ess specify validate --format json` `completeness.counts`, which runs a second synthesis and is not the
exit status of the run that wrote the suite.

## Need

Class: convenience that costs a second synthesis; a real gap for a gate that must fail on any refusal
(the gate's own job is to tell "the suite is whole" from "the suite is partial").

Consumer: a downstream gate over its own specification. Second adopter: any repository whose CI runs
synthesis and must fail on a partial suite.

## Acceptance

- `synthesize_exits_non_zero_on_a_refusal_when_asked`, in `ess-cli`'s tests: the minimal model above with
  the new flag exits 1, writes the suite file with the same bytes as without the flag, and prints the same
  lines; without the flag it exits 0 as on 0.99.8.
- `synthesize_with_the_flag_exits_zero_when_every_refusal_is_covered_by_an_authored_scenario`: the
  declared-coverage rule (`covers:`) is the one the flag reads, so a covered refusal does not fail the run.
- The reference page of the command (`website/docs/reference/cli.md`, generated from the command) and the
  `CHANGELOG.md` entry state the exit status of each form. No existing exit status changes without the flag.
- No suite byte, no `ess-conformance` version and no format moves: this is a process exit status only.

## Decisions

EX-D1, how a gate asks for the exit status (recommended: A; no answer yet).

- A (recommended): a flag on `synthesize` (and the same on `ess specify validate`, which reports the same
  refusals), off by default, named like the existing strict flags; the run writes its output as today.
- B: make exit 1 the default for any in-scope refusal. Breaking for every gate that reads 0 today and
  documented the other way in `CHANGELOG.md`; a format-style migration note would be owed.
- C: change nothing; the gate reads `ess specify validate --format json` and pays a second synthesis.

## Fit review

1. Need: an exit status that says whether the written suite is whole; reproduced above.
2. Class: gap for a gate (convenience for a person).
3. Expressible: yes at a cost (`completeness.counts` of `validate --format json`; a second synthesis).
4. Fit: the same question `--suite-format 5` already answers by exiting 1; siblings are `--check` and
   `--strict-requires` (flags that turn a condition into a refusal); no new authored surface.
5. Second adopter: any repository whose CI gates on a whole suite; written brand-free above.
6. Cost: one flag, no format, no diagnostics; the default keeps every exit status.
7. Alternatives: A, B and C above.

Decision class: accept, redesigned (a flag, not a changed default).

