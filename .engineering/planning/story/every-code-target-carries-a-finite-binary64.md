---
format: aep.planning-md/3
id: story:every-code-target-carries-a-finite-binary64
kind: story
status: draft
title: Every code target serves a finite Binary64 as a JSON number
summary: Rust, Go and web synthesis refuse the whole workspace for any Binary64; the checked finite codec the standalone libraries ship is carried into every value position of the generated servers
tags:
- adopter-report
- ess-0.97.1
relations:
- decomposes: epic:adopter-gaps-generated-code
- serves: vision:O2
- informed_by: story:binary64-structural-codecs
- informed_by: story:a-binary64-value-round-trips-through-a-conformance-suite
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/generate/ess-synth/src/failure.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/invariant.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/layout.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/mod.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/store/identity.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/invariant.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/items.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/layout.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/store/identity.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/view_query.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/go_owner.rs
- confidence: inferred
  path: docs/design/binary64-structural-codecs.md
- confidence: inferred
  path: docs/design/model-binary64.md
revision: 2
---
## Outcome

`ess generate synthesize` emits a Rust (workspace and `--layout crate`, with or without `--http`),
Go and web workspace for a model that declares a `Binary64` anywhere a value travels: a struct or
newtype, an entity field, a command input or response, an event or error field, a view field or
row, under `Optional` and `List`. On the wire the value is a JSON number in and a JSON number out;
a number that does not round to a finite binary64 is answered malformed. The OpenAPI document the
server serves already says `type: number` for it.

## Need

A downstream specification declares a relevance score from 0 to 1 as `Binary64`, inside a struct a
view returns, and wants its backend generated. Every code target refuses the whole workspace.

Reproduced on 0.97.1 with a model of our own (`format: ess/37`, domain `demo.search`: struct `Hit
{label: String, score: Binary64}`, entity `Doc {added_ms: Integer, hit: Hit}`, command `AddDoc`, an
unpaged view of `Doc`, component `reached_by: network`, an `ess-http/1` binding with one view and one
command route). `ess specify validate` exits 0; then:

```text
$ ess generate synthesize --path system.yaml --target rust --layout crate --http http.yaml --out out/rust
rust target cannot emit this workspace
types.demo.search.Hit.fields.score: this target has no qualified finite Binary64 codec
$ ess generate synthesize --path system.yaml --target go --http http.yaml --out out/go
go target cannot emit this workspace
types.demo.search.Hit.fields.score: this target has no qualified finite Binary64 codec
$ ess generate synthesize --path system.yaml --target web --out out/web
web target cannot emit this workspace
types.demo.search.Hit.fields.score: this target has no qualified finite Binary64 codec
```

Every position is refused the same way: `Optional<Binary64>` and `List<Binary64>` name
`types.demo.search.Hit.fields.score.of`; a newtype `of: Binary64` with invariants names
`types.demo.search.Score.of`; the same model with `Binary64` on the entity field, the command input
and the view field names all four positions (`command.….input.added_ms`, `entity ….fields.added_ms`,
`types.….score`, `view.….fields.added_ms`). With the score declared `Integer` the same model
synthesizes on all three targets (12 capabilities generated, 0 refused).

## Fit review

1. **Need.** A finite IEEE double is a domain value (a relevance, a ratio, a measurement) that a
   generated server must accept and answer as a JSON number, refusing NaN and infinities. The
   requester's words (theirs): "a finite Binary64 codec for rust/go/web (JSON number in, JSON number
   out, NaN and infinities refused), or the declaration form that qualifies one".
2. **Class.** Gap. `Binary64` is a declared primitive since `ess/2` and the design names the
   whole-system codec as separate, unbuilt work: "Add full-system codec support separately if it is
   required" (`docs/design/model-binary64.md:126`); "whole-system synthesis and conformance retain
   their existing refusals" (`docs/design/binary64-structural-codecs.md:5`). The refusal is
   `crate::failure::binary64` (`crates/generate/ess-synth/src/failure.rs:156-176`), called first by
   every target (`lib.rs:488`, `rust/mod.rs:192`, `go/mod.rs:319`, `web/mod.rs:300`,
   `clap/mod.rs:63`), and it reads the structural inventory `ess_compiler::binary64::locations`, so
   a position nothing reaches is refused too (`model-binary64.md:170-172`).
3. **Already expressible?** No declaration keeps the wire a JSON number and synthesizes. `Decimal`
   synthesizes on all three targets (tested, 0.97.1) but the served OpenAPI makes the field a
   string (`type: string`, `format: decimal`, a decimal-string pattern) and the Go type is
   `primitives.Decimal`, "carried as its wire rendering — a decimal string": that changes the wire,
   so it is no idiom. A scaled `Integer` changes the wire and the meaning. `ess generate types
   --target rust|go` already emits the checked finite codec for the same model (`EssBinary64`,
   `NewEssBinary64`, tested), but as a standalone data library, not a server. `Number` is no type
   (`ESS-TYPE-001`).
