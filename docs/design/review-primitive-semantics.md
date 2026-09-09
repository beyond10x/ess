# Primitive semantics: one abstract value, one predicate value, one wire spelling

Binding design for `story:review-primitive-semantics` (review finding F08,
`docs/reviews/2026-09-05-architecture-review.md:326`). It decides what a `Primitive` *is* at three
layers, what the exact in-memory representation of a number becomes, which spellings are frozen in
this wave, and where the shared vector corpus lives.

## The defect, stated once

`ess_primitives::facts::Number` was `Number(f64)`. Three consequences, each observable:

| site | before this page | consequence |
|---|---|---|
| `facts.rs:33` `pub struct Number(f64)` | every number is a binary64 | `9007199254740993` and `9007199254740992` are one value, *including when they are built from two different `i64`s* — which is the half this wave fixes; see *The round-trip law* for the half it does not |
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
| `Integer` | an exact integer in `[i64::MIN, i64::MAX]`, in process and on the wire alike | `Number` (`Repr::Exact`) | number | `{"type":"integer"}` | `int64` / `i64` |
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

**One `Integer` range, and `is_integral` is `as_i64`.** An earlier draft of this page gave the row
two halves and let the wire half reach `2^63`, on the argument that `i64::MAX` written as a binary64
comes back as `2^63`. That followed from a read door that rounded, and the door no longer does — see
*The round-trip law* and *One range, and the lane that can draw it*, which also states the one place
the float-reading lanes cannot draw the range as narrowly as Rust does.

## The round-trip law

**A `Number` is a fixed point of write∘read. Every `Number`, however it was built.** For every
value `n`:

```
read(write(n)) == n          and          is_integral(read(write(n))) == is_integral(n)
```

The first half is value stability; the second is *admission* stability, and it is the one with
teeth — a suite this repository writes must be admitted the same way when it is read back
(`admission.rs::payload_agrees_with_its_shape`), or the repository refuses its own artifact.

The law admits no in-process exemption, and the earlier draft of this page was wrong to grant one.
It said exactness beyond binary64 lived in-process and was lost at the first write; that made
`FactValue::parse_literal("1.0000000000000000001")` a value that was **not** an `Integer`, wrote as
`1.0`, and came back one — the admission half failing on a constructor this page's own table names.
A law with an exemption is not a law; it is a place the next reader gets a different answer.

**Both doors move together, or neither does.** Two constructions make the law hold, and every
`Number` is one of them:

| the value | its write | its read back |
|---|---|---|
| equals the canonical decimal of its binary64 | that **binary64** — the bytes that were always written | the same value |
| an **integer** binary64 does not carry (`i64::MAX`, `2^53 + 1`) | the **integer token** | the same integer |

Anything else — a non-integer carrying more places than binary64 does — is not built at all:
`Number::parse_decimal` collapses it to the canonical decimal of its binary64, which is what the
write was always going to say. `Repr::exact` is the one function that builds an exact value, and
holding this is its first postcondition.

**Why writing an integer token moves no published byte.** The frozen spelling
(`docs/design/review-conformance-coverage.md:172-176`, "do not coerce to integer or string", and
the `1.0` witness pin) protects values that binary64 *carries*, and those still write their
binary64: `1.0` is `1.0`, `19.99` is `19.99`, `-0.0` is `-0.0`. Every number in every artifact this
repository has published is in that class, because an artifact's numbers came from a document and
at the base commit a document's numbers were read through binary64. The second class is a value
binary64 never carried: at the base it was rounded silently on the way in and written wrong on the
way out, so writing the integer is not a *change* to a spelling any reader has received — it is the
first correct one. `cargo xtask generate --check` and the committed suites are what say so.

So **`9007199254740992` and `9007199254740993` are two values, and stay two values through a
document.** What remains for the canonical-serialization stage is the *decimal* half: an authored
`0.1000000000000000000001` is still the binary64 `0.1`, because reading a decimal token exactly
needs `serde_json`'s `arbitrary_precision` on the reader and the exact digits on the writer, and
those move together behind a format version.

### One range, and the lane that can draw it

`Primitive::Integer` admits exactly `[i64::MIN, i64::MAX]`. `Number::is_integral` **is**
`Number::as_i64().is_some()` — one question, not two — so a value admitted as an `Integer` always
carries the integer it was admitted as, and the generated `int64`/`i64` codecs decode every token
conformance admits. `tests/adversary_integer_bound_pass2.rs` asks the codec lane that directly.

The earlier draft admitted `2^63` as well, on the argument that it is what `i64::MAX` comes back as.
That was a consequence of the rounding read door, and with the door fixed it is simply wrong: `2^63`
is not an `i64`, `as_i64` says `None`, and a suite carrying that token is refused by the
implementation the same specification generates.

**The one place the four lanes do not have the same reach, stated rather than hidden.** Rust reads
the *token*, so it separates `9223372036854775807` from `9223372036854775808`. The Go conformance
runtime and the browser adapter are handed a JSON *value* — `float64` and JS `number` — and to them
those two tokens are one `f64`. They therefore admit the closed float image `[-2^63, 2^63]`, which
is wider than the declared range by exactly the point `2^63`, and they cannot be made narrower
without refusing `i64::MAX` as well. Two consequences, both deliberate:

