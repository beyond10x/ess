---
format: aep.planning-md/3
id: story:rust-binding-wraps-a-plain-copy-into-an-optional-member
kind: story
status: draft
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
revision: 3
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
