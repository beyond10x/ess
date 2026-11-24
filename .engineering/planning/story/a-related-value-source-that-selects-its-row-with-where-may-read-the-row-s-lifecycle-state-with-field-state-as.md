---
format: aep.planning-md/3
id: story:a-related-value-source-that-selects-its-row-with-where-may-read-the-row-s-lifecycle-state-with-field-state-as
kind: story
status: active
title: 'A related value source that selects its row with `where:` may read the row''s lifecycle `state` with `field: state`, as the `via:` form already may, with the same handling of no row and several rows.'
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#822
relations:
- serves: vision:O2
- decomposes: epic:generic-row-queries
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:25:02Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-24T03:25:04Z", actor: "human:timo", revision: 5}
---
# d822 part 1: a where-selected related value reads the row's state

format: `ess/38`, the one new source format of the one pull request (D77, 2026-11-24), opened by the unit in work for #640 (DF-D5 A).

Part 1 of 2 for beyond10x/ess#822, split by layer (C19): this unit admits the read, lowers it to
the IR and gives every exhaustive consumer of the IR its arm; part 2,
`d822-a-where-selected-state-is-read-by-the-interpreter-and-synthesis`, makes the interpreted target
answer it and synthesis expect it. Until part 2 lands, the interpreted target answers such a
command unsupported and synthesis records it as an obligation, by the paths every related value
source already takes where it is not decided. Both parts go into the one pull request of open
issues, part 1 first.

Parent: `epic:generic-row-queries` ("Queries over rows"). Siblings already implemented under it:
`story:a-set-effect-selection-reads-the-rows-state` (a set effect's `where:` reads `state`,
`ess/30`, #564) and `story:row-queries-navigate-a-reference` (#557, the `via:` state read). No story
in the store asks for `field: state` in a `where:`-selected value source.

Size: L- (about 470 changed lines: 40 validation, 50 compiler, 30 IR, about 130 for the arms in 17
consumer files counted below, 50 docs and design page, 170 test). Under 600 after the split.
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
$ ess specify validate --path repro/822
  - error[ESS-COMMAND-001]: `state` is not a field of `club.member.Member`
  help: `club.member.Member` holds: member_id, email
  --> domains/member.yaml:55:5
exit 1
```

Without the `state` line the same model validates (the issue; `member_id` through the same form is
admitted).

## Outcome

From `ess/38` (DF-D5 A), `field: state` in a `where:`-selected related value (and in the
ordered-head form, `{related: {entity, where, order_by, first, field: state}}`) is admitted, typed as
the entity's `State`, and lowered to its own IR variant per form (as `RelatedState` is its own
variant beside `RelatedVia`, so that no field lookup meets `state`). No row and several rows are
answered by the same validation rules as any other field of the form. Below that format it is
refused as `unsupported_format_version` naming it, with the hint every format refusal carries; a
model without it keeps its IR bytes.

## Where it lands (read in the source, not run)

Lines at `41dd4aa9e`.

- Validation: `crates/specify/ess-domain/src/command/value_expression.rs`, `check_selection`
  (2014-2102), refusal at 2086-2088 (`entity_field` → `not_a_field`, 1134-1152); the head form in
  `check_head` (2107, refusal at 2174-2176). The branch to reuse: `check_related` (959-974), which
  admits `field == EntitySpec::STATE` from `FormatVersion::V28` (`related_state_below`, 1085-1099).
- Lowering: `crates/specify/ess-compiler/src/resolve.rs`, `selection_field` (3437-3485, refusal near
  3455) and `head_field` (3491+); the `via:` state branch in `related_field` (3771-3806) emits
  `ResolvedPayloadValue::RelatedState`.
- IR: `RelatedSelection` (`crates/specify/ess-compiler/src/ir.rs:1366-1376`) and `RelatedHead`
  (1387) carry `field: String` read as a declared field; `RelatedState` (1313-1330) is its own
  variant. Two additive variants, carried only by a model that uses them.
- Consumers that match `ResolvedPayloadValue` exhaustively (every file that names `RelatedState`,
  `RelatedSelection` or `RelatedHead` outside the compiler; `git grep` at `41dd4aa9e`):
  - `crates/generate/ess-entity-runtime/src/lib.rs` (3-4 sites), `src/subset.rs` (refuses, 140)
  - `crates/generate/ess-gen/src/http.rs`
  - `crates/specify/ess-service-contract/src/lib.rs`
  - `crates/verify/ess-diff/src/diff.rs` (2 sites)
  - `crates/generate/ess-synth/src/determined.rs` (626, keeps a related source owed),
    `src/rust/behaviour.rs` (3001-3004) and `src/go/behaviour.rs` (3875-3878), `unreachable!`
    behind it
  - `crates/verify/ess-conformance/src/`: `linearize.rs`, `web.rs`, `witness.rs` (1772, 1892),
    `authored/ties.rs`, `interpret/execute.rs` (2632-2637), `interpret/execute/determined.rs`,
    `synthesize.rs` (11626, 11715, 13042, 13946, 14510), `synthesize/row_set.rs` (103, 295, 1098,
    3759, 4968), `synthesize/related.rs` (113), `synthesize/aggregate.rs`,
    `synthesize/existence.rs`, `synthesize/system_issued.rs`
  In this unit each `ess-conformance` arm answers the new variants as it answers a related value
  source it cannot decide (the interpreter: unsupported, naming the read; synthesis: an obligation);
  part 2 replaces the interpreter's and synthesis's arms with the read.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

1. Red today: `repro/822/` with `format:` raised to the chosen format validates and compiles; the
   IR holds the new variant for `Joined.state`.
2. At `ess/37` the same model is refused as `unsupported_format_version` naming `ess/38`,
   at the `state` line.
3. The ordered-head form with `field: state` validates at the chosen format and is refused below
   it, the same way.
4. Several rows matching `where:` and no row matching are answered as `field: member_id` is answered
   by the same form (the same refusal or the same validation rule, whichever applies to other
   fields today).
5. `ess generate synthesize --target rust` and `--target go` keep `Join` an obligation and generate
   without panicking; `ess verify diff` between the model and itself reports nothing; the entity
   runtime refuses the command as it refuses every related value source.
6. A model without the read keeps its IR bytes and digest (the IR pins of `ess-compiler` stay green)
   and every committed suite keeps its bytes.
7. `website/docs/reference/spec-versions.md` (the chosen format's row), the value-source table in
   `website/docs/guides/specify/values-and-views.md` and the design page (below) name the read.

## Scope

As "Where it lands": one validator branch per form, one compiler branch per form, one IR variant
per form, one arm per consumer above, docs. The design page is amended first: a construct is a
design page before it is code. The page is `docs/design/generic-language-features.md` § G2 "Queries
over rows" (line 218), which holds the selection reads and grid R; § G4 (line 450) is the
precedence rule and is not touched.

## Owned files

- `docs/design/generic-language-features.md` (§ G2)
- `crates/specify/ess-domain/src/command/value_expression.rs` (`check_selection`, `check_head`)
- `crates/specify/ess-compiler/src/resolve.rs` (`selection_field`, `head_field`),
  `crates/specify/ess-compiler/src/ir.rs` (two variants)
- each consumer file listed under "Where it lands" (one arm per site)
- `crates/specify/ess-domain/tests/row_queries_navigate.rs`,
  `crates/specify/ess-compiler/tests/row_set_ir.rs` (new `#[test]`s)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (rewritten by the stamp
  command under Gate)
