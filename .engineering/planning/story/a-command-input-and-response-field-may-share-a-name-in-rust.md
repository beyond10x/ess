---
format: aep.planning-md/3
id: story:a-command-input-and-response-field-may-share-a-name-in-rust
kind: story
status: draft
title: Rust synthesis scopes a command's input and response fields apart, and honours naming.code on a field
summary: Rust synthesis refuses an input and response field of one name, and the refusal's naming.code remedy does not clear it (adopter item 9, beyond10x/a2a); defect, 0.69.0.
tags:
- adopter-report
- defect
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- depends_on: story:go-synthesis-repairs-a-served-component-named-server
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/alias.rs
- confidence: cited
  path: crates/generate/ess-synth/src/lib.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/feasibility.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/items.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/layout.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/feasibility.rs
revision: 8
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (item 9 of 9, 2026-10-22). A
command whose input field and response field share a name cannot be synthesized for Rust, and the
refusal's own remedy does not clear it. Reproduced on ess 0.67.0 (beyond10x/a2a at 917e417, with
the response field renamed from `reply` back to `message`):

```text
rust target cannot emit this workspace
a2a.tasks.SendMessage.message: `message` is allocated 2 times in `fields:a2a.tasks.SendMessage` (field, field); give one of them `naming: { code: … }` to choose the identifier it is emitted as, which changes no wire name, no document schema and no qualified name
```

(and the same for `SendStreamingMessage`). Adding `naming: {code: …}` to either field leaves the
refusal unchanged, on both versions.

Minimal reproduction (brand-free): `Send(message: Message) -> response { message: Message }`;
`ess generate synthesize --target rust`.

## Where it lives

- `crates/generate/ess-synth/src/rust/feasibility.rs:111-121`: fields are allocated in
  `fields:{owner}` regardless of `scope`; input at `:476` and response at `:484-488` use the same
  owner; refusal text `:124-155` (format at `:147`).
- The field identifier comes from `name::value_ident(&field.name)`; `naming.code` is read only for
  declarations (`crates/generate/ess-synth/src/alias.rs:24`, `rust/layout.rs:39,195-200`).

## 0.68.0

No. Rerun on 0.67.0 and 0.68.0: the same refusal.

## Cause

Verified by run plus code. Input and response fields share one allocation scope, and
`naming: {code: …}` is never read for fields, so the remedy the refusal names cannot work. It also
contradicts `docs/design/unique-wire-field-identity.md:12-20`, where a command's field namespace is
its input fields.

## Fit review

1. **Need.** A command may echo an input field's name in its response; the generated Rust must
   compile it. No syntax proposed.
2. **Class: defect.** ESS contradicts its own design page (`unique-wire-field-identity.md:12-20`)
   and its own refusal's remedy.
3. **Already expressible?** Renaming the response field changes the wire a published protocol fixes.
4. **Fit.** No authored surface. Input and response are separate Rust types, so their fields get
   separate scopes (`fields:{owner}.input` / `.response`, or per generated type); and
   `naming.code` on a field is honoured wherever the refusal suggests it. Check Go and TypeScript
   feasibility for the same scope sharing (sibling targets).
5. **Second adopter.** `Rename(name) -> {name}` in any directory service.
6. **Cost.** None in formats; generated Rust identifiers for existing documents keep their bytes
   unless they hit the collision, which today refuses.
7. **Alternatives.** (a) Change nothing: adopters rename wire fields. (b) Only honour
   `naming.code`: the collision still needs an authored workaround for a non-collision. (c) Chosen:
   both.

## Decisions

**accept.** Scope a command's input and response fields apart in Rust synthesis, and honour
`naming.code` on a field.

2026-10-22: depends on `story:go-synthesis-repairs-a-served-component-named-server` (unit A6),
which lands first. Surfaces include `crates/generate/ess-synth/src/alias.rs:24` (`code_aliases`,
which today collects `naming.code` for declarations only) and its export at
`crates/generate/ess-synth/src/lib.rs:63`. Go and TypeScript are a check only: a test shows the
same collision does not occur there; any Go or TypeScript change is out of scope, and if the check
finds the collision, the implementor files a follow-up story instead of fixing it here.

## Acceptance

Fixture: `Send(message: Message) -> response { message: Message }`, written by the test.

- **Input and response may share a name.** Transition: `ess generate synthesize --target rust`.
  Before: refused, `message is allocated 2 times in fields:<command> (field, field)`. After:
  synthesized, and the generated workspace builds. Test:
  `crates/generate/ess-synth/tests/feasibility.rs`, case
  `an_input_and_response_field_may_share_a_name`. Failing before: the case is refused on the
  current tree. Mutation check: putting input and response back in one allocation scope
  (`rust/feasibility.rs:111-121`) refuses the case again.
- **`naming.code` on a field is honoured.** Transition: `naming: {code: other}` on the response
  field. Before: the identifier is unchanged. After: the Rust identifier is `other`, the wire name
  and document schema unchanged. Same file, case `naming_code_on_a_field_renames_its_rust_identifier`.
- **A real collision still refuses, and its remedy clears it.** Transition: two fields of one
  generated type whose identifiers collide, then the same with `naming.code` on one. Before the
  remedy: refused; after it: synthesized. Same file, case
  `a_collision_inside_one_type_clears_with_naming_code`.
- **Go and TypeScript do not collide (check only).** Transition: the fixture synthesized for
  `--target go` and `--target typescript`. After: both exit 0 with no collision refusal. Same file,
  case `an_input_and_response_field_share_a_name_in_go_and_typescript`. If this case fails, the
  change does not touch Go or TypeScript; it files a follow-up story.
- **Bytes elsewhere.** Every existing Rust synthesis pin in `tests/feasibility.rs`,
  `tests/served_codec_names.rs` and `tests/underscore_field_names.rs` keeps its bytes.

## Schedule

Release 0.70.0 (defect, wave A unit A7, after A6). The release tag on the artifact marks it.
