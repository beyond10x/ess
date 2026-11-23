---
format: aep.planning-md/3
id: story:synthesis-arranges-the-rows-a-when-related-guard-names-through-a-member-of-a-struct-input-and-the-interpreted
kind: story
status: active
title: Synthesis arranges the rows a when_related guard names through a member of a struct input, and the interpreted target reads the member
tags:
- wave-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:synthesis-witnesses-a-row-addressed-through-a-struct-input-member
- depends_on: story:at-ess-38-a-when-related-via-an-all-element-and-a-missing-payload-source-name-a-member-of-a-struct-input-a
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/await_outcome.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/related.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_list.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/threads.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest
- confidence: inferred
  path: crates/verify/ess-conformance/tests/input_path_addressing.rs
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T21:48:27Z", actor: "human:timo", revision: 11}
- {from: "proposed", to: "active", at: "2026-11-23T21:48:29Z", actor: "human:timo", revision: 12}
---
## Outcome

- `ess verify conform synthesize` on s1's model (`struct-input-member-related-rows.yaml.in`)
  writes the related-row witnesses with the arranged identity INSIDE the struct input:
  `shop.route.Send/outcome/unknown-depot` sends `settings` with `depots` holding one arranged
  depot and then an identity no row carries, expects `shop.route.UnknownDepot`, and its
  `{missing: input.settings.depots}` payload `depot_id` equals that identity;
  `Send/outcome/sent` sends only arranged depots; `Check`'s `no-depot`, `closed` and `checked`,
  `Pair`'s branches (the `all:` row pair) and `Probe`'s branches (the route through the
  `Optional` member `extra`) are witnessed. 0 refusals under `shop.route.`.
- The interpreted target reads the reference at the path (`settings.depots`, `settings.primary`,
  `settings.extra.spare`), so every scenario of that suite passes against `Interpreted`, and a
  target that reads a top-level `depots` in place of `settings.depots` fails
  `Send/outcome/unknown-depot`.
- Awaiting a system-issued command whose `when_related:` guard names its row through a member is
  answered as unsupported, naming the path, where today the interpreter would mint the struct and
  report that nothing reached the awaited outcome (N6). Remaining limit, named: the interpreter
  does not await such a command (follow-up line in `s-map.txt`).
- Every model without a member address keeps its suite bytes: no corpus table row moves.

## Why

- Parent `story:synthesis-witnesses-a-row-addressed-through-a-struct-input-member` (serves O2):
  once s1 admits the address, a target that reads the wrong member must fail, or the construct is
  admitted unchecked.
- This unit carries only what a `when_related:` guard needs: the related-row positions (`via:`,
  `all:`, the `List` member and `{missing:}`), the `bound` half of `supply` and the sent-value
  helpers; s4 carries `instance:` and `each: {in:}`, including the subject's `named` half of
  `supply` (X4).

## Where

Read at commit `d3252eda9` (tree named in s1; re-read there for D48). Built on the wave branch
with s0 and s1 merged (s0 opened `ess/38`, the only format at which a model names a member path;
every model synthesized here declares it), in round 2 beside s2 (X1: the only thing this unit took from s2 was
`input_path::reads_member`, which s1 now adds; s2's changes touch no model this unit
synthesizes). Paths under `crates/verify/ess-conformance/`. Every site works on the compiled IR,
where s1 put the path in `ResolvedRelatedVia::Input { field }` (`"settings.depots"`) and in
`ResolvedPayloadValue::MissingElement { field }`. Synthesis keeps keying `setup.bound`
(`BTreeMap<String, InstanceName>`) by `via.field()`, so a path is a key there; what changes is
every place a value is put into or read from an input map by that key.

The rule for every site below, one helper per map kind (D1):

- an input under construction, `BTreeMap<String, Node>`: write with `set_at` (`src/synthesize.rs`
  13266, path-aware, `false` where the struct is not there), read with
  `ess_compiler::ir::read_input` (`crates/specify/ess-compiler/src/ir.rs` 1468, path-aware);
  for a top-level field both behave exactly as `insert`/`get`, so one-segment bytes hold;
- a sent input, `BTreeMap<String, ScenarioValue>`: read with `supplied_at` (13129, extended
  below), write with the new `supply_at`.

`src/synthesize.rs`:

