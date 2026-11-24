---
format: aep.planning-md/3
id: story:a-row-set-branch-s-search-arranges-the-rows-of-a-selector-once-per-distinct-prefix-of-counts-not-once-per
kind: story
status: active
title: A row-set branch's search arranges the rows of a selector once per distinct prefix of counts, not once per combination tried.
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:33:06Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T01:33:07Z", actor: "human:timo", revision: 3}
---
# pf1-arrange-reuses-prefix

Size: S (one function split in two, one counter, one test; about 200 changed lines).

## Need

`row_set::prepare_with` tries the combinations of row counts of a command's selectors in a fixed
order, the first selector varying slowest (`COUNTS = [1, 2, 0, 3]`, cut at `ARRANGEMENTS = 4096`).
For every combination, `Plan::arrange` (`crates/verify/ess-conformance/src/synthesize/row_set.rs`,
`fn arrange`, line 2753 on `integration/gaps-2026-11-24`) arranges every selector's rows from
scratch, decoys included, although two consecutive combinations differ only in their later
selectors. Each row is made through the creating commands (`arrange_row` ->
`arrange_toward_with` -> `advance`), which is where every profile sample of the reported run sits.

## Outcome

`Plan::arrange` keeps, per selector, what the selectors up to it left (steps, taken names, rows,
witness gaps, related-row tests still failing) under the key "count of this selector, `forall` it
refutes". An attempt starts from the longest prefix of levels whose keys it shares with the last
attempt and arranges only the selectors after it. A selector whose rows could not be arranged is
kept as that failure. The beside searches a reused level cut are added again
(`related_guard::add_cuts`), as `made.rs` and `run_memo.rs` do. No suite byte, refusal, diagnostic
or verdict changes; only the work.

Starting point: a patch written on tag 0.98.0 and handed to the implementor with the brief (it is not in
the repository). It
applies to the integration head except its last hunk (the head appended `taken_tried`/`add_tried`
after `counting`): put the new counter after those. The patch lacks the thread transfer of its
counter (below).

## Acceptance

1. Red today: on the integration head, the new test `a_command_refusing_on_several_row_sets_arranges_each_prefix_once`
   in `row_set_many_selectors_work.rs` fails, `row_set_selectors_arranged` reporting more than the
   bound. On tag 0.98.0 the counts were (instrumented release build, the test's model):
   K = 3: 873 selectors arranged unchanged, 404 with reuse, 308 arrangements tried either way;
   K = 4: 4,842 unchanged, 1,646 with reuse, 1,236 tried. The implementor recounts both on the head
   before fixing the bound (the counter can be added first, alone, to read the unchanged figure).
2. Green: the same test passes with the bound of decision D1; `row_set_arrangements_tried` is the
   same number before and after (the attempts are unchanged, only their work).
3. `row_set_selectors_arranged(|| synthesize(&ir))` counts selectors arranged rather than reused, on
   every thread: the counter is carried across caller readings like `tried` in
   `synthesize/threads.rs` (`taken_*`/`add_*` pair, a `Work` field, and a `Counter` row in
   `STATE`, which `every_static_of_the_crate_is_classified` and `work_names_every_counter_row`
   require).
4. No byte moves: the byte pins of `## Gate` pass unchanged, and the generic model's suite
   (`ess verify conform synthesize --target ir`) is `cmp`-identical between the unchanged head's
   release binary and the fixed one for K = 2, 3, 4, 5 (model text: the test's, written to a scratch
   directory). On tag 0.98.0 the same comparison, and `cmp` of `specify validate --format json` on
   the reporter's specification, were identical.
5. The new test runs under 10 s at a 1-minute load of 60 or less (time and load in the report).

## Scope

- `Plan` gains `levels: RefCell<Vec<Level>>`; `Level { key: (usize, Option<usize>), left:
  Result<Left, usize>, cuts: usize }`; `Left` holds `steps`, `taken`, `arranged`, `gaps`, `failing`
  (cited: patch).
- The loop body of `Plan::arrange` moves to `Plan::arrange_selector(&self, ir, at, (counts,
  violate), actors, supplied, &mut Left) -> Result<(), usize>`; the closing `failing` pass stays in
  `arrange` (cited: patch).
- Inferred, check first: `Plan::arrange` is called only from the search closure of `prepare_with`,
  with `before = &setup.steps`, `supplied` and `actors` constant for the Plan's life (read on the
  tag: one call site, line 3956 there). If a second caller exists on the head, key on those too.
- Inferred, check first: `crowded` is only ever set to `true` and read after the search, so a
  reused level that set it leaves the same value (tag: `set(true)` at one site).

## Owned files

- `crates/verify/ess-conformance/src/synthesize/row_set.rs`
- `crates/verify/ess-conformance/src/synthesize.rs` (the `#[doc(hidden)] pub fn
  row_set_selectors_arranged`, beside `row_set_arrangements_tried`)
