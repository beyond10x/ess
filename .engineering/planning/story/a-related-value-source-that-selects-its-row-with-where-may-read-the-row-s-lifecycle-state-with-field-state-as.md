---
format: aep.planning-md/3
id: story:a-related-value-source-that-selects-its-row-with-where-may-read-the-row-s-lifecycle-state-with-field-state-as
kind: story
status: draft
title: 'A related value source that selects its row with `where:` may read the row''s lifecycle `state` with `field: state`, as the `via:` form already may, with the same handling of no row and several rows.'
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#822
relations:
- serves: vision:O2
- decomposes: epic:generic-row-queries
revision: 1
---
# d822: a where-selected related value reads the row's state

format: decided before acceptance (DF-D5; default: the open source format, `ess/38`).

Parent: `epic:generic-row-queries` ("Queries over rows"). Siblings already implemented under it:
`story:a-set-effect-selection-reads-the-rows-state` (a set effect's `where:` reads `state`,
`ess/30`, #564) and `story:row-queries-navigate-a-reference` (#557, the `via:` state read). No story
in the store asks for `field: state` in a `where:`-selected value source.

Size: M (about 330 changed lines: 120 source across validation, IR, compiler and interpreter; 30
docs; 180 test).
Fixed suite bytes of committed models: none (a model using the new read is refused today; every
other model keeps its IR and suite bytes, as the `via:` precedent did).
Patch-release candidate: no (new meaning at a new source format).

## Need

beyond10x/ess#822. An event (or a stored field) cannot carry the lifecycle state of a row that the
command finds by a predicate rather than by an identity in its input. The guard can read that row's
`state` in the same `where:`, so the value is decided; only the value source cannot name it. Today an
author writes `{generated: true}` (an implementation invents a state) or one outcome per state with
a literal (callers see different outcomes).

Requester's proposal, theirs: allow `field: state` in `{related: {entity, where, field}}`, with the
same none/several handling as other fields.

## Reproduction

Reproduced on 0.98.0. Model `repro/822/` (`format: ess/37`): `club.member.Member {member_id, email}`
with states `Pending`, `Active`; `Join(email)` with `not-invited`
(`when_related: {entity: Member, where: email == input.email, exists: false}`) and `joined`, whose
`Joined` payload reads `member_id` and `state` through
`{related: {entity: club.member.Member, where: email == input.email, field: …}}`.

```console
$ ess specify validate --path .
  - error[ESS-COMMAND-001]: `state` is not a field of `club.member.Member`
  help: `club.member.Member` holds: member_id, email
  --> domains/member.yaml:55:5
exit 1
```

Without the `state` line the same model validates (the issue; `member_id` through the same form is
admitted).

## Outcome

From the format DF-D5 picks, `field: state` in a `where:`-selected related value (and in the
ordered-head form, `{related: {entity, where, order_by, first, field: state}}`) is admitted, typed as
the entity's `State`, and read as the selected row's state before the outcome. No row and several
rows are answered as for any other field of the form. Below that format it is refused as
`unsupported_format_version` naming it, with the hint every format refusal carries; a model without
it keeps its IR bytes. The interpreted target and synthesis read it; generated Rust and Go keep the
command an obligation, as they do for every related value source today.

## Where it lands (read in the source, not run)

- Validation: `crates/specify/ess-domain/src/command/value_expression.rs`, `check_selection`
  (2014-2102), refusal at 2086-2088 (`entity_field` → `not_a_field`, 1134-1152); the head form in
  `check_head` (2174-2176). The branch to reuse: `check_related` (959-974), which admits
  `field == EntitySpec::STATE` from `FormatVersion::V28` (`related_state_below`, 1085-1099).
- Lowering: `crates/specify/ess-compiler/src/resolve.rs`, `selection_field` (3437-3485, refusal near
  3455) and `head_field` (3491+); the `via:` state branch in `related_field` (3771-3806) emits
  `ResolvedPayloadValue::RelatedState`.
- IR: `RelatedSelection` (`crates/specify/ess-compiler/src/ir.rs:1366-1376`) carries `field: String`
  read as a declared field; `RelatedState` (1313-1330) is its own variant so that no field lookup
  meets `state`. The where form needs one additive variant (and one for the head form), carried only
  by a model that uses it.
- Interpreter: `crates/verify/ess-conformance/src/interpret/execute.rs:2632-2637` reads
  `RelatedSelection` through `row_set::Reader::one` and `determined::selected`
  (`interpret/execute/determined.rs:666`); the state read of the `via:` form is
  `determined.rs:85-120`.
- Synthesis readers that need an arm: `synthesize/row_set.rs:103, 295, 1098`, `witness.rs:1772,
  1892`, `synthesize.rs:13042, 13946`, `synthesize/related.rs:113`.
- Generated targets: `crates/generate/ess-synth/src/determined.rs:603-610` keeps a related source
  owed; `rust/behaviour.rs:3001-3004` and `go/behaviour.rs:3875-3878` are `unreachable!` behind that.
  Entity runtime refuses it (`crates/generate/ess-entity-runtime/src/subset.rs:140`).

## Acceptance

1. Red today: `repro/822/` with `format:` raised to the chosen format validates.
2. At `ess/37` the same model is refused as `unsupported_format_version` naming the chosen format,
   at the `state` line.
3. Synthesis of the reproduction (format raised) writes a `joined` scenario whose `Joined` event
   expects the state of the member the scenario arranged (`Pending`, and `Active` where a scenario
   arranges an active member), and the suite passes on the interpreted target.
4. Several rows matching `where:` and no row matching are answered as `field: member_id` is answered
   by the same form (the same refusal or the same validation rule, whichever applies to other
   fields today).
5. `ess generate synthesize --target rust` and `--target go` keep `Join` an obligation and generate
   without panicking.
6. A model without the read keeps its IR bytes and digest (the IR pins of `ess-compiler` stay green).
7. The reference page `website/docs/reference/spec-versions.md` row of the chosen format and the
   value-source table in `website/docs/guides/specify/values-and-views.md` name the read.

## Scope

As "Where it lands": one validator branch (and the head's), one compiler branch per form, one IR
variant per form, one interpreter arm, the synthesis arms listed, docs.

## Owned files

- `crates/specify/ess-domain/src/command/value_expression.rs` (`check_selection`, `check_head`)
- `crates/specify/ess-compiler/src/resolve.rs` (`selection_field`, `head_field`),
  `crates/specify/ess-compiler/src/ir.rs` (new variant)
- `crates/verify/ess-conformance/src/interpret/execute.rs`, `interpret/execute/determined.rs`, and
  the synthesis files listed above (one arm each)
- `crates/specify/ess-domain/tests/row_queries_navigate.rs`,
  `crates/verify/ess-conformance/tests/a_command_with_two_related_selectors_arranges_both.rs`
  (new `#[test]`s)
- `website/docs/reference/spec-versions.md`, `website/docs/guides/specify/values-and-views.md`,
  `CHANGELOG.md`

## Not yours

- Opening the format: the active story that opens `ess/38` (s0 of
  `story:at-ess-38-instance-and-each-in-name-a-member-of-a-struct-input-and-the-documents-list-every-addressing`,
  operator decision D48). This story only adds a construct to it.
- `{related: {via: payload.<Event>.<field>, …}}` (the issue's third form tried): a different read,
  not asked.
- Generated Rust and Go realization of related value sources.

## Tests first

- `crates/specify/ess-domain/tests/row_queries_navigate.rs` (existing; the `via:` state read at line
  61): `a_where_selected_related_value_reads_state_from_the_new_format` and
  `a_where_selected_state_read_below_it_names_the_format` (acceptance 1, 2).
- `crates/verify/ess-conformance/tests/a_command_with_two_related_selectors_arranges_both.rs`
  (existing): `a_where_selected_state_is_carried_by_the_event` (acceptance 3).
Each under 5 s (bound 60 s per test, 120 s per step).

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy` and `cargo test --locked` for `-p ess-domain` (`--test row_queries_navigate`),
  `-p ess-compiler`, `-p ess-conformance` (`--test
  a_command_with_two_related_selectors_arranges_both`, one `--test` at a time), `-p ess-synth`,
  `-p ess-cli` (table: `ess-domain`/`ess-compiler` also gate `ess-cli`)
- `task test-xtask` (format version, diagnostic, `website/`, `CHANGELOG.md`); `cargo xtask schema`
  then `task projection-check` if `RawSpecFile` changes
- byte pins: `ess-conformance` `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes`, `external_beside_held_guard`; `ess-synth`
  `served_codec_names`, `go`; `ess-openapi` `adversary_ess_issues_2_u6`; `ess-compiler`
  `string_alphabet_and_length`; `ess-cli` `tutorial_page`

## Depends on

DF-D5; the story that opens `ess/38` (its s0) if DF-D5 A is taken.

## Changelog

Under `Added` (it is a construct; the issue is not labelled a bug):

- From `format: ess/38`, a related value that selects its row with `where:`, or the head of an
  ordered selection, may read the row's lifecycle state, `{related: {entity, where, field: state}}`;
  earlier formats refuse it naming `ess/38`, and a model without it keeps its IR bytes (#822).

## Fit review

1. Need: carry the state of a row found by a predicate. Requester's syntax above, labelled theirs.
2. Class: gap (`ess/28` admitted the read for `via:` only; `spec-versions.md:71`), and a sibling
   inconsistency: the guard's `where:` reads `state` (`ess/20`, `spec-versions.md:63`), the set
   effect's `where:` reads it (`ess/30`, `:73`), the value source's `where:` form does not.
3. Expressible today: only through `via: input.<id>`, which needs the identity in the input; the
   issue's command has the email. The two workarounds the issue names move or invent the figure.
4. Fit (grid R, "what can be read where"): the column "`state` of a row read through a relation or
   a selection" is open in guards, set effects and `via:` values; this opens the last cell of the
   column, both forms of selection together. Same key, same spelling (`field: state`), same typing.
5. Second adopter: an order found by its external reference whose state an acknowledgement event
   carries.
6. Cost: one construct in the open format; an additive IR variant; no new diagnostic code (the
   format refusal and ESS-COMMAND-001 exist).
7. Considered: change nothing (rejected: the guard decides the value and the value cannot be named);
   allow `via: payload.<Event>.<id>` instead (rejected: a second spelling for one read); admit
   `field: state` in both selection forms (chosen, as proposed, plus the head form for symmetry).

## Decisions

- Decision: accept as proposed, widened to the ordered-head form.
- DF-D5: the format. A (default) the open format `ess/38`; B the next format after it; C no format
  (lift the refusal at every format: rejected by precedent, every `state` lift so far was gated).
