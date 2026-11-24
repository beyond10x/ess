---
format: aep.planning-md/3
id: story:generated-rust-and-go-behaviour-read-a-related-row-and-the-addressed-instance-through-a-member-of-a-struct
kind: story
status: active
title: 'Generated Rust and Go behaviour read a related row and the addressed instance through a member of a struct input and pass their synthesized suite, and a List member or an each: over one keeps its command owed by name'
tags:
- wave-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:generated-targets-read-a-reference-nested-in-a-struct-input
- depends_on: story:at-ess-38-a-when-related-via-an-all-element-and-a-missing-payload-source-name-a-member-of-a-struct-input-a
- depends_on: story:at-ess-38-instance-and-each-in-name-a-member-of-a-struct-input-and-the-documents-list-every-addressing
- depends_on: story:synthesis-witnesses-an-instance-and-an-each-in-that-name-a-member-of-a-struct-input-on-the-parent-s-minimal
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-xtask/tests/slow_lane.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/determined.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/struct_member_address.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T21:48:35Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-23T21:48:36Z", actor: "human:timo", revision: 9}
---
## Outcome

- The generated Rust and Go behaviour of the parent's minimal model (s4's fixture), whose `Check`
  reads a related row by `when_related: {via: input.settings.primary}` and whose `Note` updates the
  row addressed by `instance: settings.primary`, build with no hand-written code and pass every
  scenario that model synthesizes (`ess verify conform run` against each served binary), in the new
  `ess-synth` test target `struct_member_address` that builds both. A faulty control that looks
  the bin up by `settings.backup` where `settings.primary` is meant fails `Check/outcome/checked`
  and `Check/outcome/closed`, the scenarios where only the primary's row is arranged.
- With `via: input.settings.depots`, a `List` member, `ess generate synthesize` for Rust and Go
  keeps the command an obligation whose reason names `input.settings.depots`, as the top-level list
  does today; its `{missing: input.settings.depots}` stays owed with it.
- An `each: {in: input.batch.lines}` keeps its command owed by name in both targets, as a top-level
  `each:` does; generating every model of this feature panics in neither target (N5).
- Every committed generated tree and every byte pin of `ess-synth` keeps its bytes: a one-segment
  field emits exactly today's text.

## Why

- Parent `story:generated-targets-read-a-reference-nested-in-a-struct-input` (serves O2): a model
  that s1 and s2 admit must not make the generators emit `input.settings.primary` as one
  identifier (Rust `name::value_ident`) or panic (Go `items::member_ident`, `unreachable!`).
- Parent acceptance 1 and 2; acceptance 3-5 (Entity Runtime, OpenAPI) are s6.

## Where

