---
format: aep.planning-md/3
id: story:a-writer-true-conformance-entry-admits-a-senders-narrowing-of-types-and-value-constraints
kind: story
status: active
title: A writer true conformance entry admits a sender's narrowing of types and value constraints
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:a-writer-side-conformance-admits-the-narrowing-a-sender-may-do
- depends_on: story:a-writer-true-conformance-entry-under-ess-composition-6-decides-presence-by-what-the-sender-may-write
scope:
- confidence: cited
  path: crates/specify/ess-composition/src/conformance.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/writer_conformance/common.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/writer_conformance/narrowing.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:03:21Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:03:23Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

- A `conformances` entry with `writer: true` (`ess-composition/6`, opened by the previous unit `w1`) admits every narrowing in the table below and nothing else. A pair that a plain entry calls `type_conformance_drift` composes where the local type can only send values the imported type's reader accepts; a pair where the local type can send a value the reader refuses stays one `type_conformance_drift`, naming the field.
- Narrowing covers newtypes, an enum sent where the imported field is `String`, a subset of an enum's variants or a union's variants, an object sent where the imported field is `Map<String, Json>`, a subset of an imported struct's fields, extra fields against a struct or command response that ignores undeclared fields, and the value constraints `/5` compares (alphabet, prefix, invariants).
- What `w1` decided (presence and optionality, at any position) is unchanged; plain, `reader: true` and every `/1` to `/5` entry keep their verdicts, messages and bytes.
- Not in this unit: documents, the diagnostic catalogue text and the CLI help (`w3`).

## Why

Parent: `story:a-writer-side-conformance-admits-the-narrowing-a-sender-may-do` (accepted in full, decision D19). A sender that keeps its own copy of an imported message narrows it: a stricter newtype, a fixed enum, an object shape. The dual of what `reader: true` already admits for a receiver. `w1` opened the key and the presence half; this unit builds the rest of the table, cell for cell.

## Where

Read at commit `16f2fc88c2` (`crates/specify/ess-composition` is identical at `f4e13cde68` and at `80794bbd698`, `plan/wave-2026-11-23`). This unit starts from the head of `w1`, which adds about 25 lines to `Comparison`, `field_naming` and `references` and moves every later line down; locate by function name, and record your base with `git rev-parse HEAD`. Line numbers below are the unmodified tree's.

