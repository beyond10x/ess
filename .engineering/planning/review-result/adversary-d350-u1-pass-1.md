---
format: aep.planning-md/3
id: review-result:adversary-d350-u1-pass-1
kind: review-result
status: active
title: 'D350 U1 adversary pass 1: nullable Binary64 fields and responseless response roots'
relations:
- reviews: story:a-null-when-absent-binary64-field-realizes
- reviews: story:a-response-root-requires-a-declared-response
revision: 1
---
unit: D350 U1 (story:a-null-when-absent-binary64-field-realizes, story:a-response-root-requires-a-declared-response), c47ab4f64 on unit/ess-d350-u1, reviewed against bf5cf1873
verdict: CONFIRMED (1 warning, 1 note, 1 out of scope)
cases: executed 0→5 (written under the cargo hold; run at "go" by the implementor at 7c5fd30a0), red 0
origin: introduced 2 / pre-existing 0 / undecided 1
wrote-outside-worktree: none (scratch the wave scratch directory u1/adv-scratch created, empty)
needs-coordinator: the cases could not run under the 13:26Z hold; the implementor ran them at go

**Cases.** `crates/generate/ess-gen/tests/adversary_d350_u1_pass1.rs` (story 25): every response root selects exactly when `bound_command_roots` lists it, else one `unknown_root` error naming that root (exactly 2 refused); `Pin.response` beside three valid roots gives exactly one error; the `--all-commands` set selects as a whole, with and without the binding. `crates/generate/schema-contract/tests/adversary_d350_u1_pass1.rs` (story 15): a present null written back as a present null (`"value":null`, `"spare":null`); `1e400` refused at `value` and `spare` in struct, response and input; `9007199254740993` read as `9007199254740992.0`; escaped wire names `a/b~` and `c~1` give `/properties/a~1b~0/anyOf/0` and `/properties/c~01/anyOf/0`, Rust and Go realize, TypeScript `model_binary64` obligations equal exactly those pointers. Run at go: ess-gen 3 passed, schema-contract 2 passed.

**Findings (from reading).**
- F1 (warning, introduced), `crates/generate/schema-contract/tests/binary64_null_when_absent.rs:225` and `:242`: the response and struct cases read null back through `Value`'s index, which answers null for a missing key; 1.5 and null decode without the Binary64 codec. Fixed at 7c5fd30a0: whole re-encoded documents compared, fields read as `Option<EssBinary64>`, `-0` sign bit checked, `1e400` refused.
- F2 (note, introduced), `crates/generate/ess-gen/src/model_types.rs:368` (detail :293-296): `Pin.response` got the generic `unknown_root` detail. Fixed at 7c5fd30a0: the detail says the command declares no response.
- F3 (note, out of scope, undecided), `crates/generate/ess-gen/src/types.rs:995-1003` against `model_types.rs:529-549`: a shorthand struct (`ess/27`) is published as `oneOf [short, object]` while `body_numeric_locations` points at `/$defs/X/properties/…`; not probed.
- F4 (residue): Go only plans (`plan.go(...)` Ok); no Go code is compiled or decoded on this host (the `go-typecheck` lane needs go1.26.5; installed 1.27.0, 1.26.1/2/10). The Go realization claim is not verified here.

**Read and not broken.** Reverting the `is_nullable`/`property_type` fix or dropping `OmittedOrNull` from `is_nullable` fails the unit's exact location sets; removing the `response.is_empty()` guard fails the new `command_view_roots` test; the CLI's three callers do not use the response-less root; `Optional<List/Map<Binary64>>` locations walk correctly (not probed).

```findings
[
{"file":"crates/generate/schema-contract/tests/binary64_null_when_absent.rs","line":225,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"the response and struct cases read null back through Value's index, which answers null for a missing key, and 1.5/null decode without the Binary64 codec, so dropping the key or losing the codec at anyOf/0 stays green"},
{"file":"crates/generate/ess-gen/src/model_types.rs","line":368,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"a response root of a command without a response is refused with the generic unknown_root detail claiming it is not a command's .response, not that the command declares none"},
{"file":"crates/generate/ess-gen/src/types.rs","line":995,"category":"judgement","severity":"note","verdict":null,"origin":"undecided","message":"out of scope - a shorthand struct publishes oneOf [short, object] while body_numeric_locations still points Binary64 fields at /properties, so its locations may not resolve; not probed"}
]
```
