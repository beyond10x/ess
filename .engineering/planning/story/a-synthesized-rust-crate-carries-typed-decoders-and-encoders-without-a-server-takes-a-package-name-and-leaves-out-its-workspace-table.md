---
format: aep.planning-md/3
id: story:a-synthesized-rust-crate-carries-typed-decoders-and-encoders-without-a-server-takes-a-package-name-and-leaves-out-its-workspace-table
kind: story
status: draft
title: A synthesized Rust crate in the crate layout carries typed decoders and encoders without a server, takes a package name and leaves out its workspace table
summary: generate synthesize --layout crate writes the wire module only with a server; a consumer's model crate needs decoders, a name of its own and no [workspace] table (owner's order of 2026-11-25; TD-D1..TD-D6)
tags:
- external-request
- gap-reports-2026-11-24
- synthesize-rust
relations:
- serves: vision:O2
- informed_by: story:a-generated-type-s-doc-comment-names-a-decoder-only-where-the-crate-carries-one
- informed_by: story:generated-decoders-check-struct-invariants
- informed_by: story:generated-rust-and-go-decoders-check-a-structs-distinct-invariant
- informed_by: story:a-published-event-payload-has-a-generated-decoder
revision: 1
---
## Need

`ess generate synthesize --target rust --layout crate` emits typed decoders and encoders for the declared types without a server, so
a consumer's model crate is wholly generated: types, invariant checks, lifecycles and decoders. The crate takes the package name its
consumer gives it and carries no `[workspace]` table of its own, so it can be a member of the consumer's workspace. The owner ordered
the story on 2026-11-25 (about 03:18 CET): typed decoders from `generate synthesize` without a server. The consumer's smallest useful
slice is its first three gaps, which this story holds; the further gaps are the stories named under Fit review 9.

Measured by a consumer on 0.98.0: the synthesized crate holds a JSON reader and writer and no typed decoder; `ess generate types`
holds decoders and leaves 116 obligations to the consumer, 12 of them model invariants (55 statements).

What the code holds at `origin/main` (release 0.99.0):

- The Rust target's types derive no `Serialize` or `Deserialize`: the wire module is the crossing, emitted beside them
  (`crates/generate/ess-synth/src/rust/wire.rs:1-23`).
- The wire module (`decode_<type>`, `encode_<type>`, `decode_event_<event>`) is written only by the server crate:
  `crates/generate/ess-synth/src/rust/http.rs:127-135` (`server_crate` returns nothing when no component is `reached_by: network`)
  and `:176-183` (the `wire.rs` artifact).
- In `--layout crate` that file lands at `src/server/wire.rs`, and the whole server module is behind the `server` feature
  (`crates/generate/ess-synth/src/rust/single.rs:13-14`, `:189`).
- The JSON reader and writer is `crates/generate/ess-synth/src/rust/json.rs`: dependency-free by design (`json.rs:7`: the emitted
  tree builds with zero third-party crates), written into the types crate only when a model uses `Json` (`rust/mod.rs:218-224`).
- `ess generate types` is another generator: `crates/edge/ess-cli/src/model_types.rs:89` (`run`), through
  `crates/edge/ess-cli/src/schema_bundle.rs:245` (`type_files`) and `crates/generate/schema-contract/src/realize/rust.rs` (serde
  decoders, `CODECS` at `:14`; the newtype decoder calls `broken_constraint` at `:949`). Its obligations are listed in
  `types-report.json` (`realize.rs:1282`, `ess-types-report/3`). It shares no code with the wire module, and has `--package`;
  `generate synthesize` has none.

## Reported case

From a consumer, 2026-11-25, on ess 0.98.0, minimal specification of three files (system `shop`, one domain `shop.catalog` with a
newtype, an enum, two structs one of which has `any: [defined(min), defined(max)]`, and one `provided` entity with a one-state
lifecycle; no component, no `--http`). Re-run on a 0.99.1 build (0.99.0 plus three synthesis fixes and one UI check fix, none in the
generators), 2026-11-25 03:22 CET, a copy of the specification and the output under a scratch directory:

```
$ ess generate synthesize --path spec --target rust --layout crate --out out1/crate
6 capabilities: 6 generated, 0 obligation(s), 0 refused
6 artifact(s), written to out1/crate
files: Cargo.toml  PLAN.md  plan.json  src/lib.rs  src/catalog.rs  src/primitives.rs  .ess-output/state.json
Cargo.toml:6: name = "shop"    Cargo.toml:8: version = "1.0.0"    Cargo.toml:13: [workspace]
src/catalog.rs:42: /// ... the generated decoder refuses a value that breaks it.
```

