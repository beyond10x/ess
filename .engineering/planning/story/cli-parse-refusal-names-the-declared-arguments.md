---
format: aep.planning-md/3
id: story:cli-parse-refusal-names-the-declared-arguments
kind: story
status: draft
title: A cli_parse refusal names the declared arguments it is missing, never the input
relations:
- serves: vision:O2
revision: 2
---
## Outcome

A generated `ess-cli/1` CLI that refuses its command line as `cli_parse` (exit 2) says what to
repair, in terms the binding declares: the command path it reached, the kind of failure, and the
argument ids that are required and missing. It never echoes a word from the command line. Today
every parse failure answers `{"error":{"code":"cli_parse","data":{}},"ok":false}`
(`crates/generate/ess-cli-project/src/runtime.rs:742`, `docs/design/cli-presentation-binding.md:119`,
`:216`), so a caller sees that its line was refused and nothing about why. Requested by an adopter
on 2026-10-08, reproduced on its CLI with `--output json operations describe` (one required
argument missing).

## Fit review

1. **Need.** A caller (often an agent) that sent an incomplete command line must learn which
   declared argument to add without parsing human help text. Domain fact: "the command `describe`
   requires `operation`, and it was not given."
2. **Class.** Presentation of an existing refusal; no new noun, command or outcome.
3. **Already expressible?** No. `invalid_input:` (design :167-176) maps invalid *values* to a
   declared error with `{}` data and excludes `cli_parse`, which fails before a callable is known.
4. **Fit.** Holds the existing rule that refusals never carry input text (design :216). Everything
   the new data names comes from the plan: the reached command path (declared words), clap's
   missing-argument ids (the binding's field ids) and a closed reason vocabulary. An unknown
   argument, an unknown subcommand and a non-UTF-8 word are reported by reason only, never by the
   word.
5. **Second adopter.** Any generated CLI driven by a script: `deploy release` without `--version`
   should answer `missing: ["version"]`.
6. **Cost.** `cli_parse` data stops being `{}`. It is answer data, not a source format, so no
   `ess/N` bump; the generated reference and README text change; consumers asserting `{}` byte for
   byte (this repository's `ess-cli-project` tests: `adversarial_r2.rs`, `trailing_arguments.rs`,
   `optional_globals.rs`, `adversary_466_pass1.rs`) are updated. Whether the `ess-cli/1` contract
   version must move is decided in the design page first.
7. **Alternatives.** (a) Change nothing: callers keep guessing. (b) Echo clap's message: refused,
   it carries input text. (c) This design: reason plus declared ids only.

## Decisions

- Accept, redesigned to name only declared ids. Not in 0.57.0 or 0.58.0; scheduled when a release
  plan has room.

## Acceptance

- `cli_parse` data for a missing required argument is
  `{"reason":"missing_argument","command":[<declared words>],"missing":[<field ids>]}`.
- An unknown argument, unknown subcommand and non-UTF-8 word each answer their own `reason` and no
  word from the command line; a test feeds a secret-looking word and asserts it is absent from
  stdout and stderr.
- The design page states the data shape and the closed `reason` list before code.

## Scope

- `docs/design/cli-presentation-binding.md` (:112-120, :167-176, :210-217).
- `crates/generate/ess-cli-project/src/runtime.rs` (`internal(preliminary, 2, "cli_parse")`, :742).
- `crates/generate/ess-cli-project/tests/` (the four files named above).
- Held files: none.
