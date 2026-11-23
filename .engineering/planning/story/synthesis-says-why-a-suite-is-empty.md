---
format: aep.planning-md/3
id: story:synthesis-says-why-a-suite-is-empty
kind: story
status: draft
title: Synthesis says why a suite is empty
tags:
- from-finding
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: website/docs/guides/verify-conformance.md
- confidence: inferred
  path: website/docs/guides/verify/mutation-audit.md
revision: 6
---
## Outcome

- `ess verify conform synthesize` on a specification that declares no command and no view writes one note on standard error that says what the specification does not declare. The suite bytes, the standard output and the exit code do not change.
- A reader who meets the empty suite, or the `ESS-MUTATE-003` refusal of `ess verify conform mutate` that follows it, learns from the first command that the specification declares nothing to check, and is not left to take the tool for broken.

## Need

Found while reproducing a downstream reader's report on release 0.97.4 with a generic model of one entity and no command. A reader took the empty suite for a failure of the tool.

- `ess verify conform synthesize` prints `0 scenario(s) (0 authored), 0 refusal(s), written to …` and exits 0, with no reason, when the specification declares no command and no view.
- `ess verify conform mutate` on the same specification refuses with `ESS-MUTATE-003`, naming mutant classes that find no site. The refusal says what the audit could not find, not why: nothing in the specification gives a site.
- A specification that is a model only is legal; an empty suite is the right answer for it. What is missing is the sentence that says so.

Where the output is made, as read (to be confirmed by the implementor):

- `crates/edge/ess-cli/src/main.rs`, the text branch of `conform_synthesize` prints `note:` lines for `synthesis.notes` on standard output and the count line after them. The JSON branch prints the suite bytes only: there is no notes list in the JSON output, so the note cannot be added there without changing the suite or the output's shape, and neither changes.
- `crates/verify/ess-conformance/src/synthesize.rs` holds `Synthesis::notes` (`Note`), the list of questions the specification does not answer.

## Fit review

- Origin: a downstream reader's report. Adopted, redesigned: the request was for the tool to fail on an empty suite. It does not: an empty suite of a specification with nothing to check is correct, and a changed exit code would break every caller that reads 0 as "synthesis ran". A note is the smaller change.
- Considered and rejected: a new refusal code, a non-zero exit, a note inside the suite file (it would move the suite bytes and the digests that pin them).

## Design

- One note, written to standard error by the CLI after synthesis, when the suite holds 0 scenarios and 0 refusals: it names what the specification does not declare (no command, no view, or neither) and says that an empty suite is the expected answer then.
- Decided in `ess-conformance` from the compiled model, as a function the CLI calls, so another caller can print the same sentence; the CLI only prints.
- The same sentence is added to `ess verify conform mutate`'s `ESS-MUTATE-003` text where the model declares no command and no view, or the refusal's help line points at the first command's note. The refusal code and its other texts stay.
- The guide pages that explain synthesis and the mutation audit say, in one sentence each, that a specification without a command and a view yields an empty suite and an audit with no site.

## Acceptance

- A generic one-entity model with no command and no view: `synthesize` writes the same suite bytes as before (compared with `cmp` against a suite written by the release before), exits 0, prints the same standard output, and prints the note once on standard error.
- A model with one command and no view, and one with a view and no command, each name only what is missing.
- A model with at least one scenario prints no note. `--format json` writes the same bytes on standard output as before.
- A test for each of the three models, in a test target of `ess-cli` that already runs a synthesis, each under 5 s.
- `ess verify conform mutate` on the empty model still refuses `ESS-MUTATE-003` with the same code.
- The two guide pages carry the sentence; `task site-build` or the repository's docs check passes.

## Decisions

- D1. Where the note goes.
  - A (recommended): standard error only; the JSON output has no notes list and the suite bytes do not move.
  - B: also add a `notes` member to the JSON output. This changes the output's shape and needs its own format decision.
- D2. The exit code.
  - A (recommended): unchanged, 0.
  - B: non-zero on an empty suite, behind a flag.