4. **Fit.** The cell is the `Binary64` column of the code-target row: today every code target
   refuses the whole column. The decision opens the whole column on Rust, Go and web (web reaches
   the Rust target's crates, `crates/generate/ess-synth/src/web/layout.rs:1-14`), for every
   position the conformance suite already carries (`ess_compiler::binary64::uncarried_uses`,
   `crates/specify/ess-compiler/src/binary64.rs:53-61`: entity fields, command input and response,
   event and error fields, view fields and row). It reuses, not reinvents, the checked wrapper the
   standalone libraries ship (`crates/generate/schema-contract/src/realize/rust_binary64.rs.txt`,
   `go_binary64.go.txt`; API in `binary64-structural-codecs.md` § Checked native API). Siblings:
   map keys stay refused by the compiler (`binary64-structural-codecs.md:36-37`); a view parameter
   or filter on a `Binary64` keeps the query an obligation (`view_query.rs:204`), named; Clap gets
   the same codec or keeps its refusal by name; Entity Runtime is out of the seam. No source key,
   no format version, no new diagnostic.
5. **Second adopter.** A sensor service: `reading: Binary64` in an event payload and a view of the
   latest readings, served as JSON numbers.
6. **Cost.** No source format, IR or suite format change: suites carry a reached `Binary64` from
   `ess-conformance/62` (`model-binary64.md` § Suite contract). Generated code for models without
   `Binary64` keeps its bytes (the wrapper is emitted only where used, `model-binary64.md:115`).
   Generator work, read from the code, rough: about 20 `Primitive::Binary64` arms that are now
   `unreachable!` or refusals in `crates/generate/ess-synth/src` (`rust/layout.rs:522`,
   `rust/wire.rs:863,879,1011,1042`, `go/layout.rs:1217`, `go/http.rs:998,1014,1179,1204`, plus
   `rust/invariant.rs:756`, `rust/feasibility.rs`, `rust/store/identity.rs`, `go/invariant.rs`,
   `go/store/identity.rs`, `go/mod.rs` constructors, `view_query.rs:204`, `determined.rs`), the
   fixed primitives modules of both targets, and the generated Rust items, which derive `Eq`
   (`rust/items.rs:76,115`) where `EssBinary64` derives only `PartialEq`. About 14 files in one
   crate, plus the `ess-synth` byte pins.
7. **Alternatives.** (a) Change nothing: the requester declares `Decimal` and changes its wire, or
   keeps hand-written handlers. (b) A declaration form that "qualifies" a codec (a key on the field):
   refused, because the codec is the target's duty for a type ESS already defines, and a key would
   leave every other `Binary64` refused. (c) The finite codec on every code target for every
   position the suite carries: chosen.

## Decisions

accept, redesigned. The need (a finite double as a JSON number in a generated server) is adopted
as one generic feature: every code target carries `Binary64` in every value position. The
requester's second option, a declaration form that qualifies a codec, is refused: no new key; the
existing type gets its codec.

Design points the implementation must settle and write into `docs/design/model-binary64.md` (a new
section "Whole-system codecs"):

- Decoding reads the number token from the raw request text (as `rust-json-keeps-raw-text` keeps
  it) and rounds to nearest-even; a token that overflows to an infinity is answered malformed
  (`400`), naming the field. JSON has no NaN token; a constructed NaN is refused by the checked
  constructor.
- Encoding writes a finite round-tripping spelling and keeps `-0.0`
  (`model-binary64.md:19`).
- Equality in the generated types is IEEE on finite values (`+0 == -0`), so the wrapper may
  implement `Eq` lawfully; `Hash` and the in-memory store's identity key canonicalize `-0` to `+0`,
  or a `Binary64` identity stays refused by name.
- Ordering a view by a `Binary64` field ranks as the runner's `ranked` does.

## Acceptance

- The reproduction above synthesizes on `--target rust --layout crate --http`, `--target rust`
  (workspace), `--target go --http` and `--target web`: exit 0, no cause naming `Binary64`.
- The generated Rust and Go servers decode `{"label":"a","score":0.73}` and answer the view with
  `"score":0.73`; `-0.0` round-trips as `-0.0`; `1e400` is answered `400` naming the field; a
  string `"0.73"` is answered `400`. One `ess-synth` test per target runs the built server.
- The reproduction's conformance suite (`ess verify conform synthesize`, at `ess-conformance/62` or
  later) passes against the generated Rust and Go servers, with `Optional<Binary64>` and
  `List<Binary64>` variants of the model.
- A `Binary64` map key, a `Binary64` view parameter and a `Binary64` identity keep a refusal or an
  obligation that names the position; none emits code that compiles and ignores it.
- Every committed example tree under `generated/` keeps its bytes (`ess-synth`'s `feasibility`,
  `served_codec_names`, `go` byte pins green).
- `docs/design/model-binary64.md` and `binary64-structural-codecs.md:5` no longer say whole-system
  synthesis refuses `Binary64`; `CHANGELOG.md` says what is served.

## Out of scope

- Arithmetic on `Binary64` (grid O row `+ - * /`: no) and `Binary64` measures.
- Cross-language byte-identical float spelling (`model-binary64.md:19`).
- TypeScript standalone types, which keep their reported obligation
  (`types-report.json` rule `model_binary64`).
- Paged views: `story:a-generated-server-serves-a-paged-view`.

## Scope

- inferred: crates/generate/ess-synth/src/failure.rs
- inferred: crates/generate/ess-synth/src/rust/layout.rs
- inferred: crates/generate/ess-synth/src/rust/wire.rs
- inferred: crates/generate/ess-synth/src/rust/items.rs
- inferred: crates/generate/ess-synth/src/rust/invariant.rs
- inferred: crates/generate/ess-synth/src/rust/store/identity.rs
- inferred: crates/generate/ess-synth/src/go/layout.rs
- inferred: crates/generate/ess-synth/src/go/http.rs
- inferred: crates/generate/ess-synth/src/go/mod.rs
- inferred: crates/generate/ess-synth/src/go/invariant.rs
- inferred: crates/generate/ess-synth/src/go/store/identity.rs
- inferred: crates/generate/ess-synth/src/view_query.rs
- inferred: crates/generate/ess-synth/tests/go_owner.rs
- inferred: docs/design/model-binary64.md
- inferred: docs/design/binary64-structural-codecs.md
- inferred: CHANGELOG.md
confidence: medium
