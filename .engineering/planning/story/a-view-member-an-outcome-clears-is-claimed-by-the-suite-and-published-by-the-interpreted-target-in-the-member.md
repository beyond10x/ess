---
format: aep.planning-md/3
id: story:a-view-member-an-outcome-clears-is-claimed-by-the-suite-and-published-by-the-interpreted-target-in-the-member
kind: story
status: draft
title: A view member an outcome clears is claimed by the suite and published by the interpreted target in the member's declared presence, with no presence read as "left out" as the row schema already says.
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#799
relations:
- serves: vision:O2
revision: 1
---
# d799 (suite and interpreter): a cleared view member is claimed and published in its declared presence

One of two units for beyond10x/ess#799; the other is
`d799-generated-servers-write-null-for-an-absent-null-when-absent-member` (generated servers). Split
by cause: this one is `ess-conformance` (synthesis claim, interpreter row), the other `ess-synth`.

Size: M (about 220 changed lines: 30 source, 190 test).
Fixed suite bytes of committed models: some may move, only where a branch writes an absent value
(`{cleared: true}`, or an `Optional` input sent absent) into an entity field a view publishes with
no presence or `omitted_when_absent`: the `null` claim of that member is dropped. No committed model
with presence on a view-projected entity field and a cleared write was found (the 10 committed
models carrying `presence:` are at `ess/15`-`ess/29`, have no view, or carry it on an input or a
struct); models with no presence and a written absent value were not counted. The byte pins decide.
Patch-release candidate: no (suite bytes and interpreter answers change).

## Need

beyond10x/ess#799. An `Optional` entity field that an outcome clears and a view publishes is spelled
differently by ESS's own outputs: the synthesized suite claims `null` under every `presence:`, the
interpreted target leaves the member out under `omitted_when_absent` and so fails ESS's own suite,
and with no `presence:` the suite and the interpreter say `null` while the published row schema
admits only a string or a missing member.

## Reproduction

