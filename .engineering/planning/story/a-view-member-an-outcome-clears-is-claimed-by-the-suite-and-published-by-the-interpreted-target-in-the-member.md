---
format: aep.planning-md/3
id: story:a-view-member-an-outcome-clears-is-claimed-by-the-suite-and-published-by-the-interpreted-target-in-the-member
kind: story
status: active
title: A view member an outcome clears is claimed by the suite and published by the interpreted target in the member's declared presence, with no presence read as "left out" as the row schema already says.
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#799
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T02:31:16Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T02:31:18Z", actor: "human:timo", revision: 4}
---
# d799 (suite and interpreter): a cleared view member is claimed and published in its declared presence

One of two units for beyond10x/ess#799; the other is
`d799-generated-servers-write-null-for-an-absent-null-when-absent-member` (generated servers). Split
by cause: this one is `ess-conformance` (synthesis claim, interpreter row), the other `ess-synth`.

Size: M (about 250 changed lines: 30 source, 220 test).
Patch-release candidate: no (suite bytes and interpreter answers change; breaking for stored suites,
see Changelog).

## Committed bytes that move, and why the old bytes were wrong

Exactly three committed models write an absent value (`{cleared: true}`) into an entity field that a
view publishes, all with no `presence:` (`git grep -l "cleared: true"` over every fixture, example
and model: these three, the rest are `docs/**/*.md`). Synthesized on 0.98.0 (copies under
`repro/799-pins/`), each suite claims the member as `null`:

| model | member | scenarios holding the `null` claim | pinned rows that move |
|---|---|---|---|
| `crates/verify/ess-conformance/tests/fixtures/updating-commands-keep-scenarios.yaml` (`ess/30`) | `demo.agents.Agents.embed_token` | `demo.agents.RevokeEmbedToken/outcome/revoked` | `synthesis-suite-bytes-table.tsv` line 359 |
| `crates/verify/ess-conformance/tests/fixtures/defined-over-optional-aggregates.yaml` (`ess/16`) | `demo.queue.QueueById.metrics` | `demo.queue.ResumeQueue/outcome/resumed`, `demo.queue.Queue/transition/resume/by/demo.queue.ResumeQueue/resumed` | `synthesis-suite-bytes-table.tsv` line 194 |
| `crates/generate/ess-synth/tests/fixtures/declared-behaviour/` (`ess/19`; `ticket.yaml:188`, `:250`) | `desk.ticket.Tickets.note`, `.maybe_stats` | `desk.ticket.ReopenTicket/outcome/reopened`, `desk.ticket.Ticket/transition/reopen/by/desk.ticket.ReopenTicket/reopened`, `desk.ticket.ForgetStats/outcome/forgotten` | `synthesis-suite-bytes-table.tsv` line 411 (`system-dir:`), `synthesis-component-suites-table.tsv` line 62 (`desk-service`) |

Why the old bytes were wrong: each claim requires the member present with `null`, a spelling the
same model's published row schema refuses (a member with no presence is `type: <T>`, not
nullable, `additionalProperties: false`; `ess-gen/src/types.rs`, `object_annotated` 839-875). An
implementation that follows its own schema (the generated Rust and Go servers leave the member out)
fails these scenarios. Each re-pinned row names #799 beside it. The `ess-synth` harness that runs the
`declared-behaviour` suite against generated code (`tests/declared_behaviour.rs`) is run in the gate,
since its suite changes.

## Need

beyond10x/ess#799. An `Optional` entity field that an outcome clears and a view publishes is spelled
differently by ESS's own outputs: the synthesized suite claims `null` under every `presence:`, the
interpreted target leaves the member out under `omitted_when_absent` and so fails ESS's own suite,
and with no `presence:` the suite and the interpreter say `null` while the published row schema
admits only a value or a missing member.

## Reproduction

Reproduced on 0.98.0. The issue's model (`format: ess/37`) in one directory per variant, run with
`--path repro/799-none`, `repro/799-omitted_when_absent`, `repro/799-null_when_absent`,
`repro/799-omitted_or_null` (entity field `note: Optional<String>` with that `presence:`;
`AddBin/added` sets `note: {cleared: true}`; view `Bins` publishes `note`). Script
`repro/799/run-variant.sh <dir>` runs synthesize, run, Rust and OpenAPI generation for one variant.

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
  refuses.
- The view field carries the entity field's presence from `ess/31`
  (`crates/specify/ess-compiler/src/resolve.rs:4517-4538`).

## Acceptance

1. Red today: under `omitted_when_absent` the suite of the reproduction claims `null` for `note`
   and the interpreted target fails 1 of 6 (`depot.bins.AddBin/outcome/added`). Green: no `note`
   claim in that step, 6 passed.
2. Red today: under no presence the claim is `null` and the interpreted target writes `null`.
   Green: no `note` claim for the written absent value, the interpreted target leaves `note` out of
   the row, 6 passed.
3. Under `null_when_absent` the claim stays `null` and the suite bytes of that variant are
   unchanged.
4. In every variant each row the interpreted target answers for `Bins` validates against
   `depot.bins.Bins.Row` of the same model's OpenAPI document (an in-process check in the test).
5. Below `ess/31` (a view field carries no presence there) the rule for "no presence" applies.
6. A suite written by 0.98.0 for the no-presence variant (claim `"note": null`), run against the
   interpreted target after the change, fails `depot.bins.AddBin/outcome/added`: the break the
   changelog announces is pinned.