Read at commit `d3252eda9` (tree named in s1; re-read there for D48); built on the wave
branch with s0 to s4 merged (round 4; s6, merged in round 3, shares no file with this unit but
`CHANGELOG.md`). Depends on s1, s2 (tests 5 and 6 read s2's fixture
`struct-input-member-addressed-rows.yaml.in`) and s4, and on s0 through s1: every model this unit
reads declares `ess/38` (s1's, s2's and s4's fixtures), the only format that admits a member path.
This unit writes no model of its own and adds no format check: it reads no `format()` of the IR;
s1 and s2 refuse a member path below `ess/38`. Paths under `crates/generate/ess-synth/`. After s1/s2 the IR carries the path in
`ResolvedRelatedVia::Input { field }` and in `ResolvedInstance::Supplied { field }.name`.

`src/rust/behaviour.rs` (each site names an input field directly as `input.{value_ident(field)}`):

- `input_value` (2756-~2810): walks a dotted field through struct members, `Optional` and
  newtype bodies (ess/22) and returns an owned `….clone()`. Split it: new `fn input_place(&self,
  field: &str) -> (String, bool)` returns the place expression and whether it is an `Option<&T>`
  (the walk as it is, without the final clone); `input_value` becomes `input_place` plus today's
  clone suffix. For a one-segment field `input_place` returns exactly `input.{value_ident(field)}`
  and `false` (`true` only where today's text reads `.as_ref()`), so no byte moves.
- `related_row` (1308-1338, 1318-1327) and `present_related` (1528-~1555, 1544-1550): the
  `reference` (`Some(&input.x)` / `input.x.as_ref()`) and the label `input.x` come from
  `input_place`.
- The subject-identity sites: `addressed_existence` (1404, 1435), `check_subject` (1962, 1978),
  2194, `take` (2241; 2260-2341, `&input.{identity}` in the get, update and delete lines), 2386-2408
  (the re-key filter), and 1645 and 1783-1790 where they name a `via` or an identity: each
  `input.{value_ident(&field.name)}` becomes the `input_place` expression.
- Every other `value_ident(&field.name)` (2658, 2677, 2725 write struct literal members; 2826;
  3326-3346 `Root::Input`) is not an input read by an address: unchanged.

`src/go/behaviour.rs`:

- `input_member` (2449-2455) calls `items::member_ident(&self.command.input, field)`
  (`src/go/items.rs` 652-660), which panics on a dotted field. Leave `input_member` for one
  segment; at the via sites `related_row` (1957, 1967) and `present_related` (2157, 2179) and the
  identity sites (2058, 2082, 2275, 2798, 3081, and 2622/3235 where they read an address) call
  `self.input_value(field)` (2461, path-aware since ess/22, emitting nil checks past an `Optional`
  before the expression) when `field` contains `.`, and `input_member` otherwise, so one-segment
  output is byte-identical.

`src/determined.rs` (which command each target generates, and the reason it owes the others):

- the list obligation (226-235) prints `` the list `{via}` `` and `ResolvedRelatedVia`'s
  `Display` writes `input.{field}`, so a path is named with no change: pinned by test 1. A
  `{missing: input.<path>}` exists only on the branch of such a list guard (s1 keeps
  `related_list::check` refusing it anywhere else), so it is owed with that command and reaches no
  emitter.
- `affects` (1896-~1990): every `affects:` entry with `each:` keeps its command owed (1901-1911
  with `repeated:`, 1925-1927 without a filter), the reason starting `` effects on filtered rows
  (`affects:`) `` (1897): an `each: {in: input.<path>}` takes the same path and is pinned by
  test 5. N5's choice: both positions stay owed by name in both targets, because no emitter reads
  a row per element (the parent's own list rule) and reading them is not what parent 3 asks.
- If a check in `related` (1689-~1760) or `supported` refuses a dotted `via` or identity before
  generation, admit the path there (one-segment unchanged) and name the line in the report.

New test target `tests/struct_member_address.rs` (parent scope names it), with `#[allow(dead_code)]
mod related_guard_served; use related_guard_served as served;` as
`tests/related_guard_unknown_instance_served.rs` (12-14) does: the module builds through
`go`/`cargo` and is already in `ALLOWED` of `crates/edge/ess-xtask/tests/no_test_builds.rs` (81);
the new file spawns nothing itself, so it does not enter `ALLOWED` (which only shrinks).

How often a program is built (N4). The shared module builds a Rust program into a target directory
keyed by the test process (`tests/related_guard_served/mod.rs` 145-152: `related-guard-rust-build-<pid>`
under `CARGO_TARGET_TMPDIR`, behind an in-process mutex) and a Go program with `go build` (130-142,
the host's Go build cache). In this file each program is built exactly once per run: test 3 is the
only test that builds Rust (the served program, then the faulty control in the same process and
target directory, so the dependency graph compiles once), test 4 the only one that builds Go. No
second process builds the same product, so no cross-process marker is needed. The cross-process
form the routing names (a shared target under `CARGO_TARGET_TMPDIR` reused by later processes) is
`crates/edge/ess-cli/tests/browser_product_adversary.rs` `build_host` (51-110, shared target at
84-108); no once-built WebAssembly host of initial-state probes was found under
`crates/generate/ess-synth/tests/` (searched `initial`, `probe`, `once` beside `wasm`). Changing
the shared module's per-process target directory would touch every served test and enlarge an
`ALLOWED` file's spawning code: not done here.

The served model is s4's fixture
`crates/verify/ess-conformance/tests/fixtures/struct-input-member-served.yaml.in` (text in s4;
domain `shop.stock`: `AddBin`, `Check` with `via: input.settings.primary`, `Note` with
`instance: settings.primary`, `unknown_instance:` and a `via: input.settings.backup` predicate;
s4's test 9 proves its suite has no refusal and passes on `Interpreted`), read with `const TEXT:
&str = include_str!("../../../verify/ess-conformance/tests/fixtures/struct-input-member-served.yaml.in");`.
s1's model A (`struct-input-member-related-rows.yaml.in`) and s2's model B
(`struct-input-member-addressed-rows.yaml.in`) are read the same way.

`crates/edge/ess-xtask/tests/slow_lane.rs` `MARKED` (17-…): one new row.

## Tests first

In `tests/struct_member_address.rs`. The base of this unit is the wave branch with s0 to s4
merged: the models validate and synthesize, and the generators still name an input by
`value_ident(field)` / `member_ident`.

1. `a_list_member_keeps_its_command_owed_and_named` (parent acceptance 2; fast, no build): model A
   compiled with `served::model`; for `Target::Rust` and `Target::Go`, `synthesize_for(&ir,
   target)`'s disposition of `CapabilityKind::CommandBehavior` `"shop.route.Send"` is
   `SynthesisDisposition::Obligation` whose `reason.describes()` contains
   `` the list `input.settings.depots` `` (shape of `tests/view_reference_owed.rs` 107-124 and its
   `owed` helper, 22-27). Covers the `{missing: input.<path>}` position too (N5: it lives on that
   branch only). Red on the base: none expected (the reason prints the path through `Display`):
   control, green at once; kept because it is the parent's acceptance 2 and pins the path in the
   reason. Expected under 2 s.
2. `the_served_model_plans_both_commands_generated` (fast, no build): on `TEXT`,
   `shop.stock.Check` and `shop.stock.Note` are `SynthesisDisposition::Generated` for both targets.
   Red on the base: the plan's markdown (`synthesis.plan.to_markdown()`), or a panic in Go's
   `member_ident` (`items.rs` 660). Expected under 2 s.
3. `rust_reads_a_struct_member_reference_and_passes_its_suite` and
4. `go_reads_a_struct_member_reference_and_passes_its_suite` (parent acceptance 1), both
   `#[ignore = "slow lane: builds and serves the generated Rust and Go behaviour"]`, sharing one
   `fn member_served(target: Target)` shaped as `unknown_instance_served` in
   `tests/related_guard_unknown_instance_served.rs` (56-95): `served::model(&served::served(TEXT,
   "stock"))`, `served::emit`, `served::build(&root, target, "shop", "stock", false)`,
   `served::run`, `served::assert_passes` over `Check/outcome/no-bin`, `Check/outcome/closed`,
   `Check/outcome/checked`, `Note/outcome/unknown`, `Note/outcome/backup-closed`,
   `Note/outcome/noted`, and that `run` reports no scenario other than `Passed`. Then the faulty
   control: `served::mutate(&served::behaviour_path(&root, target, "shop"), from, to)` replacing
   the generated `Check` lookup of the bin by `settings.primary` with one by `settings.backup`
   (copy `from` exactly from the first green emission: Rust reads like
   `BinStorage::get(&self.ports, &input.settings.primary)`, Go like
   `b.ports.BinStorage.Get(input.Settings.Primary)`; replace every lookup of the bin in `Check`'s
   behaviour and say how many), rebuild, and assert `Check/outcome/checked` and
   `Check/outcome/closed` are not `Passed`: there only the primary's row is arranged, so a lookup
   by `settings.backup` finds no row and answers `UnknownBin` (`no-bin` is not asserted: an
   unarranged backup answers it like the right code). Test 4 asserts `served::go_available()`
   first, as the existing Go case does. Red on the base: test 3 panics in emission or fails the
   build with `input.settings.primary` written as one identifier (print the build error); test 4
   panics in `member_ident`. Time each (a served build is expected over 60 s on a cold cache: it
   is slow-lane, which is why it is marked); report both times.
5. `an_each_over_a_list_member_keeps_its_command_owed_and_named` (N5; fast, no build): model B; for
   both targets `shop.parcel.Label` is an `Obligation` whose reason contains `` effects on filtered
   rows (`affects:`) `` (`determined.rs` 1897). Red on the base: none expected; control, green at
   once, kept to pin N5's choice for the `each:` position.
6. `generating_every_model_of_the_feature_panics_in_neither_target` (N5; fast, no build): for model
   A, model B and `TEXT`, each with `served::served(text, "<component>")`, and for both targets,
   `served::emit(&ir, target, "<case>")` returns; for model B the emitted behaviour of
   `shop.parcel.Relabel` contains `input.target.parcel_id` (Rust) / `input.Target.ParcelId` (Go),
   the member read. Red on the base: a panic `resolved field has a native name` (`items.rs` 660)
   for Go, or Rust text holding `input.target_parcel_id`-like single identifiers (print the line).
   Expected under 5 s per model and target; remove each emitted root after the assertion.

Existing targets that must stay green unchanged (each one `--test`): `view_reference_owed`,
`related_guard_unknown_instance_served`, `unknown_instance`, and the byte targets in Gate.

## Steps

1. Start from the wave branch with s0 to s4 merged; `git rev-parse --short HEAD` in the report.
2. Tests 1-6; run 1, 2, 5 and 6 and record their red lines (or "green at once" for 1 and 5); run
   3 with `--ignored` once and record its red line (panic text of the generator or the build
   error).
3. Rust `input_place`, then the Rust sites; Go sites; `determined.rs` only if a test shows it.
4. Run tests 1, 2, 5, 6, then 3-4 with `-- --ignored`; then the existing-green list and the byte
   targets. A committed generated file that differs is a defect of this unit: the one-segment
   path must emit today's text; fix the emitter, never re-pin.
5. Add the `MARKED` row in path order (after `crates/generate/ess-synth/tests/generated_view_queries.rs`
   and any other `ess-synth` row sorting before `struct_member_address.rs`, before the first
   `crates/verify/` row), names sorted:
   `("crates/generate/ess-synth/tests/struct_member_address.rs", &["go_reads_a_struct_member_reference_and_passes_its_suite", "rust_reads_a_struct_member_reference_and_passes_its_suite"])`.
   Run `slot cargo xtask slow-list --package ess-synth --targets` and see `struct_member_address`.
6. `CHANGELOG.md`; `slot task fmt-check`.

## Owned files

- `crates/generate/ess-synth/src/rust/behaviour.rs`
- `crates/generate/ess-synth/src/go/behaviour.rs`
- `crates/generate/ess-synth/src/determined.rs` (only if Step 3 needs it)
- `crates/generate/ess-synth/tests/struct_member_address.rs` (new)
- `crates/edge/ess-xtask/tests/slow_lane.rs` (one `MARKED` row)
- `CHANGELOG.md` (one bullet)

## Not yours

- `crates/generate/ess-synth/tests/related_guard_served/mod.rs` (use it, do not change it),
  `src/go/items.rs`, every committed generated tree, `crates/edge/ess-xtask/tests/no_test_builds.rs`.
- `crates/generate/ess-entity-runtime/`, `crates/generate/ess-gen/` (s6); every `ess-conformance`,
  `ess-domain`, `ess-compiler` file (s1-s4); s1's, s2's and s4's fixtures (read only; a
  declaration that must change for the served build is reported, not edited).

Shared files: `MARKED` in `slow_lane.rs` is shared with every unit of the wave that marks a slow
test: insert this row at its sorted path, change no other row (sorted order, not append, is what
`every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked` checks); on a merge
conflict keep both rows in path order. `CHANGELOG.md`: after s4's bullet, at the end of `### Added`
(s6 writes under `### Changed`). This unit is alone in round 4.

## Gate

`AGENTS.md` § Gate: the crate `ess-synth`; a change that could move a committed example tree also
runs `feasibility`, `adversary_related_guard_behaviour_pass1` and `http_binding_rust`; the byte
pins `served_codec_names` and `go`. `slow_lane.rs` changed, so `task test-xtask`. Every cargo
command carries `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0`; a pool job ends after 10
minutes unless it names its time (`SLOT_TIMEOUT=30m`, at most 30); a test over 60 s is killed in
the ordinary lane (tests 3-4 are slow-lane); the ordinary step ends within 120 s; named `--test`
targets one per command; `slot cargo xtask`; no Python, nothing in `/tmp`, no `RUSTC_WRAPPER`,
`SCCACHE_*` or `-j`. `go` must be on `PATH` for test 4 and for `go`; `ESS_TYPES_NODE` is not needed.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-synth --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test struct_member_address
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test struct_member_address -- --ignored
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test view_reference_owed
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test unknown_instance
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test related_guard_unknown_instance_served
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test served_codec_names
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test go
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test feasibility
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test adversary_related_guard_behaviour_pass1
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test http_binding_rust
slot --timeout 30m task test-slow PACKAGES=ess-synth
slot --timeout 30m task test-xtask
```

Do not start `task check` or a whole-workspace run.

## Acceptance

1. Tests 1, 2, 5 and 6 pass in the ordinary lane, each under 5 s (times in the report).
2. Tests 3 and 4 pass with `--ignored` and under `task test-slow PACKAGES=ess-synth`: every
   scenario of the served model passed on both targets, and each faulty control fails
   `Check/outcome/checked` and `Check/outcome/closed` (parent acceptance 1). Both times in the
   report.
3. Test 1: the reason names `` the list `input.settings.depots` `` for Rust and Go (parent
   acceptance 2).
4. `served_codec_names`, `go`, `feasibility`, `adversary_related_guard_behaviour_pass1`,
   `http_binding_rust` pass with no committed file changed.
5. `task test-xtask` passes with the new `MARKED` row (`every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked`).
6. `git diff --stat` shows only the owned files.

## Decisions

- D1. A path is emitted through the existing ess/22 path walkers (Rust `input_value`'s walk,
  Go `input_value`), never by a second walker; a one-segment field keeps today's emitter call, so
  no committed generated byte moves.
- D2. The served test reads s4's `.yaml.in` fixture, the parent's minimal model, holding only
  commands both targets generate, whose suite s4 proved on `Interpreted`; s1's model A (with
  obligations for `Send`'s list and `Pair`'s `all:`) and s2's model B (with `Label`'s `each:`)
  are used for the obligation and generation tests only.
- D3. The served tests are slow-lane (they build Rust and Go programs); the obligation, plan and
  generation tests stay in the ordinary lane. No Taskfile harness is added: each program is built
  once per run (N4, Where).
- D4. A route through an `Optional` member is not in the served fixture; the Rust and Go walkers
  already handle `Optional` for ess/22 reads, so it is not refused; the served proof covers the
  non-optional route the parent names.
- D5. N5: `each: {in: input.<path>}` and `{missing: input.<path>}` keep their commands owed by name
  in both targets (`determined.rs` 1897-1927 and 226-235); generation of every feature model is
  tested not to panic.

## Open for the operator

None.

## Changelog

Under `## [Unreleased]`, `### Added`, after s4's bullet, at the end of the section's `### Added`:

- Format `ess/38`: generated Rust and Go behaviour read a `when_related:` `via:` and an `instance:` that name a member of a struct input; a `List<…>` member and an `each:` over one keep the command an obligation that names it.