- `supplied_at` (13129-13149): also walk a `ScenarioValue::Members` root and `Members` members
  (today only a `Literal` root, 13137); a member that is `Instance` or `List` is returned as is.
- New `pub(crate) fn supply_at(supplied: &mut BTreeMap<String, ScenarioValue>, field: &str,
  value: ScenarioValue) -> bool` beside it: a top-level field is `insert`; for a path, the root
  and every struct on the way that is a `Literal { value: Node::Map(..) }` becomes `Members`
  (each of its members `Literal`), then the last segment is set to `value`; `false` (map
  unchanged) where a segment on the way is absent or no map. `Members` is the existing suite form
  for a struct holding an `Instance` (`src/scenario.rs` 1993-2001, suite/32 from
  `src/structured_values.rs` 29), so no suite format moves.
- `supply` (9941-10035), the `bound` half only: in the per-field fold (10014-10027) a top-level
  key takes today's code; after the map is built, for every `(field, owner)` of `bound` with
  `input_path::is_path(field)`, `supply_at(&mut supplied, field, ScenarioValue::instance(owner))`;
  and the `related_list::empty(command)` substitution (10028-10034) uses `supply_at` for a path
  `listed` in place of the key compare. The `named` match (9949-9952, 10019-10021: the subject's
  `instance:` field) is s4's (X4): do not change those lines.
- `run_as` (5685): `supplied.insert(list.field, list.value)` (about 5948) becomes `supply_at`.
- `with_listed` (9180-9205): `input.insert(field, ScenarioValue::List { items })` (9193) becomes
  `supply_at`.
- `refused_here` (17142): only its list line, `supplied.insert(field, ScenarioValue::List { items
  })` (17228, after `related_list::driven`), becomes `supply_at`.
- `fresh_identity` (18705-18760), its two reads only, because the `via:` sites call it with the
  via path (`overlaps`, `missing_row_send`): `input.get(field)` (18717) becomes `read_input`, and
  the type of its `WitnessGap` (18722-18727) comes from `input_type(ir, command, field)` (13222).
  What it does for a subject's `instance:` path (`keeps_branch` 18735, `guided_identity` 18738,
  `singleton::names_the_one_row` 18744) is s4's (X4).
- `determined_fields` (about 11613) and `expression_value_at` (about 13018) call
  `related_list::first_missing`: unchanged here, `first_missing` itself is fixed below.

`src/synthesize/related_list.rs` (top-level list reads, all keyed by the via field):

- `fresh_element` (90-~130): `input.get(field)` (95) becomes `read_input`; the declared type
  (103-107) comes from `super::input_type(ir, command, field)`.
- `arrange` (144-~230), `refuted_by_empty` (244), `driven` (286), `backed` (308), `empty_after`
  (423): each `input.get(field)` / `input.insert(field, …)` becomes `read_input` / `set_at`.
- `guards_on_list` (232-237): compares a guard's root with `field`; for a path compare the read's
  segments with the path (`ess_domain::command::input_path::reads_member`, added by s1).
- `first_missing` (457-467): `supplied.get(field)` becomes `super::supplied_at(supplied, field)`.

`src/synthesize/related_guard.rs` (the identity-addressed related row):

- `absent_input` (4146): `input.remove(via.field())` (4168) becomes `set_at(input, field, None)`;
  for a route through an `Optional` (D3) the segment removed is the first `Optional` on the
  route: walk the prefixes of the path with `input_type` and remove the first prefix whose type
  `is_optional()`.
- `overlaps` (4601): `sent.insert(via.field(), fresh_identity(..)?)` (4647-4650) becomes `set_at`.
- `missing_row_send` (5280): `missing.insert(field, fresh)` (5300) becomes `set_at`.
- `unclaimed` (5392, insert at 5418), `without_row_each` (5487, insert at 5541), `on_row` (5927,
  insert at 5934), `point_at_missing` (6097, insert at 6104): each `insert` becomes `set_at`.
- `guards_read` (5913-5925): `path.namespace() == field` becomes `reads_member` over the guard
  path's segments.
- `reads_row` (279-286): already reads `read_input` for the input; `bound.contains_key(via.field())`
  holds for a path key; unchanged.
- `leaving_out` (1103-1112): drops related outcomes by `via.field()` and input fields by name; a
  path is no input name, so the root struct is kept, which is right; unchanged.