Reproduced on 0.98.0. Model `repro/799/<variant>/` (the issue's model, `format: ess/37`), entity
field `note: Optional<String>` with no `presence:`, `omitted_when_absent`, `null_when_absent` or
`omitted_or_null`; `AddBin/added` sets `note: {cleared: true}`; view `Bins` publishes `note`.
Script `repro/799/run-variant.sh` runs synthesize, run, Rust and OpenAPI generation per variant.

- Claim, all 4 variants, step 6 of `depot.bins.AddBin/outcome/added`:
  `"note":{"kind":"literal","value":null}`.
- Interpreted target: `omitted_when_absent` → `6 scenarios: 5 passed, 1 failed` (`failed
  depot.bins.AddBin/outcome/added`); the other three 6 passed.
- Row schema `depot.bins.Bins.Row.note`: none and `omitted_when_absent` → `type: string`, not
  required, `additionalProperties: false`; `null_when_absent` → `anyOf [string, null]`, required;
  `omitted_or_null` → `anyOf`, not required.

## Outcome

Under DF-D3 A (default) and DF-D4 A (default):

| presence of the member | suite claim for a written absent value | interpreted target writes |
|---|---|---|
| none | no claim of the member (either spelling passes the suite) | member left out (as the row schema) |
| `omitted_when_absent` | no `null` claim | member left out (as today) |
| `null_when_absent` | `null`, exactly (as today) | `null` (as today) |
| `omitted_or_null` | no claim (as for an unwritten absent value) | `null` (as today, both admitted) |

The interpreted target passes the suite in every variant, and its rows validate against the
published row schema in every variant.

## Cause (read in the source, not run)

- `crates/verify/ess-conformance/src/synthesize.rs`: `settled` turns `{cleared: true}` into
  `Literal{Null}` (line 14471; an absent `Optional` input at 14497-14499); `subject_row` (14295) →
  `shown` (15232) → `unclaimed_absence` (15262) → `ViewExpectation::Contains` (14308).
  `unclaimed_absence` (15267-15271) drops a `null` only for a field the branch does not write; a
  written absent value keeps its claim under every presence.
- `crates/verify/ess-conformance/src/interpret/views.rs` (449-458): a missing `Optional` is left out
  only under `OmittedWhenAbsent`; with no presence it writes `Node::Null`, which the row schema
  (`crates/generate/ess-gen/src/types.rs`, `object_annotated` 839-875, `is_nullable` 766) refuses.
- The view field carries the entity field's presence from `ess/31`
  (`crates/specify/ess-compiler/src/resolve.rs:4517-4538`).

## Acceptance

1. Red today: under `omitted_when_absent` the suite of the reproduction claims no `null` for `note`
   and the interpreted target passes all 6 scenarios.
2. Under no presence the suite claims no `note` for the written absent value, the interpreted target
   leaves `note` out of the row, and the 6 scenarios pass.
3. Under `null_when_absent` the claim stays `null` and the suite bytes of that variant are
   unchanged.
4. In every variant each row the interpreted target answers for `Bins` validates against
   `depot.bins.Bins.Row` of the same model's OpenAPI document (an in-process check in the test).
5. Below `ess/31` (a view field carries no presence there) the rule for "no presence" applies.
6. The byte pins under Gate keep every row a written absent value does not reach; a moved row names
   #799 beside its re-pin.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs`, `unclaimed_absence` (15262-15273): a written
  absent value is unclaimed unless the view field is `NullWhenAbsent`, as an unwritten one is.
- `crates/verify/ess-conformance/src/interpret/views.rs` (449-458): write `Node::Null` for a missing
  `Optional` only under `NullWhenAbsent` and `OmittedOrNull`; leave it out otherwise (DF-D3 A).

## Owned files

- `crates/verify/ess-conformance/src/synthesize.rs` (`unclaimed_absence`)
- `crates/verify/ess-conformance/src/interpret/views.rs` (the field-by-field row builder)
- `crates/verify/ess-conformance/tests/omitted_or_null.rs` (new `#[test]`s; it holds #677's
  `a_view_carrying_its_source_presence_passes_its_own_suite`, line 160)
- `CHANGELOG.md`

## Not yours

- A suite claim that a member is left out (DF-D4 B): a view row shape in a new suite major, the
  natural home of `story:a-view-read-checks-its-rows-shape` (implemented; it promised a view row
  shape that `ExpectView` does not carry on this tree). Not in this unit under DF-D4 A.
- Generated servers: `d799-generated-servers-write-null-for-an-absent-null-when-absent-member`.
- The row schema (`ess-gen`): already as documented, unchanged.
- `story:an-interpreted-struct-member-follows-its-declared-presence` (draft): a struct member inside
  an event payload, `interpret/execute.rs`; neighbouring, no overlap.

## Tests first

In `crates/verify/ess-conformance/tests/omitted_or_null.rs` (existing target):

- `a_cleared_view_member_follows_its_presence_in_suite_and_interpreter`: the reproduction model, one
  text per variant, synthesize and run on the interpreted target, assert acceptance 1-3.
- `interpreted_view_rows_validate_against_their_row_schema`: acceptance 4, the row schema built in
  process with `ess-gen` (already a dependency of the crate's tests if `field_presence.rs` uses it;
  otherwise compare against the presence rule directly).

Four small synthesize-and-run passes per test: under 10 s expected (bound 60 s per test, 120 s per
step). Time them; one over 5 s is named in the report.

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test omitted_or_null`, then `--test field_presence`,
  `--test field_presence_synthesized`, `--test absent_payload_read`,
  `--test adversary_t2_interp_presence_pass1`
- `cargo test -p ess-cli --locked`
- byte pins: `ess-conformance` `--test synthesis_suite_bytes_table`,
  `--test synthesis_suite_bytes_table_cli_read`, `--test adversary_related_values_all_bytes`,
  `--test external_beside_held_guard`; `ess-synth` `--test served_codec_names`, `--test go`;
  `ess-openapi` `--test adversary_ess_issues_2_u6`; `ess-compiler` `--test
  string_alphabet_and_length`; `ess-cli` `--test tutorial_page`; `task test-slow` for the byte
  comparison of `synthesis_suite_bytes_table`.

## Depends on

DF-D3 and DF-D4 (defaults A proceed after 15 minutes). Independent of the server unit.

## Changelog

Under `Fixed`:

- A view member an outcome clears is claimed and published in its declared presence: the suite no
  longer claims `null` for an `omitted_when_absent` or unspecified member, which ESS's interpreted
  target failed, and the interpreted target leaves an absent member with no presence out of the row,
  as its published row schema says (#799).

## Fit review

1. Need: the suite, the interpreter, the row schema and the servers must agree on one spelling of an
   absent view member per presence. No proposed syntax.
2. Class: defect. `website/docs/reference/spec-versions.md:74`: a view field carries its source
   field's presence, "its row schema and suite follow the policy, and the interpreted target leaves
   an absent member out under `omitted_when_absent`"; the suite does not follow it.
3. Expressible: not applicable.
4. Fit: no authored surface; the rule an unwritten absent value already follows
   (`unclaimed_absence`'s doc, beyond10x/ess#285) is applied to a written one.
5. Second adopter: any "clear the optional note/assignee/due date" outcome shown in a view.
6. Cost: suite bytes where a branch writes an absent value into a view member without
   `null_when_absent`; interpreter answers with no presence; no format.
7. Considered: change nothing; claim absence with a new suite major (DF-D4 B); make no presence mean
   `null` (DF-D3 B, moves the schema and both servers); unclaim and follow the schema (chosen).

## Decisions

- Decision: accept, redesigned: the issue's "claims that the member is left out" becomes "claims no
  `null`" under DF-D4 A; a positive absence claim waits for a view row shape.
- DF-D3 (default A): no `presence:` on a view member means the member is left out when absent, as
  `website/docs/reference/formats.md:215` states ("A field with no presence writes and reads only a
  missing key, as its published schema does") and `website/docs/guides/specify/wire-names.md:157-159`
  ("`omitted_when_absent` keeps it optional and not nullable, which is what an `Optional` field
  without a policy already publishes").
- DF-D4 (default A): no new suite format in this unit.
