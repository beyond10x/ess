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
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/synthesize_layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/lib.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust
- confidence: cited
  path: crates/generate/ess-synth/src/web/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/tests
- confidence: inferred
  path: website/docs/guides/synthesize.md
- confidence: inferred
  path: website/docs/reference/cli.md
revision: 26
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

- TD-D1: A, taken by the coordinator on 2026-11-25 (D113), agreed by the operator session: the synthesized crate's own wire module, emitted without a server, dependency-free.

- TD-D2: A, taken by the coordinator on 2026-11-25 (D113), agreed by the operator session: the wire module is always emitted in `--layout crate`, at the crate root and outside the `server` feature.

- TD-D3: A, taken by the coordinator on 2026-11-25 (D113), agreed by the operator session: `--layout crate` only.

- TD-D4: A, taken by the coordinator on 2026-11-25 (D113), agreed by the operator session: the crate's root `json.rs`, emitted whenever the wire module is.

- TD-D5: A, taken by the coordinator on 2026-11-25 (D113), agreed by the operator session: an optional `--package <name>` for the Rust target in `--layout crate`; the system's name without it.

- TD-D6: A, taken by the coordinator on 2026-11-25 (D113), agreed by the operator session: a flag that leaves out the `[workspace]` table; the default is unchanged.

