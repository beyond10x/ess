# How a hand-written document is read into its entities

`story:a-specification-declares-how-a-document-is-read-into-its-entities-and-the-generated-decoder-applies-it`,
rules a to e. Builds on the typed decoders of
`story:a-synthesized-rust-crate-carries-typed-decoders-and-encoders-without-a-server-takes-a-package-name-and-leaves-out-its-workspace-table`.
Settled before this page: TD-D9 A (the strict decoder stays string-only for a `Decimal`; reading a
number token is a declared document-form rule, rule e), TD-D10 A (the consumer reads YAML and hands
over a `json::Value`; ESS changes nothing there).

**TD-D11: NOT CONFIRMED: rules b to e describe one document form (a file of many entities) that the
HTTP surface, the served views, the interpreter and the conformance suite do not read, and the
repository places such a reading in a sibling document pinned to the specification, never in
`ess/N`; only rule a has an `ess/N` precedent.**

The owner decides before anything is built: DF2-D1 (placement) and DF2-D2 (rule a), § 7.

## 1. The need

A domain of its own names. The specification is unchanged by anything on this page:

```yaml
types:
  - {name: depot.stock.Rotation, kind: enum, variants: [fifo, lifo]}
entities:
  - name: depot.stock.Bin
    identity: {name: id, type: String}
    fields: [{name: site, type: String}, {name: capacity, type: Decimal}, {name: tags, type: List<String>}]
  - name: depot.stock.Shelf
    identity: {name: id, type: String}
    fields: [{name: site, type: String}, {name: bin, type: String}, {name: code, type: String},
             {name: rotation, type: depot.stock.Rotation}]
```

The document people write and keep in a repository, one of several files of one set:

```yaml
format: depot-stock/1
site: north
bins:
  b-17:
    capacity: 12.50
    shelves:
      - {code: s1, rotation: lifo}
      - {code: s2}
```

It is read as one `Bin` `{id: b-17, site: north, capacity: "12.50", tags: []}` and two `Shelf`
rows, `{id: b-17/s1, site: north, bin: b-17, code: s1, rotation: lifo}` and `{id: b-17/s2, …,
rotation: fifo}`. Today the strict decoder refuses the file and a hand-written reader sits between
the file and the entities.

## 2. The rules

| rule | written | read as | strict decoder today | declared by (§ 4) |
|---|---|---|---|---|
| a | `rotation` absent; `tags` absent | the variant `fifo`; `[]` | a required member absent is refused, "never a default" (`crates/generate/ess-synth/src/rust/json.rs:1143-1156`); only an `Optional` reads absence (`rust/wire.rs:961-970`) | `defaults:` |
| b | `bins: {b-17: {…}}` | a `Bin` whose identity is the key | the identity is a member of the entity's object (`docs/design/unique-wire-field-identity.md:17`); no decoder reads a map entry as an entity | `kinds[].keyed: true` |
| c | `site` once at the root; `bin` from the holding entry | the field on every entity; identity `b-17/s1` | each entity's object must carry every required member (`json.rs:1145`) | `root_fields:`, `children:` |
| d | a root with `format`, root fields and one map per kind; several files | one document set, ids unique across files | the decoders' roots are a declared type, a command input and view parameters (`rust/wire.rs:244`, `:561`, `:606`); none is a document | `marker:`, `kinds:` |
| e | `capacity: 12.50` (number token) | the text `"12.50"`, never through a float | a string only (`rust/json.rs:814-827`, `rust/wire.rs:1051-1053`; Go `go/http.rs:1327`, `:2742`); the published schema is a string (`crates/generate/ess-gen/src/types.rs:133-135`) | `decimals: number_or_text` |

Rule e's token grammar is the one the specification's own `Decimal` literals already use,
`ess_primitives::facts::Number::decimal_literal`: `+1`, `01`, `.5`, `5.` and `1e3` refused
(`docs/design/value-expressions.md:33-38`). The quoted text is accepted too.

