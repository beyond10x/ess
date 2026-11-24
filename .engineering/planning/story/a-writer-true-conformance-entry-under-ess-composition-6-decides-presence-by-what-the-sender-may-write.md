---
format: aep.planning-md/3
id: story:a-writer-true-conformance-entry-under-ess-composition-6-decides-presence-by-what-the-sender-may-write
kind: story
status: active
title: A writer true conformance entry under ess-composition 6 decides presence by what the sender may write
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:a-writer-side-conformance-admits-the-narrowing-a-sender-may-do
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/specify/ess-composition/src/conformance.rs
- confidence: cited
  path: crates/specify/ess-composition/src/lib.rs
- confidence: cited
  path: crates/specify/ess-composition/tests/owner_types.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/writer_conformance/common.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/writer_conformance/ends.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/writer_conformance/format.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/writer_conformance/main.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/writer_conformance/narrowing.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/writer_conformance/presence.rs
- confidence: cited
  path: website/docs/reference/formats.md
revision: 19
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:02:28Z", actor: "human:timo", revision: 18}
- {from: "proposed", to: "active", at: "2026-11-23T06:02:30Z", actor: "human:timo", revision: 19}
---
## Outcome

- `ess-composition/6` exists. A `conformances` entry under it may carry `writer: true`, the sender's assertion: the local type only sends the imported one, so the pair conforms when the local type can send no value the imported type's reader refuses.
- Under `writer: true` a field's presence and optionality are decided by that rule, cell for cell by the two tables below. A local field `null_when_absent` against an imported `omitted_or_null` field composes; a local `Optional` field against an imported required one is `type_conformance_drift`, where a plain entry admits it today.
- The same rules apply to an `event` end and to a `command` end with `part: response`, because a payload is compared as a struct of its fields.
- `writer: true` with `reader: true` on one entry is refused when the document is read, naming both keys. `writer`, whatever its value (`null` included), under `ess-composition/2` to `/5` is refused as `unsupported_format` naming `ess-composition/6`.
- Every `/1` to `/5` document keeps its authored and compiled bytes. A `/6` document compares value constraints and variant attributes as `/5` does, and admits `roles` and payload ends as `/5` does.
- Not in this unit: everything the narrowing table lists beyond optionality (newtype, enum, variants, union, object as a map, field subset, extra fields, value constraints). Under `writer: true` those stay exactly as a plain entry compares them until `w2`.

## Why

Parent: `story:a-writer-side-conformance-admits-the-narrowing-a-sender-may-do` (its `## Design`, `## Fit review` and `## Decisions`; the operator accepted the full rule, decision D19). The consumer gap: a sender that keeps its own copy of an imported message and always writes an optional member cannot assert the copy today, because direction cannot be declared for a writer and presence is compared by equality whatever the mode.
This unit opens the shared format `ess-composition/6` (parent Decision 5, A) and builds the presence and optionality half of the writer mode. `story:reader-true-refused-for-closed-readers` and `story:a-synthesized-suite-sends-an-omitted-or-null-member-both-ways` are not in this wave; whichever of them is built later reuses what this unit adds (see `## Decisions`, 1).

## Where

Read at commit `16f2fc88c2` (`integration/wave-2026-11-22`) and re-checked at `80794bbd698` (`plan/wave-2026-11-23`): `crates/specify/ess-composition/` and the composition rows of `docs.rs` are identical there; `formats.md` moved (the `/5` rows are lines 213-214 there). Line numbers are as read; locate by name once edits move them. Record your base with `git rev-parse HEAD` before you start.

`crates/specify/ess-composition/src/lib.rs`

