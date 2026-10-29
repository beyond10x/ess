---
format: aep.planning-md/3
id: story:a-command-input-and-response-field-may-share-a-name-in-rust
kind: story
status: implemented
title: Rust and web synthesis scope a command's input and response fields apart
summary: Rust and web synthesis refuse a command whose input and response fields share a name, and the refusal's naming.code remedy does not clear it (beyond10x/ess#659); defect, still on 0.76.0.
tags:
- adopter-report
- defect
refs:
- provider: github
  reference: beyond10x/ess#659
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-synth/src/rust/feasibility.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/feasibility.rs
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T05:17:22Z", actor: "human:timo", revision: 13, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
- {from: "proposed", to: "active", at: "2026-10-29T05:17:22Z", actor: "human:timo", revision: 14, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
- {from: "active", to: "implemented", at: "2026-10-29T10:39:48Z", actor: "human:timo", revision: 17, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (item 9 of 9, 2026-10-22;
beyond10x/ess#659). A command whose input field and response field share a name cannot be
synthesized for Rust, and the refusal's own remedy does not clear it. Reproduced on ess 0.67.0
(beyond10x/a2a at 917e417, with the response field renamed from `reply` back to `message`):

```text
rust target cannot emit this workspace
a2a.tasks.SendMessage.message: `message` is allocated 2 times in `fields:a2a.tasks.SendMessage` (field, field); give one of them `naming: { code: … }` to choose the identifier it is emitted as, which changes no wire name, no document schema and no qualified name
```

(and the same for `SendStreamingMessage`). Adding `naming: {code: …}` to either field leaves the
refusal unchanged, on both versions.

Minimal reproduction (brand-free): `Send(id, text) -> response { id }`;
`ess generate synthesize --target rust`.

## 0.68.0

No. Rerun on 0.67.0 and 0.68.0: the same refusal. Rerun on 0.74.0 (2026-10-28) and on the 0.76.0
release binary (2026-10-29, a repro outside the repository): still the same refusal for
`--target rust` and `--target web`; `--target go` and `--target clap` synthesize.

## Where it lives

Lines against `main` at 0.76.0.

- `crates/generate/ess-synth/src/rust/feasibility.rs:143-155`: `Inventory::fields` allocates every
  field in `fields:{owner}` and ignores its `scope` argument for the field scope.
- The command input (`:519`) and the response (`:527-531`) pass the same owner, so they share one
  field scope. The response already passes its own reference scope (`response:{command}`), which
  `fields` uses only for type references.
- Refusal text `:157-196`; the remedy for a non-enum, non-union collision is `:180-182` and the
  message is formatted at `:188`.
- `naming.code` is read only for declarations (`code_aliases`, `crates/generate/ess-synth/src/alias.rs:19-32`)
  and enum variants (`variant_code_aliases`, `:41-60`). Field identifiers come from
  `name::value_ident(&field.name)`, which has 132 call sites across 20 files under
  `crates/generate/ess-synth/src/`.
- The emitted input and response are distinct Rust structs, `{ty}` and `{ty}Response`
  (`crates/generate/ess-synth/src/rust/items.rs:206-230`).
- `web` runs the same check: `crates/generate/ess-synth/src/web/mod.rs:307` calls
  `rust::feasibility::checked`.

## Cause

Verified by run plus code. Input and response fields share one allocation scope although they
are emitted as two structs, and `naming: {code: …}` is never read for fields, so the remedy the
refusal names cannot work. The shared scope also contradicts
`docs/design/unique-wire-field-identity.md:14-21`, where a command's field namespace is its input
fields (`:18`).

## Fit review

1. **Need.** A command may echo an input field's name in its response; Rust synthesis must emit
   it. No syntax proposed. Reproduced on 0.74.0 and again on 0.76.0 with `Send(id, text) ->
   response {id}`: `ess generate synthesize --target rust` and `--target web` both refuse with
   "demo.msg.Send.id: `id` is allocated 2 times in `fields:demo.msg.Send` (field, field); give one
   of them `naming: { code: … }`…". `--target go` and `--target clap` synthesize. Adding
   `naming: {code: reply_id}` to the response field gives the same refusal.
2. **Class: defect.** It contradicts `docs/design/unique-wire-field-identity.md:14-21` and the
   refusal's own remedy. Input and response are separate structs (`rust/items.rs:206-230`), so the
   shared scope is a false positive.
3. **Already expressible?** Renaming the response field changes a published wire. For a genuine
   collision inside one type (`fooBar` and `foo_bar` both become `foo_bar`; the field pattern
   `^_*[A-Za-z][A-Za-z0-9_]*$` at `crates/specify/ess-domain/src/types.rs:494` admits both), an
   idiom works today: rename one field and keep its spelling with `naming: {wire: fooBar}`. Shown
   running on 0.76.0 in a repro outside the repository: the colliding struct is refused, and the
   same struct with `{name: foo_bar2, type: String, naming: {wire: fooBar}}` synthesizes
   ("9 artifact(s)").
4. **Fit.** No authored surface. `web` shares the fix through `web/mod.rs:307`. Go does not
   collide. Synthesis has no TypeScript target: `--target` takes rust, go, web and clap, so the
   earlier TypeScript check is dropped. Honouring `naming.code` on a field would have to be read
   at every field-identifier site, in Rust and in Go (`alias.rs:5-7` exists "so Rust and Go
   cannot disagree about a name somebody wrote down"); that is its own story.
5. **Second adopter.** `Rename(name) -> {name}` in any directory service.
6. **Cost.** None in formats. One remedy string changes, for field/field collisions only; no test
   pins it. Generated bytes do not change for anything that synthesizes today.
7. **Alternatives.** (a) Change nothing: adopters rename wire fields. (b) The earlier decision,
   scope apart plus honour field `naming.code`: the second half reaches about 20 files and both
   code targets, too big for one batch unit. (c) Chosen: scope apart now, make the field remedy
   name the idiom that works, and file field `naming.code` as its own story.

The scoper's claims checked against 0.76.0: the `feasibility.rs`, `items.rs`, `web/mod.rs` and
`ess-domain/src/types.rs` lines still hold. Corrected here: `alias.rs:19-56` is two functions
(`:19-32` declarations, `:41-60` variants); `value_ident` has 132 call sites in 20 files, not 134
in 21; the design table is `unique-wire-field-identity.md:14-21`, not `:12-20`.

## Decisions

**accept, redesigned.** Format: none.

- Rust and web allocate a command's response fields in their own scope,
  `response fields:{command}` (a key no qualified declaration name can spell; adversary round). Input fields stay in `fields:{command}`. A collision between two
  response fields still refuses, in the response scope.
- A collision whose entries are all fields names the idiom that works: "rename one of them and
  keep its wire spelling with `naming: { wire: … }`, which changes no wire name and no document
  schema". It no longer offers `code` for fields. Declaration, enum-variant and union-label
  collisions keep their remedies (`feasibility.rs:172-182`).
- Field `naming.code` moves to its own draft story,
  `story:a-field-naming-code-chooses-its-identifier-in-rust-and-go`. When it lands, it restores
  `code` in the field remedy.
- The `depends_on story:go-synthesis-repairs-a-served-component-named-server` edge was wave-A
  sequencing. That story is still draft and shares no file with this one; the edge is dropped.
- The same message reached through `ess verify conform synthesize --http` has the one origin,
  `feasibility.rs:188` (inferred; not run).
- CHANGELOG, `[Unreleased]` `### Fixed` (written by the integration branch): "Rust and web
  synthesis no longer refuse a command whose response field shares a name with an input field;
  the response is its own struct and now its own field scope (beyond10x/ess#659). A collision
  between two fields of one struct now names the remedy that works, a rename that keeps the wire
  spelling with `naming: { wire: … }`."
- 2026-10-22 (superseded): the earlier decision accepted both halves and checked Go and
  TypeScript; replaced by the above under batch 2 (`docs/plan/wave-ess-issues-1.md` § Unit state,
  U8).

## Acceptance

Fixture: `Send(id, text) -> response {id}` over a `Message` entity, written by the test with the
file's `core` / `fixture` helpers.

- **Input and response may share a name.** Transition: `synthesize` for the Rust target. Before:
  refused, "`id` is allocated 2 times in `fields:demo.…Send` (field, field)". After: synthesized,
  the input struct and the `…Response` struct both carry `pub id`, and the generated workspace
  passes `check_generated`. Test: `crates/generate/ess-synth/tests/feasibility.rs`,
  `an_input_and_response_field_may_share_a_name`. Fails on the current tree. Mutation: allocating
  the response back in `fields:{owner}` (`feasibility.rs:143-155`) refuses it again.
  `cargo test -p ess-synth --locked --test feasibility an_input_and_response_field_may_share_a_name`
- **Web synthesizes the same fixture.** Transition: `synthesize_for(&ir, Target::Web)`. Before:
  the same refusal. After: synthesized. Same file,
  `the_web_target_admits_an_input_and_response_field_of_one_name`.
  `cargo test -p ess-synth --locked --test feasibility the_web_target_admits_an_input_and_response_field_of_one_name`
- **Go has no collision (check only).** Transition: `synthesize_for(&ir, Target::Go)`. After:
  synthesized, before and after the change. Same file,
  `an_input_and_response_field_share_a_name_in_go`. If it fails, this story changes no Go code and
  a follow-up story is filed.
  `cargo test -p ess-synth --locked --test feasibility an_input_and_response_field_share_a_name_in_go`
- **A collision inside a response still refuses.** Transition: two response fields `fooBar` and
  `foo_bar`. After: refused with "`foo_bar` is allocated 2 times in `response fields:demo.core.Send`".
  Same file, `a_collision_inside_a_response_still_refuses`.
  `cargo test -p ess-synth --locked --test feasibility a_collision_inside_a_response_still_refuses`
- **A field collision names the wire-rename idiom, and the idiom clears it.** Transition: a struct
  with `fooBar` and `foo_bar`, then the same struct with `{name: foo_bar2, naming: {wire: fooBar}}`.
  Before the rename: refused, the message contains `naming: { wire: … }` and not
  `naming: { code: … }`. After it: synthesized, and the emitted wire name is `fooBar`. Same file,
  `a_field_collision_names_the_wire_rename_idiom`. Fails on the current tree (the remedy names
  `code`).
  `cargo test -p ess-synth --locked --test feasibility a_field_collision_names_the_wire_rename_idiom`
- **Other remedies and bytes stay.** `an_authored_code_alias_resolves_the_collision` and
  `normalized_fields_share_their_actual_struct_scope` pass unchanged, and every existing Rust
  synthesis pin in `tests/feasibility.rs`, `tests/served_codec_names.rs` and
  `tests/underscore_field_names.rs` keeps its bytes.
  `cargo test -p ess-synth --locked --test feasibility`
  `cargo test -p ess-synth --locked --test served_codec_names --test underscore_field_names`
- `task fmt-check`

## Scope

- `crates/generate/ess-synth/src/rust/feasibility.rs`: `Inventory::fields` `:143-155`, the
  command call sites `:519` and `:527-531`, the field remedy `:180-182`. Cited.
- `crates/generate/ess-synth/tests/feasibility.rs`: the five new cases. Cited.
- `CHANGELOG.md`: one `### Fixed` line, written by the integration branch. Inferred.
- Removed from the earlier scope (field `naming.code` moved to its own story): `src/alias.rs`,
  `src/lib.rs`, `src/rust/layout.rs`, `src/rust/items.rs`, `src/rust/wire.rs`.
- Confidence: high. The refusal is reproduced on 0.76.0 and the allocation site is read.
- Shared with: none of U5, U6, U7. `CHANGELOG.md` is shared by every unit.
- Safety fact: the emitted input and response are distinct Rust structs (`items.rs:206-230`), and
  the HTTP layer reads them through separate `input` and `response` values
  (`src/rust/http/bound.rs:1116-1128`), so a separate allocation scope matches what is emitted.
  Not proven until the build in the first acceptance case runs.

## Schedule

Batch 2 of the ESS open-issue waves, unit U8 (`docs/plan/wave-ess-issues-1.md`). The release tag
on the artifact marks the release it ships in.