## 3. Placement (the TD-D11 question)

**What the repository puts in `ess/N`.** A spelling of one value that every reader and writer of
that value shares, on every surface, and that the published schema states:

- a variant's or field's `wire:` name (`ess/5`), read by the JSON Schema and every codec
  (`docs/design/enum-variant-wire-names.md:39-40`, `:72-81`);
- `undeclared_fields: ignored` (`ess/24`, `docs/design/typed-response-outcome-payloads.md:51`);
- `shorthand: <field>` (`ess/27`): a struct written as one field, schema `oneOf`
  (`website/docs/reference/spec-versions.md:407-420`);
- `presence:` (`ess/31`), admitted at every field position (`docs/design/omitted-or-null-presence.md:18-32`).

**What it puts beside `ess/N`.** The layout of a message or document that carries model values
on one surface:

- `ess-transport/1`: "never part of `ess/N`. The model does not change when the same event is
  carried by another broker" (`docs/design/event-transport-binding.md:19-23`), including "one
  message carries a JSON array of items" (`:11-15`);
- `ess-http/1`, the same words for a route (`docs/design/http-presentation-binding.md:41-46`), and
  in it the request encodings (`:324-340`) and the body forms bare, array and wrapped (`:495-510`);
- `ess-protospec/1`, a sidecar so that "existing projections cannot accidentally ignore a new
  protocol declaration" (`docs/design/protocol-specification.md:7`);
- `ess-normalization/N`, which holds the explicit fallback for absence (rule a's operation,
  `docs/design/source-pinned-data-normalization.md:24`) and the per-path number-token reading
  `binary64_inputs` (`docs/design/model-binary64.md:34`), separate "because operations, stage
  order, overflow and external dispatch are not schema annotations"
  (`source-pinned-data-normalization.md:73-80`); a stored record and a decoded document "must
  remain separate typed roots" (`:5-8`).

**The rules against that line.**

- b, c and d describe a document holding many entities: a key standing for an identity, a field
  written once for all, a root that is no entity. Everywhere else a `Bin`'s object carries its
  identity and its own `site` as members (`unique-wire-field-identity.md:17`), and that page adds
  "no speculative flattening or runtime decoder rules" (`:26-27`). These are a reading of one
  document form, the class `ess-http/1` holds for a body.
- e is one by the owner's own decision: TD-D9 keeps the strict decoder, and so the HTTP wire,
  string-only. Declared on the field in `ess/N`, it is a key every reader but one ignores, the
  outcome `protocol-specification.md:7` placed a sidecar to prevent.
- a alone fits either side: like `shorthand:` it is a spelling of one value. In `ess/N` it binds
  every reader: the HTTP decoders, the interpreter, synthesis and the schema (`default`, member not
  required). Only the file form asks for it.
- Policy 2 of `docs/design/generic-language-features.md:598-602`: "A construct for one request
  does not get a format of its own." Policy 1 (`:592-597`) holds for a sibling too: it is pinned and
  checked against the compiled model, not prose.

| option | b–e | a | § Boundaries (`AGENTS.md:18-21`) |
|---|---|---|---|
| A. on the declarations, next free `ess/N` (TD-D11 as picked) | against every layout precedent above; each key needs a "document only" scope | fits (`shorthand:`) | holds; every projector answers or refuses each key |
| B. a sibling document pinned to the specification | as `ess-http/1`, `ess-transport/1` | fits, read by the document decoder only | holds: closed keys, concrete types, no property bag |
| C. a decoder option of the generator (flags) | no declaration a second generator or reviewer can read | as b–e | a table in a generator, the shape `enum-variant-wire-names.md:15-16` removed |
| D. an `ess-normalization/N` recipe | paths reach fields and list items, not map entries (`model-binary64.md:38`); no cross-file set; runs over schema bundles, not the model crate | fits (`fallback`) | holds |

The precedents support B. The rest of the page specifies B. Under A the same keys sit on the
declarations (sketch in § 4), and § 6 lists what moves in addition.

