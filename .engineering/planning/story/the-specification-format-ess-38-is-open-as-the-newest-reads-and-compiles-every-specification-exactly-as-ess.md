---
format: aep.planning-md/3
id: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
kind: story
status: active
title: The specification format ess/38 is open as the newest, reads and compiles every specification exactly as ess/37 does, and holds the gate the member-address constructs use below it
tags:
- wave-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:via-and-instance-may-name-a-member-of-a-struct-input
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/adversary_t1_epic_formats.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/epic_formats_open.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: cited
  path: crates/specify/ess-domain/src/command/input_path.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/format_history.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/input_path_addressing.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: cited
  path: models/toolchain/domains/specify.yaml
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
- confidence: cited
  path: website/docs/start/first-specification.md
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T21:48:19Z", actor: "human:timo", revision: 16}
- {from: "proposed", to: "active", at: "2026-11-23T21:48:21Z", actor: "human:timo", revision: 17}
---
## Outcome

- `ess specify formats` lists `ess/38`, unreleased and newest; `ess/37` is listed as before,
  released in 0.93.0, no longer newest, its row byte for byte as released.
- A specification declaring `ess/38` validates and compiles exactly as at `ess/37`: the IR is equal
  apart from the `format` field, and `examples/billing` is judged refusal for refusal alike at both.
- `ess_domain::command::input_path::addresses_members(format)` answers `true` from `ess/38` and
  `input_path::below_ess_38(at, what)` builds the `unsupported_format_version` refusal naming
  `ess/38`: the one gate s1 and s2 put every member address behind. Nothing calls them yet.
- The format's documents and pins name `ess/38` as the newest: the format and version-history
  pages, the walkthrough, the toolchain model and the rows that pin its digest.

## Why

