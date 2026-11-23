---
format: aep.planning-md/3
id: story:synthesis-witnesses-an-instance-and-an-each-in-that-name-a-member-of-a-struct-input-on-the-parent-s-minimal
kind: story
status: active
title: 'Synthesis witnesses an instance: and an each: {in:} that name a member of a struct input, on the parent''s minimal model too, the interpreted target reads both there, and no suite format moves'
tags:
- wave-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:synthesis-witnesses-a-row-addressed-through-a-struct-input-member
- depends_on: story:at-ess-38-a-when-related-via-an-all-element-and-a-missing-payload-source-name-a-member-of-a-struct-input-a
- depends_on: story:at-ess-38-instance-and-each-in-name-a-member-of-a-struct-input-and-the-documents-list-every-addressing
- depends_on: story:synthesis-arranges-the-rows-a-when-related-guard-names-through-a-member-of-a-struct-input-and-the-interpreted
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/await_outcome.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects/naming.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/set_effects/pinned.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/threads.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/struct-input-member-served.yaml.in
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest
- confidence: inferred
  path: crates/verify/ess-conformance/tests/input_path_addressing.rs
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T21:48:32Z", actor: "human:timo", revision: 15}
- {from: "proposed", to: "active", at: "2026-11-23T21:48:34Z", actor: "human:timo", revision: 16}
---
## Outcome

- `ess verify conform synthesize` on s2's model (`struct-input-member-addressed-rows.yaml.in`)
  reports 0 refusals under `shop.parcel.`. The scenarios for `Relabel` (updates), `Hold` (moves,
  through the required struct input `request`), `Drop` (deletes) and `Register`
  (`existing_instance:`) send the arranged parcel inside the struct (`target.parcel_id` an
  `Instance`), and each `unknown_instance:` branch sends a fresh identity there and expects
  `shop.parcel.UnknownParcel`.
- `Label`'s `each: {in: input.batch.lines, as: line, repeated: last_wins}` is witnessed as a
  top-level `each:` is: the same scenario ids as a control model whose list is the top-level
  `lines`, each sending under `batch.lines` what the control sends under `lines`.
- Every scenario of that suite passes against `Interpreted`.
- The parent's minimal model, written here as the fixture
  `struct-input-member-served.yaml.in` (one struct value holding both the `via:` and the
  `instance:` of `Note`), synthesizes with 0 refusals under `shop.stock.` and passes on
  `Interpreted`; s5 serves the same text.
- Awaiting a system-issued command whose `instance:` is a member path is answered as unsupported,
  naming the path (N6); the limit is named in `s-map.txt`.
- Parent acceptance 1 holds whole: model A's list witnesses (s3) and model B's instance witnesses
  in one test. No suite format moves and no corpus suite changes a byte: the suite major follows
  the suite's vocabulary, and the models' specification format `ess/38` (opened by s0) does not
  enter that choice.

## Why

- Parent `story:synthesis-witnesses-a-row-addressed-through-a-struct-input-member` (serves O2): an
  `instance:` or `each: {in:}` admitted through a member by s2 is unchecked until a suite sends the
  row there.
- s3 made the `via:` half of synthesis path-aware; this unit owns the subject's `named` half of
  `supply` and of `fresh_identity` (X4), every place that reads or writes the subject's identity
  or the `each:` list by its key, the served fixture (X2), and holds the suite format.

## Where

Read at commit `d3252eda9` (tree named in s1; re-read there for D48); built on the wave branch
with s0, s1, s2 and s3 merged, in round 3 beside s6. Depends on s1 (tests 7 and 8 read s1's
model), s2 and s3, and on s0 through them: every model here declares `ess/38`, the only format
that admits a member address. Paths under
`crates/verify/ess-conformance/`. After s2 the compiled `ResolvedInstance::Supplied { field }` has
`field.name == "target.parcel_id"` and `ResolvedEach.list == "batch.lines"`. Same rule as s3 D1:
a `BTreeMap<String, Node>` is written with `set_at` (`src/synthesize.rs` 13266) and read with
`ess_compiler::ir::read_input`; a `BTreeMap<String, ScenarioValue>` is read with `supplied_at`
(13129, extended by s3) and written with `supply_at` (s3); a declared type comes from
`input_type(ir, command, field)` (13222) in place of `command.input.iter().find(|field|
field.name == …)`. A one-segment key behaves as today.