- Gap 1, no typed decoder or encoder: `src/catalog.rs` holds the types, `Limit::broken_constraint` and the lifecycle, no function from a
  JSON value to `Entry`, `Limit` or the entity snapshot, and no `json.rs` (a model of no command and no `Json` gets none).
  The doc comment at `src/catalog.rs:42` names a decoder the crate does not carry
  (`story:a-generated-type-s-doc-comment-names-a-decoder-only-where-the-crate-carries-one` makes the comment true; it leaves the decoder).
  Needed: for every struct, enum, newtype and entity snapshot a `decode(&json::Value, at) -> Result<T, DecodeError>` and an `encode`,
  from `--layout crate` with no `--http` and no server feature; a decode refuses an undeclared member of a closed struct, ignores one
  where the struct says `undeclared_fields: ignored`, runs `broken_constraint` and the struct invariants, and names the key path of the
  first refusal; `json.rs` is emitted whenever a decoder is.
- Gap 2, the package cannot be named: `generate synthesize --help` lists no `--package`; the crate is named for the system. A
  consumer's binary crate named for the system cannot share a workspace with it. Needed: `--package <name>`, as `generate types` has it.
- Gap 3, the crate carries its own `[workspace]` table (`Cargo.toml:13`), which a member of the consumer's workspace must not.
  Needed: a switch that leaves the table out.

The consumer's own text is not copied here; the minimal specification is the test fixture of the acceptance below.

## Findings covered

- Gaps 1 to 3 of one consumer's request of 2026-11-25. Its gaps 4 to 6 are the stories named under Fit review 9.

## Fit review

1. **Need.** A crate generated for a specification with no served component decodes and encodes its own types, so the consumer writes
   no decoder and no invariant check by hand; it names the package and takes it into its own workspace. The request names a command
   and flags (`generate synthesize`, `--layout crate`, `--package`); they are evidence of the need, not the design.
2. **Class.** A feature. The wire module's header promises a pair for every type (`wire.rs:19-21`) only where it is emitted at all.
3. **Expressible today.** Partly, and not enough. `ess generate types --target rust` gives serde decoders as a second crate with its
   own types and 116 obligations in the measured case; a specification with a component `reached_by: network` gives the wire module,
   with a transport and a `server` feature the consumer does not want.
4. **Fit.** The generic feature is one decoder source for the Rust target: the wire module emitted for every presented type whether or
   not a server is generated. The synthesized crate already holds the pieces: the type with `broken_constraint` (`rust/items.rs`), the
   invariant preflight that refuses an invariant no decoder can check (`rust/mod.rs`, `invariant::preflight`), the reader and writer.
   Which obligations of `generate types` a synthesized crate already discharges is not claimed here: the minimal specification
   gives `0 obligation(s)` on the synthesized side, and the 116 of the consumer's real model are measured on it when the consumer
   reports them. `--package` and the workspace switch are options of the same command and the same crate layout.
5. **Second adopter.** Any specification generated as a library with no served component: the doc-comment story found one on
   0.98.0 (a struct with a struct invariant, three outputs, no decoder).
6. **Cost.** No format, IR or diagnostic. Generated bytes of every `--layout crate` output change when the wire module is emitted by
   default (TD-D2), so committed trees and byte pins regenerate. Docs: the wire module's header and the layout section of
   `website/docs/guides/synthesize.md`. Release train: next minor.
7. **Alternatives.** Re-use what `generate types` writes (TD-D1 B); one emitter for both generators (TD-D1 C); a switch (TD-D2 B).
8. **Existing stories.** None covers the need; four border it, each by evidence in the store:
   - `story:a-generated-type-s-doc-comment-names-a-decoder-only-where-the-crate-carries-one` borders: it makes the comment true
     "without" this change and says FM-D9 B (a wire module in the crate layout without a transport) "is recorded as a draft only and
     has no story"; this is that story, ordered by the owner.
   - `story:generated-decoders-check-struct-invariants` borders (implemented): the struct invariant check lives in the decoders that
     exist, inside a served wire module (`rust/wire.rs:328`, `struct_body`); this story gives those decoders a home without a server.
   - `story:generated-rust-and-go-decoders-check-a-structs-distinct-invariant` borders (active): `distinct:` is checked in the same
     decoders (`rust/invariant.rs:683`); its checks reach a types-only crate when this story emits the wire module there.
   - `story:a-published-event-payload-has-a-generated-decoder` borders (implemented): `decode_event_<x>` and `decode_system_event`
     are written by the served wire module only; whether a library crate carries them is TD-D2's scope.
   No amendment of an existing story is needed: none claims the need.