`crates/specify/ess-composition/src/conformance.rs` (all of this unit's source edits are here)

- Module doc (29-42, plus `w1`'s writer paragraph): one more sentence listing the narrowings a writer admits, in the table's words.
- `Shape::Payload` (103-108) gains `ignores: bool`, doc "Whether the end ignores a field it does not declare (`ess/24`): a command response that says so; an event, an error and a command input never do". `shape()` (112-166) sets it: `false` for the event (123-127), the error (131-135) and the command input (140-144); `resolved.undeclared_fields.is_ignored()` for the command response (157-161; `ResolvedCommand::undeclared_fields`, `ess-compiler/src/ir.rs:2070`).
- `Compared` (831-837) gains `ignores: bool`; `fields()` (839-863) sets it from the payload, or for a named struct from `ir.undeclared_fields(name).is_ignored()` (`ess-compiler/src/ir.rs:3525`; `UndeclaredFields::is_ignored`, `ess-domain/src/types.rs:1172`), `false` for a name that is no struct.
- `shapes` (240-260): `self.structs("", (mine.name, own), (theirs.name, sent), theirs.ignores)`.
- `structs` (306-357): a fourth parameter `ignores: bool` (the imported struct's). Edits:
  - first loop (312-325): add after `None if self.reader => {}` the arm `None if self.writer && omittable(field) => {}` with the comment "A sender may leave out an imported field whose reader accepts a missing key (`omittable`, which reads `absent_read`).";
  - second loop (326-356): the wire-key block (333-342) runs for `self.reader || self.writer`, so a local field that travels as a key the imported struct reads is compared with that field (otherwise an extra field would hide a change of type behind an ignored key); then, before the `optional` test (343), `if self.writer && ignores { continue; }` with the comment "A sender may send a field the imported reader ignores: every declared field stays required and typed, which the first loop checked."; the existing drift text is unchanged for a closed imported struct;
  - doc comment: say what a writer may do, beside the reader's sentence.
- New free fn beside `wire_name` (875):

  ```rust
  /// Whether an imported field may be absent from what a sender writes: `Optional`, and its
  /// reader accepts a missing key. The presence rule is `w1`'s `absent_read`, the one home of it.
  fn omittable(field: &ResolvedField) -> bool {
      matches!(field.type_ref, ResolvedTypeRef::Optional { .. })
          && absent_read(field.naming.presence).omitted
  }
  ```

  `absent_read` and `Absent` are `w1`'s private items in this file (the same function `writes_within` reads the imported side through): do not write a second table of which presence reads which spelling. The reader's own test at 343 and 348 stays as it is.
- `wire_names` (361-381): leave. Add `wire_subset` after it, same parameters (`(local, mine)`, `(imported, theirs)`): the imported wire names are `known`, `unknown` is every local `EnumVariant::wire` not in `known`; when `unknown` is not empty push ``"{}: `{local}` has wire names {} that `{imported}` lacks"`` with `at(path)` and `names(&unknown)`.
- `invariants` (387-417): the first loop's guard (403) `if !self.reader` is unchanged; the second loop (411-416, local conditions the imported lacks) is wrapped in `if !self.writer`, with the comment "A sender may state conditions the imported type does not: it sends fewer values.". Doc comment gets the writer's sentence.
- `text_constraint` (423-461): after the reader block (439-451) add:

  ```rust
  if self.writer {
      let sends_within = match (what, mine, theirs) {
          (_, _, None) => true,
          (_, None, Some(_)) => false,
          ("alphabet", Some(mine), Some(theirs)) => mine.chars().all(|c| theirs.contains(c)),
          (_, Some(mine), Some(theirs)) => mine.starts_with(theirs),
      };
      if sends_within {
          return;
      }
  }
  ```

  The drift text below it is unchanged. Doc comment: "A writer sends nothing the owner rejects when the owner states none, or when its alphabet holds only characters of the owner's, or its prefix starts with the owner's; a writer that states none sends any text, so it conforms only to an owner that states none."
- `attributes` (467-546): the sets of attribute names stay exact for a writer (the `agree` expression at 491-495 is untouched: only the reader has the `is_subset` branch). The variant matching closure (511-517): `if self.reader || self.writer` matches by wire name, otherwise by name. Doc comment: a writer's variants are a subset matched by wire name, with the same attributes and values.
- `bodies` (638-731):
  - the `Struct` arm (641-650) passes `self.imported.undeclared_fields(&imported.name).is_ignored()` as the fourth argument of `structs`;
  - the `Enum` arm (655-666): `if self.reader { wire_names } else if self.writer { wire_subset(path, (&local.name, mine.as_slice()), (&imported.name, theirs)) } else { spellings }`; `attributes` follows as now;
  - the `Union` arm (667-721): `let covered = if self.writer { my_names.is_subset(&their_names) } else { my_names == their_names };` and `if covered { ... } else { the existing drift }`; the tag check (677-683) and the loop body are unchanged, and the loop compares each variant through `self.references`, so each conforms as a writer. A union whose local variants are a subset but whose tag differs is still drift for the tag.
- `references` (733-772), after `w1`'s two edits; the final arm order is, with `w1`'s two marked:
  1. `Optional`/`Optional` and `List`/`List`;
  2. `w1`: local against imported `Optional` where `self.writer`;
  3. the tolerance arm, guarded `if !self.writer` by `w1`;
  4. `Primitive` equal;
  5. the reader's object arm (746-747);
  6. new, directly after 5: `(local, imported) if self.writer && reads_any_object(imported) && self.object_shaped(self.local, local) => {}` with the comment "A value that is always a JSON object is accepted by `Map<String, Json>`: the sender's struct or `String`-keyed map.";
  7. `Map`/`Map`;
  8. `Declared`/`Declared`;
  9. the reader's `widened` arm (764);
  10. new, directly after 9: `_ if self.writer && self.narrowed(path, local, imported) => {}`;
  11. the final drift.
- `object_shaped` (809-826): becomes `fn object_shaped(&self, ir: &EssIr, reference: &ResolvedTypeRef) -> bool`, reading `ir.types()` and recursing with `ir`; the reader's call (747) becomes `self.object_shaped(self.imported, imported)`. Its doc comment says which side's model it reads.
- New `narrowed` after `widened` (782-804), with the doc "A writer's narrowing of a local reference that is a declared type against an imported one that is not, or `false` when none applies: a local newtype, through any chain, is what it wraps; a local enum is `String`. The newtype's own constraints only narrow, so they are not compared. A narrowing that applies but recurses into a drift records it and still returns `true`.":

  ```rust
  fn narrowed(&mut self, path: &str, local: &ResolvedTypeRef, imported: &ResolvedTypeRef) -> bool {
      let ResolvedTypeRef::Declared { name } = local else {
          return false;
      };
      let Some(declared) = self.local.types().get(name.name()) else {
          return false;
      };
      match &declared.body {
          ResolvedBody::Newtype { of, .. } => {
              self.references(path, of, imported);
              true
          }
          ResolvedBody::Enum { .. } => matches!(
              imported,
              ResolvedTypeRef::Primitive { name: Primitive::String }
          ),
          ResolvedBody::Struct { .. } | ResolvedBody::Union { .. } => false,
      }
  }
  ```

  Local newtype against imported newtype, enum against enum, struct against struct and union against union go through arm 8 (`named`), as before.
- `compare_inputs`, `field_naming`, `Comparison`, `resolve_conformances`: `w1`'s. Not edited.

`crates/specify/ess-composition/tests/writer_conformance/narrowing.rs`

- Replaces `w1`'s stub (about 450 lines). `common.rs` is `w1`'s; this unit may add functions to it and edit none that exist.

No other file. Not touched: `lib.rs`, `docs.rs`, any document, `CHANGELOG.md`.

## Tests first

All in `crates/specify/ess-composition/tests/writer_conformance/narrowing.rs` (target `writer_conformance`, `cargo test -p ess-composition --locked --test writer_conformance -- narrowing::`). Models are inline source strings, built with `w1`'s `Pair::new(relay_domain, wire_domain)` (local service `relay`, imported `wire`, domains `relay.session` and `wire.session`) and `Pair::document`, `entry`, `composes`, `drift` of `common.rs`. No `.yaml` file joins the tree (the three model tables walk files; `w1`'s `## Tests first` says why). `format: ess/31` in each `system.yaml`. Read `tests/fixtures/constraint-conformance/wire/domains/server.yaml` (newtype `alphabet`, `prefix`, `invariants`, enum `attributes`), `tests/union_unit_variants.rs` (union and unit-variant syntax, `variants: {open:, done: String}`), `tests/adversary_compose_conformance.rs:33-40` (enum `wire:`) and `crates/verify/ess-conformance/tests/adversary_500_pass1.rs:130` (a command with `undeclared_fields: ignored`; a struct writes it as a key beside `kind: struct`) for the exact syntax. If a model does not validate, change the model until it does; the cases below name what each must declare.

Every test composes the pair as `writer: true` on `ess-composition/6` and, in the same test, as a plain `/6` entry where the line says "plain". The red line is the same for every admitted case: with `w1`'s code only, `composes` panics printing the `type_conformance_drift` detail, which names the field (for example ``field `code`: ...``); every drift case is green before and after and must stay green (it pins that nothing was widened too far).

Narrowing table, copied cell for cell; one test per row group below:

| narrowing | admitted under `writer: true` | still drift |
|---|---|---|
| optionality (`w1`) | a local required value where the imported one is `Optional`, at any position | a local `Optional` where the imported value is required (the plain entry's tolerance does not apply) |
| newtype | a local newtype, through any chain, over what the imported reference is | a different primitive |
| enum as `String` | a local enum where the imported reference is `String` | a local `String` where the imported one is an enum |
| variants | local wire names a subset of the imported wire names | a local wire name the imported enum lacks |
| union | the same tag, local variants a subset of the imported ones by tag value, each conforming as a writer | a local variant the imported union lacks |
| object as a map | a local struct or `String`-keyed map where the imported reference is `Map<String, Json>` | anything that is not always an object |
| field subset | omitting an imported field that is `Optional` and whose presence admits omission (`omitted_when_absent`, `omitted_or_null`, unstated) | omitting a required or `null_when_absent` imported field |
| extra fields | a local field the imported struct lacks, when the imported struct declares `undeclared_fields: ignored` (`ess/24`) | the same against a closed imported struct |
| value constraints (`/5` rules) | a local alphabet that is a subset, a prefix that starts with the imported prefix, a superset of the imported invariants | a wider alphabet, a shorter or other prefix, a missing imported invariant |

Enum variant attributes stay compared exactly. Wire names, map keys and lists compare as without `writer`.

Models. The wire domain `wire.session` declares: newtype `Ref` (`of: String`, `alphabet: "abc"`, `prefix: "r-"`), `Code` (`of: String`, invariant `value != "x"`), `Window` (struct `low`, `high` `Integer`, invariant `low <= high`), enum `Status` (variants `Open`, `Closed`, `Pending`, each `wire` its lower-case name), union `Item` (tag `kind`, `text: String`, `count: Integer`, `gone:`), struct `Fields` (five fields, test 6), struct `OpenRecord` (`undeclared_fields: ignored`, `id: String`; test 9 adds `note: Optional<String>` with `omitted_or_null`), struct `ClosedRecord` (`id: String`), and one holder struct per test with the field under test (`Holder` with `code`, `status`, `item`, `extra`, `record`, `ref` as each test needs). The relay domain `relay.session` declares the local narrowing per test. Each test builds its two domain texts with `format!` from one holder field line each so one `Holder` serves the table.

1. `a_newtype_chain_is_admitted_over_what_the_imported_reference_is`: local `Inner` (`of: String`, any `alphabet`) and `Outer` (`of: relay.session.Inner`), local `Holder.code: Outer`, imported `Holder.code: String`: composes under `writer`; drift as plain. Local `Holder.code: Count` where `Count` is a newtype `of: Integer` against imported `String`: drift naming ``field `code` `` and `Integer` and `String`. A local `String` against an imported newtype `wire.session.Ref`: drift under `writer` (a sender of any string is refused by the newtype), and a local enum against an imported newtype: drift.
2. `an_enum_is_admitted_where_the_imported_reference_is_string`: local `Holder.status: Status` (enum) against imported `status: String`: composes under `writer`, drift as plain. The reverse (local `String`, imported enum `Status`): drift under `writer`. A local enum against an imported newtype over `String`: drift.
3. `a_subset_of_the_enum_variants_is_admitted_by_wire_name`: local `Status` with `Open`, `Closed` against imported `Open`, `Closed`, `Pending`: composes under `writer`; drift as plain and as `reader: true` (the reader needs every imported wire name). A local `Status` that adds `Extra` (wire `extra`): drift containing `wire names` and `extra`. A local variant spelled differently with the same wire name (`Live` with `wire: open` against imported `Open`) composes under `writer` (matched by wire).
4. `a_subset_of_the_union_variants_is_admitted_under_the_same_tag`: local `Item` with `text: String`, `gone:` against imported `text`, `count`, `gone`: composes under `writer`, drift as plain. A local that adds `extra: String`: drift containing `variants`. A different tag (`tag: type`): drift containing ``tag ``. A local `text: Optional<String>` against imported `text: String`: drift (each variant compared as a writer). A local `gone: String` against imported `gone:`: drift containing `no payload` or `a payload` per the existing text (`a payload where ... carries none`).
5. `a_struct_or_a_string_keyed_map_is_admitted_where_the_imported_reference_is_any_object`: imported `Holder.extra: Map<String, Json>`. Local `extra` a struct `Meta` (one field `a: String`), a `Map<String, String>`, and a newtype over a struct: each composes under `writer`, each drift as plain. Drift under `writer`: local `Json`, `List<String>`, `Optional<Meta>`, an enum, a union, `Map<Integer, String>`; and an imported bare `Json` field against a local `Meta`.
6. `an_imported_field_may_be_left_out_only_where_its_presence_admits_omission`: imported `Fields` has `req: String`, `nul: Optional<String>` (`null_when_absent`), `omit: Optional<String>` (`omitted_when_absent`), `either: Optional<String>` (`omitted_or_null`), `none: Optional<String>` (no presence). For each of the five fields, a local struct that has the other four with the same declarations and leaves it out: composes for `omit`, `either`, `none`, is drift for `req` and `nul` (each naming ``field `<name>` ``), looped as one `#[test]` collecting wrong cells. Plain: all five drift.
7. `an_extra_field_is_admitted_only_against_a_struct_that_ignores_undeclared_fields`: here `OpenRecord` has `id: String` only (the `note` field belongs to test 9's model). Local `Record { id: String, memo: Integer }` as `Holder.record` against imported `OpenRecord`: composes under `writer`, drift as plain, drift against imported `ClosedRecord` under `writer` (``field `memo` ``). A local extra `memo: Json` composes against `OpenRecord` too. As a struct end (`type: relay.session.Record` against `type: wire.session.OpenRecord`) the same.
8. `an_extra_field_against_a_command_response_follows_the_commands_own_key`: relay command `Open` with response `{id: String, memo: Integer}`, wire command `Open` with response `{id: String}` and `undeclared_fields: ignored`: `command: ..., part: response` ends compose under `writer`; the same wire command without the key: drift; a relay event with fields `{id, memo}` against a wire event with `{id}` (events are closed): drift. Copy the commands' shape from `w1`'s `ends.rs`.
9. `an_extra_field_that_travels_as_an_imported_key_is_compared_with_that_field`: imported `OpenRecord` (`id: String`, `note: Optional<String>` `omitted_or_null`). Local `Record { id: String, memo: Integer }` where `memo` has `wire: note`: drift naming ``field `memo` `` (it sends an `Integer` where the imported field reads `Optional<String>`). Local `memo: String` with `wire: note` composes (the imported field is omittable and the sender's value conforms). Plain: drift.
10. `value_constraints_narrow_for_a_writer`: pairs of newtypes over `String` (`wire.session.Ref` is the imported one), as `Holder.ref`:
    - alphabet: local `"ab"` composes; local `"abcd"` is drift; local with no alphabet is drift; imported with none and local `"ab"` composes (a second imported newtype `Free` with no constraint);
    - prefix: local `"r-x"` composes; local `"r"` is drift; local `"q-"` is drift; local none is drift;
    - invariants (imported `Code`, `value != "x"`): local `value != "x"` and `value != "y"` composes; local `value != "y"` only is drift naming `invariant`; local none is drift; imported none (`Free`) and local with one composes;
    - struct invariants: local `Window` with `low <= high` and `low >= 0` against imported `Window` composes; local with neither is drift naming `invariant`;
    - each of these as a plain `/6` entry is drift where the constraints differ (the `/5` rules), and under `ess-composition/5` the pair composes only when the constraints are equal (control: the unit changed no `/5` behaviour).
11. `variant_attributes_stay_exact_for_a_writer`: imported `Kind` enum with `attributes: [{name: terminal, type: Boolean}]` and variants `Open` (`terminal: false`), `Closed` (`terminal: true`). Local `Kind` with the same attribute and only `Open` composes (a subset of variants, same values). Local `Kind` with no attributes: drift containing `attributes`. Local `Open` with `terminal: true`: drift containing `variant` and `terminal`. Local `Kind` that declares one more attribute: drift containing `attributes`.
12. `a_plain_v6_entry_keeps_its_verdicts`: every admitted pair of tests 1 to 10, composed as a plain `/6` entry, is drift, and every pair a plain entry admits today (a local `Optional` against a required field, the same type, an equal enum) composes both as a plain `/6` entry and under `writer: true` unless a table row says otherwise. The one control that no row changes: a local `Optional<String>` against an imported `String`, plain composes and `writer` is drift (`w1`).

Timing is measured at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`), on a warm build, as test time only: the runner's own figure (the `finished in` line), not the command's wall time. This unit adds 12 tests, each building a handful of tiny models; name any over 5 s with its time in your report. The budget is 60 s per test and 120 s for the step.

## Steps

1. `git rev-parse HEAD` (the base, `w1`'s head). Write `narrowing.rs` (replace the stub); run `cargo test -p ess-composition --locked --test writer_conformance -- narrowing::`; see the admitted cases red as above and every drift case green.
2. Edit `conformance.rs` in this order, running the target after each: `Shape::Payload.ignores`, `Compared.ignores` and `structs`; `wire_subset` and the enum arm; the union arm; `invariants` and `text_constraint`; `attributes`; `object_shaped` and the writer object arm; `narrowed`. Tests 5 and 7 to 9 go green with the last of their steps; test 12 only when all are in.
3. `cargo test -p ess-composition --locked` (all of it): every `/2` to `/5` test and `w1`'s target stays green unchanged.
4. No generated file and no pin moves: the change reads no bytes of any projection. If `cargo xtask diagnostics --check` or a model table reds, something unplanned moved: find it, do not re-pin.
5. One commit on the unit branch, body naming the story and the table.

## Owned files

- `crates/specify/ess-composition/src/conformance.rs`: `Shape`, `shape`, `Compared`, `fields`, `shapes`, `structs`, `wire_subset` (new), `invariants`, `text_constraint`, `attributes`, `bodies`, `references` (arms 6 and 10 of the list above), `object_shaped`, `narrowed` (new), `omittable` (new), the module doc's second paragraph
- `crates/specify/ess-composition/tests/writer_conformance/narrowing.rs`
- `crates/specify/ess-composition/tests/writer_conformance/common.rs`: additions only

## Not yours

- `w1` (lands first) owns, and this unit does not change: `lib.rs`, `Comparison`, `resolve_conformances`, `compare_inputs`, `field_naming`, `Absent`, `absent_written`, `absent_read`, `writes_within` (the presence rule; this unit calls `absent_read`), arms 2 and 3 of `references`, `docs.rs`, the two rows of `formats.md`, `owner_types.rs`, and `main.rs`, `presence.rs`, `ends.rs`, `format.rs` of the target. `references` is shared with `w1` in sequence: this unit starts from `w1`'s head and adds arms 6 and 10 only.
- `w3` owns the `TypeConformanceDrift` text in `lib.rs`, the design page, the reference pages, `diagnostics.md`, `cli.md`, `ess-cli/src/main.rs` and `CHANGELOG.md`.
- No `.yaml` file; none of `lowered-definitions-table.tsv`, `synthesis-suite-bytes-table.tsv`, `selection-precedence-table.tsv`.
- `story:a-synthesized-suite-sends-an-omitted-or-null-member-both-ways` and `story:reader-true-refused-for-closed-readers` are not in this wave.

## Gate

`AGENTS.md` § Gate, the crate this unit touches: `ess-composition` only (no `ess-domain`, `ess-compiler` or `ess-primitives` file, no diagnostic, no format version change, no `RawSpecFile`, no generated tree). Bounds for every command and test (wall-time figures judged at a 1-minute load of 60 or less, test time only): 60 s per test, 120 s for the test step, a pool job ends after 10 minutes unless it names its time (30 minutes at most), no Python, no files in `/tmp` (`TMPDIR` to your scratch directory), no `RUSTC_WRAPPER`, `SCCACHE_*` or `-j`.

```console
slot task fmt-check
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-composition --all-targets --locked -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-composition --locked --test writer_conformance
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-composition --locked
```

No `ess-cli` gate: nothing it compiles changes. No `task test-xtask`: no `ValidationCode`, format row or document moves.

## Acceptance

1. `cargo test -p ess-composition --locked --test writer_conformance -- narrowing::` passes: tests 1 to 12, each admitted case composing under `writer: true` on `ess-composition/6` and each drift case one `type_conformance_drift` naming its field.
2. Test 3 shows the plain entry and `reader: true` refuse the variant subset, and `/5` behaviour is the control of test 10.
3. Test 9 shows an extra field that travels as an imported key is compared with that field, even against an ignoring struct.
4. Test 8 shows a command response follows its own `undeclared_fields` and an event payload stays closed.
5. `cargo test -p ess-composition --locked` passes with no existing test edited.
6. `git diff` shows no line outside the owned files; no new `.yaml`; no model table moved.
7. The report names any test over 5 s (the runner's own figure, at a 1-minute load of 60 or less) with its time, and the base commit.
8. `git diff <base> -- crates/specify/ess-composition/src/conformance.rs | grep '^+' | grep -c 'Presence::'` prints 0: this unit adds no line naming a presence, because the presence rule has one home, `w1`'s `absent_written`, `absent_read` and `writes_within`.

## Decisions

1. **The narrowing table is the specification.** A narrowing it does not list is drift; the unit adds none. Reason: the operator accepted the full rule, not a wider one.
2. **A bare imported `Json` admits nothing under `writer`**, although a `Json` reader accepts any value: it is not a table row. A later story may add it with its own tests.
3. **A local newtype's own constraints are not compared when it narrows to what it wraps** (`narrowed`): they only restrict what is sent. Where both ends are newtypes the existing `value_constraints` runs, with the writer rules above.
4. **Variant attributes stay exact** (set of names, types, values), variants matched by wire name for a writer. Reason: an attribute is data the receiver reads; the table says exact.
5. **An extra field that shares an imported field's wire key is compared with that field** (reader's rule, extended to writers), also against an ignoring struct: otherwise a retyped key would pass as ignored.
6. **`Map` keys, list element types and union tags compare as without `writer`** (the table's last line); a union's variants may be a subset only.
7. **The drift texts are the existing ones**, plus one new text, ``"{at}: `{local}` has wire names {..} that `{imported}` lacks"``, for an enum variant a sender may not send. Reason: one wording per drift; no new diagnostic code, so `diagnostics.md` keeps its rows (`w3` extends the catalogue's prose).
8. **`ignores` is the imported end's**: a struct type through `UndeclaredFields`, a command response through its own key, never an event, an error or a command input. Reason: `ess/24` defines the key for exactly those.

## Open for the operator

None.

## Changelog

none (`w3` writes the one bullet for the story).