- Do not edit `drive_several`, `as_own_scenario`, `arranged_at`, `Around`, `own_*` (another
  wave's unit owns them); if one of them writes the input by `via.field()`, report the line
  instead of editing it.

`src/interpret/execute/related.rs` (the interpreted target):

- `missing` (60-76): `input.get(field)` (73, 75) becomes `ess_compiler::ir::read_input(input,
  field)`; `absent_optional` then receives `None` for an absent `Optional` on the route.
- `held_row` (203-~240): `input.get(field)` (218 and 221) becomes `read_input`.
- `src/interpret/execute.rs` `related_absent` (1626-1637) and `src/interpret/execute/determined.rs`
  `missing_element` (54-68) already read through `Context::get` (path-aware): unchanged.

`src/interpret/await_outcome.rs` (N6, the `via:` half): today `related_inputs` (288-308) keys the
candidates by the via field, and `choices` (315-~345) walks the top-level inputs only, so a via
`settings.primary` matches no input: `settings` is minted (`base.mint`, 339), the guard reads
a row no candidate carries, and the await answers through `nothing_reached` (201) as if the
target were wrong. In `await_system_outcome` (76-), right after `related_inputs(command)` (84),
when a key `is_path`, return `Err(TargetError::unsupported(observation, format!("a
`when_related:` guard names its row through `input.{path}`, a member of a struct input, and
awaiting a system-issued command addressed through a member is not interpreted")))`, the form
`awaited` (264-265) already uses for an unaddressed command. About 10 lines. Reading the member
instead would need `choices` to enumerate candidates per member of a struct input (over 40 lines):
not done (D4).

New test file `tests/input_path_addressing.rs` (X7: `ess-conformance` has no `tests/main.rs` and
780 test binaries; this is the target the parent's scope names; no existing file is about
addressing through a member).

## Tests first

Helpers copied in shape from `tests/related_list.rs`: `ir` (22), `scenario` (39),
`passes_on_the_interpreter` (80), and `const ROUTE: &str =
include_str!("fixtures/struct-input-member-related-rows.yaml.in");`. A `sent(scenario, command)`
helper returns the `input` of each `ScenarioStep::ExecuteCommand` of that command with the
`ExpectOutcome`/`ExpectError` step after it (as `sends`, 61). A `member(value, path)` helper walks
`ScenarioValue::Members` by segment. Each test synthesizes one 130-line model: time each; expected
under 3 s each; none may run over 60 s; the file's step stays within 120 s.

The base of this unit is the wave branch with s0 and s1 merged: s1's IR carries the path, and synthesis
still writes and reads inputs by key. The red lines below are what each test prints there; each
test prints the sent input (or the refusals) in its failure message.

1. `a_list_member_naming_a_missing_row_takes_the_refusal_and_names_it`:
   `Send/outcome/unknown-depot`'s `Send` input `settings` is `Members`, its `depots` is
   `List { items: [Instance, Literal(missing)] }`, and the `ExpectError` `fields` equal
   `{depot_id: missing}` (as `related_list.rs::a_missing_list_element_takes_the_refusal`, 96-123,
   asserts for a top-level list). Red on the base: a refusal under `shop.route.Send` naming the
   list, or a sent input with a top-level key `settings.depots` beside `settings` (print which).
2. `every_element_present_takes_the_accepting_branch`: `Send/outcome/sent` sends `depots` holding
   `Instance` elements only. Red on the base: `sent`'s `settings` is one `Literal` (no
   `Instance` inside), or a key `settings.depots` beside it.
3. `a_scalar_member_names_the_arranged_row`: `Check/outcome/closed` and `Check/outcome/checked`
   send `settings.primary` as an `Instance`; `Check/outcome/no-depot` sends a `Literal` there;
   `Pair/outcome/both-closed` sends `Instance`s at `settings.primary` and `settings.backup`. Red
   on the base: the sent input holds a top-level key `settings.primary` (the `on_row` insert at
   `related_guard.rs` 5934 keys by the via field) and `settings.primary` inside `settings` is a
   `Literal`.
