---
format: aep.planning-md/3
id: story:feature-request-466
kind: story
status: active
title: ess-cli has no source for a trailing argument list (`-- <args…>`)
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#466
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-cli-project/src/runtime.rs
- confidence: cited
  path: crates/generate/ess-cli-project/tests/projection.rs
- confidence: inferred
  path: crates/generate/ess-cli-project/tests/trailing_arguments.rs
- confidence: cited
  path: crates/specify/ess-cli-contract/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-cli-contract/src/wire.rs
- confidence: cited
  path: crates/specify/ess-cli-contract/tests/binding.rs
- confidence: cited
  path: docs/design/cli-presentation-binding.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T01:48:10Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-06T01:48:11Z", actor: "human:timo", revision: 5}
---
## Outcome
Resolve beyond10x/ess#466: ess-cli has no source for a trailing argument list (`-- <args…>`).

## Origin
beyond10x/ess#466, filed 2026-10-05; an adopter scoping a generated command that launches an operator-pinned program and must pass it everything after `--` verbatim. Reproduced minimally and fresh in `<fit-review scratch>/probe-466/`.

## Fit review
1. Need: a command whose input carries "the arguments of the program it launches" must collect every argv word after the first `--` as one `List<String>`, verbatim, with no option parsing. Minimal reproduction `probe-466/` (installed ess 0.53.0): a model with `demo.LaunchInput {connection: String, args: List<String>}` and an `ess-cli/1` binding.
   - `b-trailing.yaml` (`source: {kind: trailing}`) is refused: `unknown variant 'trailing', expected one of 'option', 'positional', 'document', 'protected'` (`b-trailing.out`).
   - `a-positional.yaml` (`{kind: positional, index: 1}`) and `c-option.yaml` (`{kind: option, long: arg}`) compile, but the field then takes one argv word holding JSON: every value other than String or enum goes through `json_argument` (`crates/generate/ess-cli-project/src/runtime.rs:624-627`), and every positional and option has `num_args(1)` (`runtime.rs:319-324`).
   - The requester's syntax (theirs): `sources: {args: {trailing: true}}` beside `{option: --adapter}`. It does not match the authored shape, which is `arguments: [{field, source: {kind: …}}]` (`crates/specify/ess-cli-contract/src/wire.rs:127-169`; `docs/design/cli-presentation-binding.md:44-55`).
2. Class: gap. The `-- <args>` convention cannot be bound at all. The JSON idiom changes what the operator types (`'["-v","x"]'` instead of `-- -v x`), so a launcher wrapping another program cannot offer it. A hand-written parser over the generated package breaks the generate-and-`--check` contract (`crates/generate/ess-cli-project/src/lib.rs:18-19` copies `runtime.rs` verbatim).
3. Existing idiom: a `List<String>` positional or option taking JSON text (above). It is partial: it is not verbatim argv and it collides with shell quoting. No source today reads past `--`. `selected_json` already stops at `--` (`runtime.rs:406-420`), so `--` is already the end of adapter parsing for output selection.
4. Fit: a fifth `ArgumentSource` variant, spelled like its siblings.
   - Key: `source: {kind: trailing}`, no other fields. The tag is the existing `kind` (`wire.rs:129`); a variant with no fields matches the closed `deny_unknown_fields` reader.
   - Admission (`resolve.rs:283-362`): the field's shape must be exactly `List<String>` (an unconstrained String newtype resolves to `String`, `resolve.rs:76-78`). `Optional<List<String>>` is refused: absent and empty would be two spellings of one value. At most one `trailing` per command. It reserves no flag and is not a stdin source (`resolve.rs:296`, `:347-351`).
   - Runtime (`runtime.rs:326-404`): one clap `Arg` with `last(true)`, `num_args(0..)`, `StringValueParser`, after every positional. Values are reachable only after the first `--`. A later `--` is a value. Absent means `[]`, always inserted, because `payload` skips absent text (`runtime.rs:619-623`) and a required list must be present (`wire.rs:72-79`). Values become a JSON array of strings with no `json_argument`. Non-UTF-8 is `cli_parse`, exit 2, without the value (`runtime.rs:658-672`).
   - Positionals: they keep their consecutive-index rule (`resolve.rs:352-360`) and are only filled before `--`. With an optional positional, `cmd -- x` puts `x` in the list, not the positional.
   - Help: clap renders `[-- <field:args>...]` (inferred), using the id convention every field already has (`runtime.rs:319-324`, no `value_name`). Renaming value names in general is out of scope.
   - Targets: the generated Rust/Clap package is the only target of `ess-cli/1`. Dynamic targets need String payload fields (`resolve.rs:244-271`), so they cannot meet a trailing field.