- Lines 20-36 (module doc): after the `PAYLOAD_COMPOSITION_FORMAT` paragraph (33-36) add a paragraph: under `WRITER_COMPOSITION_FORMAT` a conformance may carry `writer: true`, the local type only sends the imported one, so a presence the sender writes must be one the imported reader accepts and a required value is admitted where the imported one is `Optional`.
- After line 77 add `pub const WRITER_COMPOSITION_FORMAT: &str = "ess-composition/6";` with the doc line "The composition format that also admits `writer: true` on a `conformances` entry." Line 80: `SUPPORTED_COMPOSITION_FORMATS` becomes `&[1, 2, 3, 4, 5, 6]` (`cargo xtask docs` reads this line; keep it one bracketed list).
- After the constants add `pub(crate) fn at_least_payload(format: &str) -> bool { matches!(format, PAYLOAD_COMPOSITION_FORMAT | WRITER_COMPOSITION_FORMAT) }` with the doc "Whether `format` reads what `/5` introduced: payload ends, `roles` and value-constraint comparison". Every `/5`-or-later test below goes through it.
- `TypeConformance` (640-656): after `reader` add `writer: Option<WriterKey>` with the same three serde attributes as `reader` (`default`, `deserialize_with = "present"`, `skip_serializing_if = "Option::is_none"`) and a doc comment "`writer:` as authored (`ess-composition/6`): `None` when absent, `Some(None)` for an authored `null`. Absent from every `/1` to `/5` entry, which keeps its bytes; refused under `/2` to `/5` whatever its value". Field order matters: `writer` comes after `reader`, so the derived `Ord` of existing entries does not change.
- After `ReaderKey` (658-663) add `struct WriterKey(Option<bool>)` with the same derives and `#[serde(transparent)]`.
- `impl TypeConformance` (675-725): `new` (677-683) and `for_reader` (698-704) set `writer: None`. Add `pub fn for_writer(local: TypeBinding, conforms_to: TypeBinding) -> Self` (`reader: None`, `writer: Some(WriterKey(Some(true)))`) with a doc comment stating the question ("can the local type send a value the imported type's reader refuses?") and that the narrowing beyond presence arrives with the same key. Add `pub fn writer(&self) -> bool` (true only for `Some(WriterKey(Some(true)))`) after `reader()` (707-709), and `fn writer_key(&self) -> bool` (any written value) after `reader_key` (712-714).
- `RawConformance` (780-787): add `#[serde(default, deserialize_with = "present")] writer: Option<WriterKey>`. `RawConformance::read` (789-797): copy `writer` through; before building the entry, if `format == WRITER_COMPOSITION_FORMAT` and `self.reader == Some(ReaderKey(Some(true)))` and `self.writer == Some(WriterKey(Some(true)))`, return `Err(ReadRefusal::WriterWithReader)`.
- `ReadRefusal` (819-829) and its `Display` (831-849): new variant `WriterWithReader`, text exactly ``a conformance entry carries `writer: true` and `reader: true`: `writer` asserts what a sender writes and `reader` what a receiver reads, so an entry carries one of them``.
- `bind` (802-812): `if format != PAYLOAD_COMPOSITION_FORMAT` becomes `if !at_least_payload(format)`. `TryFrom<RawCompositionSpec>` (855-860): the `matches!` of formats that read `roles` becomes `ROLE_COMPOSITION_FORMAT` or `at_least_payload(&raw.format)`.
- `CompositionSpec` (727-751) doc and `with_payload_conformances` (953-967): add `with_writer_conformances` after it, same parameters, `format: WRITER_COMPOSITION_FORMAT.to_owned()`, `..Self::with_payload_conformances(...)`, doc "Creates an `ess-composition/6` input, whose conformances may be writer assertions ([`TypeConformance::for_writer`])".
- `validate_format` (1996-2074): add the arm `WRITER_COMPOSITION_FORMAT => WRITER_COMPOSITION_FORMAT` (2005); add `{WRITER_COMPOSITION_FORMAT}` to the "expected ..." list in the unsupported message (2011-2013: `..., {ROLE_COMPOSITION_FORMAT}, {PAYLOAD_COMPOSITION_FORMAT} or {WRITER_COMPOSITION_FORMAT}`); line 2021 `format != PAYLOAD_COMPOSITION_FORMAT` becomes `!at_least_payload(format)`; line 2035 the `matches!(format, ROLE_COMPOSITION_FORMAT | PAYLOAD_COMPOSITION_FORMAT)` gains `| WRITER_COMPOSITION_FORMAT`; lines 2056-2059 the formats that admit `reader` gain `WRITER_COMPOSITION_FORMAT`; after the `reader` block (2072) add one more block: if `format != WRITER_COMPOSITION_FORMAT` and any entry has `writer_key()`, push `UnsupportedFormat` with detail ``` `writer` on a conformance is an {WRITER_COMPOSITION_FORMAT} construct; {format} does not admit it, whatever its value ```.
- Not edited here: `CompositionCode` (1059-1063) and `CATALOGUE` (1153-1161); `w3` words them.

`crates/specify/ess-composition/src/conformance.rs`

