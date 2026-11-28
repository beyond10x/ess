---
format: aep.planning-md/3
id: story:a-committed-suite-is-checked-against-the-inputs-of-synthesis-without-synthesizing
kind: story
status: draft
title: A committed suite is checked against the inputs of synthesis without synthesizing
summary: No verb prints a digest of the synthesis inputs, so a committed suite cannot be found stale in a fast step without synthesizing (gap report, 2026-11-24)
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-committed-suite-is-compact-drift-checked-and-reviewable
revision: 2
---
## Summary

`ess verify conform synthesize` prints one digest of everything a synthesis reads, without synthesizing, so an adopter commits it
beside a committed suite and a fast check in its test step finds the suite stale.

## Need

A downstream application reported on 2026-11-24 (eleventh gap report; reporter name kept out). A system of five components and
about 800 scenarios took six to nine minutes to synthesize on one core, and its test steps must end within two minutes. The
organization's rule for an expensive product is a committed generated file with a digest of its sources, a fast check that compares
the digest, and tests that read the file; full regeneration runs in a separate lane that compares bytes.

What ESS offers an adopter for that today:

- Run without synthesizing: `ess verify conform run --suite <file>` (and the Go and TypeScript packages written from a suite).
- Full regeneration and byte comparison: `ess verify conform synthesize --out <file> --check` (re-synthesizes: the slow lane).
- A partial fast check: `ess verify conform run --target interpreted --path <spec> --suite <file>` refuses a suite whose
  `spec_digest` is not the compiled model's. It runs the suite as well, and `spec_digest` covers the compiled model only: not the
  authored scenario files, the synthesis seeds, the suite options or the `ess` version, each of which changes the suite bytes.
- No verb prints a digest of the synthesis inputs. `ess specify compile` prints no digest; the model digest is readable only in a
  synthesized suite or in the ownership record of a generation.

This repository's own stamp (a digest of the sources of synthesis and of its models, compared in its test step) is an internal test,
not something an adopter can call for its own specification.

## Findings covered

- The adopter cannot tell in a fast step whether its committed suite is current, so it either synthesizes in the test step or
  trusts the file.
- The model digest alone is not an input digest: a changed scenario file or an upgraded `ess` leaves it unchanged.

## Fit review

1. Need: know, without synthesizing, whether a committed suite is the one these inputs and this `ess` would write.
2. Class: gap in the tool (a missing check), not in the language.
3. Existing idiom: `--check` (slow, complete) and the interpreted target's `spec_digest` refusal (fast, partial). Policy 1 of
   `docs/design/generic-language-features.md` is not engaged: no authored construct is involved.
4. Fit: beside `synthesize --check` and `conform run --known-failing`, which already binds a run to the executable's SHA-256; no
   new subcommand, one flag on the command whose inputs it names. Every target of `synthesize` (`ir`, `go`, `typescript`) reads the
   same inputs, so the one digest serves all.
5. Second adopter: any repository that commits a suite and runs CI (`story:a-committed-suite-is-compact-drift-checked-and-reviewable`,
   the adopter of issue 571).
6. Cost: one flag, one printed spelling (`synthesis-input/1:<hex>`), no suite format, no persisted ESS document, no change to any
   suite byte. A stability promise: the digest moves exactly when one of its documented inputs moves.
7. Alternatives: record the digest in the suite's provenance (a suite format change, ST-D1 option B); change nothing and use
   `--check` in every step (the reporter's cost); make `spec_digest` cover the scenario files (changes the digest every adapter binds,
   `SuiteProvenance::spec_digest`).

## Decisions

Taken on the default of each option list, 2026-11-24 16:27 CET (the coordinator, under the owner's delegation):

- ST-D1, how an adopter checks a committed suite without synthesizing. Decided: A, a flag on `ess verify conform synthesize` prints one line `synthesis-input/1:<64 hex>`: the digest of the compiled model, the selected authored scenario files, the synthesis-seed files, `--component`, `--suite-format`, `--compact`, `--target` and the `ess` version; it synthesizes nothing and writes nothing, the adopter commits the line beside the suite and compares two lines in the test step, and `synthesize --check` stays the slow lane. No format, no suite change. B (the digest recorded in the suite's `provenance`, with a fast `--check` that reads it) changes a persisted suite format and is not taken on default; C (nothing) declined.
- ST-D2, what identifies the synthesizer in the digest. Decided: A, the release version string: portable across Linux and macOS builds that write the same bytes; a dev build between releases shares the previous release's version, and the docs say so. B was the executable's SHA-256 (stale on a second operating system); C a synthesis revision number (needs a process that is not there).
- ST-D3, what of an authored scenario file enters the digest. Decided: A, the file's bytes, by path relative to the scenarios directory; a formatting change moves the digest, which costs one slow-lane run. B was the compiled scenarios.
- ST-D7, where the two-lane recipe is documented. Decided: A, a row in `website/docs/guides/commit-generated-files.md` and a paragraph in `website/docs/guides/verify/synthesize-a-suite.md`, as part of this story's last acceptance line. B was a separate documentation story.

## Acceptance

- the_digest_names_every_input_that_moves_the_suite: for a corpus model, the digest differs after a change to the compiled model, to
  one byte of a selected authored scenario file, to one byte of a synthesis-seed file, to `--component`, to `--suite-format`, to
  `--compact`, to `--target`, and to the `ess` version; and the suite bytes written by `synthesize` differ in each of those cases
  where the digest differs (equal digests never go with unequal suite bytes: checked over the committed model corpus).
- the_digest_ignores_what_the_suite_ignores: it is equal at `--jobs 1` and `--jobs 4`, at two absolute spellings of `--path`, and
  after a comment or a key reorder in a specification file (the compiled model is hashed, not the YAML text).
- the_digest_synthesizes_nothing: a work counter of whole syntheses started (`syntheses_started`) is 0 for the command, and it writes
  no file and takes `--out` or `--check` as a usage error.
- a_stale_committed_suite_is_found_by_comparing_two_lines: a test commits the suite and its digest, edits a scenario file, and shows
  the recomputed digest differs from the committed line while `synthesize --check` also exits 1.
- the_reference_and_the_commit_guide_say_it: `--help`, `website/docs/reference/cli.md` and the commit guide
  (`website/docs/guides/commit-generated-files.md`) name the flag, the inputs, that a dev build between releases shares its
  version with the release before it, and the two-lane recipe (committed suite plus digest in the test step, `--check` in a slow
  job that runs when an input changes).

## Scope

- crates/edge/ess-cli/src/main.rs  cited — `ConformCommand::Synthesize` (flag beside `check`, `jobs`; the synthesis arm that builds the inputs)
- crates/verify/ess-conformance/src/scenario.rs  cited — `SuiteProvenance::of`, `spec_digest` (reused, not changed)
- crates/specify/ess-compiler/src/ir.rs  cited — `source_digest` (3879)
- crates/verify/ess-conformance/src/synthesis_seeds.rs  inferred — the seed bytes that enter the digest
- crates/edge/ess-xtask/src/cli_reference.rs, website/docs/reference/cli.md, website/docs/guides/commit-generated-files.md  cited
- crates/edge/ess-cli/tests/  inferred — the cases above

## Format

None. The digest is printed, not stored by ESS; no suite format, IR or document changes.
