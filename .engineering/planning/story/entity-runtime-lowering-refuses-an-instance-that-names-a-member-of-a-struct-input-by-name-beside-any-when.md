---
format: aep.planning-md/3
id: story:entity-runtime-lowering-refuses-an-instance-that-names-a-member-of-a-struct-input-by-name-beside-any-when
kind: story
status: active
title: 'Entity Runtime lowering refuses an instance: that names a member of a struct input by name, beside any when_related: refusal of the same command, its subset page says so, and OpenAPI keeps such a member inside the request body'
tags:
- wave-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:generated-targets-read-a-reference-nested-in-a-struct-input
- depends_on: story:at-ess-38-a-when-related-via-an-all-element-and-a-missing-payload-source-name-a-member-of-a-struct-input-a
- depends_on: story:at-ess-38-instance-and-each-in-name-a-member-of-a-struct-input-and-the-documents-list-every-addressing
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/fixtures/struct-member-instance.yaml.in
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/lowering.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/openapi.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest
- confidence: cited
  path: website/docs/reference/entity-runtime-lowering.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T21:48:30Z", actor: "human:timo", revision: 11}
- {from: "proposed", to: "active", at: "2026-11-23T21:48:31Z", actor: "human:timo", revision: 12}
---
## Outcome

- Lowering a model for the Entity Runtime refuses every command whose `instance:` names a member of
  a struct input with `AmbiguousInstanceBinding` at the command's path, naming the member, also
  where the command has a `when_related:` branch, which is refused beside it with
  `RelatedGuardUnsupported` at `<command>.<branch>.when_related`, as a top-level `via:` is refused
  today.
- `website/docs/reference/entity-runtime-lowering.md`, regenerated from `subset.rs`, has its
  `AmbiguousInstanceBinding` row name an `instance:` that is a member of a struct input.
- `ess generate openapi` on such a model writes each operation's request body as
  `<Command>.Input` with the struct input a member of it and the identity a member of the struct,
  and no parameter for the identity.

## Why

- Parent `story:generated-targets-read-a-reference-nested-in-a-struct-input` (serves O2),
  acceptance 3-5: the Entity Runtime host loads one instance from one top-level input field
  before a branch is chosen, so a member address must be refused by name, never lowered wrongly;
  the HTTP contract must not invent a parameter.
- This unit needs only s1 and s2 (its model validates and compiles), and s0 through s1: its model
  declares `ess/38`, the only format that admits a member `instance:` and `via:`; it runs in round
  3 beside s4 (X2, X3).

## Where

Read at commit `d3252eda9` (tree named in s1; re-read there for D48); built on the wave
branch with s0, s1 and s2 merged. This unit adds no format check: it reads no `format()` of the
IR; s1 and s2 refuse a member path below `ess/38` (as `unsupported_format_version`).

`crates/generate/ess-entity-runtime/src/lib.rs`:

- `lower_command` (1932-1941) asks `refuse_set_effects`, `refuse_related_guards` (1808),
  `refuse_compensating_refusals` and `refuse_identity_changes` "before any returns, so one does
  not hide another" (1934), then returns `false` (1940-1941) when any refused. `supplied_identity`
  (2409-2437) runs only after that return (its one caller, 2031), so a command with a
  `when_related:` branch never reaches it (design 1). New `fn refuse_member_instances(&mut self,
  command: &ResolvedCommand) -> bool`, called beside the others (after `refuse_identity_changes`,
  1938) and joined into the early-return condition (1940): for each outcome whose
  `subject.instance` is `ResolvedInstance::Supplied { field }` with `field.name` containing `.`
  (an `ess/38` member path; `ess_domain::command::input_path::is_path`), push
  `self.diagnostic(LoweringCode::AmbiguousInstanceBinding, command.name.to_string(),
  format!("`instance: {}` of outcome `{}` names a member of a struct input; the host loads the
  instance from one top-level input field before a branch is chosen", field.name,
  outcome.name))`, and return whether any was pushed. A one-segment `instance:` pushes nothing,
  so every other command lowers as today. `supplied_identity` and its two messages are unchanged.
