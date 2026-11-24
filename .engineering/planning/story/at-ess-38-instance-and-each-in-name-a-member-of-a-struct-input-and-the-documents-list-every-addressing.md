---
format: aep.planning-md/3
id: story:at-ess-38-instance-and-each-in-name-a-member-of-a-struct-input-and-the-documents-list-every-addressing
kind: story
status: active
title: 'At ess/38 instance: and each: {in:} name a member of a struct input, and the documents list every addressing position that reads an input path'
tags:
- wave-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:via-and-instance-may-name-a-member-of-a-struct-input
- depends_on: story:at-ess-38-a-when-related-via-an-all-element-and-a-missing-payload-source-name-a-member-of-a-struct-input-a
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/related_guard_ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/input_path.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/outcome_shapes.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/set_effects.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/input_path_addressing.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/struct-input-member-addressed-rows.yaml.in
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest
- confidence: cited
  path: docs/design/expression-family-source22.md
- confidence: cited
  path: docs/design/language-grids.md
- confidence: cited
  path: schemas/generated/ess.schema.json
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/predicates.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 23
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T21:48:25Z", actor: "human:timo", revision: 22}
- {from: "proposed", to: "active", at: "2026-11-23T21:48:26Z", actor: "human:timo", revision: 23}
---
## Outcome

- At `format: ess/38` (opened by s0), `instance: target.parcel_id` addresses the row of an `updates:`, a `moves:`
  and a `deletes:` branch and the `unknown_instance:` branch that answers it; an
  `existing_instance:` branch answers a `creates:` whose published identity is
  `input.target.parcel_id`; and `affects: [{each: {in: input.batch.lines, as: line, …}}]` walks a
  `List` member of a struct input. `ess specify validate` exits 0 on the model below.