4. `a_route_through_an_optional_member_is_left_out_for_its_absence`: among `Probe`'s scenarios one
   sends `settings` with no `extra` member and expects `probed`; `no-spare` sends a `Literal` at
   `settings.extra.spare`. Red on the base: no `Probe` scenario sends `settings` without `extra`
   (`absent_input` removes the key `settings.extra.spare`, which is absent, 4168).
5. `the_suite_has_no_refusals_and_passes_on_the_interpreter` (parent acceptance 3, first
   sentence): `synthesis.refusals` holds nothing under `shop.route.`; `passes_on_the_interpreter`.
   Red on the base: the refusals under `shop.route.` (printed), or a scenario not `Passed`.
6. `a_target_reading_a_top_level_list_fails_the_missing_row` (parent acceptance 3, second
   sentence): a wrapper `TopLevelDepots(Interpreted)` implementing `ConformanceTarget` exactly as
   `AbsentAsMissing` in `tests/related_guard_multiple_vias.rs` (479-520), whose `execute_command`,
   for `shop.route.Send`, replaces `input.settings.depots` by the top-level `input.depots` (absent,
   so the empty list `Node::Seq(vec![])`) before delegating: the run of
   `Send/outcome/unknown-depot` is not `Passed`. Red on the base: test 1's red line (no scenario
   that sends the list inside `settings`).
7. `every_branch_of_the_model_has_its_scenario` (replaces the count test): the set of scenario ids
   of the form `shop.route.<Command>/outcome/<branch>` in the suite contains every branch the
   model declares, derived in the test from the compiled IR (`ir.commands()` of `shop.route.`,
   each `outcome.name`): `AddDepot/outcome/added`, `Send/outcome/unknown-depot`,
   `Send/outcome/sent`, `Check/outcome/no-depot`, `Check/outcome/closed`, `Check/outcome/checked`,
   `Pair/outcome/no-primary`, `Pair/outcome/no-backup`, `Pair/outcome/both-closed`,
   `Pair/outcome/paired`, `Probe/outcome/no-spare`, `Probe/outcome/spare-closed`,
   `Probe/outcome/probed` (13). The test asserts the set, not a count, and no time. Red on the
   base: the ids missing from the suite (printed). (The crate's work counters, `Work::NAMES` in
   `src/synthesize/threads.rs` 324, count synthesis steps, not branches; no counter is asserted.)
8. `awaiting_a_system_issued_command_guarded_through_a_member_is_unsupported` (N6, via half): copy
   the model text of `tests/system_issued_interpreter.rs::model` (48-143) into a `fn` of this
   file at `format: ess/38` (the copied text declares an older format: replace its `format:` line;
   below `ess/38` the member `via:` is refused before synthesis), with `Assign`'s input `worker` replaced by `pick:
   queueing.entry.Pick` (a new struct type with one member `worker: queueing.entry.WorkerId`) and
   every `input.worker` by `input.pick.worker`; drive it as that file's
   `the_first_free_held_row_is_the_input_the_system_decides` (468-) does (`begin`, `created` for
   a free worker and an entry, then `target.await_outcome(ask(..))`); the answer is
   `Err(TargetError)` whose text contains `` `input.pick.worker`, a member of a struct input ``.
   Red on the base: an `Ok(AwaitAnswer::…)` other than `Reached`, or a model refusal before s1's
   admission (print it).

Existing targets that must stay green unchanged: `related_list`, `related_guard`,
`related_guard_multiple_vias`, `interpreted_related_guards`, `adversary_761_list_identity_guard`,
`adversary_related_via_optional_pass1`, `related_guard_state`, `system_issued_interpreter`.

## Steps

1. Start from the wave branch with s0 and s1 merged; `git rev-parse --short HEAD` in the report. Run
   test 1 on that head before any edit and keep its red line.
2. Tests 1-8; record the red lines.
3. `supply_at` and the `supplied_at` extension; `supply`'s `bound` half; then `run_as`,
   `with_listed`, `refused_here`'s list line, `fresh_identity`'s two reads; then
   `related_list.rs`; then `related_guard.rs`; then the interpreter (`related.rs`,
   `await_outcome.rs`).
4. Run the file and the existing-green list. A refusal still printed under `shop.route.` names its
   site in `related_guard.rs` or `related_list.rs`: fix it by the rule above (a key write by
   `via.field()` moves to `set_at`), never by widening a condition; report each such site.
5. No thread state is added. If any `static`/`thread_local!` is added after all, classify it in
   `STATE` of `src/synthesize/threads.rs` (53) and, for a counter, in `work!` (271), `Work::add`
   (369) and `Work::NAMES` (324); `every_static_of_the_crate_is_classified` (514) fails otherwise.
   `threads.rs` is then an owned file of this unit (conditional), and s4 appends after s3's lines.
6. Re-stamp the synthesis sources from the repository root:
   `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
   then `slot --timeout 30m task test-slow PACKAGES=ess-conformance` (the slow-lane byte
   comparison of every corpus model). s2 lands in the same round and re-stamps too: the second of
   s2 and s3 to land re-runs this command on the merged tree. No row of
   `synthesis-suite-bytes-table.tsv`, `synthesis-component-suites-table.tsv`,
   `selection-precedence-table.tsv` or `lowered-definitions-table.tsv` may move: the feature model
   is a `.yaml.in` outside the corpus (s1 D3) and every one-segment site behaves as before. A moved
   row is a defect of this unit: stop and report the row and the model.
