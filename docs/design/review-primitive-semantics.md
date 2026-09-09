# Primitive semantics: one abstract value, one predicate value, one wire spelling

Binding design for `story:review-primitive-semantics` (review finding F08,
`docs/reviews/2026-09-05-architecture-review.md:326`). It decides what a `Primitive` *is* at three
layers, what the exact in-memory representation of a number becomes, which spellings are frozen in
this wave, and where the shared vector corpus lives.

## The defect, stated once

`ess_primitives::facts::Number` was `Number(f64)`. Three consequences, each observable:

| site | before this page | consequence |
|---|---|---|
| `facts.rs:33` `pub struct Number(f64)` | every number is a binary64 | `9007199254740993` and `9007199254740992` are one value |
| `facts.rs:69-73` `From<i64>` via `as f64` | `i64::MAX` becomes `9223372036854775808` | a fact built from an `i64` is not the `i64` |
| `facts.rs:64-66` `is_integral` | `false` for every `\|v\| >= 2^53` | `Primitive::Integer` refuses an integer it declares admissible |

and, separately, admission was by *shape* only: `input.rs:583-603` bound `Uuid` to any `Node::Text`,
`go/runtime.go:2532-2545` admitted `"uuid"` as any Go `string`, and `coverage-admission.js:294`
checked only that the kind *name* was in a vocabulary. A schema published a `pattern`
(`ess-gen/src/types.rs:509`) that no reader in this repository enforced.

## The matrix

Per `Primitive` variant (`ess-domain/src/types.rs:42-61`): the abstract value, the
`ess_primitives::facts::FactValue` a predicate compares, and the wire spellings. **Every spelling in
the last three columns is byte-identical to what this repository emitted at the base commit of this
wave.**

| `Primitive` | abstract value | predicate value (`FactValue`) | JSON node (`Node`) | JSON Schema (`ess-gen`) | generated Go / Rust |
|---|---|---|---|---|---|
| `String` | Unicode text | `Text` | string | `{"type":"string"}` | `string` / `String` |
| `Boolean` | true or false | `Bool` | boolean | `{"type":"boolean"}` | `bool` / `bool` |
| `Integer` | an exact integer in `[i64::MIN, i64::MAX]` | `Number` (`Repr::Exact`) | number | `{"type":"integer"}` | `int64` / `i64` |
| `Decimal` | an exact decimal, `units × 10⁻ˢᶜᵃˡᵉ` | `Number` (`Repr::Exact`) | number | `{"type":"string","format":"decimal","pattern":DECIMAL_PATTERN}` | `Decimal(string)` / `Decimal(String)` |
| `Binary64` | a finite IEEE-754 binary64, signed zero preserved | `Number` (`Repr::Binary64`) | number | `{"type":"number"}` | — (ess/2 only) |
| `Timestamp` | an instant | `Text` | string | `{"type":"string","format":"date-time"}` | `string` / `String` |
| `Duration` | a length of time | `Text` | string | `{"type":"string","format":"duration"}` | `string` / `String` |
| `Uuid` | a UUID in the canonical hyphenated form | `Text`, **admitted only when it matches `UUID_PATTERN`** | string | `{"type":"string","format":"uuid","pattern":UUID_PATTERN}` | `Uuid(string)` / `Uuid(String)` |
| `Bytes` | opaque bytes | `Text`, **admitted only when it is padded base64** | string | `{"type":"string","pattern":BASE64_PATTERN,"contentEncoding":"base64"}` | `string` / `String` |

`Decimal` is the one row where the schema column and the `Node` column disagree, and that
disagreement predates this page: `ess-gen` publishes a decimal as a *string* so a JSON reader cannot
round it (`types.rs:496-501`), while a conformance candidate carries it as a JSON number
(`input.rs:585`). This wave does not reconcile them — see *What stage two changes*.

`Binary64` has no `primitive_value` row and no witness row; it is refused before either
(`witness.rs:499-501`) and admitted only in `ess/2` (`primitive_admission.rs:14-19`).