- `refuse_related_guards` (1808, path `{command}.{outcome}.when_related`, 1816 and 1829) already refuses
  every `ResolvedCondition::Related`: unchanged; test 1 pins it for a member `via:`.

`crates/generate/ess-entity-runtime/src/subset.rs`:

- the `AMBIGUOUS` row (474-479): `Needs::NoTarget("the host loads one instance, from one top-level
  input field, before a branch is chosen; an `instance:` that names a member of a struct input
  has no single source")`, and cite the new test beside the existing one:
  `&["tests/lowering.rs::command_shape_diagnostics_cover_multiple_targets_and_identity_bindings",
  "tests/lowering.rs::an_instance_named_by_a_struct_member_is_refused_by_name"]`.
- the code text (849-853): "The instance a command acts on is not named by one top-level input
  field (a member of a struct input is not one), or a creation does not observe its identity in an
  emitted event."
- `AMBIGUOUS` (61), the construct name, is unchanged (`assert_constructs_are_catalogued`).
- The old text "not named by one input field" occurs only at `subset.rs` 852 and the generated
  page's line 112 (checked with `grep -rn` at `d3252eda9`).

`website/docs/reference/entity-runtime-lowering.md` rows 64 and 112: regenerated, never edited by
hand: `ESS_LOWERING_REFERENCE=write cargo test -p ess-entity-runtime --locked --test
lowerable_subset` (the switch at `tests/lowerable_subset.rs` 757-762).

