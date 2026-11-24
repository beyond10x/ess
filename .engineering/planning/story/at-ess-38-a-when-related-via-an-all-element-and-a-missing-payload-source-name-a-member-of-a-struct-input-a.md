---
format: aep.planning-md/3
id: story:at-ess-38-a-when-related-via-an-all-element-and-a-missing-payload-source-name-a-member-of-a-struct-input-a
kind: story
status: active
title: 'At ess/38 a when_related via, an all: element and a {missing:} payload source name a member of a struct input, a List of references included, and ess/37 keeps its reading of a struct value written {missing: input.a.b}'
tags:
- wave-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:via-and-instance-may-name-a-member-of-a-struct-input
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
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
  path: crates/specify/ess-domain/src/command/fixture_inputs.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/input_path.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_list.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/input_path_addressing.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/struct-input-member-related-rows.yaml.in
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest
- confidence: cited
  path: schemas/generated/ess.schema.json
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 20
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T21:48:22Z", actor: "human:timo", revision: 19}
- {from: "proposed", to: "active", at: "2026-11-23T21:48:24Z", actor: "human:timo", revision: 20}
---
## Outcome

- A specification at `format: ess/38` (opened by s0) validates and compiles where a
  `when_related:` guard reads a row through a member of a struct input: `via:
  input.settings.primary` with `exists: false` and with `predicate:`, an `all:` element `{via:
  input.settings.primary, predicate: …}`, and a list member `via: input.settings.depots, exists:
  false` with the per-element meaning of `ess/28`, its branch publishing `{missing:
  input.settings.depots}`.
- Below `ess/38` (so at `ess/37`, released in 0.93.0, too) each of these is refused as
  `unsupported_format_version` naming `ess/38`, through s0's `input_path::below_ess_38`, where today
  the guard gets ESS-COMMAND-001 (`` `settings.depots` is not an input of … ``): a refusal stays a
  refusal, and no accepted document changes meaning.
- A path to a member the struct does not declare is refused as `undeclared_reference` naming the
  struct's members; a member that is no entity identity (or `List<…>` of one), and a path that
  crosses a list, a map or a union, are refused as `type_mismatch` naming the type.
- The IR carries the path as the dotted `field` of `ResolvedRelatedVia::Input` and of
  `ResolvedPayloadValue::MissingElement`, as `ResolvedPayloadValue::InputField` carries an `ess/22`
  path; every model that names no path keeps its IR bytes.
- A `fixture_inputs:` binding over a struct input whose member a `via:` reads is refused as one
  over a top-level `via:` input is.
- `input_path::reads_member` (whether a read reaches a member path) exists for s2 and s3.
- A struct value written `{missing: input.a.b}` (a member named `missing` set from an input path)
  keeps its nested-struct reading at `ess/22` to `ess/37`, byte for byte as before this wave; the
  same text at `ess/38` is the missing-element source (D4). Nothing is breaking.

## Why

- Parent `story:via-and-instance-may-name-a-member-of-a-struct-input` (serves O2): a command whose
  reference sits inside a struct input cannot name it where a row is addressed; flattening the
  input changes a wire shape a published protocol fixes. The operator's decision D48 (2026-11-23):
  the feature opens `ess/38`; `ess/37` keeps its meaning.
- s0 opened `ess/38` and its gate (`addresses_members`, `below_ess_38`); this unit opens the
  related-row positions (`via:`, `all:` elements, `{missing:}`) and the shared path helpers; s2
  opens `instance:` and `each: {in:}` and the documents; s3 to s6 carry synthesis, the interpreted
  target and the generated targets.

## Where

Read in the repository root (paths below are relative to it) at commit `d3252eda9`
(branch `integration/wave-2026-11-23`; re-read at the same commit for D48). This unit is built on
the wave branch with s0 merged, which adds lines to `input_path.rs` and `system.rs`: re-read each
site by name at the commit you start from; numbers move, names hold.

From s0, used here and not changed: `FormatVersion::V38` (`system.rs`), FORMAT_HISTORY row 38 (a
placeholder sentence this unit replaces), `input_path::addresses_members(format:
Option<FormatVersion>) -> bool` (`true` from `ess/38`, `None` admits) and
`input_path::below_ess_38(at: &ConstructRef, what: &str) -> ValidationError` (code
`UnsupportedFormatVersion`, message `` {what} names a member of a struct input, which requires
specification format ess/38 ``, hint `` write `format: ess/38` on the source that declares the
system ``), and the test file `crates/specify/ess-domain/tests/input_path_addressing.rs` with
s0's one test.

`crates/specify/ess-domain/src/command/input_path.rs` (268 lines at `d3252eda9`; the `ess/22` path
rule; a public module, `command.rs` 213 `pub mod input_path;`):

