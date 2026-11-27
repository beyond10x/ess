---
format: aep.planning-md/3
id: story:rust-binding-wraps-a-plain-copy-into-an-optional-member
kind: story
status: active
title: The Rust target wraps a plain binding copy into an Optional member
tags:
- adopter-report
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-synth/src/rust/feasibility.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/system.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/binding_optional_wrap_go.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:46:00Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:46:01Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

The Rust synthesis target emits an event binding that copies a plain value into an Optional
member (wrapping it, `Some(value.clone())`), as the Go target already does, instead of refusing the
workspace.

## Problem

An adopter on ess 0.56.0: the Rust target refuses "binding `<b>` emits a plain clone of `<Doc>`
for `payload` of type `Optional<…>`", and the same for a topic-to-channel member of type
`Optional<Channel>`; Go synthesizes the same model (56 capabilities, 0 refused).

Still so on 0.92.0. Minimal reproduction (`format: ess/33`): event `MessageSent {message_id: String,
channel: String}`; command `RecordMessage {message_id: String, channel: Optional<String>}`; a
binding `recorded` on `MessageSent` invoking `RecordMessage` with `channel: event.channel`.

```
$ ess specify validate --path relay.yaml
relay v1 — 1 file(s), valid
$ ess generate synthesize --target rust --path relay.yaml --out rust-out
rust target cannot emit this workspace
recorded, relay.chat.MessageSent.channel, relay.chat.RecordMessage.channel: binding `recorded` emits a plain clone of `String` for `channel` of type `Optional<String>`; this assignment needs an explicit target representation
(exit 1)
$ ess generate synthesize --target go --path relay.yaml --out go-out
10 capabilities: 10 generated, 0 obligation(s), 0 refused
```

`crates/generate/ess-synth/src/rust/feasibility.rs`, `fn bindings`, raises `BindingAssignment`
when there is no conversion, the source and target types differ and the value is not proved
present. The only mismatch it accepts is the unwrapping direction (an Optional source the
binding's condition proves present); wrapping a plain source into an Optional target is never
emitted. Go emits the wrap since
`story:go-synthesis-refuses-an-optional-into-required-assignment`
(`crates/generate/ess-synth/tests/binding_optional_wrap_go.rs`).

## Acceptance

- `rust_binding_wraps_a_required_copy_into_an_optional_member`: for the reproduction,
  `ess generate synthesize --target rust` exits 0 with the binding capability generated and
  0 refused; the generated binding passes `Some(event.channel.clone())` for `channel`; a
  `Doc`-typed struct copied into an `Optional<Doc>` member is wrapped the same way.
- The Go target's output for the same model keeps its bytes.
- An Optional source into a required target that no condition proves present is still refused
  (by validation as `type_mismatch`, as today).

## Scope

- cited: crates/generate/ess-synth/src/rust/feasibility.rs
- inferred: crates/generate/ess-synth/src/rust/system.rs
- inferred: crates/generate/ess-synth/tests/binding_optional_wrap_go.rs
- inferred: CHANGELOG.md

confidence: medium

The emitter site for the copy (`system::proved_expression`, named in the feasibility comment) is
where the wrap is rendered.

## Reported again on 0.99.2

A downstream control plane reported it again on ess 0.99.2 (2026-11-27), as one of three kinds of
refusal that stop the Rust target on its model ("rust target cannot emit this workspace"): one
binding, with the message "binding `<binding>` emits a plain clone of `Integer` for `<field>` of
type `Optional<Integer>`; this assignment needs an explicit target representation". The same message
holds for a newtype identifier copied into an `Optional<identifier>` member. The report is a second
shape of this story's reproduction (a scalar and a newtype where it has a `String` and a `Doc`), and
the story's acceptance covers a plain source into an Optional target of the same type without
naming a newtype source.

State on `main` (`e29c029fab`, 2026-11-27): the story is not delivered. `fn bindings` in
`crates/generate/ess-synth/src/rust/feasibility.rs` still raises `BindingAssignment` with this
message when a plain source type differs from the target type and no condition proves it present;
`crates/generate/ess-synth/tests/binding_optional_wrap_go.rs` is the only wrap test, for Go.
CHANGELOG 0.99.3 to 0.99.8 and `[Unreleased]` hold no entry for it.

Acceptance, added:

- The wrap holds for every plain source type whose target is `Optional` of that same type: a scalar
  (`Integer`), a newtype identifier, a struct and a `String`; a test row names each.
- A newtype source into an `Optional` of a different newtype is still refused as
  `BindingAssignment`: the wrap changes presence, not representation.

Evidence of weight: with this binding dropped from its model, and the two other kinds removed, the
reporter's model synthesizes with exit 0 in 15 s (3019 capabilities, 2735 generated, 284
obligations, 0 refused, 7 crates).

## Corrections, 2026-11-27

Two corrections to the text above, from the unit's two adversary passes (`review-result:adversary-e3-pass-1`,
`review-result:adversary-e3-pass-2`). The sections above stay as written.

- **A newtype into an `Optional` of another newtype is refused as `type_mismatch`, not `BindingAssignment`.** The
  acceptance line "A newtype source into an `Optional` of a different newtype is still refused as `BindingAssignment`"
  cannot hold: validation refuses the binding first, as `type_mismatch`, so the Rust target never reaches its own
  refusal. The test in `binding_optional_wrap_go.rs` pins `type_mismatch`. The line stands corrected to that refusal.
- **The enum selection compares the variant's name.** In the base, the Rust selection over an enum member printed the
  domain variant with its debug form, so a binding the plan reported generated did not compile. The first correction
  compiled but answered the wire spelling while the predicate compares the variant's name, so a variant with a declared
  wire spelling was never selected; the Go arm had the same defect. Rust and Go now compare the variant's name. These
  are two defects fixed in the unit: the first was in the base, the second came in with the first correction.

## Status

Merged to `main` 2026-11-27 21:10 CET in [PR 844](https://github.com/beyond10x/ess/pull/844) (`fd6049733d`); not released.