5. Second adopter: a sandbox CLI `sandbox exec --profile ci -- cargo test -p core`. Also a database tool `db shell --env staging -- --no-psqlrc -c 'select 1'`. Both wrap another program's argv.
6. Cost: one new `kind` value, two refusals, no new error code, and no format bump.
   - Precedent: `invalid_input` was added to `ess-cli/1` and `ess-cli-plan/1` in 0.51.0 with no bump (`CHANGELOG.md:417`; commit `482bc33c2` touched `wire.rs`, `lib.rs`, `resolve.rs`). An older ess refuses the new kind by name (probe `b`), as it refused `invalid_input`. Every existing binding means what it did, and its `binding.json` bytes do not change.
   - Generated `src/wire.rs` and `src/runtime.rs` bytes change for every package (`lib.rs:18-19`; probe `gen-a/src/runtime.rs` is byte-identical to the tree). So `ess generate cli --check` reports drift until the package is regenerated, as with any runtime change.
7. Alternatives:
   - (a) Change nothing and point to the JSON list idiom. Refused: it is not `--` argv, and launchers cannot use it.
   - (b) `ess-cli/2` carrying the new kind. Refused: no existing document changes meaning, and the 0.51.0 precedent extended `/1`. A bump would add a reader branch and a migration for nothing.
   - (c) `{kind: positional, index: N, variadic: true}`. Refused: a variadic positional before `--` would swallow option-like words and conflict with the "optional positionals last" rule. It is also not verbatim.
   - (d) Chosen: the requester's semantics with the existing key shape, `{kind: trailing}`.

## Decisions
accept, redesigned — `ArgumentSource::Trailing` (`source: {kind: trailing}`) binds one required `List<String>` field per command to every argv word after the first `--`, verbatim. The field is `[]` when absent, and help shows `[-- <field:args>...]`. Changed from the request: it is spelled as a `kind` under `arguments[].source`, not `{trailing: true}` under `sources`. `Optional<List<String>>` is refused. No format bump: `ess-cli/1` and `ess-cli-plan/1` gain the kind compatibly, following `invalid_input` in 0.51.0. Generated packages need regeneration (`--check` drift), and the CHANGELOG says so. Not ess/23 and not ess-conformance.

## Acceptance
- trailing_collects_everything_after_double_dash_verbatim: `demo launch --connection c -- --flag -x v -- --output json` hands the handler `args: ["--flag","-x","v","--","--output","json"]`, and output stays human.
- trailing_absent_or_bare_double_dash_is_empty_list: `demo launch --connection c` and `demo launch --connection c --` both hand `args: []`.
- trailing_words_before_double_dash_are_parse_failures: `demo launch --connection c extra` exits 2 with `cli_parse`, and stderr does not contain `extra`.
- trailing_after_optional_positional: with an optional positional at index 1, `demo launch p -- a b` gives positional `p`, `args [a,b]`; `demo launch -- p` gives no positional, `args [p]`.
- trailing_help_shows_double_dash_list: the leaf `--help` output contains `[-- <field:args>...]`.
- trailing_non_utf8_value_is_parse_failure: a non-UTF-8 word after `--` exits 2 with `cli_parse` and no value bytes.
- binding_refuses_second_trailing_source: two `trailing` sources in one command are refused, naming "at most one trailing source".
- binding_refuses_trailing_on_other_shapes: `String`, `List<Integer>` and `Optional<List<String>>` fields with `trailing` are each refused.
- older_kind_refusal_names_trailing: the closed reader's unknown-variant message lists `trailing` among the expected kinds.
- existing_binding_plan_bytes_unchanged: the projection fixture's `binding.json` is byte-identical before and after.
- design page `docs/design/cli-presentation-binding.md` states the source, admission and runtime rules beside `positional`. A test reading the page holds the `{kind: trailing}` example.

## Scope
- crates/specify/ess-cli-contract/src/wire.rs  cited — `ArgumentSource` :127-159 gains `Trailing`
- crates/specify/ess-cli-contract/src/resolve.rs  cited — `validate_command` :283-362, shape and at-most-one checks
- crates/specify/ess-cli-contract/tests/binding.rs  cited — refusal tests beside :287-309
- crates/generate/ess-cli-project/src/runtime.rs  cited — `leaf_command` :326-404, `argument_text` :459-501, `payload` :605-643
- crates/generate/ess-cli-project/tests/trailing_arguments.rs  inferred — process tests with a recording handler, as `invalid_input.rs`
- crates/generate/ess-cli-project/tests/projection.rs  cited — existing `binding.json` byte pin
- docs/design/cli-presentation-binding.md  cited — :124-130 argument-source paragraph
- CHANGELOG.md  inferred — Added entry and the regenerate note
