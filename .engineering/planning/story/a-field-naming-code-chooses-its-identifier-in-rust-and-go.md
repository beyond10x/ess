---
format: aep.planning-md/3
id: story:a-field-naming-code-chooses-its-identifier-in-rust-and-go
kind: story
status: draft
title: A field's naming.code chooses its identifier in Rust and Go
summary: Code emitters read naming.code for declarations and enum variants but not for fields; split from beyond10x/ess#659.
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- depends_on: story:a-command-input-and-response-field-may-share-a-name-in-rust
- decomposes: epic:adopter-gaps-generated-code
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/alias.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/accessor.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/behaviour/query.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/condition.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/entity.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/invariant.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/items.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/mod.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/selection.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/store/identity.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/system.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/accessor.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/behaviour/query.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/condition.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/entity.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/feasibility.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/invariant.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/items.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/name.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/selection.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/store/identity.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/system.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/wire.rs
revision: 9
---
## Need

Split from beyond10x/ess#659 (`story:a-command-input-and-response-field-may-share-a-name-in-rust`,
batch 2 unit U8). An author can write `naming: {code: …}` on a field and validation accepts it,
but no code emitter reads it: field identifiers come from `name::value_ident(&field.name)` (call
sites across 20 files under `crates/generate/ess-synth/src/`), and `naming.code` is collected only
for declarations (`crates/generate/ess-synth/src/alias.rs`, `code_aliases`) and enum variants
(`variant_code_aliases`). The key is silently ignored by Rust and Go.

Minimal reproduction (ess 0.92.0):

```yaml
format: ess/22
system: demo
version: v1
domain: demo.core
types:
  - name: demo.core.Body
    kind: struct
    fields:
      - {name: text, type: String, naming: {code: content}}
events:
  - name: demo.core.Sent
    fields:
      - {name: id, type: String}
commands:
  - name: demo.core.Send
    input:
      - {name: body, type: demo.core.Body}
      - {name: note, type: String, naming: {code: remark}}
    outcomes:
      - name: sent
        emits: [demo.core.Sent]
        payload:
          demo.core.Sent:
            id: {generated: true}
actors:
  - name: demo.core.Writer
    may: [demo.core.Send]
components:
  - component: messages
    owns: {domains: [demo.core]}
    accepts: {commands: [demo.core.Send]}
    publishes: {events: [demo.core.Sent]}
    reached_by: network
```

- `ess specify validate` prints `valid`, exit 0.
- `ess generate synthesize --target rust` emits `pub text: String` and `pub note: String`
  (`crates/demo-types/src/core.rs`); `--target go` emits `Text string` and `Note string`.
- A field/field collision inside one struct (`fooBar` and `foo_bar`, the second with
  `naming: {code: foo_bar_alt}`) still fails Rust synthesis with "`foo_bar` is allocated 2 times in
  `fields:demo.core.Body` (field, field); rename one of them and keep its wire spelling with
  `naming: { wire: … }`".

Today the working idiom for a collision inside one struct is a rename that keeps the wire
spelling (`{name: foo_bar2, naming: {wire: fooBar}}`), which changes the field's qualified name.
`naming.code` would let an author keep the name and choose only the identifier.

## Acceptance

- From the model above, Rust synthesis emits `pub content: String` in `Body` and `pub remark: String`
  in the `Send` input; Go emits `Content string` and `Remark string`. The JSON member names
  (`text`, `note`), the OpenAPI document and `ess generate --kind schema` output are byte-identical
  to the output without `naming.code`.
- The collision model above synthesizes on Rust with exit 0, the two fields emitted as `foo_bar`
  and `foo_bar_alt`.
- The field/field collision remedy in `crates/generate/ess-synth/src/rust/feasibility.rs` offers
  `naming: { code: … }` beside the wire-rename idiom again, asserted by a test.
- A field `naming.code` that collides with another field's identifier is refused by name, as a
  declaration alias collision is.
- Models without a field `naming.code` keep their generated bytes.

## Scope

Rust field identifiers are spelled by `name::value_ident(&field.name)` at every site that declares
or reads a field; Go field identifiers by `items::field_ident` / `unique_field` (`go/items.rs`,
`go/mod.rs`) and their callers. Each site below spells a field, input, stored or member identifier
and reads the field alias from the one lookup beside `code_aliases`. Moves no shared surface.

- cited: crates/generate/ess-synth/src/alias.rs
- cited: crates/generate/ess-synth/src/rust/feasibility.rs
- cited: crates/generate/ess-synth/src/rust/items.rs
- cited: crates/generate/ess-synth/src/rust/entity.rs
- cited: crates/generate/ess-synth/src/rust/name.rs
- inferred: crates/generate/ess-synth/src/rust/wire.rs
- inferred: crates/generate/ess-synth/src/rust/behaviour.rs
- inferred: crates/generate/ess-synth/src/rust/behaviour/query.rs
- inferred: crates/generate/ess-synth/src/rust/invariant.rs
- inferred: crates/generate/ess-synth/src/rust/condition.rs
- inferred: crates/generate/ess-synth/src/rust/selection.rs
- inferred: crates/generate/ess-synth/src/rust/accessor.rs
- inferred: crates/generate/ess-synth/src/rust/store/identity.rs
- inferred: crates/generate/ess-synth/src/rust/system.rs
- cited: crates/generate/ess-synth/src/go/items.rs
- cited: crates/generate/ess-synth/src/go/mod.rs
- inferred: crates/generate/ess-synth/src/go/accessor.rs
- inferred: crates/generate/ess-synth/src/go/entity.rs
- inferred: crates/generate/ess-synth/src/go/system.rs
- inferred: crates/generate/ess-synth/src/go/invariant.rs
- inferred: crates/generate/ess-synth/src/go/selection.rs
- inferred: crates/generate/ess-synth/src/go/condition.rs
- inferred: crates/generate/ess-synth/src/go/store/identity.rs
- inferred: crates/generate/ess-synth/src/go/behaviour.rs
- inferred: crates/generate/ess-synth/src/go/behaviour/query.rs

confidence: medium

## Fit review

1. Need: an author cannot choose a field's identifier without changing its qualified name, and
   the key that says so is accepted and ignored. Reproduced above.
2. Class: defect (a target silently ignores an accepted key; `alias.rs` header says `naming.code`
   is "where an author says which"), and a gap for the collision remedy.
3. Already expressible: partly; the wire-rename idiom clears a collision but renames the field.
4. Fit: reuses `naming.code`, already read for declarations and enum variants; read in one place
   beside `code_aliases` so Rust and Go cannot disagree (`alias.rs` header). TypeScript and web
   targets need the same answer or a refusal by name.
5. Second adopter: any struct whose field name is a target keyword or collides after case folding.
6. Cost: no format bump (the key is already accepted); generated identifiers change only where an
   author wrote it.
7. Considered: change nothing (key stays silently ignored); refuse `naming.code` on a field at
   validation (honest, but removes the remedy). Reading it wins.

## Decisions

- Accept: a field's `naming.code` chooses its identifier in Rust and Go, read in one place beside
  `code_aliases`. Not scheduled in batch 2; scoped before a batch takes it.
- Scope names files, not the `rust/` and `go/` directories: the files are those whose code spells a
  field identifier today (`value_ident(&field.name)` in Rust, `field_ident`/`unique_field` in Go).