## 4. Syntax and types (B)

```yaml
type: ess-document-form/1
specification: {system: depot, version: v1, source_digest: sha256:<hex>}
documents:
  - name: stock
    marker: {member: format, value: depot-stock/1}
    root_fields:
      - {member: site, fills: [depot.stock.Bin.site, depot.stock.Shelf.site]}
    kinds:
      - {member: bins, entity: depot.stock.Bin, keyed: true}
    children:
      - entity: depot.stock.Shelf
        within: bins
        member: shelves
        container: depot.stock.Shelf.bin
        identity: {join: [depot.stock.Shelf.bin, depot.stock.Shelf.code], separator: /}
    defaults:
      - {field: depot.stock.Shelf.rotation, variant: fifo}
      - {field: depot.stock.Bin.tags, empty: true}
    decimals: number_or_text
```

Under A the same content would be keys such as `document: {keyed: true}` on an entity and
`default: fifo` on a field, with the root (d) still needing a declaration of its own that no
`ess/N` construct has today.

Rust types (`ess-document-form` crate, names only): `RawDocumentFormSpec` and `DocumentFormSpec`,
`DocumentForm`, `FormMarker`, `RootField`, `KindMap`, `ChildList`, `ChildIdentity`, `AbsentDefault`
(`Variant`, `EmptyList`), `DecimalSpelling` (`Text`, `NumberOrText`), and the compiled
`DocumentFormIr`, persisted as `ess-document-form-ir/1`. The envelope `specification` is the closed
object `ess-transport` reads and checks (`crates/specify/ess-transport/src/lib.rs:515-523`).

**Refusals of the declaration.** Existing codes where the meaning is the same
(`generic-language-features.md:634` reuses `TypeMismatch` the same way):

| refusal | proposed code |
|---|---|
| a qualified name the compiled model does not declare | `undeclared_reference` |
| `variant:` not a variant of the field's enum; `empty:` on a field that is not a `List`; a default on an `Optional` (absence already reads absent) | `type_mismatch` |
| `keyed: true` on an entity whose identity is not spelled as text; `join:` into a field that is not `String` | `type_mismatch` |
| two rules that contradict: a field both a root field and a container field, filled and defaulted, or written by `join:` and keyed | `conflicting_declaration` |
| a member name used twice at one level (marker, root field, kind map, child list) | `duplicate_declaration` |
| a rule on an entity that no `kinds:` or `children:` entry reads | `unread_document_rule` (new) |
| `decimals: number_or_text` in a document that reaches no `Decimal` | `unread_document_rule` |

Under B these are the document's own numbered family, as `ess-http/1` numbers its refusals
(`crates/specify/ess-http/src/lib.rs:37`); under A `unread_document_rule` is a new `ValidationCode`.

**Refusals of the decoder** (a `DecodeError` naming the file label and key path): a marker that is
absent or differs; a root member no rule reads; the identity written inside a keyed entry; a field a
root field or container fills written by the entity; one id in two files, naming both labels; a
number token with an exponent or a leading `+`; a number token anywhere `decimals:` is `text`.

**Determinism.** One document has one reading:

- every member is read by exactly one rule or refused; each kind map and the root are closed;
- the result holds each kind as a map ordered by identity; file order picks no winner, because a
  repeated id is refused;
- a number token is read as its own text, `12.50` stays `"12.50"`;
- at every value position the document decoder admits every spelling the strict decoder admits
  and reads it the same; it adds only spellings the strict decoder refuses (an absent member under
  a default, a number token); where it changes a shape (keyed entry, filled field) it refuses the
  strict spelling. So a strict and a document decoder never disagree on a document both accept;
- each entity the document decoder returns, encoded strictly, decodes strictly to itself.

## 5. What each generator does