- `is_path` (20), `admitted` (25, `>= V22`), `InputPath` (32-61, `as_field` at 57), `Unresolved`
  (66-97), `resolve` (100-144), `Unresolved::refusal` (148) and `parts` (159-218: `Member` gives
  `UndeclaredReference` with the hint `` `<struct>` has: a, b `` at 188; `Opaque` gives
  `TypeMismatch` `` reads past `input.<at>`, which is `<type>` and has no members `` at 206-216),
  `below_ess_22` (221), `optional_route` (233). `resolve` refuses a step into any type
  `types.struct_fields` does not answer (114, `Opaque` at 122), so a union, a list and a map are
  all `Opaque`: that is the union rule (N2), with no new code. Add, after s0's `addresses_members`:
  - `pub fn address(command: &CommandSpec, types: &TypeRegistry, field: &str) -> Option<Field>`:
    the field an addressing position reads. Not a path: `command.input_field(field).cloned()`. A
    path where `addresses_members(types.format())`: `resolve(command, types, field).ok().map(|path|
    path.as_field())` (named by the path, at `InputPath::type_ref`, so a route through an
    `Optional` reads as `Optional<…>`). A path below `ess/38`: `None`.
  - `pub fn reads_member(read: &[&str], path: &str) -> bool` (X1; used by s2's invariant-13
    conditions and `held_distinct`, and by s3's guard-read checks in `ess-conformance`): drop a
    leading `input` segment of `read`, join the rest with `.`, and answer whether it equals `path`
    or starts with `path` followed by `.`. `reads_member(&["input", "settings", "primary"],
    "settings")` and `(…, "settings.primary")` are true; `(…, "settings.prim")` is false.
  - The refusal below the format is s0's `below_ess_38`; `what` is the written position, e.g.
    `` `via: input.settings.depots` ``. It is typed and lives in this sub-directory file, so the
    census of `docs/design/review-typed-diagnostics.md` (top-level files of `ess-domain/src` only)
    does not move.
  - Update the module doc (lines 1-11) with one paragraph: from `ess/38` the addressing positions
    read a path too (`via:`, `all:` elements, `{missing:}` here; `instance:` and `each: {in:}` by
    s2).

`crates/specify/ess-domain/src/system.rs`: `FORMAT_HISTORY` row `major: 38` (written by s0 after
row 37, which ends at 398 at `d3252eda9`): replace s0's placeholder sentence ("Opened for
addressing a row through a member of a struct input …") with the sentence under Decisions D7.
`release` stays `None`; `stricter` stays empty. Row 37 (391-398) is not touched: it is released.

`website/docs/reference/formats.md`, the row `Authored specification: format: ess/38` (s0 added it
after line 203; row 203, `ess/37`, is not touched): keep "Unreleased." and replace the rest with
the first sentence of D7 up to "the `input.<path>` `sets:` reads", then "Below it each is refused
as `unsupported_format_version` naming `ess/38`, and `{missing: input.a.b}` keeps its reading as a
struct value." (`cargo xtask docs`, run by `task projection-check`, reads the release claims of
this page.)

`crates/specify/ess-domain/src/command/related_guard.rs` (2561 lines):

- `RawRelatedGuard::via` doc (108-111): "written `input.<field>`" becomes "written
  `input.<field>`, or from `ess/38` `input.<path>` to a member of a struct input". This is a
  `RawSpecFile` doc comment: the schema moves (Steps 7).
- `RawRelatedGuard::read` (213-353): the via parse at 307-317 already keeps `input.a.b` whole as
  `RelatedVia::Input("a.b")`; unchanged.
- `related_entity` (654-680): `command.input_field(via)` (659) becomes
  `super::input_path::address(command, &spec.system().types, via)` (an owned `Field`; keep the
  rest, `identity_type(declared)` borrows the local).
- `via_is_optional` (693-706): the `Input` arm uses `address(..)` and `is_optional()` of its
  `type_ref` (a route through an `Optional` reads as `Optional<…>`: parent acceptance 10,
  "checked only when present").
- `carried` (1247-1269): `command.input_field(field)?` (1254) becomes `address(..)?`; the returned
  `&'a TypeRef` cannot borrow a local: return `TypeRef` owned (change the tuple's third element to
  `TypeRef` and its one caller, found with `grep -n 'carried(' related_guard.rs`).
- `gates` (1034-1058): first, before `optional_below_ess_22` (1036), a new
  `fn path_below_ess_38(spec, command) -> ValidationErrors`: where
  `!input_path::addresses_members(Some(spec.system().format))`, one s0 `below_ess_38` per branch whose
  `via` or `also[i].via` is `RelatedVia::Input(path)` with `is_path(path)` (one per branch, even
  where both rows of an `all:` name a path), at `site(command, outcome)` (731), `what` =
  `` `via: input.<path>` ``; non-empty means return it (as `optional_below_ess_22` returns).
  `gates` runs before `related_list::gates` (1340) and before `entity_or_refusal`, so no
  ESS-COMMAND-001 follows the refusal.
- `entity_or_refusal` (1660-1750): where `via` is a path and the format admits it, resolve with
  `input_path::resolve(command, &spec.system().types, via)`; on `Err(unresolved)` push
  `unresolved.refusal(&at, command, via)` and return `None` (`Member` → `undeclared_reference`
  naming the members; `Opaque` → `type_mismatch` naming the list, map or union crossed; `Root` →
  the existing "not an input" refusal 1673-1685). On `Ok(path)` continue with `path.as_field()` in
  place of `read` (1672). The existing checks 1688-1749 then refuse a member that is no entity
  identity as `type_mismatch` with its type (1693-1712 and 1714-1724). One-segment vias take
  exactly today's path through 1672.

`crates/specify/ess-domain/src/command/related_list.rs` (322 lines):

- `is_list` (37-44) and `uses` (47-52): take `types: &TypeRegistry` first and read the list
  through `input_path::address(command, types, field)`. Callers: `gates` (57, pass
  `&spec.system().types`), lines 50, 65, 78 in this file. No caller outside `ess-domain`
  (`ess-conformance` has its own `related_list` module; verify with
  `grep -rn 'command::related_list::' crates/`).
- `gates` (57-155): `command.input_field(list.field())` (83) becomes `address(..)`.
- `RawMissingSource::recognise` (176-190): today it admits `input.<one segment>` only
  (`is_field_name` at 182); admit `input.<seg>(.<seg>)*`, every segment
  `crate::binding::is_field_name`. `into_source` (193) and `of` (203) keep the path whole in
  `PayloadSource::MissingElement { field }`.