7. The rows that move are exactly those whose model has a view member the rule covers: a view member
   an outcome clears, or sets from an `Optional` input sent absent. Six models are known: the three
   of the table above, which move in the listed scenarios, and
   `crates/specify/ess-compiler/tests/fixtures/input-value-paths.yaml`,
   `crates/verify/ess-conformance/tests/fixtures/absent-reference-identity-view.yaml` and
   `crates/verify/ess-conformance/tests/fixtures/related-values-chained.yaml`, each a model whose
   view publishes an `Optional` member of that kind and each of which loses that member's `null`
   claim in `synthesis-suite-bytes-table.tsv`. In every moved row only that claim disappears; any
   other row of both byte tables keeps its bytes.

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
- `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` (lines 194, 359,
  411) and `synthesis-component-suites-table.tsv` (line 62): re-pinned, #799 named beside each
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (rewritten by the stamp
  command under Gate)
- `CHANGELOG.md`

## Not yours

- A suite claim that a member is left out (DF-D4 B): a view row shape in a new suite major, the
  natural home of `story:a-view-read-checks-its-rows-shape` (implemented; it promised a view row
  shape that `ExpectView` does not carry on this tree). Not in this unit under DF-D4 A.
- The `null` claim under `null_when_absent`: kept, as the active
  `story:conformance-holds-missing-keeps-and-null-clears` expects (same claim code in
  `synthesize.rs`; that story writes suite 64/65). Whichever lands second rebases on the other's
  `unclaimed_absence`/claim change and re-pins on the merged tree.
- Generated servers: `d799-generated-servers-write-null-for-an-absent-null-when-absent-member`.
- The view read's instant in `interpret/views.rs`: `d819-a-view-read-step-holds-the-read-clock`
  (a different block); this unit lands first, that one rebases.
- The row schema (`ess-gen`): already as documented, unchanged.
- `story:an-interpreted-struct-member-follows-its-declared-presence` (draft): a struct member inside
  an event payload, `interpret/execute.rs`; neighbouring, no overlap.

## Tests first

In `crates/verify/ess-conformance/tests/omitted_or_null.rs` (existing target):

- `a_cleared_view_member_follows_its_presence_in_suite_and_interpreter`: the reproduction model, one
  text per variant, synthesize and run on the interpreted target, assert acceptance 1-3.
- `interpreted_view_rows_validate_against_their_row_schema`: acceptance 4, the row schema built in
  process with `ess-gen` (if the crate's tests already depend on it, as `field_presence.rs` may;
  otherwise compare against the presence rule directly).
- `an_old_null_claim_for_a_member_with_no_presence_now_fails`: acceptance 6, the 0.98.0 claim
  written into the suite by the test, run on the interpreted target, asserting the failure.

Four small synthesize-and-run passes in the first test, one in each other: under 10 s expected
(bound 60 s per test, 120 s per step). Time them; one over 5 s is named in the report.

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test omitted_or_null`, then `--test field_presence`,
  `--test field_presence_synthesized`, `--test absent_payload_read`,
  `--test adversary_t2_interp_presence_pass1`
- `cargo test -p ess-synth --locked --test declared_behaviour` (its fixture's suite moves)
- `cargo test -p ess-cli --locked`
- byte pins: `ess-conformance` `--test synthesis_suite_bytes_table`,
  `--test synthesis_suite_bytes_table_cli_read`, `--test adversary_related_values_all_bytes`,
  `--test external_beside_held_guard`; `ess-synth` `--test served_codec_names`, `--test go`;
  `ess-openapi` `--test adversary_ess_issues_2_u6`; `ess-compiler` `--test
  string_alphabet_and_length`; `ess-cli` `--test tutorial_page`
- the sources stamp, rewritten in this change:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
  re-run after every rebase, never merge the stamp by hand
- the byte comparison, once per unit, its measured time named in the report (it re-pins the rows that
  move: the four above and the three of acceptance 7):
  `SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table -- --ignored every_repository_model_keeps_its_synthesized_bytes`
- `task test-xtask` (the unit edits `CHANGELOG.md`)

## Depends on

DF-D3 and DF-D4 (defaults A proceed after 15 minutes). Independent of the server unit. Ordered
against `story:conformance-holds-missing-keeps-and-null-clears` as in "Not yours".

## Changelog

Under `Changed`:

- **Breaking.** A view member an outcome clears, or sets from an `Optional` input sent absent, is no
  longer claimed as `null` by a synthesized suite unless it is declared `presence: null_when_absent`,
  and the interpreted target leaves such a member with no presence out of the row, as its published
  row schema says. A suite stored from an earlier release that claims `null` for such a member now
  fails on the interpreted target: synthesize it again. Under `omitted_when_absent` the interpreted
  target now passes ESS's own suite (#799).

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
6. Cost: four pinned rows of three committed models; stored suites of adopters with such members
   (breaking, announced); interpreter answers with no presence; no format.
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
- decided 2026-11-24 (D92): the three further rows (`input-value-paths.yaml`, `absent-reference-identity-view.yaml`, `related-values-chained.yaml`) are claimed if the implementor shows that the rule covers each of them; a model the rule does not cover keeps its row, and the acceptance then lists fewer than six.
