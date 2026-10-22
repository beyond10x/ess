---
format: aep.planning-md/3
id: story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant
kind: story
status: draft
title: A union can be encoded as one member named after its variant (ProtoJSON oneof)
summary: Unions are always tagged; ProtoJSON oneof needs the keyed-by-variant encoding (adopter item 4, beyond10x/a2a); gap, needs a new format version.
tags:
- adopter-report
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:union-tag-inline-with-fields
revision: 2
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (item 4 of 9, 2026-10-22). The
protocol's messages are ProtoJSON: a `oneof` is encoded as one member named after the chosen
variant (`{"text": "..."}` or `{"file": {...}}`), with no separate tag. ESS unions are always
tagged, so the specification cannot state the wire shape. No error text: nothing to author.
Requester's wording: "gap: union encoding keyed by member name (ProtoJSON oneof)".

Related, from the same report: a struct that must carry exactly one of several Optional members
(a message part) has no construct either. Whether `defined()` inside a struct invariant already
suffices was not checked.

Minimal reproduction (brand-free): `Content = Text(String) | File(FileRef)` whose wire form must be
`{"text": "hi"}` or `{"file": {"uri": "…"}}`.

## Where it lives

- `crates/specify/ess-domain/src/types.rs:1176-1188` (Union, "always tagged"); raw form
  `:1786-1795`.
- Content key `crates/generate/ess-gen/src/schema.rs:75`; schema `ess-gen/src/types.rs:922-945`.
- Rust `crates/generate/ess-synth/src/rust/items.rs:157-196`, `rust/wire.rs:161,272`.
- Design `docs/design/union-unit-variants.md:54-69`.

## 0.68.0

No.

## Cause

Not a defect: only adjacent tagging exists. Draft `story:union-tag-inline-with-fields` is internal
tagging, a different encoding.

## Fit review

1. **Need.** A union whose wire form is an object with exactly one member, the member's name being
   the variant's wire name (externally tagged, ProtoJSON `oneof`).
2. **Class: gap.** The wire fact cannot be stated; generated schema, servers and the interpreter
   all emit the adjacent form.
3. **Already expressible?** A struct of Optionals plus an exactly-one invariant approximates it,
   but loses the union in every generated type and in exhaustiveness; I don't know whether
   `defined()` in a struct invariant expresses exactly-one today.
4. **Fit.** Grid O, union encoding column. This is one of three encodings (adjacent, internal,
   external). It must share one key with `story:union-tag-inline-with-fields` so the union carries
   a single encoding selector, not one key per encoding. Every target (schema, OpenAPI, Rust, Go,
   TypeScript, the interpreter, `ess verify diff`, and a future protobuf `oneof` under
   `story:ess-projects-a-protobuf-idl-with-http-annotations`) handles each encoding or refuses it
   by name.
5. **Second adopter.** Any API built on serde's default (externally tagged) enum encoding, e.g. a
   payment method `{"card": {...}}` or `{"bank_transfer": {...}}`.
6. **Cost.** A new authored key on unions, so a new format version (inferred from AGENTS.md
   § Determinism and formats: a key that changes wire meaning). A change of encoding is breaking
   for callers in `ess verify diff`.
7. **Alternatives.** (a) Change nothing: struct-of-Optionals idiom, unions lost. (b) A key only
   for this encoding: two keys once the internal-tagging draft lands. (c) Chosen: one encoding
   selector covering adjacent (default, today's bytes), internal and external.

## Decisions

**accept, redesigned.** One encoding selector on a union, shared with
`story:union-tag-inline-with-fields`; external (keyed-by-variant) is one of its values. The key's
spelling is settled in a design page before this story leaves draft.

## Acceptance

- The minimal reproduction, with the external encoding selected, generates a JSON Schema whose
  instances are exactly the one-member objects, and Rust and Go servers that round-trip them.
- Without the selector every artifact keeps its bytes.
- Selecting the encoding under the previous format version is refused by name.

## Schedule

Gap; not tagged for 0.69.0. Needs a new format version. Prerequisite for the protobuf `oneof`
mapping (item 7).

## a2a wave 1

Relayed by the orchestrator on 2026-10-22 from the adopter session specifying A2A in
beyond10x/a2a (ESS 0.67.0, a2a main 3a18050):

- **G2** (a2a wave-1 gaps list, line 11): a union keyed by member name (protobuf `oneof`), refused
  as `missing field tag` in the adopter's `ess/tasks.yaml`. The same need as this story.