- Below `ess/38` (so at `ess/37`, released in 0.93.0, too) every addressing position that names a
  member (the five of the parent, s1's three included) is refused as `unsupported_format_version`
  naming `ess/38`, never as ESS-COMMAND-001 (`… which is no command input of it`) or
  ESS-COMMAND-009 (`` each: walks a list of the command's input, written `input.<field>` ``). Each
  was refused before this wave too, so no document at `ess/37` or below changes meaning.
- A member the struct does not declare is `undeclared_reference` naming the struct's members; a
  member of the wrong type, and a path crossing a list, a map or a union, are `type_mismatch`
  naming the type, at `instance:` and `each:` as at `via:`. An `instance:` whose route crosses an
  `Optional` is refused as an `Optional` top-level input is today.
- A `when:` guard reading the member that names the instance is refused as a guard reading a
  top-level instance input is today (invariant 13, identities are opaque).
- A `distinct:` over a `List` member (`distinct: {in: batch.lines, …}`) holds an `each:` over that
  member distinct, as one over a top-level list does.
- `docs/design/expression-family-source22.md`, `website/docs/reference/predicates.md`,
  `docs/design/language-grids.md` and `website/docs/reference/formats.md` list the new positions.

## Why

- Parent `story:via-and-instance-may-name-a-member-of-a-struct-input` (serves O2); the closing rule
  opens every addressing position at once (`docs/design/generic-language-features.md:211-216`).
- The operator's decision D48 (2026-11-23): the feature opens `ess/38`; `ess/37` keeps its
  meaning. s0 opened `ess/38` and holds the gate (`FormatVersion::V38`,
  `input_path::addresses_members`, `input_path::below_ess_38`); s1 opened `via:`, `all:` elements
  and `{missing:}` and holds `input_path::address` and `reads_member`; this unit opens the rest
  and writes the documents.

## Where

Read at commit `d3252eda9` of the tree named in s1 (re-read there for D48). This unit is built on
the wave branch after s0 and s1 landed, in round 2 beside s3. s1 and s2 both edit seven files:
`crates/specify/ess-domain/src/command/input_path.rs`, `crates/specify/ess-domain/src/system.rs`
(row 38), `crates/specify/ess-domain/src/spec.rs`, `crates/specify/ess-compiler/src/resolve.rs`,
`crates/specify/ess-compiler/src/ir.rs`, `crates/specify/ess-domain/tests/input_path_addressing.rs`
and `crates/specify/ess-compiler/tests/related_guard_ir.rs` (plus `formats.md` row `ess/38`, the
two generated files, the stamp and `CHANGELOG.md`); s0 wrote lines in the first two, in
`input_path_addressing.rs` and in `formats.md`. Their lines are in place when this unit starts:
re-read every site below by name, never by the line number given here.

`crates/specify/ess-domain/src/command/input_path.rs` (after s0 and s1; s0 added
`addresses_members` and `below_ess_38`, s1 `address` and `reads_member`; this unit uses them and
changes none):

- New `pub(crate) fn validate_instances(spec: &Specification) -> ValidationErrors`: for every
  command, every outcome whose `subject.surface()` is `InstanceSurface::CommandInput` and whose
  `subject.instance` `is_path`, at `command.site().key("outcomes").named(outcome).key("instance")`:
  - below `ess/38` (`!addresses_members(Some(spec.system().format))`): `below_ess_38(at,
    "`instance: <path>`")`, message `` `instance: target.parcel_id` names a member of a struct
    input, which requires specification format ess/38 ``;
  - else `resolve(command, &spec.system().types, path)`: an `Err(unresolved)` is
    `unresolved.refusal(&at, command, path)`;
  - else, where the entity is declared (`spec.entities().get(&subject.entity)`) and
    `path.type_ref() != entity.identity.type_ref`: `TypeMismatch` with today's text of
    `validate_instance` (entity.rs 1500-1511): `` outcome `<o>` of `<c>` names the instance by
    `<path>`, which is typed `<type>`; `<E>` is identified by `<identity type>` `` and the hint
    `` the command input must be typed `<identity type>` ``. A route through an `Optional` reads
    at `Optional<…>`, so it lands here exactly as an `Optional` top-level input does today.
  Typed sites in this (sub-directory) file: the census of
  `docs/design/review-typed-diagnostics.md` does not move.

`crates/specify/ess-domain/src/spec.rs`: call `crate::command::input_path::validate_instances(self)`
beside `related_guard::validate` (589).

`crates/specify/ess-domain/src/entity.rs` `validate_instance` (1436-1520): in the
`InstanceSurface::CommandInput` arm (1455-1463), where
`crate::command::input_path::is_path(&subject.instance)`, return `errors` empty: the path rule
above answers it. No `ValidationError` is added to this top-level file (census unchanged).

`crates/specify/ess-domain/src/command.rs` (top-level file: only conditions change, no new
`ValidationError::new`/`::at`):

- `RawOutcome::instance` doc (7551-7555): add "From `ess/38` an `updates:`, `moves:` or
  `deletes:` may name a member of a struct input, `instance: target.parcel_id`." (schema moves).
- Invariant 13 in `validate_guard` (3502): the condition `subject.surface() ==
  InstanceSurface::CommandInput && root == subject.instance` (3579) also holds where
  `input_path::reads_member(&segments_of(path), &subject.instance)`; `segments_of` is the
  `FactPath`'s segments as `&str` (use what the surrounding code already has for `path`).
- `validate_typed_guard` (3602): the filter `read.path.namespace() == subject.instance` (3669) also
  holds where `reads_member(..)` of `read.path`'s segments.

`crates/specify/ess-domain/src/command/set_effects.rs` (3595 lines):

- `RawEach::list` doc (142): "The list, `input.<field>`, or from `ess/38` `input.<path>` to a
  `List` member of a struct input." (schema moves).
- `each_entry` (954-1056): the list check (979-997) admits `input.<seg>(.<seg>)*`, every segment
  `crate::binding::is_field_name`; `Each::list` holds the path whole. The refusal text for
  anything else stays (it is the one below `ess/38` never reaches for a path, see `refuse_each`).
- `refuse_each` (3116-~3200): compute `below_38` (`!input_path::addresses_members(format)`)
  beside `below_23`/`below_27` (3134-3135); in the loop (3141), before the `below_23` arm, where
  `below_38` and the raw `each.list` stripped of `input.` contains `.`: push
  `each_path_below_ess_38(format!("{at}.each"), list)` (a new fn beside `each_below_ess_23`,
  3057, with a string location as every `each:` refusal of this file has: code
  `UnsupportedFormatVersion`, message `` `each: {in: <list>}` names a member of a struct input,
  which requires specification format ess/38 ``, the same sentence s0's `below_ess_38` builds,
  hint `FormatVersion::V38.declaration_hint()`), then drop the entry the way the `below_23` arm
  does (3143-3150) and `continue`, so no ESS-COMMAND-009 follows.
- `each_rules` (3208-~3330): the list lookup `command.input.iter().find(|field| field.name ==
  each.list)` (3220-3233) becomes `input_path::address(command, &spec.system().types,
  &each.list)`; for a path that does not resolve, push `Unresolved::refusal` at the `each` site
  instead of the "does not declare as an input" refusal. The element-type check (3234-3262) reads
  the resolved field's `type_ref` and names it in its message as today.
- `held_distinct` (3461-3485): the list match (3463-3467, `[only]` / `[root, only]`, so today
  `distinct: {in: batch.lines}` never matches `each.list == "batch.lines"`) becomes: the
  `distinct.over` segments, a leading `input` dropped, joined by `.`, equal `each.list`; so a
  `distinct:` over `batch.lines` or `input.batch.lines` holds a member list distinct as one over a
  top-level list does. `distinct: {in: <path>}` already reads a member list through the ess/22
  fact read (`crates/specify/ess-domain/src/expression.rs` 2892-2925, `self.read(&distinct.over,
  true)`). Test 11 (N3).

`crates/specify/ess-domain/src/command/outcome_shapes.rs`: only if test 3 (`Register`) is refused
by a check reading the published identity as a top-level input: `supplied_identity` (173) or
`validate_existing_pair` (391) then reads it through `input_path::address`. Read both before
Steps 4; name in the report which (if any) changed.

`crates/specify/ess-compiler/src/resolve.rs`:

- `instance` (4190-4280): in the `InstanceSurface::CommandInput` arm (4226-4238) a path candidate
  is `self.input_read(command, input, named)` wrapped as `ResolvedInstance::Supplied { field }`
  (one candidate, named by the path, at the terminal type, the root's `naming`); a top-level name
  keeps the `filter` it has.
- `resolved_each` (6332-6350): `list: each.list.clone()` already carries the path; unchanged.

`crates/specify/ess-compiler/src/ir.rs` docs: `ResolvedInstance::Supplied::field` (831) and
`ResolvedEach::list` (1127) gain "or the declared segments of a path to a member of a struct input
joined by `.` (ess/38); the field's `naming` is the root input's".

`crates/specify/ess-domain/src/system.rs` `FORMAT_HISTORY` row 38 (s0's row, s1's sentence):
append a second `added` sentence (D5). Row 37 is released and not touched.

Documents:

- `docs/design/expression-family-source22.md` § "A4: dotted input value paths" (369-418): one
  paragraph at its end, "From `ess/38` (beyond10x/ess#640) the addressing positions read the same
  path: …" naming `when_related: {via}`, an `all:` element, `instance:` on `updates:`, `moves:`,
  `deletes:` and the markers answering it, `each: {in:}` and `{missing:}`; the IR carries the path
  in the existing `field` strings (s1 D1); the Optional-route rule (`via` checked only when
  present; `instance:` refused as an Optional input).
- `website/docs/reference/predicates.md`: in the `when_related: {via: …}` row of the table (line
  29) add after "From `ess/22` `input.<field>` may be `Optional<…>`: …" the sentence "From
  `ess/38` `input.<field>` may be `input.<path>` to a member of a struct input, a `List<…>` member
  included (see [a row named by a member of a struct input](#a-row-named-by-a-member-of-a-struct-input))."
  and a subsection `### A row named by a member of a struct input` (no format in the heading, so
  its anchor is exactly the one linked) after "The rows a list of references names" (1663-1699),
  before `## A guard over the rows a selector selects` (1700), opening with "From `ess/38`": one
  example (`via: input.settings.primary`, `via: input.settings.depots` with `{missing:
  input.settings.depots}`), the refusals (undeclared member, type, below `ess/38`, and that a
  struct value written `{missing: input.a.b}` keeps its meaning below `ess/38`), and one sentence
  that `instance:` and `each: {in:}` take the same path.
- `docs/design/language-grids.md`: after the **Target** list of grid R (the bullets at 41-52), one
  paragraph: "From `ess/38` the addressing positions (`via:`, `all:` elements, `instance:`,
  `each: {in:}`, `{missing:}`) read `input.<path>` as `sets:` does; this opens the input column in
  every position that names a row (`story:via-and-instance-may-name-a-member-of-a-struct-input`)."
- `website/docs/reference/formats.md`, the `ess/38` row (s0 added it after line 203, s1 rewrote
  its text; row 203, `ess/37`, is not touched): append the D5 sentence's short form:
  "`instance:` on `updates:`, `moves:` and `deletes:`, and `each: {in:}`, take the same path."

New file `crates/verify/ess-conformance/tests/fixtures/struct-input-member-addressed-rows.yaml.in`:
the model under "The model" below (`.yaml.in`, outside the corpus walks: s1 D3).

## Tests first

Each new test is timed by the implementor; each is expected under 1 s; none over 60 s; the test
step stays within 120 s.

The base of this unit is the wave branch with s0 and s1 merged: `ess/38` is supported, s1's
positions are admitted there, and `instance:` and `each:` read a top-level input only. "Red on the
base" is what each test prints there.

Appended to `crates/specify/ess-domain/tests/input_path_addressing.rs` (s0 created it, s1 appended
its section), a section after s1's last test, with `const ADDRESSED: &str =
include_str!("../../../verify/ess-conformance/tests/fixtures/struct-input-member-addressed-rows.yaml.in");`
and s1's helpers (`spec`, `refused`, `with`; add a `with_in(model, from, to)` taking the model if
s1's `with` is bound to `MODEL`):

1. `an_instance_member_addresses_updates_moves_and_deletes`: `spec(ADDRESSED)` validates;
   `Relabel/relabelled`, `Hold/held`, `Drop/dropped` carry `subject.instance` `"target.parcel_id"`,
   `"request.parcel_id"`, `"target.parcel_id"`; each command's `unknown` branch is
   `OutcomeCondition::UnknownInstance`. Red on the base: the errors name `` `target.parcel_id`,
   which is no command input of it ``.
2. `an_each_list_member_is_walked`: `Label/labelled`'s affect has `each.list == "batch.lines"`.
   Red on the base: ``each:` walks a list of the command's input``.
3. `an_existing_instance_answers_a_creation_publishing_a_member`: `Register` validates with
   `taken` `OutcomeCondition::ExistingInstance`. Control, green at once on a copy of `ADDRESSED`
   holding only `Open` and `Register` at `format: ess/37` (and at `ess/38`): `created_identity`
   already returns the `ess/22` `InputField` path (`crates/specify/ess-domain/src/command/outcome_shapes.rs`
   160-167) and `validate_existing_pair` accepts it (391-440). It is kept because it pins that the
   existing path keeps working beside the new positions in one model (the whole `ADDRESSED`), and
   it is the parent's "`existing_instance:` branches answering it"; the `ess/37` copy pins that
   this already accepted form keeps its meaning there.
4. `at_ess_37_every_member_address_is_refused_naming_ess_38` (parent acceptance 7; the same text
   one format below): with `format: ess/38` replaced by `format: ess/37`, for `ADDRESSED` the
   errors contain
   `` `instance: target.parcel_id` names a member of a struct input, which requires specification format ess/38 ``
   (Relabel's branch, built by s0's `below_ess_38`) and
   `` `each: {in: input.batch.lines}` names a member of a struct input, which requires specification format ess/38 ``
   (Label's entry, `each_path_below_ess_38`), contain `requires specification format ess/38` at
   least once on a line naming each of `shop.parcel.Relabel`, `shop.parcel.Hold`,
   `shop.parcel.Drop` (the `instance:` refusal, one per branch whose `instance:` is a path: one
   each) and `shop.parcel.Label` (the `each:` refusal, one per entry), and contain `` write
   `format: ess/38` on the source that declares the system ``; for s1's `MODEL` at `ess/37` at
   least once on a line naming each of `Send`, `Check`, `Pair`, `Probe` (s1's code pushes one per
   branch: Send 1, Check 2, Pair 3, Probe 2), exactly as s1's test 4 words it; neither text
   contains `is not an input of`, `no command input of it` or `walks a list of the command's
   input`. Red on the base: the `ADDRESSED` text holds `` `target.parcel_id`, which is no command
   input of it `` and `` `each:` walks a list of the command's input `` and no `ess/38` line
   naming `Relabel` (the `MODEL` half is green from s1).
5. `a_member_the_struct_does_not_declare_is_refused_at_every_position` (acceptance 8): three
   edits, one at a time: `instance: target.parcel_id` of `Relabel` to `target.parcel`, `each: {in:
   input.batch.lines` to `input.batch.line`, and s1's `closed` via to `input.settings.primry`;
   each error contains `` is not a field of `shop.parcel.Target` `` / `` `shop.parcel.Batch` `` /
   `` `shop.route.Settings` `` and the member list (`parcel_id, label`; `lines, note`; `depots,
   primary, backup, extra, note`). Red on the base: the `instance:` edit prints `no command input
   of it`, the `each:` edit `walks a list of the command's input`; the `via:` edit is green from
   s1 (kept so the one statement covers every position).
6. `a_member_of_the_wrong_type_or_a_crossed_list_is_a_type_mismatch` (acceptance 9): `instance:
   target.label` (`which is typed \`String\``), `each: {in: input.batch.note` (the `List` check's
   message naming `String`), `instance: target.parcel_id.x` (`` reads past
   `input.target.parcel_id`, which is `shop.parcel.ParcelId` ``) and s1's `via:
   input.settings.depots.first` (naming `List<shop.route.DepotId>`). Red on the base: the two
   `instance:` edits print `no command input of it`, the `each:` edit `walks a list of the
   command's input`; the `via:` edit is green from s1.
7. `an_optional_route_reads_as_an_optional_reference` (acceptance 10): s1's `MODEL` validates
   with `Probe` (`via: input.settings.extra.spare` through `Optional<shop.route.Extra>`), and
   `ADDRESSED` with Hold's input `- {name: request, type: shop.parcel.Target}` replaced by `-
   {name: request, type: "Optional<shop.parcel.Target>"}` is refused with
   `` names the instance by `request.parcel_id`, which is typed `Optional<shop.parcel.ParcelId>` ``.
   Red on the base: `` `request.parcel_id`, which is no command input of it `` (the `MODEL` half
   is green from s1).
8. `a_guard_reading_the_member_that_names_the_instance_is_refused`: insert in `Relabel`, before
   `      - name: relabelled`, the branch `      - name: odd\n        when: "target.parcel_id !=
   target.parcel_id"\n        error: shop.parcel.UnknownParcel\n` (with `with`); the errors
   contain `an identity is opaque` (the text of `command.rs` 3586 or 3674). Red on the base: the text
   holds `no command input of it` and not `an identity is opaque` (the root `target` is not the
   instance name `target.parcel_id`).
10. `a_path_crossing_a_union_is_a_type_mismatch_at_instance_and_each` (N2): `with` adds the type
    `  - {name: shop.parcel.Kind, kind: union, tag: kind, variants: {parcel: shop.parcel.ParcelId,
    crate: shop.parcel.CrateId}}` after `shop.parcel.Batch` and the member `      - {name: kind,
    type: shop.parcel.Kind}` to `Target`; then, one at a time, `instance: target.parcel_id` of
    `Relabel` becomes `instance: target.kind.parcel`, and `Batch` gains `      - {name: choice,
    type: shop.parcel.Kind}` with `each: {in: input.batch.choice.lines`; each error contains
    `` reads past `input.target.kind`, which is `shop.parcel.Kind` and has no members `` /
    `` reads past `input.batch.choice`, which is `shop.parcel.Kind` `` (the `Opaque` text,
    `input_path.rs` 212-213 at `d3252eda9`). Rule: an `instance:` or `each:` route never crosses a union (s1
    D10). Red on the base: `no command input of it` / `walks a list of the command's input`.
11. `a_distinct_over_a_member_list_holds_it_distinct` (N3): `with(", repeated: last_wins}",
    "}")` on `Label`'s `each:` is refused with `` no `distinct:` holds that member distinct across
    the list `` naming `input.batch.lines` (the text of `set_effects.rs` 3321-3325); the same text
    with, inserted before `      - name: labelled`, the branch `      - name: duplicated\n        when:
    {not: {distinct: {in: batch.lines, as: d, by: d.parcel_id}}}\n        error:
    shop.parcel.Taken\n` validates, and so does the form `in: input.batch.lines`. Red on the base
    (after this unit's `each:` admission, before the `held_distinct` change): the `distinct:`
    variants are still refused with the `distinct:` text; today: `walks a list of the command's
    input`.

Appended to `crates/specify/ess-compiler/tests/related_guard_ir.rs` after s1's test:

9. `a_member_instance_and_a_member_each_list_carry_their_paths_in_the_ir`: compiled `ADDRESSED`:
   `Relabel/relabelled`'s `ResolvedInstance::Supplied { field }` has `field.name ==
   "target.parcel_id"` and `field.type_ref.written() == "shop.parcel.ParcelId"`; `Label`'s
   `ResolvedEach.list == "batch.lines"`. Red on the base: compile panics with the validation errors.

## Steps

1. Start from the wave branch with s0 and s1 merged; `git rev-parse --short HEAD` in the report.
   Control first, as s1's Step 1: the model with `format: ess/37`, each member read replaced by a
   top-level input (`parcel_id` for `target.parcel_id` and `request.parcel_id`, `lines` for
   `input.batch.lines`) validates on the base; correct only declarations that do not address a
   row if it does not, carry the correction into the fixture and say so in the report.
2. Fixture and tests 1-11 (and test 9 in the compiler file); record each red line, and for test 3
   that it is green at once.
3. `input_path.rs` (`validate_instances`), `spec.rs`, `entity.rs`, `command.rs`,
   `set_effects.rs`; then the compiler.
4. Run test 3 on the whole `ADDRESSED`; if `Register` is refused there, read `outcome_shapes.rs`
   173 and 391 and route the lookup that refuses it through `address`, and name it in the report.
5. `FORMAT_HISTORY` row 38 second sentence; `slot cargo xtask format-history`. Documents.
6. `slot cargo xtask schema` (two `RawSpecFile` doc comments), then `slot task projection-check`.
7. Re-stamp the synthesis sources with the command s1's Step 7 names and run
   `slot --timeout 30m task test-slow PACKAGES=ess-conformance`; no table row may move. s3 lands in
   the same round and re-stamps too: the second of s2 and s3 to land re-runs the stamp command on
   the merged tree.
8. `CHANGELOG.md`; `slot task fmt-check`.

## Owned files

- `crates/specify/ess-domain/src/command/input_path.rs` (`validate_instances` only; s1's
  functions are not changed)
- `crates/specify/ess-domain/src/spec.rs` (one call beside 589)
- `crates/specify/ess-domain/src/entity.rs` (`validate_instance`, the one early return)
- `crates/specify/ess-domain/src/command.rs` (the `instance` doc and the two invariant-13
  conditions)
- `crates/specify/ess-domain/src/command/set_effects.rs` (the functions named above)
- `crates/specify/ess-domain/src/command/outcome_shapes.rs` (only if Step 4 needs it)
- `crates/specify/ess-domain/src/system.rs` (row 38, second sentence)
- `crates/specify/ess-compiler/src/resolve.rs` (`instance`), `crates/specify/ess-compiler/src/ir.rs`
  (two docs)
- `crates/specify/ess-domain/tests/input_path_addressing.rs` (appended section),
  `crates/specify/ess-compiler/tests/related_guard_ir.rs` (appended test)
- `crates/verify/ess-conformance/tests/fixtures/struct-input-member-addressed-rows.yaml.in` (new)
- `docs/design/expression-family-source22.md`, `website/docs/reference/predicates.md`,
  `docs/design/language-grids.md`, `website/docs/reference/formats.md` (the `ess/38` row, one
  appended sentence)
- `schemas/generated/ess.schema.json`, `website/docs/reference/spec-versions.md` (regenerated)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (re-stamped)
- `CHANGELOG.md` (one bullet)

## Not yours

- s0's `addresses_members`, `below_ess_38`, `FormatVersion::V38`, `SUPPORTED_FORMATS` and s0's
  test; row 37 of `FORMAT_HISTORY` and `formats.md` row 203 (released, never edited).
- s1's functions in `input_path.rs` (`address`, `reads_member`), `related_guard.rs`,
  `related_list.rs`, `fixture_inputs.rs`, `value_expression.rs`, the compiler's
  `related_guard`/`related_read`/`MissingElement` arm, and s1's tests above your section.
- Everything under `crates/verify/ess-conformance/src/` and `crates/generate/` (s3 to s6).
- `crates/edge/ess-xtask/`, the `ValidationCode` list, `macro-guards.json`,
  `docs/design/review-typed-diagnostics.md` (no new code; no site added to a top-level file).
- `website/docs/guides/specify/selection-effects.md`: not changed (the parent names two pages; the
  reference pages carry the rule).

Shared files. With s0 (round 0) and s1 (round 1), both landed; this unit appends after their
lines, in other functions, and changes nothing of theirs: `input_path.rs`, `system.rs` row 38,
`spec.rs`, `resolve.rs`, `ir.rs`, `input_path_addressing.rs`, `related_guard_ir.rs`, `formats.md`
row `ess/38`, and the two generated files `ess.schema.json` and `spec-versions.md` (regenerated by
command, never merged by hand). With s3 (same round 2): only `synthesis-sources.digest` (the
second to land re-runs the stamp command on the merged tree) and `CHANGELOG.md` (both append
under `### Added` after s1's bullet, the second to land after the first's). Compared by Owned files and
every path in Steps: s3 owns only files under `crates/verify/ess-conformance/src/` and
`crates/verify/ess-conformance/tests/input_path_addressing.rs`, none of which this unit edits; no
other file is shared.

## Gate

As s1's Gate (same crates, same pins, same host limits: `CARGO_PROFILE_DEV_DEBUG=false
CARGO_INCREMENTAL=0` on every cargo command, named `--test` targets one per command, 10-minute
pool limit unless `SLOT_TIMEOUT=30m` / `slot --timeout 30m` names up to 30, `slot cargo xtask`, a
test over 60 s killed, a step within 120 s, no Python, nothing in `/tmp`), plus the set-effects
and instance targets:

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-domain -p ess-compiler --all-targets --locked -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test input_path_addressing
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test each_repeated
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test related_guard_unknown_instance
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test set_effects
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test set_effects_per_row_events
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test set_effect_identity_filter
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test expression_distinct
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --lib
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test related_guard_ir
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test selection_precedence_table
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test typed_diagnostics
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test string_alphabet_and_length
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table_cli_read
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test adversary_related_values_all_bytes
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test external_beside_held_guard
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-cli --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test tutorial_page
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test validate_parse_refusals
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test validate_undeclared_fields
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test authored_beside_existing_instance
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test specify_formats
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test epic_formats_open
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test adversary_t1_epic_formats
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test format_history
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-openapi --locked --test adversary_ess_issues_2_u6
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test served_codec_names
slot --timeout 30m task test-slow PACKAGES=ess-conformance
slot task projection-check
slot --timeout 30m task test-xtask
slot --timeout 30m task ci-lint
```

The `ess-cli` targets named are those whose sources pin validation output of `instance:`,
`each:`, an undeclared field or the format rows (`grep -ln "instance:\|each:\|ess/37"
crates/edge/ess-cli/tests/*.rs`, narrowed to validation and format tests, plus s0's two targets
that pin `ess/37` reading as before); the other `ess-cli`
targets are left to the forge's shards, which run them all. `go` as in s1 (run if `go` is on
`PATH`, else named not run).

## Acceptance

1. `cargo test -p ess-domain --locked --test input_path_addressing` passes tests 1-8, 10 and 11
   and s0's and s1's tests unchanged (parent acceptance 4, 5, 7, 8, 9, 10); test 4 shows the same
   text refused at `ess/37` with the exact `ess/38` messages.
2. `cargo test -p ess-compiler --locked --test related_guard_ir` passes test 9.
3. `ess specify validate` on the fixture copied to a scratch `.yaml` exits 0 (output in the
   report).
4. `selection_precedence_table` passes with no table edit; the stamp test passes after the
   re-stamp; `task test-slow PACKAGES=ess-conformance` passes with no table edited.
5. `slot task projection-check` and `slot --timeout 30m task test-xtask` pass (the documents and
   the regenerated pages; parent acceptance 12).
6. Every target of the Gate passes; `git diff --stat` shows only the owned files.

## Decisions

- D1. The instance path rule lives in `input_path::validate_instances`, not in
  `entity::validate_instance`, because `validate_lifecycle_causes_after` has no type registry and
  `entity.rs` is a census file; `validate_instance` steps aside for a path.
- D2. The instance path is carried in `ResolvedInstance::Supplied { field }` with `field.name` the
  path and the root's `naming`, exactly as `input_read` builds an `ess/22` path read; no IR key.
- D3. An Optional route under `instance:` is refused as an Optional top-level input is
  (`TypeMismatch`, today's message); `input_path::optional_route` is not used, so the text a
  writer sees is the one they know.
- D4. Invariant 13 extends to the member: a guard reading the member that names the instance (or
  below it) is refused like one reading the top-level instance input; only the conditions change,
  so `command.rs`'s census count stays.
- D5. Row 38's second `added` sentence, after s1's: "`instance:` on an `updates:`, `moves:` or `deletes:`
  branch, and the `unknown_instance:` and `existing_instance:` branches answering it, may name a
  member of a struct input, `instance: target.parcel_id`, and an `affects:` entry may walk a
  `List` member, `each: {in: input.batch.lines, as: line}`: a member the struct does not declare
  is refused as `undeclared_reference`, a member of the wrong type and a path crossing a list, a
  map or a union as `type_mismatch`, and an `instance:` reached through an `Optional` as an
  `Optional` input is; a `distinct:` over the member list holds it distinct. Below `ess/38` each
  is refused as `unsupported_format_version` naming `ess/38`; a model without such a path keeps
  its IR bytes (beyond10x/ess#640)."
- D6. `each:` below `ess/38` is gated in `refuse_each` on the raw document, where every other
  `each:` format gate is, so the parse-time `each_entry` can admit the path at every format; the
  gate reads s0's `addresses_members`, and its refusal has the sentence of s0's `below_ess_38`
  with the string location every `each:` refusal of `set_effects.rs` has.
- D10. Every position this unit opens is admitted only at `ess/38` (D48); at `ess/37` and below
  each stays a refusal (now `unsupported_format_version` in place of ESS-COMMAND-001/009), so no
  accepted document changes meaning and nothing is breaking.
- D7. No new `ValidationCode`, diagnostic code, keyword or dependency.
- D8. An `instance:` or `each: {in:}` route never crosses a union: s1's `resolve` refuses it as
  `Opaque`, `type_mismatch` naming the union (test 10, N2), as for a list or a map.
- D9. A `distinct:` over a member list holds it distinct (test 11, N3); `reads_member` is not used
  there because `held_distinct` needs equality with the list, not a read at or below it.

## Open for the operator

None (the format is `ess/38`, the operator's decision D48).

## Changelog

Under `## [Unreleased]`, `### Added`, after s1's "Format `ess/38`: …" bullet, at the end of that heading (s3, in the same round, also appends under `### Added`: the second to land appends after the first's):

- Format `ess/38`: `instance:` on `updates:`, `moves:` and `deletes:`, with the `unknown_instance:` and `existing_instance:` branches answering it, may name a member of a struct input, `instance: target.parcel_id`, and `affects: [{each: {in: input.batch.lines, as: line}}]` may walk a `List` member. Below `ess/38` each is refused as `unsupported_format_version` naming `ess/38`; a model without such a path keeps its IR bytes (beyond10x/ess#640).

## The model

`crates/verify/ess-conformance/tests/fixtures/struct-input-member-addressed-rows.yaml.in`, byte for
byte (Step 1 may correct a declaration that does not address a row, and says so):

```yaml
format: ess/38
system: shop
version: v1
domain: shop.parcel
summary: A request names the parcel it acts on inside a struct, and a labelling names the parcels it labels in a list inside its batch.
types:
  - {name: shop.parcel.ParcelId, kind: newtype, of: Uuid}
  - {name: shop.parcel.CrateId, kind: newtype, of: Uuid}
  - name: shop.parcel.Target
    kind: struct
    fields:
      - {name: parcel_id, type: shop.parcel.ParcelId}
      - {name: label, type: String}
  - name: shop.parcel.Line
    kind: struct
    fields:
      - {name: parcel_id, type: shop.parcel.ParcelId}
      - {name: label, type: String}
  - name: shop.parcel.Batch
    kind: struct
    fields:
      - {name: lines, type: "List<shop.parcel.Line>"}
      - {name: note, type: String}
entities:
  - name: shop.parcel.Parcel
    identity: {name: parcel_id, type: shop.parcel.ParcelId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Open
      states: [Open, Held]
      terminal: [Held]
      transitions:
        - {name: hold, from: [Open], to: Held}
  - name: shop.parcel.Crate
    identity: {name: crate_id, type: shop.parcel.CrateId}
    fields:
      - {name: note, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: shop.parcel.UnknownParcel, summary: No parcel carries this identity., fields: []}
  - {name: shop.parcel.UnknownCrate, summary: No crate carries this identity., fields: []}
  - {name: shop.parcel.NotOpen, summary: The parcel is not open., fields: []}
  - {name: shop.parcel.Taken, summary: A parcel already carries this identity., fields: []}
events:
  - name: shop.parcel.ParcelOpened
    fields: [{name: parcel_id, type: shop.parcel.ParcelId}]
  - name: shop.parcel.ParcelRelabelled
    fields: [{name: parcel_id, type: shop.parcel.ParcelId}]
  - name: shop.parcel.ParcelHeld
    fields: [{name: parcel_id, type: shop.parcel.ParcelId}]
  - name: shop.parcel.ParcelDropped
    fields: [{name: parcel_id, type: shop.parcel.ParcelId}]
  - name: shop.parcel.CrateOpened
    fields: [{name: crate_id, type: shop.parcel.CrateId}]
  - name: shop.parcel.CrateLabelled
    fields: [{name: crate_id, type: shop.parcel.CrateId}]
actors:
  - name: shop.parcel.Clerk
    may: [shop.parcel.Open, shop.parcel.Relabel, shop.parcel.Hold, shop.parcel.Drop, shop.parcel.Register, shop.parcel.OpenCrate, shop.parcel.Label]
commands:
  - name: shop.parcel.Open
    input:
      - {name: label, type: String}
    outcomes:
      - name: opened
        creates: shop.parcel.Parcel
        instance: parcel_id
        emits: [shop.parcel.ParcelOpened]
        payload: {shop.parcel.ParcelOpened: {parcel_id: {generated: true}}}
        sets: {label: input.label}
  - name: shop.parcel.Relabel
    input:
      - {name: target, type: shop.parcel.Target}
    outcomes:
      - name: unknown
        unknown_instance: true
        error: shop.parcel.UnknownParcel
      - name: relabelled
        updates: shop.parcel.Parcel
        instance: target.parcel_id
        emits: [shop.parcel.ParcelRelabelled]
        payload: {shop.parcel.ParcelRelabelled: {parcel_id: input.target.parcel_id}}
        sets: {label: input.target.label}
  - name: shop.parcel.Hold
    input:
      - {name: request, type: shop.parcel.Target}
    outcomes:
      - name: unknown
        unknown_instance: true
        error: shop.parcel.UnknownParcel
      - name: held
        moves: shop.parcel.Parcel.hold
        instance: request.parcel_id
        emits: [shop.parcel.ParcelHeld]
        payload: {shop.parcel.ParcelHeld: {parcel_id: input.request.parcel_id}}
      - name: not-open
        wrong_state: true
        error: shop.parcel.NotOpen
  - name: shop.parcel.Drop
    input:
      - {name: target, type: shop.parcel.Target}
    outcomes:
      - name: unknown
        unknown_instance: true
        error: shop.parcel.UnknownParcel
      - name: dropped
        deletes: shop.parcel.Parcel
        instance: target.parcel_id
        emits: [shop.parcel.ParcelDropped]
        payload: {shop.parcel.ParcelDropped: {parcel_id: input.target.parcel_id}}
  - name: shop.parcel.Register
    input:
      - {name: target, type: shop.parcel.Target}
    outcomes:
      - name: taken
        existing_instance: true
        error: shop.parcel.Taken
      - name: registered
        creates: shop.parcel.Parcel
        instance: parcel_id
        emits: [shop.parcel.ParcelOpened]
        payload: {shop.parcel.ParcelOpened: {parcel_id: input.target.parcel_id}}
        sets: {label: input.target.label}
  - name: shop.parcel.OpenCrate
    input:
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: shop.parcel.Crate
        instance: crate_id
        emits: [shop.parcel.CrateOpened]
        payload: {shop.parcel.CrateOpened: {crate_id: {generated: true}}}
        sets: {note: input.note}
  - name: shop.parcel.Label
    input:
      - {name: crate_id, type: shop.parcel.CrateId}
      - {name: batch, type: shop.parcel.Batch}
    outcomes:
      - name: no-crate
        unknown_instance: true
        error: shop.parcel.UnknownCrate
      - name: labelled
        updates: shop.parcel.Crate
        instance: crate_id
        emits: [shop.parcel.CrateLabelled]
        payload: {shop.parcel.CrateLabelled: {crate_id: input.crate_id}}
        sets: {note: input.batch.note}
        affects:
          - entity: shop.parcel.Parcel
            each: {in: input.batch.lines, as: line, repeated: last_wins}
            instance: line.parcel_id
            sets: {label: line.label}
views:
  - name: shop.parcel.Parcels
    source: shop.parcel.Parcel
    consistency: read_your_writes
    fields:
      - {name: parcel_id, type: shop.parcel.ParcelId}
      - {name: label, type: String}
  - name: shop.parcel.Crates
    source: shop.parcel.Crate
    consistency: read_your_writes
    fields:
      - {name: crate_id, type: shop.parcel.CrateId}
      - {name: note, type: String}
```
