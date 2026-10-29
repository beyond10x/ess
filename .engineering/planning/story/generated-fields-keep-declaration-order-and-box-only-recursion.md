---
format: aep.planning-md/3
id: story:generated-fields-keep-declaration-order-and-box-only-recursion
kind: story
status: implemented
title: Generated fields keep declaration order and box only recursion
summary: 'Adopter report N7: Generated Rust boxes every reference and sorts fields (ess#661).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#661
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/schema_bundle.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/model_type_events.rs
- confidence: cited
  path: crates/generate/ess-gen/src/model_types.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/model_types.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/go.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/rust.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/ts.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/declaration_order.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/rust_generator_options.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/rust_realization.rs
- confidence: cited
  path: docs/design/types-only-realizations.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/cli.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T05:17:21Z", actor: "human:timo", revision: 6, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
- {from: "proposed", to: "active", at: "2026-10-29T05:17:22Z", actor: "human:timo", revision: 7, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
- {from: "active", to: "implemented", at: "2026-10-29T10:39:46Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
---
## Problem

Generated types list a record's fields alphabetically by wire name, and the Rust target boxes every reference to a named type. Adopter report N7, ess 0.68.0.

Corrected against 0.76.0 (the issue's `rust.rs:253` and `:409` are stale):

- the realizer reads an object's `properties` from a sorted `serde_json::Map` into `Shape::Object { fields: BTreeMap<String, Field> }` (`crates/generate/schema-contract/src/realize.rs:490-493`, filled in `kind()` at `:1573-1606`);
- the Rust target writes every `Shape::Ref` as `::std::boxed::Box<T>` (`crates/generate/schema-contract/src/realize/rust.rs:676`).

## Fit review

1. **Need.** A generated type should list its fields in the order the specification declares them, and hold a named type by value unless the reference closes a cycle. No syntax was proposed. Reproduced with the 0.76.0 release binary in a repro outside the repository: a struct `repro.g.Outer` declaring `zeta: Inner, alpha: String, mid: Optional<Inner>`, `ess generate types --all-types` for each target.
   - Rust: `alpha`, `mid`, `zeta`, with `pub mid: EssPresence<::std::boxed::Box<ReproGInner>>` and `pub zeta: ::std::boxed::Box<ReproGInner>`.
   - Go: `Alpha string`, `Mid EssPresence[*ReproGInner]`, `Zeta *ReproGInner`.
   - TypeScript: `"alpha"`, `"mid"?`, `"zeta"`.
   - `ess generate --kind schema` on the same file keeps `zeta`, `alpha`, `mid` in `repro.g.Outer.schema.json`. The `source.schema.json` that `ess generate types` writes is sorted, because `ModelTypes` goes through `serde_json::to_value` (`crates/generate/ess-gen/src/model_types.rs:119`).
2. **Class.** Field order: defect. The typed projection keeps declaration order on purpose (`crates/generate/ess-gen/src/types.rs:430-436`, `Properties`), and `ModelTypes::select` loses it at `model_types.rs:113-137` (`to_value`). The workspace `serde_json` has no `preserve_order` (`Cargo.toml:99`). Boxing: convenience. Every nested access goes through a `Box`, and a consumer has to write `Box::new(..)` for a field that is not recursive.
3. **Already expressible?** No. `RustOptions` (`realize.rs:74-94`) and the `ess generate types` flags (`crates/edge/ess-cli/src/schema_bundle.rs:124-156`) control neither order nor boxing.
4. **Fit.** There is no authored surface. The precedent is the Rust generator options of beyond10x/ess#657, #667 and #668. Each is a command-line flag that is off by default and recorded in the report's `configuration` only when given. With all of them off the output keeps its bytes (`realize.rs:74-76`, `docs/design/types-only-realizations.md:259-262`, `website/docs/guides/generate-artifacts.md:373-374`). Private projection metadata minted from typed input is precedented by `ModelTypes::binary64_locations` (`model_types.rs:176-181`). Siblings:
   - Go: field order moves under the option. Its encoded bytes do not, because it marshals through a map. It keeps `*T`: a nil required pointer is how its encoder refuses a missing value (`go.rs:396`).
   - TypeScript: field order moves under the option. It has no boxing.
   - Bundle input (`Plan::from_bundle`, `realize.rs:572`) carries no declaration order, because its definitions are parsed into a sorted `Value` (`crates/generate/schema-contract/src/bundle.rs:182`). So the option is refused there by name and not silently ignored.
   - `ess generate client` (`crates/edge/ess-cli/src/client.rs:81-87`) calls `plan.rust`/`plan.go` with defaults, so it keeps its bytes.
5. **Second adopter.** Any model with a reference to a small value type (`Money`, `Address`) wants `invoice.total.amount` without a `Box`, and a reader comparing the generated struct with the specification wants the same field order.
6. **Cost, and D135.** As a default, both changes break consumers:
   - Rust's derived `Serialize` writes members in struct-field order, so reordering changes the key order of encoded JSON.
   - A derived `PartialOrd`/`Ord` (`--rust-derive Ord`) compares fields lexicographically in declaration order, so reordering changes how values sort.
   - A Go unkeyed composite literal `T{a, b}` stops compiling.
   - Removing `Box` changes public field types, so `Box::new(..)` at a non-recursive field stops compiling.

   Is key order on the wire a documented contract? No ESS page states the order Rust encodes members in. The design promises wire names, presence and open-object storage (`types-only-realizations.md:233-238`), and JSON objects are unordered (RFC 8259 §4). What is documented is that generated bytes do not move without an option (point 4), and that bundle input keeps its output byte for byte (`types-only-realizations.md:184-185`). A consumer hashing encoded JSON, or holding committed generated code, would see a change. Under D135 that rules out a default change. The chosen form costs two new flags, one new `Plan` method, one refusal rule and one new member each in the `configuration` of `ess-types-report/3`, which is absent by default. That is the same additive change #657 made without a format move. The authored, IR and schema formats do not change.
7. **Alternatives.**
   - (a) Change nothing.
   - (b) Change both defaults, as the issue asks ("ship together with a changelog entry"). Refused under D135 for the breaks in point 6.
   - (c) Enable `serde_json/preserve_order` workspace-wide. Refused: feature unification would reorder every canonical JSON output ESS writes.
   - (d) Box per type, as `ess-synth` does (`docs/design/optional-recursive-rust.md`). It exists for an accessor algebra this realizer does not have.
   - (e) Chosen: two opt-in options. Field order is recorded in a side table and read only under the option. Boxing is per edge, only where an edge closes a by-value cycle.

## Decisions

**accept, redesigned.** Both behaviours are opt-in, and the default output keeps its bytes (D135, `approval-record:d135-a-breaking-unit-leaves-the-batch`). No decision-blocker is needed. This replaces the batch-1 scoping design, which changed the default and moved Rust's key order on the wire.

- **Declared field order: `--declared-field-order`**, on `ess generate types` for `rust`, `go` and `typescript`.
  - `ModelTypes` mints `field_order: BTreeMap<String, Vec<String>>` (object pointer to wire names in declaration order) before `to_value`, beside the binary64 walkers (`model_types.rs:358-385`). The source is the typed `ResolvedField` order of a struct and an event payload, under the definition pointer; a shorthand struct is planned as its record under that same pointer (`realize.rs:220-223`). For `<cmd>.input`, `<cmd>.response` and `<view>.row` the source is `BodyRoot.fields`. Union variant objects (tag and content) have no declared order and are not entered.
  - The table is exposed by an accessor like `binary64_locations`. `ModelTypes::to_json` (`source.schema.json`) does not change.
  - `Plan::from_model` copies the table into `Plan`. `Plan::with_declared_field_order(self) -> Result<Self, Refused>`, beside `with_raw_json` (`realize.rs:902`), turns the option on. On bundle input it refuses with rule `declared_field_order_bundle` at `/`, because "a bundle carries no declaration order".
  - Emitters iterate the fields through one helper, `Plan::ordered_fields(pointer, fields)`. It gives wire-name order (today's) unless the option is on, and declared order when it is. Required-only members that are not in the table come after the declared ones, in wire-name order, as do fields of any object without an entry.
  - `Shape::Object` and `kind()` are **not** changed (no edit at `realize.rs:1558-1634`), so normalization and U6's `kind()` edits are untouched.
  - Rust: under the option, encoding writes members in declaration order. Decoding accepts any order. The decoded value is equal to the default build's. The doc and the CHANGELOG say that a derived `PartialOrd`/`Ord` then compares in declaration order.
  - Go: struct fields and the per-field encode and decode blocks follow the option. Encoded bytes are unchanged.
  - The report's `configuration` records `"declared_field_order": true` for each language only when the option is given. `TargetConfiguration::Typescript` becomes a struct variant whose only member is skipped when false, so its default bytes stay `{"language": "typescript"}`. Go gains the same member.
- **Rust boxing: `--rust-box-only-recursion`**, a `RustOptions` field `box_only_recursion: bool` (skipped when false). Other targets refuse it, like the other `--rust-*` flags (`schema_bundle.rs:227-238`).
  - Rule: build a by-value reference graph over `plan.definitions`. Every child is a by-value edge except array `items` (`Vec`) and object `additional` (`BTreeMap`). Optional, nullable, union variants, tuple prefix and anonymous objects all count as by-value, and an anonymous node belongs to its owning definition.
  - A `$ref` node in definition `D` to `T` is boxed if `T == D` or `T` reaches `D` in that graph. The result is a set of `$ref` node pointers, computed by a new `by_value_references` helper beside `children`/`reaches` (`realize.rs:1747-1790`).
  - `rust.rs:676` boxes a reference when the option is off (today) or when its pointer is in the set.
  - So: a mutual cycle boxes both edges; a reference into a cycle from outside it is held by value; `Optional<Self>` gives `EssPresence<::std::boxed::Box<Self>>`; `List<Self>` gives `Vec<Self>`.
- **Changed from the request:** both behaviours are opt-in rather than new defaults. Go keeps pointers. Bundle input refuses `--declared-field-order`. `ess generate client` gets neither flag in this story. The old acceptance `wire_bytes_are_unchanged` becomes "the default keeps its bytes; under the option the Rust JSON value is equal and only the key order moves".
- **Not done here:** making either behaviour the default needs the operator's call under D135 and a later release. Adding the flags to `ess generate client` is a follow-up story.
- **CHANGELOG**, `[Unreleased]` `### Added`: "`ess generate types` takes `--declared-field-order` for Rust, Go and TypeScript and `--rust-box-only-recursion` for Rust, as an adopter asked (beyond10x/ess#661). The first emits a model type's fields in the order the specification declares them, so Rust also encodes members in that order and a derived `PartialOrd`/`Ord` compares them in it; a bundle carries no declaration order and is refused. The second holds a named type by value and boxes a reference only where it closes a cycle of by-value fields, such as `Optional<Self>`; a reference inside a `List` or `Map` is not boxed. The report's configuration records each option given. Without them the generated bytes and the report are the ones they were."
- **Docs:** `docs/design/types-only-realizations.md` § the Rust generator options (`:259-285`) and § "Recursive values need size-breaking indirection" (`:236-237`). `website/docs/guides/generate-artifacts.md` § "Three more Rust options" (`:373-396`) gains both options. `website/docs/reference/cli.md` is regenerated by `cargo xtask cli-reference`.

## Acceptance

Fixture: a model with `Outer { zeta: Inner, alpha: String, mid: Optional<Inner> }` and an event payload, a command with input and response, and a view row whose fields are declared out of alphabetical order. For boxing, add `Node { next: Optional<Node>, kids: List<Node> }`, `A { b: Optional<B> }` and `B { a: Optional<A> }`, and `Holder { tree: Node }`.

- `default_output_keeps_wire_name_order`: without the option, Rust, Go and TypeScript emit `alpha, mid, zeta`, and Rust boxes every reference, exactly as on 0.76.0. Command: `cargo test -p schema-contract --locked --test declaration_order default_output_keeps_wire_name_order`.
- The existing byte pins stay green **unchanged**: `SHOP_DIGEST` and `OPEN_DIGEST` are not refrozen (`no_option_keeps_the_bytes`, `defaults_are_unchanged`), and neither is the model and bundle Rust and Go output-map digest. Command: `cargo test -p schema-contract --locked --test rust_generator_options --test binary64_structural`.
- `declared_field_order_follows_the_specification`: with `with_declared_field_order()`, Rust, Go and TypeScript emit `zeta, alpha, mid` for the struct, the event payload, `<cmd>.input`, `<cmd>.response` and `<view>.row`. It fails before the change (`alpha, mid, zeta`). Mutation: `ordered_fields` ignores the flag, and the test fails. Command: `cargo test -p schema-contract --locked --test declaration_order declared_field_order_follows_the_specification`.
- `every_model_object_has_a_declared_order`: every `Shape::Object` with declared fields in a model plan of the fixture has a side-table entry, so no object falls back silently. Command: `cargo test -p schema-contract --locked --test declaration_order every_model_object_has_a_declared_order`.
- `a_bundle_refuses_declared_field_order`: `from_bundle(..).with_declared_field_order()` is refused with `declared_field_order_bundle`. Command: `cargo test -p schema-contract --locked --test declaration_order a_bundle_refuses_declared_field_order`.
- `only_recursive_references_are_boxed`: under `box_only_recursion`, the emitted Rust has:
  - `pub zeta: ReproGInner` and `EssPresence<ReproGInner>`;
  - `Node.next` as `EssPresence<::std::boxed::Box<Node>>` and `Node.kids` as `::std::vec::Vec<Node>`;
  - both `A.b` and `B.a` boxed;
  - `Holder.tree` as `Node`.

  Mutation: box every reference, and the test fails. Command: `cargo test -p schema-contract --locked --test declaration_order only_recursive_references_are_boxed`.
- `the_report_records_each_option`, extended: Rust `configuration` gains `"box_only_recursion": true` and `"declared_field_order": true`. In `declaration_order.rs`, `the_report_records_declared_field_order` checks Go and TypeScript `configuration` with the member only when it is given. Commands: `cargo test -p schema-contract --locked --test rust_generator_options the_report_records_each_option` and `cargo test -p schema-contract --locked --test declaration_order the_report_records_declared_field_order`.
- `declared_order_and_box_only_recursion_build_and_round_trip`: the Rust crate generated with both options builds (`A`/`B` and `Node` included). `Outer` encodes with `{"zeta":` first, and decoding that text gives a value equal to the default build's decoding of the default encoding. Mutation: an empty boxed set fails to build with E0072. Command: `cargo test -p schema-contract --locked --test rust_realization declared_order_and_box_only_recursion_build_and_round_trip`.
- `model_types_record_declared_field_order`: the side table holds `zeta, alpha, mid` at `/$defs/repro.g.Outer`, the event payload, and the command and view roots, and `to_json()` keeps its bytes. Command: `cargo test -p ess-gen --locked --test model_types model_types_record_declared_field_order`.
- `declared_field_order_and_box_only_recursion_are_opt_in` in the `rust_generator_options` module:
  - both flags are accepted on a model, and `types-report.json` records them;
  - `--rust-box-only-recursion` is refused for `go` and `typescript`;
  - `--declared-field-order` on `ess generate schema types-bundle` is refused, naming the rule;
  - without either flag, `types.rs` is byte-identical to a run on the same model before the change.

  Command: `cargo test -p ess-cli --locked --test model_type_events declared_field_order_and_box_only_recursion_are_opt_in`.
- Gates: `cargo xtask cli-reference --check`, `task test-xtask` (CHANGELOG) and `task fmt-check`.

## Scope

- `crates/generate/schema-contract/src/realize.rs`, cited. The edit sites are:
  - `TargetConfiguration` `:50-71`: the Typescript and Go variants gain `declared_field_order`;
  - `RustOptions` `:74-94`: `box_only_recursion`;
  - `Plan` `:191-224`: the `field_order` table and the flag;
  - `from_model` `:613-641`: copies the table;
  - `build`'s `Ok(Self { .. })` `:716-735`: initializes the new fields;
  - a new `with_declared_field_order` and `ordered_fields` beside `with_raw_json` `:902`;
  - a new `by_value_references` beside `children` `:1747` and `reaches` `:1777`.

  Not touched: `kind()` `:1558-1634` (U6 #647/#648 edit it), `Shape` `:475-505`, and the U1 #686 regions.
- `crates/generate/schema-contract/src/realize/rust.rs`, cited: the `fields` loop `:565`, the collision key list `:541-545`, `short_form` `:950`, the Box at `:676`, and the configuration at `:60-63`.
- `crates/generate/schema-contract/src/realize/go.rs`, cited: `record` loops `:230`, `:251`, `:257`, `:281`, and the configuration at `:24-27`. `*T` at `:396` stays.
- `crates/generate/schema-contract/src/realize/ts.rs`, cited: the `object` loop `:129` and the configuration at `:13`.
- `crates/generate/ess-gen/src/model_types.rs`, cited: the struct `:36-42`, the mint in `select` `:106-137`, a new accessor beside `binary64_locations` `:179`, and a walker beside `:358-385`.
- `crates/edge/ess-cli/src/schema_bundle.rs`, cited: the `TypeOptions` flags `:124-156`, `rust_options` `:158-188`, the refusal list `:227-238`, and the Plan transform before `rust_with`/`go`/`typescript` `:241-277`.
- Tests:
  - `crates/generate/schema-contract/tests/declaration_order.rs` (new);
  - `crates/generate/schema-contract/tests/rust_realization.rs`, cited;
  - `crates/generate/schema-contract/tests/rust_generator_options.rs` (its exhaustive `RustOptions` literal `:224-229` gains the field), cited;
  - `crates/generate/ess-gen/tests/model_types.rs`, cited;
  - `crates/edge/ess-cli/tests/model_type_events.rs`, cited.

  `crates/generate/schema-contract/tests/binary64_structural.rs` is read only, as a pin that must not move.
- Docs: `CHANGELOG.md`, `docs/design/types-only-realizations.md`, `website/docs/guides/generate-artifacts.md` and `website/docs/reference/cli.md` (generated), cited.
- Confidence: medium. The defect sites are read and the repro was re-run on 0.76.0. Still untested: whether the pointers of the ess-gen walker match the realizer's object pointers for event payloads and body roots. `every_model_object_has_a_declared_order` is the check.
- Shared with:
  - U6 #647/#648: `realize.rs` (U6 edits `kind()` `:1558-1634`, this story does not) and `schema_bundle.rs` if U6 adds an import flag. Expect at most an adjacent-hunk rebase.
  - U1 #686, already merged in 0.76.0: no overlap left.
  - `CHANGELOG.md` is shared by every unit.