- `crates/verify/ess-conformance/src/synthesize/threads.rs` (counter transfer only)
- `crates/verify/ess-conformance/tests/row_set_many_selectors_work.rs`
- `CHANGELOG.md` (one bullet)

## Not yours

The search closure and its enumeration in `prepare_with` (pf2), `prepare` and any memo of it (pf3),
`made.rs`, `subject_fact.rs`.

## Tests first

Add to `row_set_many_selectors_work.rs` (no new test binary) the generic model, K a parameter:
`Order` (created by `Place`), `Line` (`order_id`, `f1..fK: Boolean`, references `Order` via
`order_id`, created by `AddLine`, refused `when_related: {via: input.order_id, exists: false}`),
`Approval`, and `Approve(approval_id, order_id)` refused by `no-order` (`via`, `exists: false`) and by
`line-f1 .. line-fK`, each `when_related: {entity: Line, where: {all: [order_id == input.order_id,
fi == true]}, exists: true}`, accepting with `approved` (creates `Approval`), plus a read-your-writes
view of `Order`. The test synthesizes it under
`row_set_selectors_arranged` and `row_set_arrangements_tried`, asserts the selector count under the
bound, the tried count equal to the head's (recorded as a constant), and the scenarios it stands
today (`Approve/outcome/line-f1`, `line-f2`, `approved`, … as the head synthesizes them; record them
from the red run). Run it red, keep the `test result:` line and the assertion message.

## Gate

Every cargo line with `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0`; one `--test` at a time;
the first `ess-conformance` test build and Clippy with `SLOT_TIMEOUT=30m`.

```console
cargo test -p ess-conformance --locked --test row_set_many_selectors_work
cargo test -p ess-conformance --locked --test row_sets
cargo test -p ess-conformance --locked --test row_set_several_selectors
cargo test -p ess-conformance --locked --test row_sets_suite
cargo test -p ess-conformance --locked --test synthesis_row_set_plan
cargo test -p ess-conformance --locked --test adversary_row_sets_precedence
cargo test -p ess-conformance --locked --test a_command_with_two_related_selectors_arranges_both
cargo test -p ess-conformance --locked --test a_row_set_branch_arranges_rows_past_the_creators_guard
cargo test -p ess-conformance --locked --test nested_ordered_heads
cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table
cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table_cli_read
cargo test -p ess-conformance --locked --test adversary_related_values_all_bytes
cargo test -p ess-conformance --locked --test external_beside_held_guard
cargo test -p ess-compiler --locked --test string_alphabet_and_length
cargo clippy -p ess-conformance --all-targets --locked -- -D warnings
slot task fmt-check
```

`synthesis_suite_bytes_table` is a known test over 60 s (wave common brief): run it with
`SLOT_TIMEOUT=30m`, report its time, do not count it against the story. The `ess-synth`
(`served_codec_names`, `go`), `ess-openapi` (`adversary_ess_issues_2_u6`) and `ess-cli`
(`tutorial_page`) pins read generators, not synthesis: not run unless the change reaches them.
Plus acceptance 4: one release build of `ess-cli` before the change and one after
(`SLOT_TIMEOUT=30m cargo build -p ess-cli --release --locked`), `target/` under 5 GB.

## Depends on

Nothing. Lands first of the three (pf2 rebases on it; see pf2).

## Changelog

`### Fixed`: "Validation and synthesis of a command with several row-set guards no longer arrange
the rows of its earlier selectors again for every combination of row counts they try."

## Fit review

Finding (a downstream application, 2026-11-24, on 0.98.0): `ess specify validate --format json`
went from about 2 s to about 1,000 s, and synthesis likewise, after row-set guards
(`entity`/`where`/`exists`) were added to refusal branches; their mutation audit and completeness
runs no longer fit a 10-minute job. Measured: validation and synthesis time grow exponentially in
the number K of row-set selectors of one command. Generic model (`Approve` with K row-set
refusals over `Line`), installed 0.98.0, user seconds, load 73-78:

| K | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| validate | 0.03 | 0.10 | 0.54 | 3.1 | 25.0 | 146.6 |

This story (tag 0.98.0, instrumented, the lean model, user s unchanged/with reuse): K = 3
0.59/0.34, K = 4 3.39/1.55, K = 5 27.1/10.8; reporter's specification 710/417 s (wall 1,292/714,
load 85-123), output byte-identical. It cuts the work per attempt, not the 4^K attempts (pf2), nor
the repeated preparation of one branch (pf3).

## Decisions

- D1. The model size and bound of the test. A (default): K = 3, bound 600 selectors arranged
  (tag: 873 unchanged, 404 with reuse; factor 2.2), as a debug test build runs the synthesis
  several times slower than the release figures above. B: K = 4, bound 2,500 (4,842 / 1,646),
  if the K = 4 test stays under 10 s at load 60 or less. Either way the numbers are recounted on
  the head (acceptance 1).
- D2. Where the counter lives. A (default): a new doc-hidden `row_set_selectors_arranged` beside
  the existing work counters. B: reuse `creations_made`; rejected unless it moves by the same
  factor (not measured).