- TD-D13: an entity snapshot gets a decoder and an encoder in the wire module of the crate layout only (the workspace layout's bytes do not move, TD-D3 A); the snapshot decoder checks the entity's invariants, as a struct's decoder checks the struct's, taken by the coordinator on 2026-11-25 (D119); reversible until the release that carries it

- TD-D14: no crate-layout tree is committed in the repository (`git ls-tree` at 0.99.1), so the acceptance item `committed_trees_are_regenerated` reads: no committed tree and no repository-wide byte pin moves, taken by the coordinator on 2026-11-25 (D119); reversible until the release that carries it

- TD-D15: the one module is `crate::wire` beside `crate::json`; with the `server` feature `crate::server::wire` and `crate::server::json` stay as re-exports of them, so the names, signatures and old paths of a served crate keep compiling, taken by the coordinator on 2026-11-25 (D119); reversible until the release that carries it

- TD-D16: a crate with no component gets the codecs of every declared type, enum, newtype, entity snapshot, event payload and error; command and view codecs stay with a component that presents them, as today, taken by the coordinator on 2026-11-25 (D119); reversible until the release that carries it

- TD-D17: the plan's `scope` records the package name and the left-out workspace table as optional members that are absent by default, as `layout` is (`crates/generate/ess-synth/src/plan.rs:107-116` at 0.99.1), and the regenerate command names both flags; no format step, because the plan is written and never read back. The implementor verifies that no reader of `plan.json` exists; if one does and refuses unknown members, the unit stops and reports, taken by the coordinator on 2026-11-25 (D119); reversible until the release that carries it


- TD-D14 amended: the crate-layout pin rows are re-pinned once with the cause beside them: the three `rust-crate` rows of `served_codec_names` and `SHOP_CRATE_DIGEST` in `http_binding_rust` moved, because a crate-layout output gains `src/wire.rs` and `src/json.rs`; no workspace, web or Go row moved; taken by the coordinator on 2026-11-25 (D124)

- Accepted by the operator session under the owner's delegation, 2026-11-25 18:07 CET (a consumer waits on it).

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

Derived 2026-11-25 by `story-scoper`, read against tag `0.99.1`. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/generate/ess-synth/src/rust/` (`mod.rs`, `single.rs`, `layout.rs`, `feasibility.rs`, `wire.rs`, `http.rs`) — cited
- **Where the crate layout is written (question 1):** — cited
  - `rust/mod.rs:177` (`single_crate_bound`) calls `emitted(.., single_crate = true)` (`:187-283`).
  - `emitted` builds a workspace: `workspace_manifest` (`:325`), `crate_manifest` (`:352`), `lib_module` (`:367`), `primitives.rs`, `json.rs` only when `json::used(ir)` (`:218-227`), `obligation`, `actor`, `behaviour`, one module per bounded context, ports, `system`, then `http::server_crate` (`:264`).
  - `single::relayout` (`single.rs:107-217`) then folds it into one crate. It drops both workspace manifests (`:133`, `:141`), writes `src/<file>` for the types crate, and writes `src/lib.rs` (`:214`).
  - The wire module is `src/server/wire.rs`, behind the `server` feature (`:189`, `:211-213`).
  - `rust/http.rs:127-135` (`server_crate`) returns nothing when no component is `reached_by: network`. Its `wire.rs` artifact is at `:174-182` (`wire::module` `:179`, then `system_event_encoder` `:272` and `system_event_decoder` `:309`). Its `json.rs` is at `:160-172`, a re-export when `Json` is used (`json.rs:1229`).
  - File list for the story's case: `Cargo.toml`, `src/lib.rs`, `src/<domain>.rs`, `src/primitives.rs`, `PLAN.md`, `plan.json`, and `.ess-output/state.json`.
  - Package name: `single.rs:249` (`ir.system().segments().join("-")`). Workspace table: `single.rs:269` (`out.push_str("\n[workspace]\n")`).
  - The same system name is the library name that `src/bin/*-server.rs` is rewritten to use (`single.rs:178`). `--package` must reach it too.
- **`generate types` (question 2):** — cited
  - Path: `ess-cli/src/model_types.rs:89` (`run`), then `schema_bundle.rs:245` (`type_files`), then `schema-contract/src/realize.rs:1245/1251` (`Plan::rust`, `rust_with`), then `realize/rust.rs` (serde `CODECS` `:14`, newtype `Deserialize` `:949`).
  - `--package` is `schema_bundle.rs:138-140`. It is required for Rust (`:288-291`). Its name check, `rust.rs:845` `package_name`, is `pub(super)`.
  - Nothing is reusable as it is. It decodes serde-derived types built from the published JSON schema (`ModelTypes`). The synthesized crate's types derive nothing (`wire.rs:5-12`) and are typestate types.
  - Under TD-D1 A (chosen) the story adds **no dependency edge**, and the generated crate keeps `[dependencies]` empty.
  - Edges the rejected options would add: `ess-synth` to `schema-contract`, and `serde` in the generated crate. `AGENTS.md` § Crate tree at 0.99.1 has no rule against `ess-synth` to `schema-contract`. TD-D1 B and C are marked "architecture: the owner's" (they also drop the zero-third-party rule of `json.rs:7`). Not designed around.
- **Files and symbols by acceptance item (question 3):**
  - `a_crate_with_no_component_decodes` — cited
    - `rust/mod.rs`: `emitted` pushes a types-crate `wire.rs` when `single_crate` and `http::served(ir)` is empty. `lib_module` `:398-415` adds `wire` and `json` to the module list. `json.rs` is emitted whenever the wire module is (`:218`, with `json::types_module`).
    - `single.rs`: `relayout` moves a served model's `wire.rs` and `json.rs` from `src/server/` to the root, and `ROOT_MODULES` (`:40`) gains `wire` and `json`.
    - `layout.rs:411-424` (`module_idents`) reserves `wire` and `json` for the crate layout.
    - `feasibility.rs`: `wire()` runs only when served (`:280-282`), so a library crate needs it too. Fixed-module symbols and paths (`:338-346`, `:373-380`).
    - Entity snapshots: `wire.rs` has no entity codec (no match for `entit` in `wire.rs` or `http.rs`). New code in `wire::module` (`:63-104`), a new method on the `Surface` trait (`:1098`), and its two impls: `Server` (`http.rs:75`) and `Bridge` (`web/mod.rs:255`). The snapshot is `<Name>Snapshot` with `state` and `data: <Name>Data` (`entity.rs:333-349`).
  - `an_invalid_value_is_refused_at_its_path` — cited. Already in `wire.rs`: `newtype_body` `:210`, `struct_body` `:328` (closure `:343`, `broken_constraint` `:366`), `model_name_refusals` `:929`. No change except the entity snapshot.
  - `the_package_is_named_by_its_consumer` and `the_workspace_table_is_left_out_on_request` — cited
    - `ess-cli/src/main.rs:320-343` (`Synthesize` args), `:1795`, `:3733-3761` (`synthesize`).
    - `ess-synth/src/lib.rs:430-511` (`synthesize_bound`, `emit`). `rust/mod.rs:161-184` (`single_crate*`).
    - `single.rs:243-271` (`manifest`).
    - `plan.rs:98-126, 487-492` (`SynthesisScope`, `regenerate`).
  - `a_served_model_keeps_its_decoders` — cited. `http.rs:414` and `:487` write `pub mod wire` and `use crate::{entry, http, json, wire}`. `rewrite` (`single.rs:279`) turns that into `crate::server::wire`. Moving the file needs a special case there.
  - `the_doc_comment_stays_true` — cited. `rust/items.rs:69`, `:107`. The sentence is still in 0.99.1 and in `generated/rust/{billing,gatepass}`.
  - `committed_trees_are_regenerated` — see question 5.
- **Tests that exist (question 3):** — cited
  - Crate layout: `ess-synth/tests/single_crate_layout.rs` (asserts `src/server/{http,wire,entry,json}.rs` at `:227-233`; builds with cargo at `:344-386`), `ess-cli/tests/synthesize_layout.rs`, `output_ownership.rs:245`, `ownership_relocation/mod.rs:38`.
  - Seven `ess-synth` tests turn `x_server::` into `x::server::` for the crate layout, so a moved `wire` or `json` breaks them: `declared_behaviour.rs:1478`, `external_request_binding.rs:278`, `feasibility.rs:1889`, `generated_view_queries.rs:356`, `optional_recursive_codecs.rs:510`, `served_codec_names.rs:706`, `served_publication.rs:537` (`http_binding/shared.rs:614` names `shop_server::wire`).
  - Decoder tests: `invariant_check` (`struct_invariants`, `distinct_invariants`, `value_constraints`), `integer_spellings`, `served_codec_names`, `json_body_closure`.
  - `generate types`: `ess-cli/tests/model_types.rs`, `schema-contract/tests/rust_realization.rs`, `normalization_rust.rs`.
  - Byte pins that do NOT move: `generated/rust/*` and every AGENTS.md § Gate pin (`synthesis_suite_bytes_table*`, `adversary_related_values_all_bytes`, `external_beside_held_guard`, `served_codec_names`, `go`, `adversary_ess_issues_2_u6`, `string_alphabet_and_length`, `tutorial_page`, `feasibility`, `adv319`, `http_binding_rust`). They hold workspace or other-target bytes, and TD-D3 A leaves those alone.
  - Moves: the doc-comment story's sentence moves `generated/rust/{billing,gatepass}` (`invoice.rs:77`, `visit.rs:35`) only if that story lands first.
  - Also: `cargo xtask cli-reference` for `website/docs/reference/cli.md:944` (checked by `projection-check`).
- **Units (question 4):** each goes red test first.
  - **U1 `--package` and the workspace flag.** Red: an `ess-synth` test on `Cargo.toml` (name, no `[workspace]`, default bytes unchanged) and `synthesize_layout.rs`. Touches `main.rs`, `lib.rs`, `plan.rs` (`regenerate` and `scope`), `rust/mod.rs` (signatures), `single.rs` (`manifest`, `:178`).
  - **U2 root `wire` and `json` without a server.** Red: a crate-layout test for a model with no component expecting `src/wire.rs`, `src/json.rs` and `pub mod wire`. Touches `rust/mod.rs`, `single.rs`, `layout.rs`, `feasibility.rs`, `single_crate_layout.rs`, the seven `.replace` tests.
  - **U3 entity snapshot codecs.** Red: a byte test on `wire::module`. Touches `wire.rs`, `http.rs:75`, `web/mod.rs:255`; `generated/web/billing` must not move.
  - **U4 build-once round trip and docs.** A Taskfile step generates and builds the crate once; a test runs the binary named by an `ESS_*` variable (pattern: `integer-spellings-harness`, `Taskfile.yml:180-200`). Also `website/docs/guides/synthesize.md:120-146`, `cli.md`, `CHANGELOG.md`.
  - **Conflicts:** U1 and U2 on `rust/mod.rs` and `single.rs`. U2 and U4 on `single_crate_layout.rs` only if U4 edits it. U3 touches none of U1's files.
  - **Order:** U1, then U2, with U3 beside U1. U4 last.
- **Left out by the story's text (question 5):** — cited
  - Workspace layout (TD-D3 A).
  - Go, web and clap targets.
  - Consumer gaps 4 to 6 (shorthand struct, document-form rules, per-run `state.json`).
  - The doc-comment fix (its own story).
  - Obligations of `generate types`.
- **Contradictions (not resolved):** — cited
  - Acceptance names `decode` and `encode` for "entity snapshot", but the wire module has none and TD-D1 A reuses it. Entity snapshot codecs are new. The story is silent on whether the snapshot decoder runs the entity's `invariants`, and on whether a served workspace gets them (it would move `generated/rust`).
  - `committed_trees_are_regenerated` names "every committed crate-layout tree". `git ls-tree 0.99.1` shows none under `generated/` or elsewhere. Only workspace trees are committed.
  - `a_served_model_keeps_its_decoders` ("name and signature") against TD-D2 A ("one module, not two"). Function names stay, but the path changes from `crate::server::wire::decode_x` to `crate::wire::decode_x`.
  - The story lists types, enums, newtypes and snapshots. `wire::module` also emits event, error, view and command codecs wherever `presents_*` is true. Which of those a model with no component gets is unsaid.
  - `plan.json` `scope`, `PLAN.md` and the file headers name the regenerate command (`plan.rs:487`). The story says nothing about carrying `--package` or the flag in it.
  - "No format" in Fit review 6, but `plan.json` `scope` gains fields when the flags are given.
- **Bounds for the implementor:**
  - No test over 60 s; the test step of a gate within 120 s.
  - A generated crate that a test compiles is built once per run (Taskfile step, path handed to the test), never per test.
  - No new test spawns cargo (`no_test_builds.rs`: `single_crate_layout.rs` is on the closed list; nothing is added).
  - A new harness task goes into `Taskfile.yml` in every place the existing ones are: `test` (`:78-99`), its hotfix copy (`:116-133`), `test-archive` (`:166-173`), the shard env (`:414`). `ci_lanes.rs` checks them.
  - No Python.
  - No time estimates.
- **Documents:** `website/docs/guides/synthesize.md`, `website/docs/reference/cli.md` (generated), `CHANGELOG.md` — inferred
- **Confidence:** high for questions 1 to 3 (read at the tag). Medium for the unit split, since it depends on the open design points above.
- **Would collide with:** any unit touching `ess-synth`'s `rust/` emitter (`mod.rs`, `single.rs`, `wire.rs`, `layout.rs`, `feasibility.rs`), `plan.rs`, or `ess-cli`'s `Synthesize` arguments in `main.rs`.
- **Safety fact:** the workspace layout stays byte-identical because every new emission is gated on `single_crate` in `emitted` (`rust/mod.rs:187-192`) or lives in `relayout`, which only the crate layout calls. Pointed at the `file:line`, unproven.

### Not verified by the scoper

- Whether `ir.types()` holds the entity state enum (`entity.state_type`, `ir.rs:436`). That decides whether the state enum already gets a codec.
- How long `single_crate_layout.rs`'s cargo test takes. It builds three models, with and without `server`, against the 60 s bound. I ran nothing.
- Whether the `Taskfile.yml` env list at `:414` is the only shard env site. I did not read `ci.yml`.
- That `cargo xtask cli-reference --check` fails on a new flag. The task comment says so; I did not run it.
- Whether a root `wire` or `json` module collides with anything beyond `layout.rs:411-424` and `feasibility.rs`. I did not read `Layout::reference` or `port.rs` for module-name assumptions.
- Whether `decode_system_event` and `encode_system_event` can sit in the root module without the `server` feature. They need the `system` module, which exists only with components or bindings.
- I did not read `.engineering/` for newer decisions on `generated/` or on the doc-comment story.
- Story text used: the draft on `plan/gap-reports-2026-11-24` (revision 7) and `DECISIONS.md`. I did not open `gap.md`.

## Delivery

Commits `0a55c9b57f` (U1), `db3b964241` (U2), `0f54c9a516` (U3) and `b070c140ab` (U4) on the branch `feat/a-synthesized-rust-crate-carries-typed-decoders-without-a-server`, pushed. The whole check ran 21 lines, exit 0. No pull request yet: this story and the shorthand story ship in one (D127).

Pull request 836, head `aa04d59d34`, open when this was written; it adds no format key; flags `--package` and `--no-workspace-table`.

## Status

Pull request 836 (https://github.com/beyond10x/ess/pull/836) is open and not delivered (2026-11-27); `main` at `a861afd90e` (release 0.99.7) is merged into its branch. Not in any release.