* the shared corpus carries no vector at `2^63`, because its answer depends on which lane reads it;
  the assertion lives in `ess-primitives/tests/primitive_corpus.rs::the_first_integer_beyond_i64_is_refused_by_the_lane_that_reads_the_token`,
  the lane that can express it;
* the residual gap is one value wide, unreachable from any token a generated `int64` codec accepts,
  and it closes in the canonical-serialization stage when the Go runtime decodes with
  `json.Decoder.UseNumber` and the adapter keeps its `NumberToken`.

**What a binary64 is carried as.** `Repr::of_binary64` records the *canonical decimal name* of the
`f64`, by two rules, because one is not enough to be truthful:

* below `2^63`, an integral binary64 is carried as **the integer it is** —
  the shortest round-tripping decimal for `2^63` is `9223372036854776000`, a different number, and
  carrying that made `is_integral` refuse the value `i64::MAX` comes back as and made `Display`
  print a number the value is not;
* otherwise, the **shortest decimal that round-trips**, which is what `f64`'s own `Display` has
  always printed and what a reader means by the value.

Both are injective on `f64`, and an integral spelling never collides with a fractional one, so the
composition is injective — which is precisely what the law needs.

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

**3. Ordering is the exact value, and the binary64 only where there is no exact value.**

```
cmp(a, b) = match (a.exact(), b.exact()) {
    (Some(l), Some(r)) => exact_cmp(l, r),
    _                  => a.get().total_cmp(&b.get()),
}
```

The exact comparison is what fixes F08: `2^53` and `2^53 + 1` are one `f64`, and comparing the
scaled integers says which is which.

**The order is total, and the argument is one function away.** The `total_cmp` arm runs only when
the two variants differ, and `Repr::exact` — the single place an `Exact` is built — refuses to build
one whose binary64 has no canonical decimal. So the variant is a function of the carried binary64,
two differing variants never carry one `f64`, that arm is never a tie, and the two arms never
disagree about one pair. The previous version of this paragraph argued it from magnitude bands, and
that argument was **false**: `parse_decimal("1e-41")` and `of_binary64(1e20)` are both `Exact`
inside the band it named. The postcondition replaced the band.

**`-0.0` and `0.0` are one value**, because `units × 10⁻ˢᶜᵃˡᵉ` has one zero and the `Decimal` row of
the matrix says so. It is also what `PartialEq` answered before this page (`eq` was `f64 ==`), what
IEEE-754 says, and what a guard `amount == 0` has to mean. The signed zero `Primitive::Binary64`
promises survives where the model actually promises it — in the **bytes**: `Serialize` writes the
carried binary64, so a field spelt `-0.0` is still written `-0.0`
(`ess-conformance/src/counts.rs:477-478` pins that spelling and is unaffected).

`PartialEq` is `cmp(..) == Equal`, which repairs a pre-existing `Eq`/`Ord` disagreement: `eq` was
`f64 ==` (so `-0.0 == 0.0`) while `cmp` was `total_cmp` (so `-0.0 < 0.0`), which is the state a
`BTreeMap<Number, _>` and a `Vec<Number>::contains` gave different answers in. The disagreement is
resolved in the direction `eq` already had.

### What `Number` gains

| method | answer |
|---|---|
| `get() -> f64` | unchanged: the binary64, for every existing caller |
| `is_integral() -> bool` | literally `as_i64().is_some()`: `true` for every `i64` including `i64::MAX`, `false` for `1.5`, for `2^63` and for anything beyond |
| `as_i64() -> Option<i64>` | the exact integer when there is one, and the same question `is_integral` answers |
| `exact_text() -> String` | the exact decimal spelling, no exponent |

`Display` is the carried decimal. Below `2^53` that is the identical string it always was, and so
every fixture in this repository is unmoved; above it, `Number::from(i64::MAX)` prints
`9223372036854775807` rather than the `9223372036854775808` its binary64 rounds to, and the `2^63`
that comes back off the wire prints `9223372036854775808` rather than the `9223372036854776000` its
shortest decimal would have claimed. `Display` reaches diagnostics only — a report quotes through
`serde_json` (`report.rs:417-422`), so no persisted byte moves.

## Admission

One grammar, three implementations, one corpus. **One grammar means one answer per vector, in every
language, including the vectors nobody thought to write** — `"AA=A"` is padding followed by data,
which `BASE64_PATTERN`, the browser regular expression and `is_padded_base64` all refuse and a
hand-written Go scan admitted. The corpus is where that is settled, so a grammar restated in a third
language is checked rather than trusted; every vector below is answered by all three lanes.

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
2. The **decimal** half of the read door: `Node::from_value` reads through `serde_json`'s
   `arbitrary_precision` **and** `Serialize` writes the exact digits, together behind one format
   version — neither alone, because either alone breaks *The round-trip law*. The integer half is
   done here; until the decimal half lands, an authored `0.1000000000000000000001` is the binary64
   `0.1`. Filed as story:primitive-canonical-serialization.
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