| consumer | with a document form | without one |
|---|---|---|
| Rust (`ess generate synthesize --target rust --document-form <file>`) | a `document` module beside the strict decoders: `decode_<name>_document` over a list of `(label, &json::Value)`, built on the per-type decoders of the depends-on story, which do not change | byte-identical |
| Go | refuses `--document-form` by name: Go's decoders exist only inside the served HTTP module (`go/http.rs:1320-1330`) (DF2-D4) | byte-identical |
| TypeScript | refuses by name: the web target decodes through the Rust code compiled to WebAssembly (`web/bridge.rs:1-10`) | byte-identical |
| JSON Schema, OpenAPI | strict schemas unchanged: they describe the wire (`enum-variant-wire-names.md:76`), `Decimal` a string. On request a separate schema of the document, the lenient form (DF2-D6); OpenAPI never, the document is no HTTP message | byte-identical |
| interpreter | unchanged; it reads commands and views, no document | unchanged |
| conformance | synthesis never sends a lenient spelling; suites unchanged. The form is held by the decoder's own tests (story acceptance) | unchanged |

## 6. Format consequence

**B.** No `ess/N`. A new document format `ess-document-form/1` and its IR, a row in
`website/docs/reference/formats.md`. An older `ess` refuses the flag and the document's `type:`; the
specification reads the same on every reader. A newer reader with an older specification refuses
a form whose `source_digest` names another source, as `ess-http/1` does. A changed specification
needs the form re-pinned. Pins that move: the typed-diagnostics count
(`docs/design/review-typed-diagnostics.md`) by the new crate's sites. Pins that do not move:
`RawSpecFile` and `schemas/generated/ess.schema.json` (`AGENTS.md:377`), `macro-guards.json`'s
digest of `validation_codes!` (`AGENTS.md:399-403`), `FORMAT_HISTORY` (`crates/specify/ess-domain/src/system.rs:54-57`),
every IR, digest, projection and suite byte of every specification.

**A, in addition.** The next free `ess/N` (a `FORMAT_HISTORY` row, `cargo xtask format-history`);
every new key refused below it as `unsupported_format_version`, while an older reader fails the file
on an unknown field; `RawSpecFile`, so `cargo xtask schema`; `unread_document_rule` in
`validation_codes!`, so `macro-guards.json`; an `ess-diff` kind per rule or each change falls to the
residual as `unclassified-changed`; every projector, the interpreter, Entity Runtime and the Go and
HTTP decoders answer or refuse each key. A specification declaring no rule keeps its bytes under
both options (`AGENTS.md:50-51`).

## 7. Out of scope and open decisions

Out of scope: rule f (a YAML 1.2 core reader yields a string for a date; the strict decoder reads
`Timestamp` from a string, to be confirmed by the first decoder test); rule g (TD-D10 A); a second
kind of root with placeholders; writing entities back into the document form; `ess-diff` over two
document forms.

- **DF2-D1, placement of rules b to e.** A: a sibling document `ess-document-form/1` pinned to the
  specification (§ 4). B: keys on the type, field and entity declarations in the next free `ess/N`
  (TD-D11 as picked), each read by the document decoder only. C: an `ess-normalization/N`
  extension. Recommended A. Architecture: the owner's.
- **DF2-D2, rule a.** A: in the sibling with b to e, read by the document decoder only. B: a
  `default:` on the field in the next free `ess/N`, read by every reader and stated in the schema,
  after `shorthand:`. Recommended A, B kept for a second surface that needs it.
- **DF2-D3, the crate.** A: a new crate `crates/specify/ess-document-form`, compiled against the
  IR like `ess-http`. B: a module of `ess-synth` with no compile step or IR. Recommended A.
  Architecture: the owner's.
- **DF2-D4, targets of the first cut.** A: Rust; Go and TypeScript refuse by name. B: Rust and Go,
  with Go decoders outside the server. Recommended A.
- **DF2-D5, rule e's scope.** A: one switch per document. B: per field. Recommended A: the grammar
  is one, and a per-field list can disagree with the fields it misses.
- **DF2-D6, a schema of the document.** A: on request, a separate output describing the lenient
  form. B: none. Recommended A.
