---
format: aep.planning-md/3
id: story:the-suite-pair-ess-conformance-66-and-67-is-open-read-wherever-64-and-65-are-and-written-by-nothing-yet
kind: story
status: active
title: The suite pair ess-conformance/66 and /67 is open for the pull request's constructs, read wherever /64 and /65 are read, and written by nothing yet
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:24:35Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T03:24:37Z", actor: "human:timo", revision: 3}
---
## Need

The one pull request carries every open issue, and about a dozen of its stories add suite
vocabulary (steps 61, 91 to 98, 100, 104, 106, 107, 112, 114; `FORMAT.md` lists them). Each names
"the new suite pair" or `ess-conformance/64` and `/65`. That pair is closed: it opened in 0.93.0
with no construct (`CHANGELOG.md`, 0.93.0 section, "nothing writes `/64`, `/65` or `/24` yet") and
is written since 0.95.0 by the coverage suite that holds ESS-SYNTH-025 (`CHANGELOG.md`, 0.95.0
section; `crates/verify/ess-conformance/src/coverage.rs:54`, `COVERED_BY_MAJOR = 65`). A reader of
0.95.0 that admits `/64` would read new vocabulary there as a suite nobody wrote.

Decision: every suite change of the pull request lands in one new pair, `ess-conformance/66`
(ordinary) and `/67` (coverage), opened empty by this story, as
`story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs` opened `/64` and `/65`, so
the stories that fill it do not race to open it. No delta version opens: `ess verify diff` keeps
its versions (see `FORMAT.md`, open choice DIFF). No source format opens: `ess/38` is opened by
`story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess`.

## Reading

- Rust: `SUPPORTED_SUITE_FORMATS` ends at 65 (`crates/verify/ess-conformance/src/scenario.rs:640-644`);
  its test reads `/66` as unsupported (`scenario.rs:3701-3705`). The opened pair's constants live
  in `src/suite_pair_64.rs:11-17` and are read by `src/synthesis_seeds.rs:111` and
  `src/go/mod.rs:477`. Admission's coverage rule names "/5 through /65" (`src/admission.rs:349`).
  Seed provenance is admitted "suite/42 through /65" (`src/synthesis_seeds.rs:186`).
- Go: `NEWEST_ADMITTED_SUITE_MAJOR = 65` (`src/go/mod.rs:446`, read at 497);
  `newestSuiteMajor = 65` and the report-format message at `src/go/runtime.go:104` and `:1912`.
- TypeScript: `src/ts/runtime.ts:2864` (report format "suite/8 through /65"), `:7197` (the
  version table ends at `'ess-conformance/65': 65`), `:7209` (odd coverage majors end at 65),
  `:7248` (coverage message "through /65").
- CLI and test messages that quote the bound: `crates/edge/ess-cli/tests/support/coverage_producer.rs:495`,
  `crates/verify/ess-conformance/tests/system_issued_vocabulary.rs:184`.
- The formats reference row for `/64` and `/65`: `website/docs/reference/formats.md:453`.

## Acceptance

1. Red today: in `crates/verify/ess-conformance/tests/adversary_t1_pair_64_65.rs` a suite whose
   `provenance.suite_version` is rewritten from `ess-conformance/64` to `/66` is admitted by the
   Rust runner and run with the same verdicts as at `/64`; today admission refuses `/66`.
2. The same holds for a coverage suite rewritten from `/65` to `/67`, for an explicit selection of
   a coverage suite at `/67` (as `an_explicit_selection_of_a_coverage_suite_65_is_admitted_as_one_of_63_is`
   holds for `/65`), and in the generated Go and TypeScript runners (`go test`, `npm test` in the
   target packages the existing cases build); `/68` is refused by every reader, naming the newest
   major it admits.
3. Nothing writes the pair: every committed model's synthesized suite keeps its version and bytes
   (`synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read` unchanged), and a
   coverage suite with ESS-SYNTH-025 is still written at `/65`.
4. Every message that names the newest admitted major says `/67` (report format, coverage rule,
   seed provenance), in Rust, Go and TypeScript alike.
5. `website/docs/reference/formats.md` has a row for `ess-conformance/66` or `/67`: open with no
   construct, read wherever `/64` and `/65` are, written by nothing yet; `CHANGELOG.md` says the same
   in one bullet.