- Decision D48 (the operator's, 2026-11-23): the feature of beyond10x/ess#640 opens `ess/38`; a
  released format keeps its meaning, so `ess/37`, released in 0.93.0 with no construct, stays
  exactly as released. Parents: `story:via-and-instance-may-name-a-member-of-a-struct-input`.
- Opening a format moves pins in seven places beside the gate (`../d48/format-opening.md`); in one
  unit ahead of s1 they are moved once, and s1 to s6 only add constructs behind the gate.

## Where

Read at commit `d3252eda9` of the wave tree (branch `integration/wave-2026-11-23`). How `ess/37`
was opened: commit `9f4f4e893` (its spec-format part); this unit does the same for `ess/38` and
opens no suite pair and no delta version (the suite major is chosen by vocabulary, not by the
specification format: `ConformanceSuite::select_fresh_format_for`,
`crates/verify/ess-conformance/src/scenario.rs` 340).

`crates/specify/ess-domain/src/system.rs`:

- `SUPPORTED_FORMATS` (53-57): append `38` after `37` (56).
- `FORMAT_HISTORY`: after row 37 (391-398, which is not touched), a row
  `FormatHistoryEntry { major: 38, release: None, added: &[PLACEHOLDER], stricter: &[] }`, with
  PLACEHOLDER the sentence under D3. The `const` assertion (402-420) holds both lists equal.
- `FormatVersion::V36` (550): after it, `pub const V38: Self = Self(38);` with the doc of D4. No
  `V37` (no construct gates it; constants exist only for gated formats).

`crates/specify/ess-domain/src/command/input_path.rs` (268 lines):

- after `admitted` (24-27):

  ```rust
  /// `true` when a specification in `format` may address a row through a member of a struct
  /// input: from `ess/38`. A registry without a format (`None`) admits it, as the gates that read
  /// `TypeRegistry::format()` do (`related_guard.rs` 769, `row_set.rs` 374).
  pub fn addresses_members(format: Option<FormatVersion>) -> bool {
      format.is_none_or(|format| format.major() >= FormatVersion::V38.major())
  }
  ```

- after `below_ess_22` (220-229):

  ```rust
  /// The refusal below `ess/38` for `what`, an addressing position written with a member of a
  /// struct input (`` `via: input.settings.primary` ``).
  pub fn below_ess_38(at: &ConstructRef, what: &str) -> ValidationError {
      ValidationError::at(
          at.clone(),
          ValidationCode::UnsupportedFormatVersion,
          format!("{what} names a member of a struct input, which requires specification format ess/38"),
      )
      .with_hint(FormatVersion::V38.declaration_hint())
  }
  ```

  The hint reads `` write `format: ess/38` on the source that declares the system ``
  (`FormatVersion::declaration_hint`, system.rs 586; `tests/format_hint_source_scan.rs` accepts
  it). Typed, in a sub-directory file: the census of `docs/design/review-typed-diagnostics.md`
  (top-level files of `ess-domain/src`) does not move; no `ValidationCode` is added, so
  `macro-guards.json` does not move.

`models/toolchain/domains/specify.yaml` 42: append `"ess/38"` to the `SpecificationFormat`
variants after `"ess/37"`.

`website/docs/reference/formats.md`: after row 203 (`format: ess/37`, not touched) the row
`` | Authored specification: `format: ess/38` | Specification `vN` | Unreleased. Opened for addressing a row through a member of a struct input (beyond10x/ess#640), with no construct yet: a specification declaring it reads exactly as at `ess/37`. Below it, each construct of the format is refused as `unsupported_format_version` naming `ess/38`. | ``
(s1 replaces the text after "Unreleased." with its constructs).

`website/docs/reference/spec-versions.md`: regenerated (Steps), never edited by hand; a row
`` | `ess/38` | unreleased | <PLACEHOLDER> | `` appears after row 80.

`website/docs/start/first-specification.md` 24 (`format: ess/37` in the `ess-tutorial` file block)
and 120 (`` `format: ess/37` is the newest version ``): `ess/38` in both.

`crates/edge/ess-cli/tests/epic_formats_open.rs` 58: `json!(true)` becomes `json!(false)` (ess/37
is no longer newest); the module doc (1-3) gains "Source `ess/38`, opened for the member-address
feature, reads exactly as `ess/37`."

## Tests first

Each test is timed by the implementor; each is expected under 2 s (one small model or a constant
read); none may run over 60 s; each target's step stays within 120 s.

1. `crates/specify/ess-domain/tests/format_history.rs`, new
   `ess_38_is_the_newest_and_unreleased_and_ess_37_keeps_its_released_row`:
   `SUPPORTED_FORMATS.last() == Some(&38)`; the last `FORMAT_HISTORY` row has `major == 38`,
   `release == None`; the row of 37 has `release == Some("0.93.0")`, `stricter` empty and `added ==
   ["The next format, open with no construct of its own: a specification declaring it reads and compiles exactly as at the format before it, and the constructs of one release are added to it, each refused as `unsupported_format_version` naming `ess/37` below it."]`
   (system.rs 395, byte for byte). Red today: `left: Some(37)`, `right: Some(38)`.
2. New file `crates/specify/ess-domain/tests/input_path_addressing.rs` (doc: "Addressing a row
   through a member of a struct input, `ess/38`, beyond10x/ess#640"; s1 and s2 append their
   sections after this test), `the_member_address_gate_opens_at_ess_38`:
   `addresses_members(Some(FormatVersion::new(37).unwrap()))` and `(Some(FormatVersion::V22))` are
   `false`; `(Some(FormatVersion::V38))` and `(None)` are `true`; and
   `below_ess_38(&ConstructRef::new(ConstructKind::Command, "shop.route.Send").key("outcomes").named("unknown-depot"), "`via: input.settings.depots`")`
   has `code == ValidationCode::UnsupportedFormatVersion`, `message ==`
   `` `via: input.settings.depots` names a member of a struct input, which requires specification format ess/38 ``,
   `hint == Some("write `format: ess/38` on the source that declares the system")` and `location
   == "command.shop.route.Send.outcomes.unknown-depot"`. Imports:
   `ess_domain::command::input_path::{addresses_members, below_ess_38}`,
   `ess_domain::FormatVersion`, `ess_primitives::error::{ConstructKind, ConstructRef,
   ValidationCode}`. Red today: does not compile (`addresses_members`, `below_ess_38` and
   `FormatVersion::V38` are not defined); say so in the report.
3. `crates/edge/ess-cli/tests/epic_formats_open.rs`, new
   `ess_38_is_listed_unreleased_as_the_newest_and_reads_as_ess_37`, shaped as
   `ess_37_is_listed_as_added_in_0_93_0_and_reads_as_ess_36` (48-83): the `ess/38` row of `ess
   specify formats --format json` has `release` `null` and `newest` `true`; `ledger_at(..,
   "older", "ess/37")` and `(.., "newer", "ess/38")` both validate, and the compiled JSON of the
   older with `ess/37` replaced by `ess/38` equals the newer's ("IR bytes differ apart from the
   format field"). Red today: panics `ess/38 is not listed: [...]`.
4. `crates/edge/ess-cli/tests/adversary_t1_epic_formats.rs`: move the closure `said` (60-72) into
   `fn judged(path: &Path) -> (Option<i32>, String)`, which also replaces `ess/38` by `ess/N`;
   the existing test calls it unchanged in meaning, and new
   `examples_billing_at_ess_38_is_judged_exactly_as_at_ess_37` compares `billing_at(..,
   "ess/37")` with `(.., "ess/38")`. Red today: the two differ, `ess/38` is refused as an
   unsupported format version and `ess/37` exits 0 (print both).
5. Existing, turned red by the `system.rs` edit and green again by this unit's other edits (no new
   test): `epic_formats_open.rs` 58 (`newest`), `crates/edge/ess-cli/tests/tutorial_page.rs` 927
   `the_walkthrough_writes_the_newest_format_and_pins_its_toolchain` (first-specification.md),
   `crates/edge/ess-xtask/tests/model_enums.rs` 217
   `the_model_lists_every_admitted_specification_format` (specify.yaml), `cargo xtask
   format-history --check` (spec-versions.md), the stamp test and the three tables (Steps 6-7).

## Steps

1. Start from `d3252eda9` or a descendant on the wave branch; `git rev-parse --short HEAD` in the
   report. Write tests 1-4; run them and keep the red lines.
2. `system.rs` (list, row 38, `V38`), then `input_path.rs` (the two functions). Tests 1-2 green.
3. `specify.yaml`, `first-specification.md`, `epic_formats_open.rs` 58 and its doc, `formats.md`.
4. `slot cargo xtask format-history`, then `slot cargo xtask format-history --check`.
5. Re-pin, from the repository root, the rows the toolchain model's new variant moves (37's commit
   moved 1, 5 and 1 lines; only rows whose label starts `system-dir:models/toolchain` may move,
   any other moved row is a defect of this unit: stop and report it):
   `LOWERED_DEFINITIONS_TABLE_WRITE="$PWD/crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowered_definitions_table`;
   `SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv" SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table every_repository_component_keeps_its_synthesized_bytes`;
   `SYNTHESIS_SUITE_BYTES_TABLE_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv" SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table -- --ignored every_repository_model_keeps_its_synthesized_bytes`
   (each command is printed by its own failing test; the last is a slow-lane test, run once here).
   `git diff` of each table shows the toolchain rows only; name their count in the report.