## The exact representation

```rust
pub struct Number(Repr);

enum Repr {
    /// units × 10^-scale, together with the binary64 this value has always been written as.
    Exact { units: i128, scale: u8, binary: f64 },
    /// A value with no exact decimal spelling in i128, or one authored as a binary64.
    Binary64(f64),
}
```

Three decisions, each with its reason.

**1. A scaled `i128`, not a decimal crate.** `units × 10⁻ˢᶜᵃˡᵉ` covers every `i64` exactly (the
`Integer` row) and every decimal literal of up to 38 significant digits (the `Decimal` row) with no
new entry in `[workspace.dependencies]` and no new line in `Cargo.lock`. Anything outside that —
`1e300`, a 40-digit literal — falls back to `Binary64`, which is exactly what it did before.

**2. The binary64 is carried, not recomputed.** `Repr::Exact` stores the `f64` alongside the exact
value, and `Serialize` writes *that field*. This is what makes the wave byte-preserving **by
construction** rather than by inspection: there is no value for which the serializer can produce
bytes different from the ones it produced before, because it serializes the same `f64` it always
did. An integral witness is still `1.0` (`witness.rs:47-52`); a report still quotes what
`quote(&Node)` quoted (`report.rs:417-422`).

**3. Ordering is the binary64 first, the exact value as the tiebreak.**

```
cmp(a, b) = a.binary.total_cmp(&b.binary).then(exact_cmp(a, b))
```

`total_cmp` first preserves the one ordering fact `Binary64` promises and nothing else does:
`-0.0 < 0.0` (`model-binary64.md`). The tiebreak is what fixes F08 — `2^53` and `2^53+1` round to
the same `f64`, `total_cmp` returns `Equal`, and the exact comparison then returns `Less`. The
composition is a correct numeric order because binary64 rounding is monotone: if `x ≤ y` then
`round(x) ≤ round(y)`, so sorting by `(round(x), x)` is sorting by `x`, with signed zero as the one
deliberate refinement.

`PartialEq` is `cmp(..) == Equal`, which repairs a pre-existing `Eq`/`Ord` disagreement: `eq` was
`f64 ==` (so `-0.0 == 0.0`) while `cmp` was `total_cmp` (so `-0.0 < 0.0`), which is the state a
`BTreeMap<Number, _>` and a `Vec<Number>::contains` gave different answers in.

### What `Number` gains

| method | answer |
|---|---|
| `get() -> f64` | unchanged: the binary64, for every existing caller |
| `is_integral() -> bool` | exact: `true` for every `i64`, including `i64::MAX`; `false` for `1.5` |
| `as_i64() -> Option<i64>` | the exact integer when there is one |
| `exact_text() -> String` | the exact decimal spelling, no exponent |

`Display` is the exact spelling. For every value expressible as an `f64` shortest-round-trip
rendering below `2^53` that is the identical string it was; above `2^53` it stops printing a number
the value is not (`9223372036854775807`, not `9223372036854775808`).

## Admission

One grammar, three implementations, one corpus.

`ess_primitives::facts` gains the two predicates — `is_canonical_uuid` and `is_padded_base64` —
because `ess-primitives` is the only crate `ess-domain`, `ess-conformance`, `ess-gen` and `infra-*`
all depend on and none depends on another. `Number::exact_text` is the third grammar, spelt as a
renderer rather than a predicate because a `Decimal` reaches this repository as a JSON number, not
as the string `DECIMAL_PATTERN` publishes. `ess-gen`'s `UUID_PATTERN` and `BASE64_PATTERN` are the
published spelling of the same grammars; a test asserts the Rust predicate and the published pattern
agree on every corpus vector, which is the check that keeps them from drifting apart.