- New `pub fn read_paths_below_ess_38(format, commands)` beside `read_below_ess_28` (211-238;
  no `read_paths_below_ess_37` exists in the tree): below `ess/38`, so at `ess/22` to `ess/37`, a
  `MissingElement` whose `field` is a path is read back as the nested mapping it was
  (`nested_missing`'s rule, 240-266, applied only to paths). Called from
  `crates/specify/ess-domain/src/spec.rs` beside line 462 (`read_below_ess_28`).
- `check` (268-322): take `types: &TypeRegistry`; `command.input_field(field)` (296) becomes
  `address(..)`. Caller `crates/specify/ess-domain/src/command/value_expression.rs:529` passes
  `&context.spec.system().types`. Its two refusal texts (282-288, 300-308) are unchanged; test 15
  quotes the second.

What a `{missing: input.a.b}` mapping is today (read for D4): the payload-source reader tries
`RawMissingSource::recognise` (`command.rs` 1807) only after every other source; with a dotted
read it answers `None` (`related_list.rs` 178-182), `missing` is not in `SOURCE_KEYWORDS`
(`command.rs` 1664-1672), so the mapping is kept as `RawPayloadSource::Nested` (`command.rs`
1822): a struct value with a member `missing` read from the `ess/22` path `input.a.b`, which a
struct type with a member `missing` accepts. Every format from `ess/22` to `ess/37` keeps that
reading through `read_paths_below_ess_38`; only a document declaring `ess/38`, which no released
`ess` accepts, reads the text as the missing-element source (D4).

`crates/specify/ess-domain/src/command/fixture_inputs.rs` (N1): `read_by_a_guard` (115-140)
compares a `via` with the fixture's top-level input by equality (`via_reads`, 122). A fixture over
`settings` replaces the identity `via: input.settings.primary` reads, which is the witness the
refusal at 161-170 protects. `via_reads` becomes `RelatedVia::Input(read) if read == field ||
read.starts_with(&format!("{field}."))` (a path is only admitted at `ess/38`, so no document at
`ess/37` or earlier changes); the `all:` element arm uses the same closure. Typed site unchanged (no new
`ValidationError`); this sub-directory file is outside the census.

`crates/specify/ess-compiler/src/resolve.rs` (7157 lines):

- `related_guard` (3611) and `related_read` (3637): take `&mut self` (their caller `outcomes`,
  2324, holds `&mut self`). In `related_read` the `Input` arm (3650-3656) reads
  `self.input_read(command, input?, field)?` (3012-3038) in place of
  `input?.iter().find(|held| held.name == *field)?`: a path becomes `ResolvedRelatedVia::Input {
  field: "settings.primary", type_ref: <terminal> }`, a top-level field is unchanged.
- `PayloadSource::MissingElement` (3391-3420): the element is read through
  `self.input_read(command, fields, field)` in place of the top-level `find` (3392-3397).
- `related_field` (3680-3720, the `{related: {via, field}}` value source) is NOT changed: that
  source is not an addressing position of the parent.

`crates/specify/ess-compiler/src/ir.rs`: docs only. `ResolvedRelatedVia::Input::field` (1849-1851)
and `ResolvedPayloadValue::MissingElement`'s field doc gain "or the declared segments of a path
joined by `.` (ess/38)", as `InputField` (1213) says for `ess/22`.

New file `crates/verify/ess-conformance/tests/fixtures/struct-input-member-related-rows.yaml.in`:
the model under "The model" below, byte for byte. The `.yaml.in` extension keeps it out of the
corpus walks (`selection_precedence_table.rs::walk`, 70-87, and the synthesis-sources digest take
`.yaml`/`.yml` only), so no table row and no model count moves in this unit (D3).

## Tests first

Every new test is timed by the implementor in a debug build; each is expected under 1 s (one small
model, validate or compile only); none may run over 60 s, and the test step of the touched crates
stays within 120 s.

The base of this unit is the wave branch with s0 merged: `ess/38` is supported and reads exactly
as `ess/37`, and nothing admits a member address yet. "Red on the base" below is what each test
prints there; at `d3252eda9` without s0 the `ess/38` tests would instead print
`unsupported_format_version` for the format itself.

`crates/specify/ess-domain/tests/input_path_addressing.rs` (created by s0 with one test, X7:
`ess-domain` has no `tests/main.rs`, each of its 203 test files is a binary; no other file is
about addressing through an input path): append this unit's section after s0's test. Helpers,
copied in shape from `crates/specify/ess-domain/tests/related_list.rs` (11-27): `const MODEL: &str
= include_str!("../../../verify/ess-conformance/tests/fixtures/struct-input-member-related-rows.yaml.in");`,
`spec(body)`, `refused(body) -> String`, `with(from, to)` (asserts `from` is present, replaces
once).

1. `a_list_member_names_a_row_per_element_at_ess_38`: `spec(MODEL)` validates; `shop.route.Send`'s
   `unknown-depot` is `OutcomeCondition::Related { via, test: Absent, .. }` with `via.field() ==
   "settings.depots"`, and its error payload `depot_id` is `PayloadSource::MissingElement { field:
   "settings.depots" }`. Red on the base: panics with the validation errors, among them
   `` `settings.depots` is not an input of `shop.route.Send` ``.
2. `a_scalar_member_names_one_row_with_exists_false_and_with_a_predicate`: `shop.route.Check`'s
   `no-depot` (`Absent`) and `closed` (`Holds`) read `settings.primary`. Red on the base: same
   panic, naming `settings.primary`.
3. `an_all_element_names_a_member`: `shop.route.Pair`'s `both-closed` holds two rows,
   `settings.primary` and `settings.backup` (`via` and `also[0].via`). Red on the base: same panic.
4. `at_ess_37_each_member_address_is_refused_naming_ess_38` (the same text one format below):
   `with("format: ess/38", "format: ess/37")`; the error text contains
   `` `via: input.settings.depots` names a member of a struct input, which requires specification format ess/38 ``
   (Send's one branch, the message s0's `below_ess_38` builds), contains `requires specification
   format ess/38` at least once on a line naming each of `shop.route.Send`, `shop.route.Check`,
   `shop.route.Pair`, `shop.route.Probe` (the code pushes one per branch: Send 1, Check 2, Pair 3,
   Probe 2), contains `` write `format: ess/38` on the source that declares the system `` and
   contains no `is not an input of`. Red on the base: the text holds `is not an input of`, not
   `ess/38`.
5. `a_member_the_struct_does_not_declare_is_refused_naming_its_members`: `with("via:
   input.settings.primary, predicate: open == false}", "via: input.settings.primry, predicate:
   open == false}")` (the `closed` branch of `Check`); the error contains `` `primry` is not a
   field of `shop.route.Settings` `` and `depots, primary, backup, extra, note`. Red on the base:
   `is not an input of`.
6. `a_member_that_is_no_identity_is_a_type_mismatch_naming_its_type`: replace the `closed`
   branch's via with `input.settings.note`; the error contains `` `input.settings.note` is
   `String` `` and `no entity's identity`. Red on the base: `is not an input of`.
7. `a_path_crossing_a_list_is_a_type_mismatch_naming_the_list`: replace `Send`'s
   `via: input.settings.depots` with `via: input.settings.depots.first`; the error contains
   `` reads past `input.settings.depots`, which is `List<shop.route.DepotId>` ``. Red on the base:
   `is not an input of`.
8. `a_route_through_an_optional_member_validates_as_an_optional_reference`: `shop.route.Probe`
   (its `via: input.settings.extra.spare` crosses the `Optional` member `extra`) validates with no
   `input_absent:` or other branch for the absent reference, as a top-level `Optional<…>` via does
   from `ess/22`; and with `with("      - {name: extra, type: \"Optional<shop.route.Extra>\"}",
   "      - {name: extra, type: shop.route.Extra}")` it still validates (the required route).
   Red on the base: `is not an input of` naming `settings.extra.spare`.
9. `at_ess_37_a_path_missing_source_is_the_nested_mapping_it_was`: at `format: ess/37` (the newest
   format that must not change), parse `MODEL` and read (through `RawSpecFile::parse` and
   `Specification::assemble`'s error text, as
   `related_list.rs::below_ess_28_the_missing_source_is_the_nested_mapping_it_was`, 100-109, does
   for `ess/27`) that `{missing: input.settings.depots}` is not read as a `MissingElement`. Read
   that test first and mirror its assertion. Control, green at once: it pins that widening
   `recognise` keeps `ess/37` and every format below it as they were (`read_paths_below_ess_38`);
   it must stay green after the change. No `ess/36` copy: it would pin the same thing.
12. `reads_member_matches_a_read_at_or_below_the_member` (X1): calls
    `ess_domain::command::input_path::reads_member` with the three cases under Where, plus
    `(&["settings", "primary"], "settings.primary")` true (no `input` segment) and
    `(&["input", "settings"], "settings.primary")` false (a read above the member). Red on the
    base: does not compile (`reads_member` is not defined); say so in the report.
13. `a_path_crossing_a_union_is_a_type_mismatch_naming_the_union` (N2): `with` adds after the
    `Settings` struct the type `  - {name: shop.route.Choice, kind: union, tag: kind, variants:
    {depot: shop.route.DepotId, shipment: shop.route.ShipmentId}}` (the shape of
    `examples/billing/domains/invoice.yaml` 44-49, written in flow form; correct the form if the
    parser wants the block form and say so), adds the member `      - {name: choice, type:
    shop.route.Choice}` to `Settings`, and replaces the `closed` branch's via with
    `input.settings.choice.depot`; the error contains
    `` `input.settings.choice.depot` reads past `input.settings.choice`, which is `shop.route.Choice` and has no members ``
    (`input_path.rs` 212-213 at `d3252eda9`). Red on the base: `is not an input of`. Rule: a
    `via:` route never crosses a union; name the union whole or a struct member.
14. `a_fixture_over_a_struct_input_a_via_reads_is_refused` (N1): `with("  - name:
    shop.route.Check\n", "  - name: shop.route.Check\n    fixture_inputs: {settings:
    provided-value}\n")` (the binding form of `fixture_inputs_guard_reads.rs` 198-206); the error
    names `command.shop.route.Check.fixture_inputs.settings` (the location form of that file's
    214) and contains `a fixture input cannot replace a field read by an outcome predicate`. Red
    on this unit's base after the `via:` work and before the `fixture_inputs.rs` change: the model
    validates (print "validated"); on the base before the `via:` work: `is not an input of`.
15. `a_member_named_missing_keeps_its_struct_reading_at_ess_37_and_is_the_missing_source_at_ess_38`
    (D4; one text, two formats): an inline model `TAG` (below) with an event field `mark` of the
    struct type `shop.tag.Mark` whose one member is `missing: shop.tag.SpotId`, published `mark:
    {missing: input.spot.id}`.
    - at `format: ess/37` (as `TAG` is written): `spec(TAG)` validates, and `Note/noted`'s payload
      `mark` is the domain's `PayloadSource::Struct { fields }` with one `PayloadField { target:
      "missing", source: PayloadSource::InputField { field: "spot.id" } }` (the reading every
      release from 0.93.0 gives it, `nested_missing`'s shape, `related_list.rs` 240-252; the test
      is in `ess-domain`, so it reads the domain model, not the IR);
    - the same text with `with("format: ess/37", "format: ess/38")`: refused, the error contains
      `` `{missing: input.spot.id}` reads the first element of `input.spot.id` that names no row, and outcome `noted` is not guarded by `when_related: {via: input.spot.id, exists: false}` over a list of references ``
      (`related_list.rs` 302-306, unchanged text).
    Red on the base: the `ess/38` half validates (the base reads it as `ess/37` does); the `ess/37`
    half is green at once and must stay green: it is the proof that `ess/37` keeps its meaning.
    `TAG`:

    ```yaml
    format: ess/37
    system: shop
    version: v1
    domain: shop.tag
    summary: A note publishes a mark whose one member is named missing.
    types:
      - {name: shop.tag.SpotId, kind: newtype, of: Uuid}
      - name: shop.tag.Spot
        kind: struct
        fields:
          - {name: id, type: shop.tag.SpotId}
      - name: shop.tag.Mark
        kind: struct
        fields:
          - {name: missing, type: shop.tag.SpotId}
    events:
      - name: shop.tag.Noted
        fields: [{name: mark, type: shop.tag.Mark}]
    actors:
      - name: shop.tag.Clerk
        may: [shop.tag.Note]
    commands:
      - name: shop.tag.Note
        input:
          - {name: spot, type: shop.tag.Spot}
        outcomes:
          - name: noted
            emits: [shop.tag.Noted]
            payload: {shop.tag.Noted: {mark: {missing: input.spot.id}}}
    ```

    If the `ess/37` half does not validate on the base for a reason other than this payload (an
    omission in `TAG`), correct `TAG` and say so; if it is refused for the payload itself, the
    struct reading was never accepted: then the `ess/37` half asserts that refusal, byte for byte
    the same before and after this unit, and the report names the refusal text.

Appended to `crates/specify/ess-compiler/tests/related_guard_ir.rs` (an existing binary; after its
last test, `issue_283_each_branch_carries_the_row_its_own_via_names` at 221), with
`const MEMBER: &str = include_str!("../../../verify/ess-conformance/tests/fixtures/struct-input-member-related-rows.yaml.in");`
and its `ir` helper (22):

10. `a_member_via_carries_its_path_and_the_members_type_in_the_ir`: in `shop.route.Check`, both
    related branches hold `ResolvedRelatedVia::Input { field: "settings.primary", type_ref }` with
    `type_ref.written() == "shop.route.DepotId"`; in `Send`, `field == "settings.depots"` and the
    type is `List<shop.route.DepotId>`; `unknown-depot`'s error payload is
    `ResolvedPayloadValue::MissingElement { field: "settings.depots", type_ref }` at
    `shop.route.DepotId`; in `Probe`, `field == "settings.extra.spare"` and `type_ref.is_optional()`
    with `type_ref.required().written() == "shop.route.DepotId"` (as
    `issue_304_an_optional_input_via_keeps_its_declared_type_in_the_ir`, 80, asserts for a
    top-level Optional). Red on the base: `ir(MEMBER)` panics with the validation errors.
11. `issue_304_ess_20_related_fixtures_compile_to_identical_ir` (104) and `LEGACY_RELATED_IR` (152)
    are the existing one-segment IR pins: control, unchanged and green. The whole-corpus IR pin is
    `selection_precedence_table` (Gate): no row moves.

## Steps

1. Start from the wave branch with s0 merged (a descendant of `d3252eda9`); write `git rev-parse
   --short HEAD` in your report. Before any edit, validate a control copy of the model in a scratch
   test run: replace `format: ess/38` with `format: ess/37` and every `input.settings.<m>` read
   with a top-level input `<m>` (add `depots`, `primary`, `backup` as inputs of the commands
   reading them, and `spare` typed `Optional<shop.route.DepotId>` for `Probe`'s
   `input.settings.extra.spare`); it must validate on the base. If it does not, correct only what
   the control's errors name in the parts of the model that do not address a row (event, error and
   view declarations) and carry the same correction into the fixture; do not change what any guard
   reads. Do not commit the control. Validate `TAG` at `ess/37` the same way (test 15's first half).
2. Write the fixture and tests 1-15; run both targets and record the red lines.
3. `input_path.rs` helpers (`address`, `reads_member`; s0's `addresses_members` and
   `below_ess_38` are used, not changed); then `related_guard.rs`, `related_list.rs`, `spec.rs`,
   `value_expression.rs:529`, `fixture_inputs.rs`, then the compiler.
4. `FORMAT_HISTORY` row 38 (D7, replacing s0's placeholder). Regenerate its page: `slot cargo
   xtask format-history`, then `slot cargo xtask format-history --check`.
5. Run the tests of the Gate. Where `cargo test -p ess-compiler --test typed_diagnostics` reports
   the inventory moved, a `ValidationError::new` or `::at` was written in a top-level file of
   `ess-domain/src`: move it into `input_path.rs` rather than re-pinning the page.
6. Schema: the `via` doc comment is part of `RawSpecFile`. `slot cargo xtask schema`, then
   `slot task projection-check` (it runs `schema --check`, `format-history --check`, `docs`).
7. Re-stamp the synthesis sources (`ess-domain` and `ess-compiler` are two of the eight hashed
   crates): `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`
   from the repository root; then run the slow-lane byte comparison of that crate:
   `slot --timeout 30m task test-slow PACKAGES=ess-conformance`. No table row may move: no corpus
   model declares `ess/38` (s0 opened it with no model), so none can name a path in these
   positions, and a struct member named `missing` set from a path keeps its reading at every
   format a corpus model declares (`grep -rn "{missing: input\.[a-z_]*\." --include='*.yaml'
   examples crates` lists any such model; report the hits, none may move). A moved row is a defect
   of this unit; stop and report it.
8. `CHANGELOG.md` (below); `slot task fmt-check`.

## Owned files

- `crates/specify/ess-domain/src/command/input_path.rs` (`address`, `reads_member` and the module
  doc)
- `crates/specify/ess-domain/src/system.rs` (row 38's `added` sentence only)
- `crates/specify/ess-domain/src/command/related_guard.rs` (the functions named under Where)
- `crates/specify/ess-domain/src/command/related_list.rs`
- `crates/specify/ess-domain/src/command/fixture_inputs.rs` (`read_by_a_guard` only)
- `crates/specify/ess-domain/src/command/value_expression.rs` (the one call at 529)
- `crates/specify/ess-domain/src/spec.rs` (one call beside 462)
- `crates/specify/ess-compiler/src/resolve.rs` (`related_guard`, `related_read`, the
  `MissingElement` arm)
- `crates/specify/ess-compiler/src/ir.rs` (two doc comments)
- `crates/specify/ess-domain/tests/input_path_addressing.rs` (this unit's section, after s0's
  test)
- `crates/specify/ess-compiler/tests/related_guard_ir.rs` (appended test and constant)
- `crates/verify/ess-conformance/tests/fixtures/struct-input-member-related-rows.yaml.in` (new)
- `schemas/generated/ess.schema.json`, `website/docs/reference/spec-versions.md` (regenerated)
- `website/docs/reference/formats.md` (the `ess/38` row's text only)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (re-stamped)
- `CHANGELOG.md` (one bullet)

## Not yours

- s0's: `SUPPORTED_FORMATS`, `FormatVersion::V38`, `input_path::addresses_members` and
  `below_ess_38`, s0's test, `models/toolchain/domains/specify.yaml`,
  `website/docs/start/first-specification.md`, `epic_formats_open.rs`,
  `adversary_t1_epic_formats.rs` and the toolchain table rows. Row 37 of `FORMAT_HISTORY`,
  `formats.md` row 203 and `spec-versions.md`'s `ess/37` row: released, never edited.

- `crates/specify/ess-domain/src/entity.rs` (`validate_instance`), `command/set_effects.rs`,
  `command.rs`, the instance resolution of the compiler (`instance`, 4190-4280), `resolved_each`
  (6332): s2.
- `docs/design/expression-family-source22.md`, `website/docs/reference/predicates.md`,
  `docs/design/language-grids.md`: s2 writes the documents for all positions at once (the parent's
  last acceptance line is one statement).
- Everything under `crates/verify/ess-conformance/src/` and `crates/generate/`: s3 to s6.
- `crates/edge/ess-xtask/` (no test file there is edited), the `ValidationCode` list,
  `crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json` and
  `docs/design/review-typed-diagnostics.md` (no new code, no new site in a top-level file).
- The corpus tables (`*-table.tsv`): no row moves in this unit.

Shared files. With s0 (round 0, landed first): `input_path.rs` (s0's two functions stay as they
are; this unit adds others), `system.rs` row 38 (this unit replaces s0's placeholder sentence),
`formats.md` row `ess/38`, `spec-versions.md` (regenerated by command), `input_path_addressing.rs`
(appended after s0's test) and the stamp (re-run on the merged tree). s1 is alone in round 1 and
lands before every later unit: `CHANGELOG.md` (s0 writes no bullet; this unit's bullet opens the
`ess/38` entries under `### Added`; each later unit appends after the last bullet of its heading
and changes nothing above it); the synthesis-sources stamp (s2 and s3 in round 2, s4 in round 3,
s6 in round 3 only if it edits `ess-gen`; each re-runs the stamp command on the merged tree, never
merges the file by hand); with s2: `input_path.rs`, `system.rs` row 38, `spec.rs`, `resolve.rs`,
`ir.rs`, `input_path_addressing.rs`, `related_guard_ir.rs`, `formats.md` row `ess/38`,
`ess.schema.json` and `spec-versions.md` (s2 appends after s1's lines in different functions and
regenerates the two generated files by command).