9. **The consumer's further gaps** are their own stories, each with its edge to this one: a shorthand struct in the Rust target (gap 4),
   the document-form rules (gap 5: a to d need a source format step, e to g do not), and the per-run difference of
   `.ess-output/state.json` (gap 6, a determinism defect).

## Decisions

Open; recommended options are in the store's decision record for this story. A decision that changes the crate tree's dependency
rules (`AGENTS.md` § Crate tree) is marked "architecture: the owner's".

- TD-D1, one decoder source. A: the synthesized crate's own wire module (`ess-synth`), emitted without a server; it stays
  dependency-free. B: the synthesized crate re-exports what `generate types` writes: the crate takes `serde`, `ess-synth` depends on
  `schema-contract`, and the zero-third-party rule of `json.rs:7` is dropped (architecture: the owner's). C: one decoder emitter
  both generators call (architecture: the owner's). Recommended A.
- TD-D2, when the wire module is emitted in `--layout crate`. A: always, at the crate root and outside the `server` feature; a served
  model keeps one module, not two. B: only when a new flag asks (FM-D9 B of 2026-11-24, taken under the owner's delegation before his
  2026-11-25 order, which names no flag). C: always, but behind the `server` feature as today. Recommended A.
- TD-D3, which layouts. A: `--layout crate` only, as ordered; a types-only workspace stays without it. B: both Rust layouts.
  Recommended A.
- TD-D4, where the JSON reader and writer live when the wire module is emitted without a server. A: the crate's root module
  `json.rs`, emitted whenever the wire module is. B: the wire module carries its own copy. Recommended A (one `Value` type).
- TD-D5, `--package`. A: an optional `--package <name>` for the Rust target in `--layout crate`; without it the name is the system's, as
  today. B: also in the workspace layout, as the name prefix of its crates. C: required, as in `generate types` (a change for every
  existing caller). Recommended A.
- TD-D6, the `[workspace]` table. A: a switch (flag) that leaves it out; the default is unchanged. B: the crate layout never writes
  it (every standalone build then fails inside another workspace's tree). Recommended A.

## Acceptance

- a_crate_with_no_component_decodes: `--layout crate` on the minimal specification of the Reported case writes `decode` and `encode`
  for every struct, enum, newtype and entity snapshot; the crate builds with warnings denied and `decode(encode(v)) == v` for each.
- an_invalid_value_is_refused_at_its_path: a value breaking an alphabet, a length, a prefix, a struct invariant (the case's
  `any: [defined(min), defined(max)]`) or a `distinct:` invariant is refused at its key path, in the words the served decoders use;
  an undeclared member of a closed struct is refused, and is ignored where the struct says `undeclared_fields: ignored`.
- the_package_is_named_by_its_consumer: with the TD-D5 option taken, `Cargo.toml` carries the given name; without the flag it is the
  system's, byte for byte as today.
- the_workspace_table_is_left_out_on_request: with the TD-D6 option taken, `Cargo.toml` holds no `[workspace]` and the crate builds as
  a member of an enclosing workspace; without the switch the table is as today.
- a_served_model_keeps_its_decoders: for a model with a component `reached_by: network`, every decoder keeps its name and signature
  and the server's request handling compiles unchanged.
- the_doc_comment_stays_true: the sentence of the doc-comment story is true in the new crate.
- committed_trees_are_regenerated: every committed crate-layout tree is regenerated in the same change and `task projection-check`
  passes.

## Scope

- cited: `crates/generate/ess-synth/src/rust/http.rs` (`server_crate`, 127-183), `crates/generate/ess-synth/src/rust/single.rs`
  (the `server` destination), `crates/generate/ess-synth/src/rust/wire.rs`, `crates/generate/ess-synth/src/rust/json.rs`,
  `crates/generate/ess-synth/src/rust/mod.rs` (`emitted`, `crate_manifest`, `workspace_manifest`),
  `crates/edge/ess-cli/src/main.rs` (the `synthesize` arguments)
- inferred: `website/docs/guides/synthesize.md`, `crates/generate/ess-synth/tests/single_crate_layout.rs`, `CHANGELOG.md`
- not touched unless TD-D1 B or C: `crates/generate/schema-contract/src/realize/rust.rs`, `crates/edge/ess-cli/src/schema_bundle.rs`
