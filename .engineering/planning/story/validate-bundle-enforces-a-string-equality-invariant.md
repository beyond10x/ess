---
format: aep.planning-md/3
id: story:validate-bundle-enforces-a-string-equality-invariant
kind: story
status: active
title: validate-bundle enforces a string-equality invariant
summary: 'spindle wave 12: format == "..." validates but validate-bundle does not enforce it (ess 0.88.0)'
tags:
- backlog
- defect
- from-consumer
- queue-0.91.0
- rank-7
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/schema_bundle.rs
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/
- confidence: inferred
  path: website/docs/guides/generate-artifacts.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:48:54Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:48:57Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A downstream adopter reported this against ess 0.88.0. An invariant that compares a string field
with a literal (`format == "…"`) passes specification validation, but `validate-bundle` does not
enforce it, so a bundle whose value breaks the invariant validates.

Cause: `ess generate --kind schema` lowers into JSON Schema keywords only a comparison with a
numeric literal (`crates/generate/ess-gen/src/types.rs:1156-1197`, `conjuncts` matches only
`Operand::Literal(FactValue::Number(_))`; `apply_bound`, `:1212`, takes only `Integer` and
`Binary64`). A string equality stays only in `x-ess-invariants`, which is an annotation
(`types.rs:136-137`). `validate-bundle` never asserts `x-ess-*` annotations
(`website/docs/guides/generate-artifacts.md:291`). An `Integer` field's `==` already becomes
`const` (`types.rs:1203-1205`), so a `String` `==` with a literal has an exact keyword
(`const`) that is not used.

Reproduction on 0.92.0 (`format: ess/20`, struct `docs.records.Envelope` with fields
`format: String`, `size: Integer` and `invariants: [format == "doc/1", size >= 0]`):

```text
$ ess generate --path system.yaml --kind schema --out schema
  "format": {"type": "string"}                  # no const
  "size":   {"type": "integer", "minimum": 0}
  "x-ess-invariants": ["format == \"doc/1\"", "size >= 0"]
$ ess generate schema import-document --path schema/schema/types/docs.records.Envelope.schema.json \
    --root Envelope --definition docs.records.Envelope --dialect draft-2020-12 --out envelope.bundle.json
$ ess generate schema validate-bundle --bundle envelope.bundle.json --root Envelope bad.json   # {"format": "other/9", "size": 3}
1 instance(s), 1 valid, root Envelope                                                         # exit 0
$ ess generate schema validate-bundle --bundle envelope.bundle.json --root Envelope negative.json   # {"format": "doc/1", "size": -1}
negative.json/size: -1 is less than the minimum of 0                                          # exit 1
```

## Acceptance

- On the reproduction's model, the generated schema for `Envelope.format` carries
  `"const": "doc/1"`, and `x-ess-invariants` still lists the statement. Where the field is
  `Optional`, the keyword still admits an absent member and `null`, as the `Integer` lowering does.
- `validate-bundle` on `bad.json` exits 1 and names `bad.json/format`. On `good.json`
  (`{"format": "doc/1", "size": 3}`) it exits 0, as before.
- A newtype of `String` with `invariants: [value == "x"]` gets the same keyword on its own definition.
- `!=`, a comparison of two fields and a dotted path stay annotations only. The reference
  (`generate-artifacts.md` or the schema mapping table in `types.rs`) says which invariant shapes
  become keywords and that the rest are not enforced by `validate-bundle`.
- Models without a string-equality invariant keep their generated schema bytes.

## Scope

- cited: `crates/generate/ess-gen/src/types.rs` (`conjuncts`, `apply_bound`)
- inferred: `crates/generate/ess-gen/tests/` (a new case beside `integer_bounds.rs`)
- inferred: `crates/edge/ess-cli/tests/schema_bundle.rs` (the validate-bundle end-to-end case)
- inferred: `website/docs/guides/generate-artifacts.md`

confidence: high