## Gate

Per `AGENTS.md` § Gate: `ess-domain` and `ess-compiler` changed, so `ess-cli` too; a `RawSpecFile`
doc comment changed, so `cargo xtask schema` and `task projection-check`; `CHANGELOG.md` and a
generated page under `website/` changed, so `task test-xtask`. Every cargo command carries
`CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0` (shown once, applies to all); named `--test`
targets only, one per command; a pool job ends after 10 minutes unless it names its time
(`SLOT_TIMEOUT=30m` on a cargo call, `slot --timeout 30m` on anything else, 30 minutes at most);
`cargo xtask` runs as `slot cargo xtask`. A test over 60 s is killed; a test step ends within 120
s. No Python, nothing in `/tmp`, no `RUSTC_WRAPPER`, `SCCACHE_*` or `-j`.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-domain -p ess-compiler --all-targets --locked -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test input_path_addressing
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test format_history
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test related_list
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test related_guard
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test related_rows_all
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test related_guard_multiple_vias
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test adversary_related_guard_pass1
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test adversary_related_guard_pass2
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test fixture_inputs_guard_reads
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --lib
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test related_guard_ir
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test selection_precedence_table
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test typed_diagnostics
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test string_alphabet_and_length
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table_cli_read
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test adversary_related_values_all_bytes
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test external_beside_held_guard
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test related_list
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-cli --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test tutorial_page
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test validate_implied_relations
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test validate_parse_refusals
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test specify_formats
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test epic_formats_open
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test adversary_t1_epic_formats
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-openapi --locked --test adversary_ess_issues_2_u6
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test served_codec_names
slot --timeout 30m task test-slow PACKAGES=ess-conformance
slot cargo xtask schema --check
slot cargo xtask format-history --check
slot task projection-check
slot --timeout 30m task test-xtask
slot --timeout 30m task ci-lint
```

The `ess-cli` targets named are those whose sources pin `when_related: {via: input.…}` output or
a format row (`grep -ln "via: input\|ess/37" crates/edge/ess-cli/tests/*.rs`, plus s0's
`epic_formats_open` and `adversary_t1_epic_formats`, which pin that `ess/37` reads as before); the other
`ess-cli` targets are left to the forge's shards, which run them all. `ess-synth`'s `go` target
needs `go` on `PATH`; run `SLOT_TIMEOUT=30m cargo test -p ess-synth --locked --test go` if it is,
else name it in the report as not run. Do not start `task check` or a whole-workspace run.

## Acceptance

1. `cargo test -p ess-domain --locked --test input_path_addressing` passes tests 1-9 and 12-15
   and s0's test, each under 1 s as timed (parent acceptance 1, 2, 3, 6); test 4 shows the same
   text refused at `ess/37` with the message of `below_ess_38`, test 15 that `ess/37` keeps its
   reading of `{missing: input.a.b}` and `ess/38` has the new one.
2. `cargo test -p ess-compiler --locked --test related_guard_ir` passes test 10 and its existing
   tests unchanged.
3. `ess specify validate` on the fixture copied to a `.yaml` file in a scratch directory exits 0
   (run once by hand with the built `ess`, output in the report).
4. `cargo test -p ess-compiler --locked --test selection_precedence_table` passes with no edit to
   `selection-precedence-table.tsv`: every corpus model keeps its IR bytes (parent acceptance 11).
5. The stamp test passes after the re-stamp, and `task test-slow PACKAGES=ess-conformance` passes
   with no table edited.
6. `slot task projection-check` and `slot --timeout 30m task test-xtask` pass.
7. Every target of the Gate passes; `git diff --stat` shows only the owned files.

## Decisions

- D1. The path lives in the existing `field` string of `ResolvedRelatedVia::Input` and of
  `MissingElement`, joined by `.`, as `ResolvedPayloadValue::InputField` holds an `ess/22` path
  (`ir.rs` 1212-1219); no new IR key. A one-segment field serializes exactly as today, which is
  the parent's "IR keeps its bytes" with no `skip_serializing_if` needed. The fit review's "carry
  a path, left out for one segment" is met by the same bytes.
- D2. One resolver, `input_path::address`, for every addressing site of `ess-domain`; it returns
  `None` for a path below `ess/38` (through s0's `addresses_members`), so `ess/37` or an older
  format can never resolve one by accident, and the format refusal (`path_below_ess_38`, built
  with s0's `below_ess_38`) answers first.
- D3. The feature's model is a `.yaml.in` fixture, outside the corpus: the corpus tables and the
  stamp's model count do not move in any unit of this feature, which is the direct proof of the
  synthesis parent's "suites of models without an input path keep their bytes". The model's own
  suites are pinned by s3's and s4's tests.
- D4. `{missing: input.<path>}` is the missing-element source at `ess/38` only (the operator's
  decision D48, 2026-11-23: a released format keeps its meaning). A struct value with a member
  named `missing` set from an input path of two or more segments (`{missing: input.a.b}` as a
  nested struct literal) keeps the reading 0.93.0 to 0.98.0 give it at `ess/22` to `ess/37`
  (`read_paths_below_ess_38`, the precedent of `read_below_ess_28`); at `ess/38`, which no released
  `ess` accepts, the same text is the missing-element source, which `related_list::check` refuses
  outside a list guard. No accepted document changes meaning, so nothing is marked breaking. Test
  15 pins both sides on one text; test 9 pins the parse at `ess/37`.
- D5. A route through an `Optional` member reads as an `Optional` reference
  (`InputPath::type_ref`), so `via_is_optional` and every `ess/22` "checked only when present"
  rule apply unchanged; `input_path::optional_route` (233) is not called for a `via`.
- D6. The `{related: {via, field}}` value source is not opened (not an addressing position of the
  parent; `related_field` and `value_expression.rs`'s own `related_entity` unchanged).
- D7. Row 38's `added` holds exactly this sentence in place of s0's placeholder (s2 appends a
  second): "A `when_related:` guard
  may name the row it reads through a member of a struct input, `via: input.settings.primary` with
  `exists: false` or `predicate:`, as an `all:` element, and a `List<…>` member of references,
  `via: input.settings.depots, exists: false`, with its `{missing: input.settings.depots}`, the
  `input.<path>` `sets:` reads: a member the struct does not declare is refused as
  `undeclared_reference` naming its members, a member that is no entity identity or a path
  crossing a list, a map or a union as `type_mismatch` naming the type, and a route through an
  `Optional` reads as an Optional reference, checked only when present. Below `ess/38` each is
  refused as `unsupported_format_version` naming `ess/38`, and a struct value written `{missing:
  input.a.b}` keeps its reading as that struct value; a model without a path there keeps its IR
  bytes (beyond10x/ess#640)." No release number is written; the release commit sets the row's
  `release`.
- D8. No new `ValidationCode`, no new diagnostic code, no new keyword, no new dependency.
- D9. `ess verify diff` needs no change: a moved path is a changed `field` string, reported as the
  existing `outcome-condition-changed`.
- D10. A `via:` route never crosses a union: `resolve` refuses it as `Opaque`, `type_mismatch`
  naming the union (test 13), as it refuses a list or a map. (N2)
- D11. A `fixture_inputs:` binding over a struct input is refused when a `via:` reads any member of
  it (N1): the fixture would choose the row the guard reads.
- D12. `reads_member` lives here (X1), so s2 and s3 both depend on s1 only and run in one round.
  The format gate (`addresses_members`, `below_ess_38`) is s0's, so s2 uses the same gate without
  depending on this unit for it.

## Open for the operator

None. The format is decided: `ess/38`, opened by s0; `ess/37` keeps its meaning and nothing is
breaking (the operator's decision D48, 2026-11-23).

## Changelog

Under `## [Unreleased]`, `### Added` (the heading of 0.91.0's "Format `ess/36`: …" entries), after
the last bullet of that heading; it is the first `ess/38` entry (s0 writes none):

- Format `ess/38`: a `when_related:` guard may name the row it reads through a member of a struct input, `via: input.settings.primary` with `exists: false` or `predicate:` and as an `all:` element, and a `List<…>` member of references, `via: input.settings.depots, exists: false`, with `{missing: input.settings.depots}` in its payload. Below `ess/38` each is refused as `unsupported_format_version` naming `ess/38`, and a struct value written `{missing: input.a.b}` keeps its meaning; a model without such a path keeps its IR bytes (beyond10x/ess#640).

## The model

`crates/verify/ess-conformance/tests/fixtures/struct-input-member-related-rows.yaml.in`, byte for
byte (Step 1 may correct a declaration that does not address a row, and says so in the report):

```yaml
format: ess/38
system: shop
version: v1
domain: shop.route
summary: A shipment names its depots inside its settings; a depot no row carries is refused and named, and a closed depot refuses a check.
types:
  - {name: shop.route.DepotId, kind: newtype, of: Uuid}
  - {name: shop.route.ShipmentId, kind: newtype, of: Uuid}
  - name: shop.route.Extra
    kind: struct
    fields:
      - {name: spare, type: shop.route.DepotId}
  - name: shop.route.Settings
    kind: struct
    fields:
      - {name: depots, type: "List<shop.route.DepotId>"}
      - {name: primary, type: shop.route.DepotId}
      - {name: backup, type: shop.route.DepotId}
      - {name: extra, type: "Optional<shop.route.Extra>"}
      - {name: note, type: String}
entities:
  - name: shop.route.Depot
    identity: {name: depot_id, type: shop.route.DepotId}
    fields:
      - {name: open, type: Boolean}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
  - name: shop.route.Shipment
    identity: {name: shipment_id, type: shop.route.ShipmentId}
    fields:
      - {name: depots, type: "List<shop.route.DepotId>"}
    relations:
      - {name: depots, kind: references, target: shop.route.Depot, cardinality: many, via: depots}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - name: shop.route.UnknownDepot
    summary: A depot the settings name does not exist.
    fields:
      - {name: depot_id, type: shop.route.DepotId}
  - {name: shop.route.ClosedDepot, summary: A depot the settings name is closed., fields: []}
events:
  - name: shop.route.DepotAdded
    fields: [{name: depot_id, type: shop.route.DepotId}]
  - name: shop.route.ShipmentSent
    fields: [{name: shipment_id, type: shop.route.ShipmentId}]
  - name: shop.route.DepotChecked
    fields: [{name: depot_id, type: shop.route.DepotId}]
actors:
  - name: shop.route.Clerk
    may: [shop.route.AddDepot, shop.route.Send, shop.route.Check, shop.route.Pair, shop.route.Probe]
commands:
  - name: shop.route.AddDepot
    input:
      - {name: open, type: Boolean}
    outcomes:
      - name: added
        creates: shop.route.Depot
        instance: depot_id
        emits: [shop.route.DepotAdded]
        payload: {shop.route.DepotAdded: {depot_id: {generated: true}}}
        sets: {open: input.open}
  - name: shop.route.Send
    input:
      - {name: settings, type: shop.route.Settings}
    outcomes:
      - name: unknown-depot
        when_related: {via: input.settings.depots, exists: false}
        error: shop.route.UnknownDepot
        payload: {shop.route.UnknownDepot: {depot_id: {missing: input.settings.depots}}}
      - name: sent
        creates: shop.route.Shipment
        instance: shipment_id
        emits: [shop.route.ShipmentSent]
        payload: {shop.route.ShipmentSent: {shipment_id: {generated: true}}}
        sets: {depots: input.settings.depots}
  - name: shop.route.Check
    input:
      - {name: settings, type: shop.route.Settings}
    outcomes:
      - name: no-depot
        when_related: {via: input.settings.primary, exists: false}
        error: shop.route.UnknownDepot
        payload: {shop.route.UnknownDepot: {depot_id: input.settings.primary}}
      - name: closed
        when_related: {via: input.settings.primary, predicate: open == false}
        error: shop.route.ClosedDepot
      - name: checked
        emits: [shop.route.DepotChecked]
        payload: {shop.route.DepotChecked: {depot_id: input.settings.primary}}
  - name: shop.route.Pair
    input:
      - {name: settings, type: shop.route.Settings}
    outcomes:
      - name: no-primary
        when_related: {via: input.settings.primary, exists: false}
        error: shop.route.UnknownDepot
        payload: {shop.route.UnknownDepot: {depot_id: input.settings.primary}}
      - name: no-backup
        when_related: {via: input.settings.backup, exists: false}
        error: shop.route.UnknownDepot
        payload: {shop.route.UnknownDepot: {depot_id: input.settings.backup}}
      - name: both-closed
        when_related:
          all:
            - {via: input.settings.primary, predicate: open == false}
            - {via: input.settings.backup, predicate: open == false}
        error: shop.route.ClosedDepot
      - name: paired
        emits: [shop.route.DepotChecked]
        payload: {shop.route.DepotChecked: {depot_id: input.settings.primary}}
  - name: shop.route.Probe
    input:
      - {name: settings, type: shop.route.Settings}
    outcomes:
      - name: no-spare
        when_related: {via: input.settings.extra.spare, exists: false}
        error: shop.route.ClosedDepot
      - name: spare-closed
        when_related: {via: input.settings.extra.spare, predicate: open == false}
        error: shop.route.ClosedDepot
      - name: probed
        emits: [shop.route.DepotChecked]
        payload: {shop.route.DepotChecked: {depot_id: input.settings.primary}}
views:
  - name: shop.route.Depots
    source: shop.route.Depot
    consistency: read_your_writes
    fields:
      - {name: depot_id, type: shop.route.DepotId}
      - {name: open, type: Boolean}
  - name: shop.route.Shipments
    source: shop.route.Shipment
    consistency: read_your_writes
    fields:
      - {name: shipment_id, type: shop.route.ShipmentId}
      - {name: depots, type: "List<shop.route.DepotId>"}
```