`crates/generate/ess-gen/src/openapi.rs`: no change expected. `operation` (848) takes path
parameters only from an `ess-http` route (`path_parameters`, 811-846), the body is
`<Command>.Input` (`request_body`, 1204), and `instance:` appears only in the description
(2972-2975). Test 3 pins the outcome. If it fails, the fix is in `openapi.rs`, and the synthesis
digest moves (`ess-gen` is one of the eight hashed crates, `synthesis_suite_bytes_table.rs`
375-384): then s4 lands first and this unit re-runs the stamp command on the merged tree (s3 Step
6's command) and `slot --timeout 30m task test-slow PACKAGES=ess-conformance`.

New fixture `crates/generate/ess-entity-runtime/tests/fixtures/struct-member-instance.yaml.in`
(X2: a model of this unit's own; text under "The fixture"): `Note` holds a `when_related:` branch
(`via: input.pick.backup`) and an `updates:` addressed by `instance: pick.primary`; `Tag` holds the
same `updates:` with no guard. `.yaml.in` keeps it out of every corpus walk (the lowering table
reads `.yaml` corpus models only). The OpenAPI test reads it from `ess-gen` by `include_str!` with
a relative path, as `ess-synth`'s tests read `ess-conformance`'s fixtures.

## Tests first

Each test compiles one 80-line model and lowers or projects it: expected under 2 s each, time
each; none may run over 60 s; each target's step stays within 120 s. The base is the wave branch
with s0, s1 and s2 merged: the model compiles, `lower_command` has no member check.

1. `crates/generate/ess-entity-runtime/tests/lowering.rs`,
   `an_instance_named_by_a_struct_member_is_refused_by_name` (parent acceptance 3): `const SHELF:
   &str = include_str!("fixtures/struct-member-instance.yaml.in");` plus a components block
   appended as `related_guard_served::served` writes it
   (`crates/generate/ess-synth/tests/related_guard_served/mod.rs` 44-48): component `shelf`,
   `reached_by: network`, `owns: {domains: [shop.shelf]}`, `accepts: {commands: [shop.shelf.AddSlot,
   shop.shelf.Note, shop.shelf.Tag]}`, `publishes: {events: [shop.shelf.SlotAdded,
   shop.shelf.SlotNoted, shop.shelf.SlotTagged]}`. Compile it (parse, `Specification::assemble`,
   `compile`, as `served::model` 24-29), then `lower(&selected(&ir, &plan, "shelf"),
   &options(&["shop.shelf.Slot"]))` (`selected` 91, `options` 104) is `Err`, and its
   `into_vec()` holds exactly these three for `Note` and `Tag`:
   - code `AmbiguousInstanceBinding`, path `shop.shelf.Note`, message
     `` `instance: pick.primary` of outcome `noted` names a member of a struct input; the host loads the instance from one top-level input field before a branch is chosen ``;
   - code `AmbiguousInstanceBinding`, path `shop.shelf.Tag`, message
     `` `instance: pick.primary` of outcome `tagged` names a member of a struct input; the host loads the instance from one top-level input field before a branch is chosen ``;
   - code `RelatedGuardUnsupported`, path `shop.shelf.Note.closed.when_related`.
   Red on the base: the codes printed hold `RelatedGuardUnsupported` for `Note` and, for `Tag`,
   either nothing or `supplied_identity`'s generic text; no diagnostic names `pick.primary`.
2. `crates/generate/ess-entity-runtime/tests/lowerable_subset.rs`: no new test; the page check
   fails until the page is regenerated (parent acceptance 4). Red line: the diff it prints after
   the `subset.rs` edit.
3. `crates/generate/ess-gen/tests/openapi.rs`,
   `a_struct_member_identity_stays_inside_the_request_body` (parent acceptance 5): the same text
   (`include_str!("../../ess-entity-runtime/tests/fixtures/struct-member-instance.yaml.in")`) and a
   local copy of the components block, compiled with the file's `inline` helper (91);
   `document(&ir, "shelf")` (193); for `Note` and `Tag`, the one operation of
   `operations(&document)` (304) whose `requestBody.content."application/json".schema."$ref"` is
   `#/components/schemas/shop.shelf.<Command>.Input` exists; that schema has
   `properties.pick."$ref"` `#/components/schemas/shop.shelf.Pick` and `required` `["pick"]`;
   `shop.shelf.Pick` has `primary` among its `properties`; the operation's `parameters` is absent
   or names no parameter `primary`, `pick` or `pick.primary`; and `assert_valid` (242) accepts the
   document. Red on the base: none expected; control, green at once (state so in the report); kept
   because it pins an outcome the parent requires.

Existing targets that must stay green unchanged: `ess-entity-runtime` `--test related_guard`,
`--test lowered_definitions_table`; `ess-gen` `--test openapi` (whole file).

## Steps

1. Start from the wave branch with s0, s1 and s2 merged; `git rev-parse --short HEAD` in the report.
2. The fixture; validate it once by hand: copy to a scratch `shelf.yaml` and run `ess specify
   validate` and `ess generate openapi` on it; output in the report. A declaration that does not
   validate is corrected in the fixture and named in the report.
3. Tests 1 and 3; record red lines.
4. `refuse_member_instances` and its call in `lower_command`; then `subset.rs`; then regenerate
   the page:
   `ESS_LOWERING_REFERENCE=write CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowerable_subset`,
   and run it again without the switch.
5. `website/` changed, so `slot --timeout 30m task test-xtask`; `task site-build` is not run
   (generated reference row only; the docs check of the pull request covers it).
6. `CHANGELOG.md`; `slot task fmt-check`.

## Owned files

- `crates/generate/ess-entity-runtime/src/lib.rs` (`refuse_member_instances` new, its call in
  `lower_command`)
- `crates/generate/ess-entity-runtime/src/subset.rs` (the `AMBIGUOUS` row and code text)
- `crates/generate/ess-entity-runtime/tests/lowering.rs` (one test, appended at the end)
- `crates/generate/ess-entity-runtime/tests/fixtures/struct-member-instance.yaml.in` (new)
- `website/docs/reference/entity-runtime-lowering.md` (regenerated)
- `crates/generate/ess-gen/tests/openapi.rs` (one test, appended at the end)
- `CHANGELOG.md` (one bullet)
- Only if test 3 fails first: `crates/generate/ess-gen/src/openapi.rs` and
  `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (re-stamped after s4)

## Not yours

- Every `ess-synth` file (s5), every `ess-conformance` source and fixture (s3, s4; this unit reads
  none of them), every `ess-domain`/`ess-compiler` file (s0, s1, s2), and every file s0 moved to
  open `ess/38` (among them `lowered-definitions-table.tsv` of this crate, whose toolchain rows s0
  re-pinned; this unit's fixture is `.yaml.in` and moves no row of it).

Shared files. With s4 (same round 3): `CHANGELOG.md` under different headings (this unit
`### Changed`, s4 `### Added`), and, only if test 3 forces an edit in
`crates/generate/ess-gen/src/openapi.rs`, `synthesis-sources.digest`: s4 lands first, this unit
re-runs the stamp command on the merged tree. No other file is shared with s4. With s1 to s3:
`CHANGELOG.md` only, and their bullets are under `### Added`; this one goes under `### Changed`,
after the last bullet of that heading, changing nothing above.

## Gate

The crates `ess-entity-runtime` and `ess-gen`; `website/` changed, so `task test-xtask`. Every
cargo command carries `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0`; a pool job ends after
10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, at most 30); a test over 60 s is killed;
the step ends within 120 s; named `--test` targets one per command; `slot cargo xtask`; no Python,
nothing in `/tmp`, no `RUSTC_WRAPPER`, `SCCACHE_*` or `-j`.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-entity-runtime --all-targets --locked -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowering
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowerable_subset
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test related_guard
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowered_definitions_table
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-gen --all-targets --locked -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-gen --locked --test openapi
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-openapi --locked --test adversary_ess_issues_2_u6
slot --timeout 30m task test-xtask
```

If `openapi.rs` changed (Owned files, last item): add the re-stamp command of s3 Step 6, `slot
--timeout 30m task test-slow PACKAGES=ess-conformance` and the byte pins of s3's Gate. Do not
start `task check` or a whole-workspace run.

## Acceptance

1. `cargo test -p ess-entity-runtime --locked --test lowering` passes with test 1 (parent
   acceptance 3).
2. `cargo test -p ess-entity-runtime --locked --test lowerable_subset` passes with the page
   regenerated; the page's `AmbiguousInstanceBinding` row contains "a member of a struct input"
   (parent acceptance 4).
3. `cargo test -p ess-gen --locked --test openapi` passes with test 3 (parent acceptance 5).
4. Step 2's CLI output in the report: the fixture validates and `ess generate openapi` exits 0.
5. `task test-xtask` passes.
6. `git diff --stat` shows only the owned files (and `openapi.rs` plus the stamp only if test 3
   failed first).

## Decisions

- D1. A member address is refused by the existing code `AmbiguousInstanceBinding` with a message
  naming the path and the outcome; no new `LoweringCode` (the parent names this code, and a new
  code would move the code catalogue and the page's code table). The message does not say
  "existing-instance outcome": it names the outcome, which is an `updates:` here.
- D2. The refusal is its own check among those `lower_command` asks before its early return, so a
  `when_related:` branch in the same command does not hide it (design 1).
- D3. `RelatedGuardUnsupported` needs no change: it refuses every `when_related:` regardless of
  the `via:` shape; test 1 pins it for a member.
- D4. No OpenAPI parameter is added for an identity inside the body: the body already carries it,
  and a second carrier would let the two disagree.
- D5. X2: this unit's model is its own fixture under the crate it tests; it neither owns nor reads
  the served fixture (s4's), so it depends on s1 and s2 only (and on s0 through s1).
- D6. The changelog bullet stays under `### Changed`, not in the `### Added` "Format `ess/38`:"
  register: it admits no construct, it narrows what the Entity Runtime lowers, and it names the
  format the constructs it refuses belong to. It is not `**Breaking**`: no model below `ess/38`
  can write a member `instance:`, so nothing lowered before is refused now.

## Open for the operator

None.

## Changelog

Under `## [Unreleased]`, `### Changed`, after the last bullet of that heading:

- Entity Runtime lowering refuses an `instance:` that names a member of a struct input, which `ess/38` admits, as `AmbiguousInstanceBinding`, naming the member and its outcome, also beside a `when_related:` branch; the lowering reference says so.

## The fixture

`crates/generate/ess-entity-runtime/tests/fixtures/struct-member-instance.yaml.in`, byte for byte
(Step 2 may correct a declaration that does not validate, and says so in the report):

```yaml
format: ess/38
system: shop
version: v1
domain: shop.shelf
summary: A pick names the slot it acts on inside a struct; a note is refused for a closed backup slot, a tag is not guarded.
types:
  - {name: shop.shelf.SlotId, kind: newtype, of: Uuid}
  - name: shop.shelf.Pick
    kind: struct
    fields:
      - {name: primary, type: shop.shelf.SlotId}
      - {name: backup, type: shop.shelf.SlotId}
      - {name: note, type: String}
entities:
  - name: shop.shelf.Slot
    identity: {name: slot_id, type: shop.shelf.SlotId}
    fields:
      - {name: open, type: Boolean}
      - {name: note, type: String}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
errors:
  - {name: shop.shelf.UnknownSlot, summary: The slot the pick names does not exist., fields: []}
  - {name: shop.shelf.ClosedSlot, summary: The backup slot the pick names is closed., fields: []}
events:
  - name: shop.shelf.SlotAdded
    fields: [{name: slot_id, type: shop.shelf.SlotId}]
  - name: shop.shelf.SlotNoted
    fields: [{name: slot_id, type: shop.shelf.SlotId}]
  - name: shop.shelf.SlotTagged
    fields: [{name: slot_id, type: shop.shelf.SlotId}]
actors:
  - name: shop.shelf.Clerk
    may: [shop.shelf.AddSlot, shop.shelf.Note, shop.shelf.Tag]
commands:
  - name: shop.shelf.AddSlot
    input:
      - {name: open, type: Boolean}
      - {name: note, type: String}
    outcomes:
      - name: added
        creates: shop.shelf.Slot
        instance: slot_id
        emits: [shop.shelf.SlotAdded]
        payload: {shop.shelf.SlotAdded: {slot_id: {generated: true}}}
        sets: {open: input.open, note: input.note}
  - name: shop.shelf.Note
    input:
      - {name: pick, type: shop.shelf.Pick}
    outcomes:
      - name: unknown
        unknown_instance: true
        error: shop.shelf.UnknownSlot
      - name: closed
        when_related: {via: input.pick.backup, predicate: open == false}
        error: shop.shelf.ClosedSlot
      - name: noted
        updates: shop.shelf.Slot
        instance: pick.primary
        emits: [shop.shelf.SlotNoted]
        payload: {shop.shelf.SlotNoted: {slot_id: input.pick.primary}}
        sets: {note: input.pick.note}
  - name: shop.shelf.Tag
    input:
      - {name: pick, type: shop.shelf.Pick}
    outcomes:
      - name: unknown
        unknown_instance: true
        error: shop.shelf.UnknownSlot
      - name: tagged
        updates: shop.shelf.Slot
        instance: pick.primary
        emits: [shop.shelf.SlotTagged]
        payload: {shop.shelf.SlotTagged: {slot_id: input.pick.primary}}
        sets: {note: input.pick.note}
views:
  - name: shop.shelf.Slots
    source: shop.shelf.Slot
    consistency: read_your_writes
    fields:
      - {name: slot_id, type: shop.shelf.SlotId}
      - {name: open, type: Boolean}
      - {name: note, type: String}
```
