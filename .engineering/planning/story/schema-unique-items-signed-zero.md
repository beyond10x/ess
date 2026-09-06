---
format: aep.planning-md/1
id: story:schema-unique-items-signed-zero
kind: story
status: draft
title: Correct size-dependent schema uniqueness for signed zero
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: Cargo.lock
- confidence: inferred
  path: Cargo.toml
- confidence: inferred
  path: crates/generate/schema-contract/Cargo.toml
- confidence: cited
  path: crates/generate/schema-contract/src/bundle.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/source.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/schema_unique_items.rs
revision: 3
---
# Correct size-dependent uniqueItems behavior for signed zero

ESS source `6c78676c35193423fe326b9dde21b8fc21681b8a` exposes a pre-existing
jsonschema-value 0.52.1 inconsistency: an array containing equal +0/-0 values
fails `uniqueItems:true` through length 15 but can pass at length 16. The same
problem affects recursively equal records containing signed zero. This predates
the proposed TypeScript normalization target.

## Reproduction and independent evidence

Use a source-built CLI at
[`6c78676`](https://github.com/beyond10x/ess/commit/6c78676c35193423fe326b9dde21b8fc21681b8a).
The independently exercised binary SHA-256 was
`b5e3d24372ff557ea89431f183e426afda0502e072ec6fc78d1e38120adb3181`.
It reports `ess 0.19.0`; this identifies the tested source build and does not
claim any later capability was released under that version.

Save this exact schema as `unique.schema.json`:

```json
{"type":"array","uniqueItems":true}
```

Save these exact inputs as `fifteen.json` and `sixteen.json`, respectively:

```json
[0,-0.0,1,2,3,4,5,6,7,8,9,10,11,12,13]
```

```json
[0,-0.0,1,2,3,4,5,6,7,8,9,10,11,12,13,14]
```

Run:

```sh
ess generate schema import-document --path unique.schema.json --root Root --dialect draft-2020-12 --out unique.bundle.json
ess generate schema validate-bundle --bundle unique.bundle.json --root Root fifteen.json
ess generate schema validate-bundle --bundle unique.bundle.json --root Root sixteen.json
```

Expected: both validations refuse with one non-unique-items diagnostic. Adding a
distinct element must not make existing duplicates unique. Numeric schema
equality treats +0 and -0 as equal, independently of their serialized spelling.

Actual independent execution used the same schema and input bytes through
anonymous file descriptors. Import exited 0. The 15-element validation exited 1
with this exact stdout and empty stderr:

```text
/proc/self/fd/4: [0,-0.0,1,2,3,4,5,6,7,8,9,10,11,12,13] has non-unique elements
1 instance(s), 0 valid, root Root
```

The 16-element validation exited 0 with this exact stdout and empty stderr:

```text
1 instance(s), 1 valid, root Root
```

Controls independently confirmed:

- `[0,-0.0]` and `[1,1.0]` each refuse as non-unique.
- Reversing the zeros in the 16-element input still passes.
- `[{"z":0},{"z":-0.0},1,2,3,4,5,6,7,8,9,10,11,12,13,14]` passes.
- `[9007199254740992,9007199254740993]` passes as distinct exact integers.

These are actual source-6c CLI observations, not predictions from a prospective
TypeScript validator or a reconstruction of a service input.

## Source explanation

ESS [Cargo.lock at 6c78676](https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/Cargo.lock)
pins jsonschema and jsonschema-value 0.52.1. The latter's checksum is
`da4ab4cbe58181a117d8c3582844ce20b07184319b51ba6d756596c1c451aebc`.
Inspection of those exact dependency sources establishes:

- `jsonschema/src/keywords/unique_items.rs:11–16` delegates to Array::is_unique;
  `jsonschema-value/src/serde_json.rs:186–187` delegates to the shared algorithm.
- `jsonschema-value/src/unique.rs:61,67–98` uses pairwise semantic comparison
  through length 15, then AHashSet.
- Its `HashedValue` equality at `unique.rs:14–15` uses schema equality, but
  numeric hashing at `:26–28` hashes the raw f64 bits. Signed zeros compare
  equal while supplying different hash inputs. Recursive array/object hashing
  at `:36–49` propagates this mismatch.
- `jsonschema-value/src/cmp.rs:99–148` uses NumCmp and recursive schema equality;
  it correctly distinguishes large exact integer neighbors and equates 1/1.0.

This violates the equality/hash consistency required by the set algorithm.
The measured outcomes demonstrate the defect; they do not promise identical
behavior for every possible randomized hash collision.

## Bounded investigation and acceptance

Find the smallest dependency or ESS boundary correction that restores consistent
schema uniqueness without changing normalization's numeric input admission,
integer-token eligibility, negative-zero serialization or typed Binary64 rules.
Determine whether a corrected upstream dependency is available or a narrowly
scoped upstream fix is required; record and qualify any dependency change.
Do not replace exact mixed numeric equality with Number/f64 coercion.

Acceptance should include:

- Reference validation rejects equal signed-zero pairs at lengths 2, 15, 16
  and 17, including reordered pairs and recursively nested records/arrays.
- Equal integer/floating representations remain duplicates; distinct exact
  integers around 2^53 and signed/unsigned bounds remain distinct.
- Empty/singleton arrays and genuinely unique arrays still pass; false
  uniqueItems remains nonrestrictive.
- ESS bundle validation and any admitted normalization uniqueness path retain
  the expected located finding count and no successful output on refusal.
- Relevant existing corpus and target compatibility checks pass, with the
  resulting behavior and dependency identity documented.

Until correction and qualification, a new target may explicitly preflight-refuse
`uniqueItems:true` at its source-qualified keyword pointer when it is outside
required coverage. It must not silently reproduce this defect, discard the
keyword, or drop a required fixture to preserve that capability boundary.