- Lines 29-42 (module doc): after the reader paragraphs add one: a writer assertion (`writer: true`, `ess-composition/6`) asks the dual question, and for presence admits a field whose written absent spellings are among those the imported field reads; and a required local value where the imported one is `Optional`.
- Imports 53-56: drop `PAYLOAD_COMPOSITION_FORMAT`, add `at_least_payload` (`crate::at_least_payload`).
- `resolve_conformances` (59-98), the `Comparison` literal at 71-78: add `writer: asserted.writer(),` after `reader`; `constraints: specification.format() == PAYLOAD_COMPOSITION_FORMAT` (75) becomes `constraints: at_least_payload(specification.format())`.
- `compare_inputs` (176-219), literal 180-187: add `writer: false,`. A role's input comparison never reads a writer assertion.
- `Comparison` (222-235): add after `reader` the field `writer: bool` with the doc "A writer assertion (`ess-composition/6`): the local type only sends the imported one, so a difference is admitted where the local type can send no value the imported type's reader refuses." The struct then has three bools, which `clippy::struct_excessive_bools` (limit: more than three) still accepts; do not add a fourth.
- `field_naming` (278-302): the presence check at 292-301. Both fields `Optional`: where `self.writer`, the pair agrees when `writes_within(own.naming.presence, field.naming.presence)`; otherwise it agrees when the two presences are equal (unchanged). The drift text stays byte for byte: ``field `{at}`: presence {own} where `{imported}` has {theirs}`` (the `presence(...)` helper at 966-968 prints backticks, or `unstated`).
- New private items after `presence` (966-968). This is the one home of the presence rule: the two spelling functions and `writes_within`; `w2` calls `absent_read` and never restates the table.

  ```rust
  /// The absent spellings a field can carry on the wire.
  #[derive(Clone, Copy)]
  struct Absent {
      omitted: bool,
      null: bool,
  }

  /// What a sender of a field with `presence` writes when the value is absent. A field with no
  /// presence writes either spelling.
  fn absent_written(presence: Option<Presence>) -> Absent {
      match presence {
          Some(Presence::NullWhenAbsent) => Absent { omitted: false, null: true },
          Some(Presence::OmittedWhenAbsent) => Absent { omitted: true, null: false },
          Some(Presence::OmittedOrNull) | None => Absent { omitted: true, null: true },
      }
  }

  /// What a reader of a field with `presence` accepts when the value is absent. As the published
  /// schema reads it, a field with no presence accepts only a missing key.
  fn absent_read(presence: Option<Presence>) -> Absent {
      match presence {
          Some(Presence::NullWhenAbsent) => Absent { omitted: false, null: true },
          Some(Presence::OmittedWhenAbsent) | None => Absent { omitted: true, null: false },
          Some(Presence::OmittedOrNull) => Absent { omitted: true, null: true },
      }
  }

  /// Whether every absent spelling a writer's `own` presence can produce is one the imported
  /// field's `theirs` reads. Equal forms always conform.
  fn writes_within(own: Option<Presence>, theirs: Option<Presence>) -> bool {
      if own == theirs {
          return true;
      }
      let (writes, reads) = (absent_written(own), absent_read(theirs));
      (!writes.omitted || reads.omitted) && (!writes.null || reads.null)
  }
  ```

  `Absent`, `absent_written`, `absent_read` and `writes_within` stay private to `conformance.rs`, where `w2` also works. The pin of `absent_read` is the 25-cell test below (its `required`-free columns read the three spellings) and `w2`'s test 6 (the field-subset rule reads `absent_read(..).omitted`).

- `references` (733-772). Two edits, nothing else in the function: (a) directly after the first arm (735-738, `Optional`/`Optional` and `List`/`List`) add an arm `(local, ResolvedTypeRef::Optional { of: theirs }) if self.writer => self.references(path, local, theirs),` with the comment "A writer: a required value where the imported one may be absent writes what every reader accepts, at any position."; (b) the tolerance arm at 740, `(ResolvedTypeRef::Optional { of: mine }, theirs) => ...`, gets the guard `if !self.writer`, and its comment says the tolerance is a reader's. A local `Optional` against an imported required value then falls to the final drift arm (765-770): ``field `codec`: `Optional<String>` where the imported type has `String` ``. That is the drift this unit reports for the optionality cells.
- `structs`, `bodies`, `widened`, `object_shaped`, `invariants`, `text_constraint`, `attributes`, `value_constraints`, `Shape`, `Compared`, `fields`: not edited by this unit (`w2` owns them).

`crates/edge/ess-xtask/src/docs.rs`

- After the `/5` row (257-259) add, with the same comment style:
  `// A conformance entry may assert a sender's copy (`writer: true`).` then `("ess-composition", 6, None),`. A `None` row is the one case a document may still call unreleased. Without the row `cargo xtask docs` refuses "supported format versions with no release recorded".

`website/docs/reference/formats.md`

