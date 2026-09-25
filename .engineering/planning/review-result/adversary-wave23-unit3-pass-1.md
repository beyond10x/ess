---
format: aep.planning-md/2
id: review-result:adversary-wave23-unit3-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: the fix does not reach the validator the crate emits'
relations:
- reviews: story:schema-unique-items-signed-zero
revision: 1
---
## Pass

`aep-drive:adversary`, pass 1, against `wt-0d3d95df677f` over base `46c7281e`.
Verdict **NEEDS-CHANGE**. Cases executed 164 → 171, red 5. Origin: introduced 5.

Ten cases added in `crates/generate/schema-contract/tests/schema_unique_items_adversary.rs`.

## The blocker

`src/realize/normalize/rust_runtime.rs.txt:72` — the Rust normalization runtime this crate **emits**
— still builds its validator from plain `jsonschema::options()`. The unit changed the three
in-process sites and left the emitted one, so the reference and its own projection now disagree
about the story's exact input. **At the base commit the two were byte-identical constructions and
agreed** (both defective), so the unit created the divergence. Same line in `legacy_v1_v3`,
`legacy_v4`, `legacy_v5`.

Measured twice: a validator built exactly as that template builds it **accepts**
`[0,-0.0,1000..1013]` while `Bundle::validate` on the same schema refuses it; and
`Plan::rust("adversary-probe")` on the crate's own `fixtures/normalization_v1.rs` plan emits a crate
whose `src/lib.rs` carries the unchecked call.

`src/realize.rs:571` admits `uniqueItems` for every normalization target with the obligation
"validate this constraint against the source schema at runtime"; the TypeScript-only refusal at
`typescript_schema.rs:76` does not cover Rust or Go. `tests/normalization_rust.rs` compiles and
executes the emitted crate, so it is a shipped artifact rather than a fixture.

## Why the guard is green while that stands

`tests/schema_unique_items.rs:160` filters on `extension == "rs"`, so the four `rust_runtime.rs.txt`
templates **inside the directory it walks** are invisible to it. The same walk with `.txt` included
returns five sites, not one.

## Attacked and could not break

The bucketing key, head on: 36 numeric spellings parsed from JSON text — signed zeros in five
spellings, 2^53 neighbours, `u64::MAX`, 2^64, `i64::MIN` as integer and float, 2^127 and 2^128 as
literals and in exponent notation, subnormals, 1e308 — all 630 ordered pairs at lengths 2/15/16/17/64,
**3150 bundle validations** asserting the verdict equals `jsonschema::json::cmp::equal` exactly. No
pair that is equal and keys apart. Nesting through arrays and objects, signed zero as an object key
(stays distinct, correctly), the error pointer under `items` (reports `/1`, not `""`), and
`uniqueItems: false` non-restrictive in both implementations. NaN and infinity are not reachable:
`json!(f64::NAN)` is `Value::Null` and `serde_json` rejects both `[NaN,NaN]` and `[1e400]` in text.

```findings
- file: crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt
  line: 72
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the Rust normalization runtime this crate emits still builds its validator from plain jsonschema::options(), so the shipped projection accepts the sixteen-element signed-zero array the fixed reference refuses, a divergence that did not exist at the base commit where both constructions were identical.
- file: crates/generate/schema-contract/tests/schema_unique_items.rs
  line: 160
  category: mutant
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the class guard filters on `extension == "rs"` and therefore cannot see the four rust_runtime.rs.txt templates inside the very directory it walks, which is why it is green while the emitted runtime carries the defect.
- file: crates/generate/schema-contract/src/uniqueness.rs
  line: 20
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the module header claims nothing else calls jsonschema::options(), but tests/bundle.rs:240 and tests/normalization_typescript_native.rs:114 do, and the first of those is the oracle a bundle's own verdict is asserted equal to.
- file: crates/generate/schema-contract/tests/schema_unique_items.rs
  line: 162
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the guard matches one literal spelling, so a future site using jsonschema::draft202012::options(), validator_for, is_valid or options_for reinstates the length-dependent uniqueItems with the guard still green.
- file: crates/generate/schema-contract/src/uniqueness.rs
  line: 39
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the error text is byte-identical only under Display; under MaskedValidationError the Custom kind writes the message unchanged and leaks the whole instance where the built-in substitutes the placeholder, but neither is reachable through this crate's public surface.
```