## Scope

- `crates/verify/ess-conformance/src/scenario.rs` (`SUPPORTED_SUITE_FORMATS`, its test)
- `crates/verify/ess-conformance/src/suite_pair_66.rs` (new module, the shape of `suite_pair_64.rs`:
  `ORDINARY = 66`, `COVERAGE = 67`, `ADMITTED`), `src/lib.rs` (its `mod`)
- `crates/verify/ess-conformance/src/synthesis_seeds.rs` (111, 186), `src/go/mod.rs` (446, 477),
  `src/admission.rs` (349), `src/count_json.rs` and `src/coverage.rs` wherever they list admitted
  majors (the earlier opener changed both)
- `crates/verify/ess-conformance/src/go/runtime.go` (104, 1912)
- `crates/verify/ess-conformance/src/ts/runtime.ts` (2864, 7197, 7209, 7248) and the TypeScript
  tests the earlier opener changed (`runtime.test.ts`, `predicate.test.ts`)
- tests: `adversary_t1_pair_64_65.rs` (new cases), `system_issued_vocabulary.rs` (184), and the
  ess-cli support file `tests/support/coverage_producer.rs` (495) with ess-cli's
  `epic_formats_open.rs` (a case for `/66`)
- `website/docs/reference/formats.md` (one row)

## Owned files

The files under Scope, plus `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest`
(`ess-conformance/src` is hashed by the stamp; rewritten by the command under Gate) and
`CHANGELOG.md` (one bullet).

## Shared surfaces after this story

`suite_pair_66.rs`, the `/66`/`/67` row of `formats.md` and `CHANGELOG.md` are append-only for the
stories that fill the pair: each proposes its vocabulary line and the implementor of the one pull
request writes it in turn.

## Not yours

- Any vocabulary in the pair: each filling story adds its own.
- `ess-diff` versions (open choice DIFF in `FORMAT.md`).
- The source format (`ess/38`, its own opener).

## Tests first

New `#[test]`s in `crates/verify/ess-conformance/tests/adversary_t1_pair_64_65.rs` (existing
target): `a_suite_relabelled_66_is_admitted_as_one_at_64`, `a_coverage_suite_relabelled_67_is_admitted_as_one_at_65`,
`suite_68_is_refused_by_every_reader`. The Go and TypeScript cases reuse the packages the target
already builds (`typescript_package`, its Go counterpart); time each and name any over 5 s (bound
60 s per test, 120 s per test step). If the Go or TypeScript case exceeds 60 s, it is split, not
given a longer timeout.

## Gate

One at a time; no job in the build pool over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`;
  `cargo clippy -p ess-cli --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test adversary_t1_pair_64_65`
- `cargo test -p ess-conformance --locked --test system_issued_vocabulary`
- `cargo test -p ess-conformance --locked --lib` (the `scenario.rs` unit test)
- `cargo test -p ess-cli --locked --test epic_formats_open`
- `task test-xtask` (`website/` and `CHANGELOG.md` changed)
- the sources stamp, rewritten in this change:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
  re-run after every rebase, never merge the stamp by hand
- byte pins, each alone, none may move: `--test synthesis_suite_bytes_table`,
  `--test synthesis_suite_bytes_table_cli_read`, `--test adversary_related_values_all_bytes`,
  `--test external_beside_held_guard` (ess-conformance); `--test served_codec_names`, `--test go`
  (ess-synth)
- the slow lane only if a pin moves (it must not):
  `SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table -- --ignored every_repository_model_keeps_its_synthesized_bytes`

## Depends on

Nothing. Every story that writes suite vocabulary depends on this one (`APPLY.md` lists the edges).

## Changelog

Under `Added`:

- Suite pair `ess-conformance/66` (ordinary) and `/67` (coverage) is open with no construct: every
  reader (Rust, Go, TypeScript) admits it wherever it admits `/64` and `/65`, and nothing writes it
  yet.

## Fit review

Not an external request: an opener with no authored surface, recorded for the plan.

## Decisions

- One suite pair for the pull request, `/66` and `/67`, opened empty (coordinator, 2026-11-24, step
  49 of ORDER). No delta version and no source format here.