7. Run once by hand, output in the report: copy the fixture to a scratch `route.yaml`, then
   `ess verify conform synthesize` and `ess verify conform run --target interpreted` on it (the
   CLI spelling of test 5).
8. `CHANGELOG.md`; `slot task fmt-check`.

## Owned files

- `crates/verify/ess-conformance/src/synthesize.rs` (`supplied_at`, `supply_at` new, `supply`'s
  `bound` half and `related_list::empty` substitution, `run_as`'s list insert, `with_listed`,
  `refused_here`'s list insert, `fresh_identity`'s two reads at 18717 and 18722-18727)
- `crates/verify/ess-conformance/src/synthesize/related_list.rs`
- `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (the eight functions named
  above only)
- `crates/verify/ess-conformance/src/interpret/execute/related.rs` (`missing`, `held_row`)
- `crates/verify/ess-conformance/src/interpret/await_outcome.rs` (the `via:` check in
  `await_system_outcome` only)
- `crates/verify/ess-conformance/src/synthesize/threads.rs` (only if Step 5 applies)
- `crates/verify/ess-conformance/tests/input_path_addressing.rs` (new)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (re-stamped)
- `CHANGELOG.md` (one bullet)

## Not yours

- In `synthesize.rs`: `supply`'s `named` lines (9949-9952, 10019-10021), what `fresh_identity`
  does for a subject's `instance:` path (`keeps_branch`, `guided_identity`,
  `singleton::names_the_one_row` calls, 18734-18745), `unknown_instance` (17856): s4 (X4).
  `subject_fact.rs`, `existence.rs`, `set_effects.rs` and `set_effects/`,
  `interpret/execute/set_effects.rs`, `awaited`'s instance check in `await_outcome.rs`: s4.
- `drive_several`, `as_own_scenario`, `arranged_at`, `Around`, the `own_*` counters in
  `related_guard.rs`.
- `src/synthesize/related.rs` (the `{related: {via, field}}` value source, not opened: s1 D6).
- Every `ess-domain`, `ess-compiler` and `crates/generate/` file; the fixtures `.yaml.in` (s1, s2,
  s4).
- Every corpus table `.tsv` (no row moves).

Shared files. With s2 (same round 2): only `synthesis-sources.digest` (the second to land re-runs
the stamp command on the merged tree, never merges it by hand) and `CHANGELOG.md` (the second to
land appends after the first's bullet under `### Added`). Compared by Owned files and every path in
Steps: s2 owns no file under `crates/verify/ess-conformance/` except its `.yaml.in` fixture and the
stamp; no other file is shared. With s4 (round 3, lands after this unit): `src/synthesize.rs`
(different functions and lines, listed under Not yours), `src/interpret/await_outcome.rs` (s4 adds
its check in `awaited`, after this unit's), `tests/input_path_addressing.rs` (s4 appends after
this unit's last test), `threads.rs` if either adds state (s3 first, s4 appends), the stamp and
`CHANGELOG.md`.

## Gate

Per `AGENTS.md` § Gate: synthesis in `ess-conformance` changed, so `ess-cli` too, and
`ess-conformance` runs one `--test` at a time; the repository-wide byte pins run. Every cargo
command carries `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0`; a pool job ends after 10
minutes unless it names its time (`SLOT_TIMEOUT=30m` for the first `ess-conformance` build of a
fresh worktree, for `ess-cli` and `ess-synth`; `slot --timeout 30m` otherwise; 30 at most); a test
over 60 s is killed; the step ends within 120 s; `slot cargo xtask`; no Python, nothing in
`/tmp`, no `RUSTC_WRAPPER`, `SCCACHE_*` or `-j`.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-conformance --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --lib -- synthesize::tests synthesize::instant_refusal_tests synthesize::unclaimed_refusal_tests synthesize::threads::
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test input_path_addressing
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test related_list
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test related_guard
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test related_guard_multiple_vias
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test interpreted_related_guards
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test adversary_761_list_identity_guard
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test adversary_related_via_optional_pass1
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test related_guard_state
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

The `--lib` line runs the unit tests of the files this unit touches: the three test modules of
`synthesize.rs` (22865 `tests`, 23028 `instant_refusal_tests`, 23113 `unclaimed_refusal_tests`)
and `synthesize::threads` (`every_static_of_the_crate_is_classified`, 514); the whole lib (234
unit tests across the crate) is left to the forge's shards because its time on this host is not
known. `ess-cli`: `interpreted_target` pins the interpreted run's output; the other targets are
left to the forge's shards. `go` as in s1. Do not start `task check` or a whole-workspace run.

## Acceptance

1. `cargo test -p ess-conformance --locked --test input_path_addressing` passes tests 1-8, each
   timed in the report.
2. Test 5: 0 refusals under `shop.route.` and every scenario `Passed` against `Interpreted`
   (parent acceptance 3, first sentence).
3. Test 6: the target reading a top-level `depots` does not pass `Send/outcome/unknown-depot`
   (parent acceptance 3, second sentence).
4. The existing-green list and the `--lib` line pass unchanged.
5. The stamp test passes after the re-stamp; `task test-slow PACKAGES=ess-conformance` and the
   four table targets pass with no `.tsv` edited.
6. Step 7's CLI runs exit 0 with 0 refusals and every scenario passed (output in the report).
7. `git diff --stat` shows only the owned files.

## Decisions

- D1. Synthesis keys the arranged row by the path string (as it keys by the field today) and
  writes and reads the input through the path-aware helpers; the whole struct value is still
  built by the witness builder, and the arranged identity replaces its member. No second input
  representation and no flattened key reaches a suite.
- D2. A struct holding an arranged identity is sent as `ScenarioValue::Members`, the existing
  suite/32 form; the suite version is decided as today, from the suite's vocabulary and the
  model's features (`ConformanceSuite::select_fresh_format_for`, `src/scenario.rs` 340), never
  from the specification format, so the models' `ess/38` moves no suite major either (s4's test 8
  pins it).
