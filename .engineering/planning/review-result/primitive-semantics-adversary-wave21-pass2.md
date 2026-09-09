---
format: aep.planning-md/1
id: review-result:primitive-semantics-adversary-wave21-pass2
kind: review-result
status: active
title: Primitive semantics adversary, wave 21, pass 2
relations:
- reviews: story:review-primitive-semantics
revision: 1
---
unit: story:review-primitive-semantics — worktree ess-primitive-semantics-wave21 at HEAD 33f4568 (base 2900f628)
verdict: red
cases: executed 1094→1099, red 4
origin: introduced 3, pre-existing 1, undecided 0
wrote-outside-worktree: none (2 logs written to the unit's TMPDIR parent were moved into assigned scratch; see part 6)
needs-coordinator: no

Recorded by the wave-21 coordinator from the aep-drive:adversary 0.8.1 (Opus) report as returned, pass 2, 2026-09-09. Harness accounting: 136,921 sub-agent tokens, 55 tool uses, 13.4 min. Workstation path prefixes removed; nothing else changed.

## 1. `git --no-pager diff --stat`

`git --no-pager diff --stat` is **empty** — both files I added are new. `git status --short`:

```
?? crates/specify/ess-primitives/tests/adversary_round_trip_law_pass2.rs
?? crates/verify/ess-conformance/tests/adversary_integer_bound_pass2.rs
```

```
 .../ess-primitives/tests/adversary_round_trip_law_pass2.rs    |  98 ++++++++++
 .../ess-conformance/tests/adversary_integer_bound_pass2.rs    | 126 ++++++++++++
```

Both paths are test files. No implementation file, no `.engineering/planning/` file, no corpus file was touched.

## 2. The cases, and their red output when written (each run alone, before the suite)

**A1** `crates/specify/ess-primitives/tests/adversary_round_trip_law_pass2.rs:39` — `a_predicate_literal_keeps_its_admission_across_one_write`. **RED now.**

```
thread 'a_predicate_literal_keeps_its_admission_across_one_write' (1653543) panicked at crates/specify/ess-primitives/tests/adversary_round_trip_law_pass2.rs:39:9:
assertion `left == right` failed: 1.0000000000000000001: built integral=false, written 1.0, read back integral=true
  left: true
 right: false
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out
```

**A2** `.../adversary_round_trip_law_pass2.rs:88` — `exact_text_is_always_the_decimal_grammar_it_says_it_is`. **GREEN.** Theory disproved (Rust `f64` `Display` never emits an exponent, so `exact_text` on a `Repr::Binary64` still matches `DECIMAL_PATTERN`). Kept as added coverage; not a finding.

**A3** `crates/verify/ess-conformance/tests/adversary_integer_bound_pass2.rs:65` — `every_integer_the_corpus_admits_is_one_the_generated_codec_decodes`. **RED now.**

```
thread 'every_integer_the_corpus_admits_is_one_the_generated_codec_decodes' (1657303) panicked at crates/verify/ess-conformance/tests/adversary_integer_bound_pass2.rs:65:9:
integer-two-to-63: conformance admits the token 9223372036854775808 as an Integer and the generated codec's int64/i64 decoder refuses it
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out
```

**A4** `.../adversary_integer_bound_pass2.rs:97` — `a_value_admitted_as_an_integer_yields_the_integer_it_was_admitted_as`. **RED now.** It fails on the *first* token, `9223372036854775807` — not on `2^63`:

```
thread 'a_value_admitted_as_an_integer_yields_the_integer_it_was_admitted_as' (1657631) panicked at crates/verify/ess-conformance/tests/adversary_integer_bound_pass2.rs:97:9:
assertion `left == right` failed: 9223372036854775807: admitted as an Integer = true, as_i64 = None
  left: true
 right: false
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out
```

**A5** `.../adversary_integer_bound_pass2.rs:122` — `two_declared_integers_are_not_one_value_in_an_expect_event_payload`. **RED now.**

```
thread 'two_declared_integers_are_not_one_value_in_an_expect_event_payload' (1662169) panicked at crates/verify/ess-conformance/tests/adversary_integer_bound_pass2.rs:122:5:
assertion `left != right` failed: an expect_event asserting i64::MAX is satisfied by an event carrying i64::MAX - 511
  left: Number(Number(Exact { units: 9223372036854775808, scale: 0, binary: 9.223372036854776e18 }))
 right: Number(Number(Exact { units: 9223372036854775808, scale: 0, binary: 9.223372036854776e18 }))
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out
```

## 3. The gate, after the cases existed

All prefixed with the unit's `TMPDIR`, run from the worktree root.

| command | exit |
|---|---|
| `cargo fmt -p ess-primitives -p ess-domain -p ess-conformance -p ess-gen -p ess-synth -- --check` | 0 |
| `cargo clippy --locked -p ess-primitives -p ess-domain -p ess-conformance -p ess-gen -p ess-synth --all-targets -- -D warnings` | 0 |
| `cargo test --locked --no-fail-fast -p ess-primitives -p ess-domain -p ess-conformance -p ess-gen -p ess-synth` | **101** |
| `cargo xtask generate --check` → `projections are up to date` | 0 |
| `cargo xtask schema --check` → `schemas/generated/ess.schema.json: current` | 0 |
| `task consumer-check` | 201 (expected: the 9 `Number`/`Repr` cells only — 2 stale, 7 unaccounted; nothing else) |

Suite totals, summed from every `test result:` line (full log in scratch, `suite-after-nff.log`):

```
passed 1095  failed 4  executed 1099
```

`1099 − 5 (my cases) = 1094`, which is the number the implementing state reported in commit `33f4568` (`test 0 (1088 -> 1094 executed)`). The two failing binaries:

```
test result: FAILED. 0 passed; 3 failed; ...   (ess-conformance adversary_integer_bound_pass2)
test result: FAILED. 1 passed; 1 failed; ...   (ess-primitives  adversary_round_trip_law_pass2)
error: test failed, to rerun pass `-p ess-conformance --test adversary_integer_bound_pass2`
TEST_EXIT:101
```

## 4. Findings

| ID | file:line | category | severity | verdict | origin | finding |
|---|---|---|---|---|---|---|
| F1 | `crates/specify/ess-primitives/src/facts.rs:571` | property | blocker | CONFIRMED | introduced | **The round-trip law's admission half fails for `FactValue::parse_literal`.** `parse_literal("1.0000000000000000001")` → `Repr::Exact{units:10000000000000000001, scale:19}` → `is_integral()==false`, `as_i64()==None`; written it is `1.0`, and read back `is_integral()==true`, `as_i64()==Some(1)`. **What was measured:** `adversary_round_trip_law_pass2.rs:39`, exit 101. **What reaches it:** `predicate.rs:267` (`Operand::parse` → `parse_literal` on any authored guard whose right-hand side has >17 significant digits) and `predicate.rs:1221` (a quoted element of an `any_of:` list); the value is written back out through `Predicate::serialize` (`predicate.rs:1359`) / `to_node` (`predicate.rs:1121` → `value_node:1206` → `Node::Number`), which emits the binary64. **Origin:** at `2900f628` `parse_literal` built `Number(f64)` (`facts.rs:230-233` at base), so admission was stable; the exact `Repr` introduces the flip. **The fix I am naming and not applying:** `Number::is_integral` must ask the same question of a `Repr::Exact` with `scale > 0` that `Repr::of_binary64` would ask of its `binary` field — or `parse_decimal` must not keep an exact value whose `binary` is integral while the exact value is not. Note the direction: `false → true`, i.e. a value refused as an `Integer` before a write is admitted after one. |
| F2 | `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json:44` | acceptance | blocker | CONFIRMED | introduced | **The corpus admits an `Integer` no generated codec can decode.** Vector `integer-two-to-63` (`9223372036854775808`) is `admitted: true`, and all three admission lanes agree. `ess-synth` maps `Primitive::Integer` to Go `int64` (`crates/generate/ess-synth/src/go/layout.rs:867`) and decodes it through `json.Number.Int64()` = `strconv.ParseInt(_,10,64)` (`crates/generate/ess-synth/src/go/http.rs:1304-1313`), which refuses that token with a range error; `serde_json::from_str::<i64>` refuses it identically. The JSON Schema publishes `{"type":"integer"}` (`ess-gen/src/types.rs:492`), which accepts it. **What was measured:** `adversary_integer_bound_pass2.rs:65`, exit 101. **What reaches it:** any `expect_event`/`eventually_event` payload or command input carrying that token for an `Integer` field — admitted at `input.rs::primitive_value:591`, at `admission.rs::payload_agrees_with_its_shape:60`, at `runtime.go::integral` and at `coverage-admission.js::primitiveAdmits`, and refused by the implementation the same specification generates. **Origin:** at base `is_integral` was `fract()==0 && |v| < 2^53` (`facts.rs:65-67` at base), so the token was refused by the Rust admitter; the inclusive `2^63` bound is what admits it. **Fix I am naming:** the four admitters' `Integer` range should be the range `as_i64` answers on, i.e. exclude `+2^63`; the design page's own `2^63` argument then belongs to the shortest-decimal carrier, not to the admitted range. |
| F3 | `crates/specify/ess-primitives/src/facts.rs:138` | boundary | blocker | CONFIRMED | introduced | **`is_integral` and `as_i64` disagree on `i64::MAX` itself, not only on `2^63`.** The ordinary token `9223372036854775807` — the top of the `Integer` range the design page declares — is read through `Node::from_value` as the binary64 `2^63`, so `Holds::admits` says yes and `Number::as_i64()` says `None`. The doc's carve-out is written for "the `2^63` that comes back off the wire"; the value a reader actually types is `i64::MAX`, and it lands in the same hole. **What was measured:** `adversary_integer_bound_pass2.rs:97`, exit 101, failing on the first token. **What reaches it:** any suite or candidate document naming `9223372036854775807` for an `Integer` field. **Origin:** introduced (at base the token was not admitted at all). Distinct from F2: F2 is one token nobody would type, F3 is the declared maximum. |
| F4 | `crates/specify/ess-primitives/src/node.rs:52` | acceptance | warning | CONFIRMED | pre-existing | **Two `Integer`s the model declares distinct are one value in a payload.** `9223372036854775296` and `9223372036854775807` are both inside `[i64::MIN, i64::MAX]` and both admitted, and both become `Exact{units: 9223372036854775808}` on the read door, so an `expect_event` asserting one is satisfied by an implementation publishing the other. This is the acceptance statement's "without losing promised integer precision", for the `Integer` row. **What was measured:** `adversary_integer_bound_pass2.rs:122`, exit 101. **What reaches it:** every `expect_event` payload comparison; the Go runtime is worse, since `encoding/json` into `any` is also `float64`. **Origin:** pre-existing — `Node::from_value`'s number arm is comment-only in this diff, and at base `Number` was `Number(f64)`. **Note for routing:** the design page names this and defers it to the canonical-serialization stage ("What stage two changes"), so the story's acceptance is not met for the `Integer` row within this wave's declared bound. This is a story/blocker call, not a code fix inside U1. |
| F5 | `docs/design/review-primitive-semantics.md` (§ *The exact representation*, decision 3) | judgement | note | CONFIRMED | introduced | The justification for `Ord`'s totality is factually wrong, even though the conclusion holds. It says "no `Exact` value can have a binary64 in that range" (magnitude ≥ 10³⁸ or < 10⁻³⁸). `parse_decimal("1e-41")` yields `Repr::Exact{units:1, scale:41}` with `binary = 1e-41`, and `Repr::of_binary64(1e20)` yields `Exact{units:10²⁰, scale:0}`. The two arms *do* straddle magnitudes in that band. I could not construct an ordering violation from it (two `Exact` values sharing one `binary` are always integral-in-range, and no `Binary64` can carry such an `f64`), so the order is total by an argument the page does not make. **What reaches it:** the next reader who relies on the stated invariant. |

## 5. Attacked and could not break

- **The base64 grammar across three lanes.** Exhaustive cross-check of `is_padded_base64` (Rust logic), `runtime.go::paddedBase64` (compiled and run) and the browser regex (run under `node`) over all 9,331 strings of length 0–5 from `{A,=,!,/,z,9}`: **0 divergences**. `BASE64_PATTERN` and `base64.StdEncoding.DecodeString` agree with them on `AB==`, `AAB=`, `//==`, `++==` and the empty string.
- **The UUID grammar across three lanes.** 783 single-character insert/delete/replace mutants of a canonical UUID (including `-`, `+`, space, tab, `é`, Cyrillic `а`): **0 divergences** between `is_canonical_uuid`, `canonicalUUID` in Go and the browser regex. JS `$` does not match before a trailing newline, so the trailing-newline vector holds.
- **`exact_text` vs `DECIMAL_PATTERN`** — A2, green. Rust's `f64` `Display` never emits an exponent, so the `Repr::Binary64` fallback still spells a bare decimal at `1e300`, `-1e300`, `1e-300` and `f64::MIN_POSITIVE`.
- **`Number::new` as a fixed point of write∘read**, and `From<i64>`/`From<usize>`/`From<u32>` admission stability across a write — all hold.
- **The `-0.0` / `0.0` pair.** `Ord`'s exact arm makes them equal, `Serialize` still writes `-0.0` (corpus `negative-zero`, `json: "-0.0"`), and no reader I found compares number bytes rather than values.
- **`payload_agrees_with_its_shape` vs `payloadAgreesWithShape`.** The two skip the same things (absent field, explicit `null`) and dispatch the same five `Holds` arms; I found no input on which they disagree.
- **The corpus lanes.** Both the Go and the node harnesses iterate every one of the 45 admission vectors with no `continue`, and both fail loudly on an empty selection. No vector is unanswered in any lane.
- **The removed classification entry.** `facts::fn::exact_of_binary64` is gone from the tree entirely; `task consumer-check`'s stale/unaccounted list is exactly the expected 9 cells (2 `Number`, 7 `Repr`) and nothing else.

## 6. Paths written outside the worktree

**None remaining.** Two files were written to the parent of the unit's `TMPDIR` (`suite-after.log`, `suite-after-nff.log`, via a `$TMPDIR/../` redirect) and have been moved into the assigned scratch directory `target/review-boundaries-21/scratch/`. Also created, all inside assigned scratch: `scratch/b64/{main.go,js.mjs}`, `scratch/uuid/{main.go,js.mjs}`, `scratch/cases.txt`, `scratch/ucases.txt`, `scratch/{js,go,ujs,ugo}.txt`. `go run` used the assigned `TMPDIR`; nothing was written under `/tmp`.

```findings
- file: crates/specify/ess-primitives/src/facts.rs
  line: 571
  category: property
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'FactValue::parse_literal of a decimal whose binary64 is integral (1.0000000000000000001, 9007199254740992.5) builds a Number with is_integral()==false and as_i64()==None that becomes is_integral()==true and as_i64()==Some after one write and read, so the round-trip law''s admission half fails on the constructor the design page''s own table names.'
- file: crates/specify/ess-primitives/tests/vectors/primitive-semantics.json
  line: 44
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'the corpus vector integer-two-to-63 asserts 9223372036854775808 is an admitted Integer and all three admitters agree, while the codec the same specification generates decodes it through json.Number.Int64 / i64 and refuses it, so admission is not equivalent across conformance and generated codecs.'
- file: crates/specify/ess-primitives/src/facts.rs
  line: 138
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'the ordinary token 9223372036854775807 — the declared top of the Integer range — is read as the binary64 2^63, so Holds::admits says yes while Number::as_i64 returns None, and the design page''s carve-out is written only for the 2^63 that comes back off the wire.'
- file: crates/specify/ess-primitives/src/node.rs
  line: 52
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: '9223372036854775296 and 9223372036854775807 are both declared-admissible Integers and both become one Node::Number, so an expect_event asserting one is satisfied by an implementation publishing the other and the acceptance statement''s "without losing promised integer precision" does not hold for the Integer row.'
- file: docs/design/review-primitive-semantics.md
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'the stated reason Ord is total — "no Exact value can have a binary64 in that range" — is false, since parse_decimal("1e-41") and Repr::of_binary64(1e20) are both Exact with binary64s in the named band; the conclusion happens to hold for a reason the page does not give.'
```