- `website/docs/reference/spec-versions.md`, `website/docs/guides/specify/values-and-views.md`,
  `CHANGELOG.md`

## Not yours

- Opening the format: the active story that opens `ess/38` (s0 of
  `story:at-ess-38-instance-and-each-in-name-a-member-of-a-struct-input-and-the-documents-list-every-addressing`,
  operator decision D48). `ess/38` does not exist at `41dd4aa9e` (no `V38` in
  `ess-domain/src/system.rs`, no row in `spec-versions.md`), so this unit starts after that s0.
- The interpreter's read and synthesis's expectation: part 2.
- `{related: {via: payload.<Event>.<field>, …}}` (the issue's third form tried): a different read,
  not asked.
- Generated Rust and Go realization of related value sources.

## Tests first

- `crates/specify/ess-domain/tests/row_queries_navigate.rs` (existing; the `via:` state read at line
  61): `a_where_selected_related_value_reads_state_from_the_new_format`,
  `a_where_selected_state_read_below_it_names_the_format`,
  `an_ordered_head_state_read_follows_the_same_format` (acceptance 1-3).
- `crates/specify/ess-compiler/tests/row_set_ir.rs` (existing):
  `a_where_selected_state_read_lowers_to_its_own_variant` (acceptance 1, 6).
Each under 5 s (bound 60 s per test, 120 s per step).

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy -p <crate> --all-targets --locked -- -D warnings` and `cargo test -p <crate>
  --locked` for each of `ess-domain` (`--test row_queries_navigate`), `ess-compiler`, `ess-diff`,
  `ess-entity-runtime`, `ess-gen`, `ess-service-contract`, `ess-synth`, `ess-cli` (table:
  `ess-domain`/`ess-compiler` also gate `ess-cli`); `ess-conformance` clippy, and its tests one
  `--test` at a time: `--test a_command_with_two_related_selectors_arranges_both`, then the byte pins
- `task test-xtask` (format version, `website/`, `docs/`, `CHANGELOG.md`); `cargo xtask schema`
  then `task projection-check` if `RawSpecFile` changes
- byte pins: `ess-conformance` `--test synthesis_suite_bytes_table`,
  `--test synthesis_suite_bytes_table_cli_read`, `--test adversary_related_values_all_bytes`,
  `--test external_beside_held_guard`; `ess-synth` `--test served_codec_names`, `--test go`;
  `ess-openapi` `--test adversary_ess_issues_2_u6`; `ess-compiler` `--test
  string_alphabet_and_length`; `ess-cli` `--test tutorial_page`
- the sources stamp, rewritten in this change:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
  re-run after every rebase, never merge the stamp by hand
- the byte comparison, once per unit, its measured time named in the report:
  `SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table -- --ignored every_repository_model_keeps_its_synthesized_bytes`

## Depends on

story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess (opens `ess/38`).

## Changelog

Under `Added` (it is a construct; the issue is not labelled a bug), one bullet for both parts,
written by part 1 and completed by part 2:

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
6. Cost: one construct in the open format; two additive IR variants with an arm in 17 consumer
   files; no new diagnostic code (the format refusal and ESS-COMMAND-001 exist).
7. Considered: change nothing (rejected: the guard decides the value and the value cannot be named);
   allow `via: payload.<Event>.<id>` instead (rejected: a second spelling for one read); a
   `reads: Field | State` member on the existing variants instead of new variants (rejected: every
   consumer that reads `field` would still need the case, and a lookup could meet `state`, which
   `RelatedState` exists to prevent); admit `field: state` in both selection forms (chosen, as
   proposed, plus the head form for symmetry); one unit (rejected: over 600 lines).

## Decisions

- Decision: accept as proposed, widened to the ordered-head form, split in two.
- DF-D5: the format. A (default) the open format `ess/38`; B the next format after it; C no format
  (lift the refusal at every format: rejected by precedent, every `state` lift so far was gated).
  Taken: A (D77: `ess/38`, opened by the unit in work for #640).