- D3. The absence witness of a reference whose route crosses an `Optional` leaves out the first
  `Optional` member on the route (`settings.extra`), the value an `Optional` top-level reference
  is left out with today; a terminal `Optional` member is left out itself.
- D4. `interpret/await_outcome.rs` answers a system-issued command whose `via:` is a member path
  as unsupported, by name (N6). Reading the member would need `choices` (315-) to enumerate
  candidates inside a struct input, over 40 lines; the limit is named in the Outcome and as a
  follow-up in `s-map.txt`. s4 does the same for an `instance:` path.
- D5. The `{related: {via, field}}` value source (`src/synthesize/related.rs` `input_read`,
  772-795, which drops paths) is not changed (s1 D6).
- D6. No memo, no new counter, no new thread state.
- D7. X4: this unit changes only what a `when_related:` guard needs; the subject's `named` path in
  `supply` and the instance-specific calls in `fresh_identity` are s4's, tested by s4's tests.

## Open for the operator

None.

## Changelog

Under `## [Unreleased]`, `### Added`, after s1's "Format `ess/38`: …" bullet, at the end of that
heading (s2, in the same round, also appends there: the second to land appends after the
first's):

- Format `ess/38`: synthesis witnesses a `when_related:` guard that names its row through a member of a struct input, `via: input.settings.primary`, an `all:` element and a `List<…>` member with `{missing: …}`: the arranged identity is sent inside the struct, and the interpreted target reads it there. Suites of models without such a path keep their bytes and their suite format.
