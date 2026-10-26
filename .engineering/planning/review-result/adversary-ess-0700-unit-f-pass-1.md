---
format: aep.planning-md/3
id: review-result:adversary-ess-0700-unit-f-pass-1
kind: review-result
status: active
title: Adversary pass 1, unit F (empty struct, state wire names)
relations:
- reviews: story:a-struct-that-ignores-undeclared-fields-may-declare-none
- reviews: story:a-lifecycle-state-carries-a-wire-name
revision: 1
---
## Adversary pass 1, unit F (ess-0700 wave)

Worktree ess-0700-f, base d740b1f9e, implementor's changes uncommitted; adversary added
`tests/adversary_f_pass1.rs` under ess-synth and schema-contract (4 cases, 3 red).

Held: the state wire spelling everywhere a state reaches the wire in served Rust and Go (view
rows, `{subject: state}` in events, errors and `sets:`, `wrong_state` payloads, a `State` view
parameter, JSON Schema, OpenAPI and AsyncAPI enums with `x-ess-variants`); a spelling equal to
another state's name, two states sharing one, and one state with two spellings are refused; a
case-only spelling is admitted; diff reports `variant-wire-name-changed`; Entity Runtime refuses by
name. Go was not installed at the version the tests pin, so Go-side claims were read, not run.

```findings
[
{"file":"crates/generate/ess-synth/src/rust/wire.rs","line":238,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"served Rust decoder of a fields: [] struct is Ok(T {}) with no object check; 42, a string, an array and null decode as the struct"},
{"file":"crates/generate/schema-contract/src/realize/ts.rs","line":99,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a closed empty struct realizes as TypeScript {}, which admits a string, a number and an array under tsc --strict"},
{"file":"crates/generate/ess-synth/src/view_query.rs","line":370,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the order guard matches only Kind::Enum; an order_by over a respelled state is generated and ranks by declared name while the runner ranks by wire text"},
{"file":"crates/specify/ess-domain/src/primitive_admission.rs","line":665,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"the below-ess/32 refusal is located at the entity line, not the state's line; every lifecycle diagnostic has the same granularity"},
{"file":"crates/generate/ess-synth/src/rust/wire.rs","line":238,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"served Rust decoders do not refuse unknown members of a closed struct"}
]
```