`src/synthesize.rs`:

- `supply` (9941-10035), the `named` half (X4): `named` (9949-9952) is the subject's
  `instance:` field. In the fold, `(Some(named), Some(subject)) if named == field` (about
  10019-10021) matches a top-level key only, as today. After the fold (beside s3's `bound` loop),
  when `named` `is_path` and `instance` is `Some(subject)`,
  `supply_at(&mut supplied, named, ScenarioValue::instance(subject.clone()))`.
- `fresh_identity` (18705-18760), the instance-specific calls (X4): `keeps_branch` (18735),
  `guided_identity` (18738) and `singleton::names_the_one_row` (18744) receive the subject's path;
  read each and make every input write or read by `field` in them path-aware by the rule above
  (`set_at` / `read_input` / `input_type`). s3 made the two reads at 18717 and 18722-18727
  path-aware; do not change those.
- `unknown_instance` (17856-17900): `input.insert(field.to_owned(), fresh)` (17887) becomes
  `set_at`. `names_existing` (17426) and `preserves_supplied` (17442) already return
  `field.name`, the path.
- `unknown_instances` (17499): no key access of its own; unchanged unless a refusal names it.

`src/synthesize/subject_fact.rs`:

- `absent` (7999-~8035): `input.get(&field.name)` (8023) becomes `read_input`.
- `unknown_identity_refusal` (8075-~8130): `input.insert(field.name.clone(), identity.clone())`
  (8111) becomes `set_at`.

`src/synthesize/existence.rs`:

- `called_instance` (1560-1580): `input.get(field.name.as_str())` (1574, a sent input) becomes
  `super::supplied_at(input, &field.name)`.

`src/synthesize/set_effects.rs`:

- the `each_lists` call site (3075-3078): `input.insert(each.each.list.clone(), …)` (3077)
  becomes `set_at`;
- `each_lists` (3360-~3460): the list type (3371-3378) through `input_type`; `sent.insert(each.list
  .clone(), list.clone())` (3425) becomes `set_at`;
- `first_call` (3464, insert 3488), `empty_call` (3644, insert 3659): `set_at`;
- `carry` (3559-~3640): `input.insert(field.name.clone(), held.clone())` (3597) becomes `set_at`;
- `refused_writes_none` (4119): `input.get(&each.list)` (4155) becomes `read_input`;
- `selected_row_naming` (1229) and `affects_segment_on` (3002): read them; a key access by
  `field.name` or `each.list` follows the rule above.

`src/synthesize/set_effects/naming.rs`:

- `affects_segment` (167-~440): the list type (230-236) through `input_type`;
  `supplied.insert(each.list.clone(), sent(&each.member, &listed))` (434) becomes `supply_at`;
  `decided.insert(each.list.clone(), …)` (416) is a map keyed by the list name for the report,
  unchanged.
- `skipped_row` (532-~610): `supplied.insert(each.list.clone(), …)` (607) becomes `supply_at`.

`src/synthesize/set_effects/pinned.rs` (433-436, 758-771): keyed by a pinned `Source::Input`
field. An owned file only if a test of this unit sends through it (then `supply_at` /
`read_input` at those lines, named in the report); otherwise unchanged.

`src/interpret/execute/set_effects.rs` (the interpreted target):

- `subject` (441-~465): `supplied.get(&field.name)` (455, a `BTreeMap<String, Node>`) becomes
  `read_input`.
- `ElementFacts::rebind` (378-388): builds `[self.list, index, rest…]` with `"batch.lines"` as one
  segment; split `self.list` on `.` into its segments before the index.
- `elements` (236-~300) reads through `Context::get`, already path-aware: unchanged.
- Every other interpreter read of the subject identity (`execute.rs` `held_rows` 987,
  `addressed_unknown` 1521, `selected_subject_refusal` 1575, `stored_reference` 1749, `take`
  2289; `row_set.rs` `subject` 427; `determined.rs` `selected_subject` 33) reads through
  `Context::get` (`execute/history.rs` 42, `read_input`): unchanged.

`src/interpret/await_outcome.rs` (N6, the `instance:` half): `awaited` (248-267) takes the
identity input from `identity_input` (271), which returns the path `target.entry_id`; `choices`
(315-) places the awaited instance only where `field.name == identity` (325), a top-level
comparison, so the instance is never placed, the struct is minted and the await answers through
`nothing_reached` as if the target were wrong. In `awaited`, after `identity_input(command)`
(264-265), when the identity `is_path`, return `Err(TargetError::unsupported(observation,
format!("outcome addresses its record through `input.{identity}`, a member of a struct input, and
awaiting a system-issued command addressed through a member is not interpreted")))`. About 8
lines, after s3's check in `await_system_outcome`. Placing the instance inside the minted struct
would need `choices` to build struct candidates (over 40 lines): not done (D6).

Not changed, and checked by test 8: `src/scenario.rs` (`ScenarioValue`,
`SUPPORTED_SUITE_FORMATS` 640-644, ending at 65), `src/structured_values.rs` (`ORDINARY` 32,
`COVERAGE` 33). A struct holding an arranged identity is `ScenarioValue::Members`, the suite/32
vocabulary a top-level list of references already uses. The ordinary suite major is chosen by
`ConformanceSuite::select_fresh_format_for` (`src/scenario.rs` 340, with `counted_claims_floor`
308 and `select_model_format_for` 360) and the coverage major by `coverage_version`
(`src/coverage_build.rs` 668), both from the features a suite and its model use; neither reads
the specification format, so opening `ess/38` (s0) moves no `ess-conformance/<N>` and opens no
suite pair (`/64` and `/65` stay as 0.93.0 opened them).

New fixture `tests/fixtures/struct-input-member-served.yaml.in` (X2; text under "The fixture";
`.yaml.in` keeps it out of every corpus walk and of the synthesis digest, s1 D3). s5 reads it.

`tests/input_path_addressing.rs`: s3 created it; this unit appends after s3's last test.

## Tests first

Appended to `tests/input_path_addressing.rs` with `const PARCEL: &str =
include_str!("fixtures/struct-input-member-addressed-rows.yaml.in");` and `const STOCK: &str =
include_str!("fixtures/struct-input-member-served.yaml.in");`, using s3's `ir`, `scenario`,
`sent`, `member` and `passes_on_the_interpreter` helpers. Each synthesizes one model of 90 to 170
lines (tests 5 and 8 two): expected under 3 s each, time each; none may run over 60 s; the file's
step stays within 120 s.

The base of this unit is the wave branch with s0 to s3 merged: the IR carries the instance
path, `supply` places `bound` paths, and every instance site still writes and reads by key. The
red lines below are what each test prints there; each test prints the sent input (or the
refusals) in its failure message.

1. `an_update_through_a_member_names_the_arranged_row_and_a_fresh_one`:
   `Relabel/outcome/relabelled` sends `target` as `Members` with `parcel_id` an `Instance` (the
   parcel `Open` arranged), and `Relabel/outcome/unknown` sends a `Literal` there that no arranged
   row carries and expects `shop.parcel.UnknownParcel`. Red on the base: `relabelled` sends
   `target` as one `Literal` (the `named` half of `supply` matches top-level keys only, 10019),
   and `unknown` sends a top-level key `target.parcel_id` (17887), or a refusal under
   `shop.parcel.Relabel` (print which).
2. `a_move_through_a_struct_member_names_the_arranged_row`: `Hold/outcome/held` and
   `Hold/outcome/not-open` send `request.parcel_id` as an `Instance`; `Hold/outcome/unknown` a
   fresh `Literal`. Red on the base: as test 1, for `request`.
3. `a_delete_through_a_member_names_the_arranged_row`: `Drop/outcome/dropped` and `Drop/outcome/
   unknown` as test 1. Red on the base: as test 1, for `Drop`.
4. `existing_instance_through_a_member_names_the_taken_row`: `Register/outcome/taken` sends
   `target.parcel_id` as an `Instance` and expects `shop.parcel.Taken`; `Register/outcome/
   registered` sends a `Literal` there. Red on the base: `taken` sends `target` as one `Literal`
   (the published identity is arranged by key), or a refusal under `shop.parcel.Register`.
5. `an_each_over_a_list_member_is_witnessed_as_a_top_level_each_is` (parent acceptance 2): build
   the control text from `PARCEL` by two exact replacements: the line
   `      - {name: batch, type: shop.parcel.Batch}` becomes that line followed by
   `      - {name: lines, type: "List<shop.parcel.Line>"}`, and `in: input.batch.lines` becomes
   `in: input.lines`. Assert: the ids of `shop.parcel.Label` scenarios are equal in both suites,
   and for each pair of `Label` `ExecuteCommand` steps the path suite's `member(batch, "lines")`
   equals the control's top-level `lines`. Red on the base: a refusal under `shop.parcel.Label`
   (`each_lists` finds no top-level input `batch.lines`, `set_effects.rs` 3374), printed.
6. `the_addressed_suite_has_no_refusals_and_passes_on_the_interpreter`: no refusal under
   `shop.parcel.`; `passes_on_the_interpreter(PARCEL)`. Red on the base: the refusals of tests
   1-5, printed.
7. `parent_acceptance_one_holds_on_both_models` (parent acceptance 1, whole): on s1's `ROUTE`,
   `Send/outcome/unknown-depot` and `Send/outcome/sent` as s3's tests 1 and 2 assert them, and the
   suite has no refusal; on `PARCEL`, tests 1-4's `Instance` and fresh-`Literal` assertions for
   every `updates:`/`moves:`/`deletes:` branch. One test, so the acceptance is one checkable
   statement. Red on the base: the `PARCEL` half, as test 1 (the `ROUTE` half is green from s3).
8. `no_suite_format_moves` (parent acceptance 4, format half): each suite's
   `provenance.suite_version.major()` equals the major of the same model's control without the
   member path, and `SUPPORTED_SUITE_FORMATS.last() == Some(&65)` (no suite major was added for
   `ess/38`). The controls keep `format: ess/38` (they are the same text with one path moved to a
   top-level input, and the other paths still need `ess/38`), so the comparison isolates the
   member address. The two controls, built in the test by text replacement:
   - `PARCEL`'s control is test 5's (the list moved to the top-level input `lines`);
   - `ROUTE`'s control: in `Send`, the line `      - {name: settings, type:
     shop.route.Settings}` (the first after `  - name: shop.route.Send\n    input:\n`) gains the
     line `      - {name: depots, type: "List<shop.route.DepotId>"}` after it, and every
     `input.settings.depots` becomes `input.depots`.
   Also no sent input of either suite has a key containing `.` (a flattened path never reaches a
   suite). Control, green at once on the base if both suites already synthesize there; red on the
   base where `PARCEL` refuses (test 6's red line); kept because it pins the parent's statement
   after this unit's changes, and fails naming both majors if they differ.
9. `the_minimal_model_synthesizes_without_refusals_and_passes_on_the_interpreter` (X2, for s5):
   on `STOCK`, no refusal under `shop.stock.`; `Note/outcome/noted` sends `settings` as `Members`
   with `primary` an `Instance` and `Note/outcome/backup-closed` sends `Instance`s at
   `settings.primary` and `settings.backup` (one struct value holding both the `instance:` and the
   `via:` row); `Check/outcome/checked` and `Check/outcome/closed` send `settings.primary` as an
   `Instance`; `passes_on_the_interpreter(STOCK)`. Red on the base: `Note`'s scenarios send
   `settings` with `primary` a `Literal`, or a refusal under `shop.stock.Note` (printed; the
   `Check` half is green from s3).
10. `awaiting_a_system_issued_command_addressed_through_a_member_is_unsupported` (N6, instance
    half): s3's test 8 model (at `format: ess/38`) with, in addition, `Assign`'s input `entry_id` replaced by `target:
    queueing.entry.Target` (a struct with one member `entry_id: queueing.entry.EntryId`), its
    `instance: entry_id` by `instance: target.entry_id` and every `input.entry_id` by
    `input.target.entry_id`; the await (as s3's test 8) answers `Err(TargetError)` whose text
    contains `` `input.target.entry_id`, a member of a struct input ``. This unit's check in
    `awaited` (`await_outcome.rs` 83) runs before s3's check after `related_inputs` (84), so it
    answers first; `worker` stays top-level in this test (no `pick`) so s3's check has nothing to
    refuse. Red on the base: an `Ok` other than `Reached` (printed).

Existing targets that must stay green unchanged: `each_repeated`, `related_guard_unknown_instance`,
`input_path_addressing` (s3's tests), `related_list`, `related_guard`, `system_issued_interpreter`,
and the byte pins in Gate. Find the `each:` targets with `grep -l "each:"
crates/verify/ess-conformance/tests/*.rs` and run the ones whose names start with `each` one at a
time.

## Steps

1. Start from the wave branch with s0 to s3 merged; `git rev-parse --short HEAD` in the
   report. Run test 1 before any edit and keep its red line.
2. The served fixture; validate it once with `ess specify validate` on a scratch copy (a
   declaration that does not validate is corrected and named in the report). Tests 1-10; record
   the red lines.
3. `supply`'s `named` half and `fresh_identity`'s instance calls; `unknown_instance`;
   `subject_fact.rs`; `existence.rs`; `set_effects.rs`; `naming.rs`; `pinned.rs` only if a test
   sends through it; then the interpreter's `subject`, `rebind` and the `awaited` check.
4. Run the file and the existing-green list. A refusal still printed under `shop.parcel.` or
   `shop.stock.` names its site; fix it by the rule above, never by widening a condition; report
   each site.
5. No thread state is added. If any `static`/`thread_local!` is added after all, classify it in
   `STATE` of `src/synthesize/threads.rs` (53) and, for a counter, in `work!` (271), `Work::add`
   (369) and `Work::NAMES` (324), after any line s3 added there; `every_static_of_the_crate_is_classified`
   (514) fails otherwise.
6. Re-stamp from the repository root:
   `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
   then `slot --timeout 30m task test-slow PACKAGES=ess-conformance`. No row of the four corpus
   tables moves (s3 Step 6); a moved row is a defect of this unit: stop and report row and model.
   s6 runs in the same round and re-stamps only if it edits `ess-gen/src/openapi.rs`: this unit
   lands first; s6 then re-runs the stamp command on the merged tree.
7. Run once by hand, output in the report: copy each fixture (`PARCEL`, `STOCK`) to a scratch
   `.yaml`, then `ess verify conform synthesize` and `ess verify conform run --target interpreted`
   on it.
8. `CHANGELOG.md`; `slot task fmt-check`.

## Owned files

- `crates/verify/ess-conformance/src/synthesize.rs` (`supply`'s `named` half, `fresh_identity`'s
  instance calls, `unknown_instance`)
- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs`
- `crates/verify/ess-conformance/src/synthesize/existence.rs`
- `crates/verify/ess-conformance/src/synthesize/set_effects.rs`
- `crates/verify/ess-conformance/src/synthesize/set_effects/naming.rs`
- `crates/verify/ess-conformance/src/synthesize/set_effects/pinned.rs` (only if a test sends
  through it)
- `crates/verify/ess-conformance/src/synthesize/threads.rs` (only if Step 5 applies; after s3's
  lines)
- `crates/verify/ess-conformance/src/interpret/execute/set_effects.rs`
- `crates/verify/ess-conformance/src/interpret/await_outcome.rs` (the `awaited` check only)
- `crates/verify/ess-conformance/tests/input_path_addressing.rs` (append after s3)
- `crates/verify/ess-conformance/tests/fixtures/struct-input-member-served.yaml.in` (new)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (re-stamped)
- `CHANGELOG.md` (one bullet)

## Not yours

- In `synthesize.rs`: `supply_at`, `supplied_at`, `supply`'s `bound` half and `related_list::empty`
  substitution, `fresh_identity`'s two reads (18717, 18722-18727), `run_as`, `with_listed`,
  `refused_here`; and `related_list.rs`, `related_guard.rs`, `interpret/execute/related.rs`,
  `await_system_outcome`'s `via:` check: s3. If one of them is wrong for an instance path, report
  the line; do not edit it.
- `src/scenario.rs`, `src/structured_values.rs`, `src/coverage_build.rs`, every `.tsv` table, s1's
  and s2's fixtures, every `ess-domain`, `ess-compiler` and `crates/generate/` file.

Shared files. With s3 (landed in round 2): `src/synthesize.rs` (different lines, listed above),
`src/interpret/await_outcome.rs` (this unit's check goes in `awaited`, s3's in
`await_system_outcome`), `tests/input_path_addressing.rs` (append after s3's last test, change
nothing above), `threads.rs` if either adds state (s3 first, this unit appends). With s6 (same
round 3): `CHANGELOG.md` under different headings (this unit `### Added`, s6 `### Changed`) and,
only if s6 edits `crates/generate/ess-gen/src/openapi.rs`, the stamp: this unit lands first, s6
re-runs the stamp command on the merged tree. No other file is shared with s6 (s6 owns files under
`crates/generate/ess-entity-runtime/`, `crates/generate/ess-gen/tests/` and `website/`). With s5
(round 4): s5 reads the served fixture and changes it never.

## Gate

Per `AGENTS.md` § Gate: synthesis in `ess-conformance` changed, so `ess-cli` too, and
`ess-conformance` runs one `--test` at a time; the repository-wide byte pins run. Every cargo
command carries `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0`; a pool job ends after 10
minutes unless it names its time (`SLOT_TIMEOUT=30m`, at most 30); a test over 60 s is killed;
the step ends within 120 s; `slot cargo xtask`; no Python, nothing in `/tmp`, no
`RUSTC_WRAPPER`, `SCCACHE_*` or `-j`.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-conformance --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --lib -- synthesize::tests synthesize::instant_refusal_tests synthesize::unclaimed_refusal_tests synthesize::threads:: synthesize::set_effects
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test input_path_addressing
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test each_repeated
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test related_guard_unknown_instance
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test related_list
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test related_guard
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test system_issued_interpreter
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table_cli_read
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test adversary_related_values_all_bytes
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test external_beside_held_guard
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test selection_precedence_table
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test string_alphabet_and_length
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowered_definitions_table
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test served_codec_names
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-openapi --locked --test adversary_ess_issues_2_u6
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-cli --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test tutorial_page
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test interpreted_target
slot --timeout 30m task test-slow PACKAGES=ess-conformance
slot --timeout 30m task test-xtask
```

Plus each `each*` target found in Tests first, one per command. The `--lib` line runs the unit
tests of the files this unit touches (the three test modules of `synthesize.rs` at 22865, 23028,
23113, `synthesize::threads`'s `every_static_of_the_crate_is_classified` at 514, and any test
module under `synthesize::set_effects`); the whole lib (234 unit tests across the crate) is left
to the forge's shards because its time on this host is not known. `ess-cli`: `interpreted_target`
pins the interpreted run's output; the other targets are left to the forge's shards. `go` as in
s1. Do not start `task check` or a whole-workspace run.

## Acceptance

1. `cargo test -p ess-conformance --locked --test input_path_addressing` passes tests 1-10 of this
   unit and s3's tests, each timed in the report.
2. Test 7 passes: parent acceptance 1 on both models.
3. Test 5 passes: parent acceptance 2.
4. Tests 6 and 9 pass: 0 refusals under `shop.parcel.` and `shop.stock.`, and every scenario of
   both suites passed on `Interpreted`.
5. The stamp test passes after the re-stamp; `task test-slow PACKAGES=ess-conformance`, the four
   table targets and the byte pins pass with no `.tsv` edited (parent acceptance 4, bytes half).
6. Test 8 passes, and `git diff --stat` names neither `src/scenario.rs` nor
   `src/structured_values.rs` (parent acceptance 4, format half).
7. Step 7's CLI runs exit 0 with 0 refusals and every scenario passed (output in the report).
8. Every target of the Gate passes; `git diff --stat` shows only the owned files.

## Decisions

- D1. The `unknown_instance:` witness keeps the struct value its witness builder made and replaces
  only the member the path names, with the fresh identity: the rest of `target` stays as the
  accepting run sends it, so the branch is reached by the identity alone.
- D2. `Hold`'s `request` is a required struct input (s2's fixture); an `instance:` whose route
  crosses an `Optional` is s2's refusal (s2 D3, test 7) and gets no witness.
- D3. Parent acceptance 2 is checked against a control model built from the fixture by text
  replacement in the test, not against a second fixture file: the two differ by exactly the list's
  position. Test 8 uses the same control and one built the same way for `ROUTE`.
- D4. Parent acceptance 1 is held by one test of this unit over both models (test 7), so it maps
  to exactly one unit; s3's tests stay as the finer statements of its list part.
- D5. No memo, no new counter, no new thread state, no new `ScenarioValue` variant.
- D6. `interpret/await_outcome.rs` answers a system-issued command whose `instance:` is a member
  path as unsupported, by name (N6); placing the instance inside a struct candidate is left as the
  limit named in `s-map.txt`.
- D7. X2: the parent's minimal model is a fixture of this unit, synthesized and interpreted here,
  so s5's served run starts from a suite already proven on `Interpreted`.

## Open for the operator

None.

## Changelog

Under `## [Unreleased]`, `### Added`, after the last bullet of that heading (s2's and s3's bullets
are there; s6, in the same round, writes under `### Changed`):

- Format `ess/38`: synthesis witnesses an `instance:` and an `each: {in: …}` that name a member of a struct input: the arranged row, or a fresh identity for `unknown_instance:`, is sent inside the struct, and the suite format is unchanged.

## The fixture

`crates/verify/ess-conformance/tests/fixtures/struct-input-member-served.yaml.in`, byte for byte
(Step 2 may correct a declaration that does not validate, and says so in the report). The
parent's minimal model: `Check` reads a bin by `via: input.settings.primary`, `Note` updates the
bin addressed by `instance: settings.primary` beside a `via: input.settings.backup` predicate.

```yaml
format: ess/38
system: shop
version: v1
domain: shop.stock
summary: A check reads the bin its settings name first, and a note updates that bin.
types:
  - {name: shop.stock.BinId, kind: newtype, of: Uuid}
  - name: shop.stock.Settings
    kind: struct
    fields:
      - {name: primary, type: shop.stock.BinId}
      - {name: backup, type: shop.stock.BinId}
      - {name: note, type: String}
entities:
  - name: shop.stock.Bin
    identity: {name: bin_id, type: shop.stock.BinId}
    fields:
      - {name: open, type: Boolean}
      - {name: note, type: String}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
errors:
  - name: shop.stock.UnknownBin
    summary: The bin the settings name does not exist.
    fields:
      - {name: bin_id, type: shop.stock.BinId}
  - {name: shop.stock.ClosedBin, summary: The bin the settings name is closed., fields: []}
events:
  - name: shop.stock.BinAdded
    fields: [{name: bin_id, type: shop.stock.BinId}]
  - name: shop.stock.BinChecked
    fields: [{name: bin_id, type: shop.stock.BinId}]
  - name: shop.stock.BinNoted
    fields: [{name: bin_id, type: shop.stock.BinId}]
actors:
  - name: shop.stock.Clerk
    may: [shop.stock.AddBin, shop.stock.Check, shop.stock.Note]
commands:
  - name: shop.stock.AddBin
    input:
      - {name: open, type: Boolean}
      - {name: note, type: String}
    outcomes:
      - name: added
        creates: shop.stock.Bin
        instance: bin_id
        emits: [shop.stock.BinAdded]
        payload: {shop.stock.BinAdded: {bin_id: {generated: true}}}
        sets: {open: input.open, note: input.note}
  - name: shop.stock.Check
    input:
      - {name: settings, type: shop.stock.Settings}
    outcomes:
      - name: no-bin
        when_related: {via: input.settings.primary, exists: false}
        error: shop.stock.UnknownBin
        payload: {shop.stock.UnknownBin: {bin_id: input.settings.primary}}
      - name: closed
        when_related: {via: input.settings.primary, predicate: open == false}
        error: shop.stock.ClosedBin
      - name: checked
        emits: [shop.stock.BinChecked]
        payload: {shop.stock.BinChecked: {bin_id: input.settings.primary}}
  - name: shop.stock.Note
    input:
      - {name: settings, type: shop.stock.Settings}
    outcomes:
      - name: unknown
        unknown_instance: true
        error: shop.stock.UnknownBin
        payload: {shop.stock.UnknownBin: {bin_id: input.settings.primary}}
      - name: backup-closed
        when_related: {via: input.settings.backup, predicate: open == false}
        error: shop.stock.ClosedBin
      - name: noted
        updates: shop.stock.Bin
        instance: settings.primary
        emits: [shop.stock.BinNoted]
        payload: {shop.stock.BinNoted: {bin_id: input.settings.primary}}
        sets: {note: input.settings.note}
views:
  - name: shop.stock.Bins
    source: shop.stock.Bin
    consistency: read_your_writes
    fields:
      - {name: bin_id, type: shop.stock.BinId}
      - {name: open, type: Boolean}
      - {name: note, type: String}
```