| reader | function | change |
|---|---|---|
| Rust conformance | `input.rs::primitive_value` | `Uuid`/`Bytes` text is matched against the grammar, not merely typed |
| Go conformance runtime | `runtime.go::primitive` | `"uuid"` and `"bytes"` get the same grammar; `"integer"` gets integrality |
| browser adapter | `coverage-admission.js::primitiveAdmits` | a step carrying both `payload` and `shape` has its payload checked against the shape's primitive kinds |
| Rust replay admission | `admission.rs::payload_agrees_with_its_shape` | the identical rule, so the two admitters cannot disagree about one document |

The Go runtime previously had no `"bytes"` case at all, so a `Bytes` field accepted a boolean, and
`"integer"` did not check integrality, so it accepted `1.5` where `input.rs` refused it.

**Only the fields the payload names.** A shape is a claim about the declaration; a payload is a
partial claim about values (`scenario.rs:1712`). Requiring every declared leaf to be present would
refuse suites this repository already writes — including
`ess-cli/tests/support/coverage_cases.rs:94`, a shape with no payload at all — which is a different
rule from the one F08 names.

### The corpus

`crates/specify/ess-primitives/tests/vectors/primitive-semantics.json` — one JSON document, read by

* `crates/specify/ess-primitives/tests/primitive_corpus.rs` (Rust: `Number`, the grammars),
* `crates/verify/ess-conformance/tests/primitive_corpus.rs` (Rust: `primitive_value`; and it
  compiles the vectors through `go run` against `runtime.go`, and through `node` against
  `coverage-admission.js`).

It lives beside `ess-primitives` because that crate owns both the number and the grammars and is
below every reader of them. It is read, never generated: a vector is added by hand, and the three
languages must agree about it or a lane goes red.

## What stage two changes, and why it is not here

The canonical-serialization stage is a **format change** and belongs to
`obligation:review-contract-rollout-coordination`, which already lists F08 as a migration candidate.
`docs/design/review-conformance-coverage.md:172-176` freezes suite and report numeric spelling to
what `serde_json` 1.0.151 writes — "do not coerce to integer or string" — and AEP's
`aep-ess-evidence::adapt_json` reads `ess-conformance-report/1` today. Changing `1.0` to `1` is
therefore a change to a document a pinned external reader parses, and it needs a format version, an
ADR and the relying-reader inventory, none of which this story carries.

So stage two, not this wave:

1. `Serialize` writes `1` for an integral `Number` and the exact decimal string for a `Decimal`,
   behind `ess-conformance/6` and `ess-conformance-report/2`.
2. `Node::from_value` reads through `serde_json`'s `arbitrary_precision`, so a decimal *arrives*
   exact instead of being exact only when constructed in Rust. Until then the exactness this page
   delivers is exactness of construction, comparison and admission — a `Number` that has been
   through JSON is the binary64 it was written as.
3. The `Decimal` disagreement between the schema (a string) and the conformance node (a number) is
   resolved in one direction.

Nothing in this wave may be read as having chosen any of those three.

## What this costs the consumer-coverage baseline, and who pays it

`Number` changing shape is visible to `cargo xtask consumer-check` in two ways, and only the first
is inside this unit's reservation.

1. **New concrete entries.** `is_canonical_uuid`, `is_padded_base64`, `Number::as_i64`,
   `Number::exact_text`, `Repr` and its variants, `payload_agrees_with_its_shape` and the rest are
   unclassified until named. They are named, in sorted position with their siblings' class and
   reason, in `ess-xtask/src/consumer_coverage/entry-classifications.json`.
2. **A changed model shape, and seven new models.** `rust:ess_primitives::facts::Number` and
   `.../Number/field/0` have new shape hashes, and `Repr` adds seven obligations —
   `87 profiles × 9 models = 783` cells the frozen baseline does not cover. The baseline is
   byte-pinned by a constant in `consumer_coverage/mod.rs:403`, and
   `docs/design/review-consumer-coverage.md:146-148` says an old `BaselineUnknown` does not transfer
   to a changed obligation. **Re-freezing it is root's act, not this unit's**, so the measured patch
   that does it is left unapplied for the coordinator.