- After the `/5` rows (the two `ess-composition/5` rows, lines 213-214 at `80794bbd698`; locate by their text, since `h1` also edits this file elsewhere, near its `ess-http` rows) add the authored and compiled `ess-composition/6` rows. `cargo xtask docs` refuses a supported version that no reference page names, so these are required in this unit; `w3` extends the authored row. Text of the two rows, in the register of the rows above them:

  `| Authored composition: `format: ess-composition/6` | As the earlier format; a `conformances` entry may add `writer: true` | Not yet in a release. A writer entry asserts that the local type only sends the imported one, so the pair conforms when the local type can send no value the imported type's reader refuses. A field's presence conforms when every absent spelling it writes is one the imported field reads: `null_when_absent` against `null_when_absent` and `omitted_or_null`; `omitted_when_absent` against `omitted_when_absent`, `omitted_or_null` and a field with no presence; `omitted_or_null` against `omitted_or_null`; a field with no presence against `omitted_or_null` and a field with no presence. A field with no presence reads only a missing key, as its published schema does. A local required value is admitted where the imported one is `Optional`, at any position; a local `Optional` where the imported value is required is `type_conformance_drift`, which a plain entry admits. Everything else is compared as without `writer`. `writer: true` with `reader: true` on one entry is refused when read, naming both keys. An earlier-format document carrying `writer`, whatever its value (`null` included), is refused as `unsupported_format`; earlier-format documents keep their meaning and bytes. `roles`, payload ends and value-constraint comparison are admitted as under `/5`. [Source][composition] |`

  `| Compiled composition: `format: ess-composition/6` | As the earlier format, with each checked conformance's `reader` and `writer` as authored | Not yet in a release. Serialize-only; the compiled format echoes the authored one, and the client plan is unchanged. [Source][composition] |`

`crates/specify/ess-composition/tests/owner_types.rs`

- Lines 173-180, `the_composition_format_admits_every_version`: line 174 `&[1, 2, 3, 4, 5]` becomes `&[1, 2, 3, 4, 5, 6]` and one assertion is added, `assert_eq!(WRITER_COMPOSITION_FORMAT, "ess-composition/6");` (import it at the `use` at the top, near line 13). This is the one existing assertion this unit moves, and it moves only because the list gains `6`. Claim it in your report.

New test target `crates/specify/ess-composition/tests/writer_conformance/` (a directory target named `writer_conformance`, so `--test writer_conformance` works; one binary, as `AGENTS.md` asks for few test binaries):

- `main.rs`: `mod common; mod presence; mod ends; mod format; mod narrowing;`
- `narrowing.rs`: a stub that holds only the doc comment "Filled by the next unit (`w2`): one admitted and one drift case per row of the narrowing table beyond optionality." so that `main.rs` compiles. `w2` replaces it.
- `common.rs`, `presence.rs`, `ends.rs`, `format.rs`: below.

## Tests first

Every test uses inline source strings and builds its two models itself (`RawSpecFile::parse`, `Specification::assemble`, `ess_compiler::compile`), as `tests/adversary_reader_conformance.rs` lines 140-153 (`model(...)`) do. No `*.yaml` or `*.yml` file joins the tree. `crates/generate/ess-entity-runtime/tests/lowered_definitions_table.rs` (`walk`, lines 79-96) walks every `.yaml` and `.yml` file of the repository for `format: ess/` and pins each system directory in `lowered-definitions-table.tsv` (header `models=488 lines=669`); the sibling tables `synthesis-suite-bytes-table.tsv` and `selection-precedence-table.tsv` use the same walker. Inline Rust strings are not walked, so none of the three tables moves. If any of them reds after your change, the fixture leaked into a file: fix that, do not re-pin.

Sources are `ess/31` (the lowest format that has `presence: omitted_or_null`; `undeclared_fields` needs `ess/24`, used by `w2`). Each model is `system.yaml` plus one domain file, like `tests/fixtures/payload-conformance/wire/` (read `system.yaml` and `domains/server.yaml` there for the component and command shapes).

`common.rs` (about 90 lines)

- `fn model(files: &[(&str, &str)]) -> EssIr`, the body of `adversary_reader_conformance.rs:140-153`.
- `struct Pair { relay: EssIr, wire: EssIr }`; `Pair::new(relay_domain: &str, wire_domain: &str) -> Pair` wraps each domain text with a fixed `system.yaml` (system `relay`, component `relay-component`, domain `relay.session`; system `wire`, component `wire-server`, domain `wire.session`).
- `Pair::document(&self, format: &str, entries: &str) -> String`: the YAML of `Models::document` in `payload_conformance.rs:80-91` (services `relay` and `wire`, `source_digest: {SourceDigest::of(..)}`), followed by `entries`.
- `Pair::entry(local: &str, imported: &str, extra: &str) -> String`: one `conformances:` list item `- local: {service: relay, <local>}`, `conforms_to: {service: wire, <imported>}`, then `extra` (for example `    writer: true\n`), as `payload_conformance.rs:94-101` writes it.
- `Pair::compose(&self, text: &str) -> Result<EssCompositionIr, CompositionDiagnostics>` (reads with `CompositionSpec::from_yaml`, panics with the text if reading fails, compiles with both services), `composes(&self, text) -> EssCompositionIr`, and `drift(&self, text) -> String` that expects exactly one diagnostic, asserts its code is `type_conformance_drift`, and returns its `detail()`.