6. Re-stamp the synthesis sources (`system.rs` is under a hashed crate, `specify.yaml` is a model):
   `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
   then `slot --timeout 30m task test-slow PACKAGES=ess-conformance`.
7. `slot task projection-check` (`schema --check`, `docs`, `format-history --check`: the schema
   does not move, `format` is a pattern); the Gate; `slot task fmt-check`.

## Owned files

- `crates/specify/ess-domain/src/system.rs` (`SUPPORTED_FORMATS`, row 38 as a placeholder, `V38`)
- `crates/specify/ess-domain/src/command/input_path.rs` (`addresses_members`, `below_ess_38`)
- `crates/specify/ess-domain/tests/format_history.rs` (one test)
- `crates/specify/ess-domain/tests/input_path_addressing.rs` (new, one test)
- `crates/edge/ess-cli/tests/epic_formats_open.rs`, `crates/edge/ess-cli/tests/adversary_t1_epic_formats.rs`
- `models/toolchain/domains/specify.yaml` (line 42)
- `website/docs/reference/formats.md` (the new `ess/38` row), `website/docs/reference/spec-versions.md`
  (regenerated), `website/docs/start/first-specification.md` (24, 120)
- `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv`,
  `crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv`,
  `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` (toolchain rows,
  re-pinned by command), `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest`

## Not yours

- Row 37 of `FORMAT_HISTORY`, `formats.md` row 203 and `spec-versions.md` row 80: released, never
  edited. `crates/edge/ess-xtask/src/docs.rs` (the `ess` family is read from `FORMAT_HISTORY`).
- Every construct: `address`, `reads_member` and every call of the gate (s1, s2); every
  `ess-compiler`, `ess-conformance/src` and `crates/generate/` source; `CHANGELOG.md` (D5).
- The release work: row 38's `release`, `changes/*.yaml`, the dated `Language:` line.

Shared files (s0 is alone in round 0 and lands first): with s1, `system.rs` (s1 replaces row 38's
placeholder), `input_path.rs` (s1 adds other functions), `input_path_addressing.rs` (s1 appends),
`formats.md` row `ess/38` (s1 rewrites the text after "Unreleased."), `spec-versions.md` (s1
regenerates) and the stamp (s1 re-stamps on the merged tree). Never merged by hand.

## Gate

`ess-domain` changed, so `ess-cli`; a format version and `website/` changed, so `task test-xtask`;
`task projection-check`. Every cargo command carries `CARGO_PROFILE_DEV_DEBUG=false
CARGO_INCREMENTAL=0`; named `--test` targets one per command; a pool job ends after 10 minutes
unless it names its time (`SLOT_TIMEOUT=30m` / `slot --timeout 30m`, 30 at most); a test over
60 s is killed, a test step ends within 120 s; no Python, nothing in `/tmp`, no `RUSTC_WRAPPER`,
`SCCACHE_*` or `-j`.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-domain --all-targets --locked -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test format_history
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test input_path_addressing
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test format_hint_source_scan
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --lib
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test typed_diagnostics
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test selection_precedence_table
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test string_alphabet_and_length
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowered_definitions_table
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table_cli_read
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test adversary_related_values_all_bytes
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test external_beside_held_guard
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-cli --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test epic_formats_open
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test adversary_t1_epic_formats
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test specify_formats
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test adversary_460_pass1
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test tutorial_page
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-openapi --locked --test adversary_ess_issues_2_u6
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test served_codec_names
slot --timeout 30m task test-slow PACKAGES=ess-conformance
slot task projection-check
slot --timeout 30m task test-xtask
slot --timeout 30m task ci-lint
```

`ess-cli`: the targets that read the newest format or the `ess/37` row; the others are left to
the forge's shards. `ess-synth`'s `go` pin: run `SLOT_TIMEOUT=30m cargo test -p ess-synth --locked
--test go` if `go` is on `PATH`, else name it not run. Do not start `task check` or a
whole-workspace run.

## Acceptance

1. `cargo test -p ess-domain --locked --test format_history` passes test 1 (`ess/37`'s row as
   released; `ess/38` newest, unreleased).
2. `cargo test -p ess-domain --locked --test input_path_addressing` passes test 2 (the gate and its
   exact message and hint).
3. `cargo test -p ess-cli --locked --test epic_formats_open` and `--test
   adversary_t1_epic_formats` pass tests 3 and 4 and their existing tests.
4. `tutorial_page`, `specify_formats`, `adversary_460_pass1` pass; `task test-xtask` passes
   (`model_enums`); `slot task projection-check` passes.
5. The three tables changed in toolchain rows only (count in the report); the stamp test and `task
   test-slow PACKAGES=ess-conformance` pass.
6. Every target of the Gate passes; `git diff --stat` shows only the owned files.

## Decisions

- D1. `ess/38` opens with no construct, like `ess/37` in `9f4f4e893`; s1 and s2 add theirs behind
  this unit's gate in the same wave, so the released row names constructs as rows 22-36 do.
- D2. No suite pair and no delta version: the suite major follows vocabulary (`scenario.rs` 340),
  and `ess-conformance/64`/`65` and `ess-diff/24` already exist for later vocabulary.
- D3. Row 38's placeholder: "Opened for addressing a row through a member of a struct input
  (beyond10x/ess#640), with no construct yet: a specification declaring it reads and compiles
  exactly as at `ess/37`, and each construct the format gains is refused as
  `unsupported_format_version` naming `ess/38` below it." s1 replaces it.
- D4. `V38`'s doc: "Addressing through a member of a struct input: `via:`, `all:` elements,
  `instance:`, `each: {in:}` and `{missing:}` read the `input.<path>` `sets:` reads
  (`story:via-and-instance-may-name-a-member-of-a-struct-input`, beyond10x/ess#640); `ess/37`,
  released in 0.93.0 with no construct, keeps its meaning."
- D5. No `CHANGELOG.md` bullet: a format that ships constructs is announced by them (0.91.0,
  "Format `ess/36`: …"), and s1's `### Added` bullet starts "Format `ess/38`:"; an "opened with no
  construct" bullet would be false at the release.
- D6. The gate lives in `input_path.rs` beside `admitted`/`below_ess_22`, the `ess/22` path rule
  the feature extends; `None` admits, as `TypeRegistry::format()` gates do.
- D7. Row 37, its `formats.md` and `spec-versions.md` rows, and its tests stay as released (D48).

## Open for the operator

None (D48 decided the format).

## Changelog

None (D5).