`presence.rs` (about 220 lines; the model sources are two `const &str` of about 40 lines each)

Local `relay.session` and imported `wire.session` each declare, in `ess/31`, the structs `Required` (`codec: String`), `NullWhenAbsent`, `OmittedWhenAbsent`, `OmittedOrNull` (each `codec: Optional<String>` with that `presence:`), and `Unstated` (`codec: Optional<String>`, no `presence:`); plus `Items` (`items: List<String>`) and `ItemsOfOptional` (`items: List<Optional<String>>`) on both sides.

- `the_presence_table_cell_by_cell`. One `#[test]` that loops a `const CELLS: [(&str, &str, bool); 25]` (local struct, imported struct, conforms) and, per cell, composes `/6` with `writer: true`. A cell that conforms composes with no diagnostic. A cell that does not yields exactly one `type_conformance_drift` whose detail contains exactly one ``field `codec` `` and the strings in the second table. It collects every wrong cell into one `Vec<String>` and asserts it is empty, each entry naming `local -> imported`, so one run prints every failure. Red today: `from_yaml` refuses the document, ``unknown field `writer` `` (the entry key is not read yet), so the test panics on the first cell.

  The presence table, local row (what the sender writes), imported column (what the imported reader reads); this is the specification, copy it into `CELLS` unchanged:

  | local writes \ imported reads | required | `null_when_absent` | `omitted_when_absent` | `omitted_or_null` | unstated |
  |---|---|---|---|---|---|
  | required | yes | yes | yes | yes | yes |
  | `null_when_absent` | drift | yes | drift | yes | drift |
  | `omitted_when_absent` | drift | drift | yes | yes | yes |
  | `omitted_or_null` | drift | drift | drift | yes | drift |
  | unstated | drift | drift | drift | yes | yes |

  Expected drift text per cell (`I` is the imported struct's full name, for example `wire.session.OmittedWhenAbsent`; `codec` is the field):

  | cell kind | the detail contains |
  |---|---|
  | local `Optional` (any presence) against the imported `Required` column | ``field `codec`: `Optional<String>` where the imported type has `String` `` |
  | both `Optional`, a drift cell | ``field `codec`: presence `<local presence>` where `I` has `<imported presence>` `` for the four forms with a presence, and the word `unstated` without backticks for no presence on either side, for example ``presence `null_when_absent` where `wire.session.Unstated` has unstated`` |

  Count: 13 cells conform and 12 are drift. Of the 12, four are in the `required` column (a local `Optional` against an imported required field, first kind in the text table) and eight have both ends `Optional` (second kind).

- `a_sender_that_always_writes_the_key_composes_where_a_plain_or_reader_entry_refuses`: local `NullWhenAbsent`, imported `OmittedOrNull`. `writer: true` composes. The same pair as a plain `/6` entry and as `reader: true` is each one `type_conformance_drift` containing ``presence `null_when_absent` where `wire.session.OmittedOrNull` has `omitted_or_null` ``. Red today: the `writer` document does not read; the other two halves are green and stay green.
- `a_local_optional_against_an_imported_required_field_is_drift_under_writer_and_admitted_plain`: local `Unstated`, imported `Required`. The plain `/6` entry composes (the tolerance at `conformance.rs:739-740`, unchanged); the same pair under `writer: true` is drift with the text of the first table. The mirror, local `Required` against imported `Unstated`, composes under `writer: true` and is drift as a plain entry (``field `codec`: `String` where the imported type has `Optional<String>` ``). Red today: the `writer` halves do not read.
- `optionality_holds_at_any_position`: local `Items` against imported `ItemsOfOptional` composes under `writer: true` and is drift as a plain entry; local `ItemsOfOptional` against imported `Items` is drift under `writer: true` (``field `items`: `List<Optional<String>>` where the imported type has `List<String>` ``). Red today: the `writer` halves do not read.

`ends.rs` (about 120 lines; two more model sources)

The `relay` model publishes the events `relay.session.Opened` (`codec: Optional<String>`, `null_when_absent`), `relay.session.Closed` (`codec: Optional<String>`, no presence) and `relay.session.Sent` (`codec: String`), and accepts the commands `relay.session.Open` (response `codec` `null_when_absent`), `relay.session.Close` (response `codec: Optional<String>`, no presence) and `relay.session.Send` (response `codec: String`). The `wire` model publishes `wire.session.Opened` (`codec` `omitted_or_null`) and `wire.session.Closed` (`codec: String`) and accepts `wire.session.Open` (response `codec` `omitted_or_null`) and `wire.session.Close` (response `codec: String`). Each command is `input: [{name: channel, type: String}]` with one outcome `{name: done, returns: true}`; copy the component's `accepts`, `publishes` and `reached_by: network` from `tests/fixtures/payload-conformance/wire/system.yaml`.

- `an_event_end_applies_the_same_rules`: `event: relay.session.Opened` against `event: wire.session.Opened` composes under `writer: true` and is drift as a plain entry; `event: relay.session.Sent` (required) against `wire.session.Opened` composes; `event: relay.session.Closed` against `event: wire.session.Closed` is drift naming ``field `codec`: `Optional<String>` where the imported type has `String` ``. The compiled JSON of the first has `"writer": true` on the entry. Red today: the `writer` document does not read.
- `a_command_response_end_applies_the_same_rules`: the same three pairs with `command: relay.session.Open, part: response` against `command: wire.session.Open, part: response`, `Send` against `Open`, `Close` against `Close`. Red today: as above.

`format.rs` (about 150 lines; reuses the presence model for its documents)

- `writer_together_with_reader_is_refused_when_read_naming_both_keys`: a `/6` document whose entry has `writer: true` and `reader: true` fails `CompositionSpec::from_yaml`, the error string containing ``writer`` and ``reader`` and `a conformance entry carries`. `writer: true` with `reader: false`, and `writer: false` with `reader: true`, both read and compose (the reader one compares as a reader). Red today: the first assertion fails with ``unknown field `writer` ``, which does not name `reader`.
- `writer_under_an_earlier_format_is_refused_as_unsupported_format_naming_the_new_one`: for each of `ess-composition/2`, `/3`, `/4`, `/5` and each written value (`true`, `false`, `null`, and the bare `writer:`), the document reads and compiles to exactly one diagnostic with code `unsupported_format` whose detail contains `ess-composition/6`. (A `/5` end is `type`; `/2` to `/4` ends are `type` too.) Red today: `from_yaml` refuses the key as unknown before compile, so `.expect("reads")` panics.
- `a_document_without_writer_keeps_its_authored_and_compiled_bytes`: the same one-entry document without `writer` read under `/5` serialises (`to_canonical_json` of the `CompositionSpec`) with no `writer` substring, and its compiled `to_canonical_json` equals a literal string pinned in the test. Capture that literal from the unmodified tree before you edit any source file, and paste it; it is the proof that `/5` bytes did not move. Green today and after.
- `the_writer_key_is_kept_as_authored_in_the_compiled_composition`: `/6` with `writer: true` compiles to JSON whose entry has `"writer": true` after `"conformances"`'s `local` and `conforms_to`; `writer: false` is kept as `false`; `writer: null` as `null` (and `TypeConformance::writer()` is false for both). `CompositionSpec::with_writer_conformances` with `TypeConformance::for_writer` produces a document that `to_canonical_json` writes with `"format": "ess-composition/6"` and `"writer": true`, and that reads back equal through `from_json`. Red today: does not compile (`for_writer`, `with_writer_conformances`, `WRITER_COMPOSITION_FORMAT` do not exist); that compile error is the first red of the whole target.
- `a_v6_document_reads_roles_and_payload_ends_and_compares_value_constraints_as_v5_does`: a `/6` document with `roles: []` reads (under `/3` it is refused, ``unknown field `roles` ``, as `adversary_k7_pass1.rs:251-277` pins). One `event` end pair composes under `/6`. A pair of newtypes over `String` with different `alphabet` (local `abc`, imported `ab`, both `kind: newtype`, no other constraint) as a plain entry is drift under `/6` and under `/5`, and conforms under `/4`. Red today: `/6` is `unsupported_format`.
- `the_composition_formats_are_one_to_six`: not a new test; the edit to `owner_types.rs` above.

Existing tests that must stay green unchanged except the one line above: `cargo test -p ess-composition --locked` (every `/2` to `/5` assertion).

Timing is measured at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`), on a warm build, as test time only: the runner's own figure (the `finished in` line of the target and `--report-time` where the runner offers it), not the command's wall time. The target holds 11 tests (`presence` 4, `ends` 2, `format` 5; the `owner_types.rs` edit is not one of them), each building two or four tiny models; name any test over 5 s with its time in your report. The budget is 60 s per test and 120 s for the step.

## Steps

1. `git rev-parse HEAD` (record). Capture the `/5` compiled-bytes literal for `format.rs` from the unmodified tree: write the test with a placeholder, run it with `SLOT_TIMEOUT=30m cargo test -p ess-composition --locked --test writer_conformance -- a_document_without_writer` after temporarily commenting the other `mod` lines out, copy the printed canonical JSON into the test, restore the `mod` lines.
2. Write `main.rs`, `common.rs`, `presence.rs`, `ends.rs`, `format.rs`, the `narrowing.rs` stub, and the `owner_types.rs` edit. Run the target: it does not compile (the new API); then add only the API signatures, run again, and see the red per test named above.
3. Edit `lib.rs` (constants, `at_least_payload`, `WriterKey`, `TypeConformance`, `RawConformance::read`, `ReadRefusal`, `bind`, `TryFrom`, `with_writer_conformances`, `validate_format`).
4. Edit `conformance.rs` (`Comparison`, the two literals, `field_naming`, `writes_within`, the two edits in `references`, the module doc).
5. Edit `docs.rs` (the row) and `formats.md` (the two rows). Run `cargo xtask docs` (reads the tree, no network; set `TMPDIR` to your scratch directory, not `/tmp`): it must pass.
6. Run the gate below. No pin table moves in this unit; if a model table, `synthesis-suite-bytes-table.tsv` or a `*_TABLE_WRITE` command prints, something walked a new `.yaml`: remove it.
7. One commit on the unit branch with a body naming the story and the format row. No `CHANGELOG.md` edit (`w3` writes the one bullet).

## Owned files

Only this unit edits, in this wave:

- `crates/specify/ess-composition/src/lib.rs` (all but the `CATALOGUE` entry at 1153-1161 and the `CompositionCode` doc at 1059-1063)
- `crates/specify/ess-composition/src/conformance.rs`: `Comparison`, `resolve_conformances`, `compare_inputs`, `field_naming`, `presence`'s neighbour `writes_within`, `references` (the two edits above), the module doc, the imports
- `crates/specify/ess-composition/tests/writer_conformance/main.rs`, `common.rs`, `presence.rs`, `ends.rs`, `format.rs`, and the stub `narrowing.rs`
- `crates/specify/ess-composition/tests/owner_types.rs` (lines 173-180 only)
- `crates/edge/ess-xtask/src/docs.rs` (the one row)
- `website/docs/reference/formats.md` (the two new rows after the `ess-composition/5` rows; `h1` edits other rows of the file: both are row insertions at different places and merge textually)

## Not yours

- `w2` owns, in `conformance.rs`: `structs`, `bodies`, `wire_names`, `invariants`, `text_constraint`, `attributes`, `value_constraints`, `widened`, `object_shaped`, `Shape`, `Compared`, `fields`, `shapes`, and a second pair of edits in `references` (new arms after yours); and `tests/writer_conformance/narrowing.rs` (replace the stub). `w2` starts from this unit's commit, so the two share `references` in sequence, never in parallel.
- `w3` owns the `CompositionCode::TypeConformanceDrift` doc (1059-1063) and its `CATALOGUE` entry (1153-1161) in `lib.rs`, `docs/design/*`, `spec-versions.md`, the rest of `formats.md`, `diagnostics.md`, `CHANGELOG.md`, and `crates/edge/ess-cli/src/main.rs`.
- `story:reader-true-refused-for-closed-readers` and `story:a-synthesized-suite-sends-an-omitted-or-null-member-both-ways` (not in this wave) edit `conformance.rs`, `lib.rs`, `formats.md` and `field_naming`. When one of them is built later it reuses `WRITER_COMPOSITION_FORMAT`, `at_least_payload`, the `docs.rs` row and the `/6` rows, and adds only its own key.
- Shared files of the wave: `CHANGELOG.md`, `docs/`, `website/` belong to the coordinator at integration (`.engineering/waves/wave-2026-11-23.md`); the two lines this unit puts in `formats.md` and the one row in `docs.rs` are required for `cargo xtask docs` to pass and are expected at integration.
- No `.yaml` file anywhere; no `lowered-definitions-table.tsv`, `synthesis-suite-bytes-table.tsv`, `selection-precedence-table.tsv`.

## Gate

`AGENTS.md` § Gate, the crates this unit touches: `ess-composition` and `ess-xtask` (`docs.rs`, `docs` lane). Bounds for every command and test run here (wall-time figures judged at a 1-minute load of 60 or less, test time only): 60 s per test, 120 s for the test step, a pool job ends after 10 minutes unless it names its time (30 minutes at most), no Python in anything that stays in the repository, no files in `/tmp` (set `TMPDIR` to your scratch directory), no `RUSTC_WRAPPER`, `SCCACHE_*` or `-j`. Every `cargo` call goes through the shim; `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0` on each.

```console
slot task fmt-check
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-composition --all-targets --locked -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-composition --locked --test writer_conformance
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-composition --locked
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo xtask docs
slot --timeout 30m task test-xtask
```

The change touches no `ess-domain`, `ess-compiler` or `ess-primitives` file, no `ValidationCode` and no `RawSpecFile`, so no `ess-cli` gate, no `cargo xtask schema` and no `macro-guards.json` move. The model tables (`lowered_definitions_table`, `synthesis_suite_bytes_table`, `selection_precedence_table`) are not run: this unit adds no `.yaml`. `task test-xtask` runs `docs.rs`'s unit tests, which hold the new row to `SUPPORTED_COMPOSITION_FORMATS`; its inner command sets an empty `RUSTC_WRAPPER` by the repository's own task, which this unit does not change.

## Acceptance

1. `cargo test -p ess-composition --locked --test writer_conformance -- the_presence_table_cell_by_cell` passes: 13 cells compose and 12 are one `type_conformance_drift` each with the text above, under `writer: true` on `/6`.
2. In the same target `a_sender_that_always_writes_the_key_composes_where_a_plain_or_reader_entry_refuses` passes: `null_when_absent` against `omitted_or_null` composes under `writer: true` and is drift as a plain entry and under `reader: true`.
3. In the same target the optionality tests pass: a local `Optional` against an imported required field, at the field and inside `List<..>`, is drift under `writer: true` and admitted by a plain entry; a local required value against an imported `Optional` composes under `writer: true` and is drift as a plain entry.
4. In the same target `an_event_end_applies_the_same_rules` and `a_command_response_end_applies_the_same_rules` pass.
5. In the same target `writer_together_with_reader_is_refused_when_read_naming_both_keys` passes, and `writer_under_an_earlier_format_is_refused_as_unsupported_format_naming_the_new_one` passes for `/2` to `/5` with `true`, `false`, `null` and the bare key.
6. `a_document_without_writer_keeps_its_authored_and_compiled_bytes` passes against a literal captured from the tree before the change; `cargo test -p ess-composition --locked` passes with every `/2` to `/5` assertion unchanged except `owner_types.rs:174`, named in the report.
7. `a_v6_document_reads_roles_and_payload_ends_and_compares_value_constraints_as_v5_does` passes (the `at_least_payload` edits).
8. `cargo xtask docs` and `slot --timeout 30m task test-xtask` pass; `SUPPORTED_COMPOSITION_FORMATS` is `&[1, 2, 3, 4, 5, 6]`.
9. `git status` shows no new `.yaml` or `.yml` file and none of the three model tables edited.
10. The report names any test over 5 s (the runner's own figure, at a 1-minute load of 60 or less) with its time, and the base commit.

## Decisions

1. **This unit opens `/6`** (parent Decision 5 A): the constants, `SUPPORTED_COMPOSITION_FORMATS`, the format history row, the old-reader test (item 5 above) and the two reference rows. Reason: it is the first of the three composition stories to be built, and `AGENTS.md` § Determinism and formats asks for a new major when an entry gains a key with meaning (`TypeConformance` is a closed DTO). The constant is named `WRITER_COMPOSITION_FORMAT` after the construct that opens it, as the others are; a later story that adds `tolerant:` reuses it and does not add a seventh format.
2. **Both keys on one entry** (parent Decision 2 A): refused when read, only when both are `true` and the document is `/6`. Reason: `false` and `null` assert nothing, and under an earlier format the `writer` key itself is the refusal.
3. **An imported field with no presence reads `{omitted}`** (parent Decision 3 A), a local one writes `{omitted, null}`: sound against an implementation that validates with the published schema. Equal forms always conform, so `unstated -> unstated` is `yes`.
4. **`writer` is stored as authored**, `Option<WriterKey>` apart from its value, like `reader`: a `/1` to `/5` entry serialises without it, so no old bytes move.
5. **The drift text is not new.** A both-`Optional` presence drift keeps the existing wording; the required-column drift is the existing final arm of `references`. Reason: one wording per drift, and `diagnostics.md` needs no new row (no new code).
6. **No new `ValidationCode` or diagnostic code**: drift stays `type_conformance_drift`; the read refusal is a `ReadRefusal` message, not a diagnostic, like the `roles` and payload-end refusals. `macro-guards.json` and `review-typed-diagnostics.md` do not move.
7. **Inline sources, one test binary directory.** The models are Rust strings in the test files; the target is `tests/writer_conformance/main.rs` with modules, so the crate gains one test binary, not two. Reason: the organization AGENTS.md § Tests stay fast (few binaries) and the three shared model tables walk `.yaml` files.
8. **`Json` as an imported target is not admitted under `writer`** (a local value sent where the imported type is `Json`): it is not in the parent's table, and the table is the specification.
9. **A role's `implements` comparison never reads `writer`** (`compare_inputs` sets `false`): parent fit review 4 puts a response comparison there out of scope.
10. **The `ess specify compose` help text** (`crates/edge/ess-cli/src/main.rs:179` and `:183`, "`/1` to `/5`") is `w3`'s: changing it moves the generated `website/docs/reference/cli.md` and needs an `ess-cli` build, which this unit avoids.

## Open for the operator

None.

## Changelog

none (the one `### Added` bullet for the whole story is written by `w3`, after the last unit).
