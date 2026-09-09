---
format: aep.planning-md/1
id: review-result:primitive-semantics-adversary-wave21-pass1
kind: review-result
status: active
title: Primitive semantics adversary, wave 21, pass 1
relations:
- reviews: story:review-primitive-semantics
revision: 1
---
unit: story:review-primitive-semantics — worktree ess-primitive-semantics-wave21 at HEAD d107b53 (base 2900f628)
verdict: red
cases: executed 1088→1092, red 4
origin: introduced 4, pre-existing 0, undecided 0
wrote-outside-worktree: none
needs-coordinator: no

Recorded by the wave-21 coordinator from the aep-drive:adversary 0.8.1 (Opus) report as returned, pass 1, 2026-09-09. Harness accounting: 117,737 sub-agent tokens, 42 tool uses, 12.3 min. Workstation path prefixes removed from the report; nothing else changed.

## 1. `git --no-pager diff --stat`

```
(empty)
```
`git status --short`:
```
?? crates/verify/ess-conformance/tests/primitive_divergence.rs
```
One path, a test file. No tracked file changed; no implementation file touched.

## 2. Cases added — all in `crates/verify/ess-conformance/tests/primitive_divergence.rs`

All four are **red now**. Each was written before anything was run and run alone first.

**A. `a_node_number_is_the_number_it_was_after_a_round_trip_through_its_own_serializer`** — a `Node` parsed from an integer token equals the `Node` reparsed from what `Serialize` wrote.
```
running 1 test
thread 'a_node_number_is_the_number_it_was_after_a_round_trip_through_its_own_serializer' (1217408) panicked at crates/verify/ess-conformance/tests/primitive_divergence.rs:249:9:
assertion `left == right` failed: 9007199254740993 was written as 9007199254740992.0 and came back a different number
  left: Number(Number(Exact { units: 9007199254740992, scale: 0, binary: 9007199254740992.0 }))
 right: Number(Number(Exact { units: 9007199254740993, scale: 0, binary: 9007199254740992.0 }))
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
error: test failed, to rerun pass `-p ess-conformance --test primitive_divergence`
EXIT:101
```

**B. `an_integer_payload_is_admitted_the_same_way_before_and_after_it_is_written`** — the same value admitted as `Integer` on the way in and on the way back.
```
running 1 test
thread 'an_integer_payload_is_admitted_the_same_way_before_and_after_it_is_written' (1218610) panicked at crates/verify/ess-conformance/tests/primitive_divergence.rs:270:5:
i64::MAX was written as 9.223372036854776e+18 and is no longer an Integer when read back
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
error: test failed, to rerun pass `-p ess-conformance --test primitive_divergence`
EXIT:101
```

**C. `the_three_admitters_agree_about_padding_that_is_not_a_suffix`** — Rust `is_padded_base64`, Go `paddedBase64` and the browser `paddedBase64` regex on `"AA=A"`, `"AAAAAA=A"`, `"A==A"`.
```
running 1 test
thread 'the_three_admitters_agree_about_padding_that_is_not_a_suffix' (1223883) panicked at crates/verify/ess-conformance/tests/primitive_divergence.rs:205:5:
one grammar, three implementations:
corpus_test.go:33: Go bytes-padding-followed-by-data: primitive("bytes", AA=A) admitted=true, the design page says false
corpus_test.go:33: Go bytes-padding-followed-by-data-long: primitive("bytes", AAAAAA=A) admitted=true, the design page says false
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.49s
error: test failed, to rerun pass `-p ess-conformance --test primitive_divergence`
EXIT:101
```
Rust and node produced no line — they refuse both; only Go admits.

**D. `the_three_admitters_agree_about_the_top_of_the_declared_integer_range`** — `i64::MAX` and `i64::MIN` as `Integer` across the three lanes.
```
running 1 test
thread 'the_three_admitters_agree_about_the_top_of_the_declared_integer_range' (1224836) panicked at crates/verify/ess-conformance/tests/primitive_divergence.rs:227:5:
one grammar, three implementations:
corpus_test.go:33: Go integer-i64-max: primitive("integer", 9.223372036854776e+18) admitted=false, the design page says true
integer-i64-max: primitiveAdmits(integer, 9223372036854776000) = false, the design page says true
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.35s
error: test failed, to rerun pass `-p ess-conformance --test primitive_divergence`
EXIT:101
```
`i64::MIN` is admitted by all three; only the top of the range diverges.

(After the four red runs above, `cargo clippy --all-targets -- -D warnings` rejected three lints in my own file — `unreadable_literal` ×2, `needless_pass_by_value`. I fixed those in my file only; the four cases are still red in part 3.)

## 3. Suite run — the gate from `target/review-boundaries-21/brief.md`, package-scoped

| command | exit |
|---|---|
| `cargo fmt -p ess-primitives -p ess-domain -p ess-conformance -p ess-gen -p ess-synth -- --check` | 0 |
| `cargo clippy --locked -p … --all-targets -- -D warnings` | 0 |
| `cargo test --locked --no-fail-fast -p …` | **101** |
| `cargo xtask generate --check` → `projections are up to date` | 0 |
| `cargo xtask schema --check` → `schemas/generated/ess.schema.json: current` | 0 |
| `task consumer-check` | 201 — the stale `rust:ess_primitives::facts::Number` cells the brief excludes; not a finding |

```
running 4 tests
test an_integer_payload_is_admitted_the_same_way_before_and_after_it_is_written ... FAILED
test a_node_number_is_the_number_it_was_after_a_round_trip_through_its_own_serializer ... FAILED
test the_three_admitters_agree_about_the_top_of_the_declared_integer_range ... FAILED
test the_three_admitters_agree_about_padding_that_is_not_a_suffix ... FAILED
test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
error: test failed, to rerun pass `-p ess-conformance --test primitive_divergence`
TEST-EXIT:101
```
Totals across all binaries: **AFTER passed=1088 failed=4 executed=1092**, exit 101.
`<before>` measured by the same command with my four cases deselected (`--skip the_three_admitters_agree_about --skip a_node_number_is_the_number_it_was --skip an_integer_payload_is_admitted_the_same_way`): **passed=1088 failed=0 executed=1088**, exit 0. The tree without my cases is green; the four failures are mine alone.

## 4. Findings

| ID | file:line | category | severity | verdict | origin | finding |
|---|---|---|---|---|---|---|
| F1 | `crates/specify/ess-primitives/src/facts.rs:90` (`impl Serialize for Number`) | acceptance | blocker | CONFIRMED | introduced | `Serialize` writes the binary64 while `node.rs::from_value:47` reads an integer token exactly, so `Node` no longer round-trips through its own format. **Measured:** case A, `tests/primitive_divergence.rs:249`, exit 101 — `9007199254740993` written as `9007199254740992.0`, read back as a `Number` its own new `PartialEq` calls unequal. **Reaches it:** every suite that is emitted, persisted and re-read — `AdmittedSuite::from_json` on a written suite, a witness written by `ess conform synthesize` and reparsed, an `expect_event` payload compared to an observed one. The acceptance statement is "equivalent admission and comparison results … without losing promised integer precision"; comparison is not equivalent across one write. **Base:** `Number(f64)` with `eq = f64 ==` and `from_value` via `as_f64`, so the round trip was stable. **Fix (not applied):** either keep `from_value` on `as_f64` in this wave, or make `PartialEq` compare `get()` rather than `cmp()`; the byte-preserving `Serialize` is the constraint, so the exactness has to stop at the same door. |
| F2 | `crates/specify/ess-primitives/src/facts.rs:90` (same site, admission face) | acceptance | blocker | CONFIRMED | introduced | The same defect flips *admission*, not only equality. **Measured:** case B, `tests/primitive_divergence.rs:270`, exit 101 — `9223372036854775807` is admitted as `Primitive::Integer`, written as `9.223372036854776e+18`, and refused on re-read because `as_i64` then fails. **Reaches it:** `admission.rs:91 payload_agrees_with_its_shape`, added by this unit, turns that into `InvalidSuite` for a suite this repository itself wrote and re-read. **Base:** `is_integral` refused `i64::MAX` both times — stable. **Fix (not applied):** as F1. |
| F3 | `crates/verify/ess-conformance/src/go/runtime.go:2570` (`paddedBase64` final `return`) | contract-drift | blocker | CONFIRMED | introduced | Go admits `=` at *any* of the last two positions, not only as a suffix: `"AA=A"` and `"AAAAAA=A"` pass Go and fail `is_padded_base64`, `BASE64_PATTERN` and the browser regex. **Measured:** case C, `tests/primitive_divergence.rs:205`, exit 101, Go `admitted=true` on both. **Reaches it:** the design page's own "one grammar, three implementations, one corpus" and the corpus's own note ("A vector added here must be answered the same way by all three"); a Go conformance target admits a `Bytes` value the Rust runner and the browser replay refuse for the same suite. The corpus carries no vector with `=` anywhere but last, so no lane notices. **Base:** `primitive` had no `"bytes"` case at all and Rust admitted any text — all three agreed. **Fix (not applied):** replace the loop with "strip up to two trailing `=`, then require every remaining byte in the alphabet", i.e. the Rust shape; the trailing `return` expression is dead in every reachable case and should go with it. |
| F4 | `crates/verify/ess-conformance/src/go/runtime.go:2585` (`integral`) and `crates/verify/ess-conformance/assets/coverage-admission.js:307` (`primitiveAdmits` `integer`) | boundary | warning | CONFIRMED | introduced | Both lanes compare a float64 against `2^63` after JSON parsing has already rounded, so every integer above `9223372036854775296` — including `i64::MAX` — is refused, while Rust admits it. **Measured:** case D, `tests/primitive_divergence.rs:227`, exit 101; Go `primitive("integer", 9.223372036854776e+18) admitted=false`, node `primitiveAdmits(integer, 9223372036854776000) = false`. `i64::MIN` is fine (exactly representable). **Reaches it:** the design page's `Integer` row declares the abstract value "an exact integer in `[i64::MIN, i64::MAX]`", and this unit changed `node.rs` specifically so those values arrive exact; a suite carrying one is admitted by the Rust runner and refused by the Go and browser ones. **Base:** Go and JS admitted it (no integrality check at all) and Rust refused it — the disagreement existed but in the other direction; the code that refuses it now is entirely new in this diff. **Fix (not applied):** read the integer token, not the float — `json.Decoder.UseNumber()` plus `strconv.ParseInt` in Go, and the raw token via a reviver in JS — or state the narrower range in the design page and add the boundary to the corpus. |
| F5 | `crates/specify/ess-primitives/src/facts.rs:359` (`impl Ord for Number`) | judgement | note | CONFIRMED | introduced | `PartialEq` is now `cmp(..) == Equal` with `total_cmp` first, so `-0.0 != 0.0` for `Integer` and `Decimal` facts as well as `Binary64`. The design page's `Decimal` row says the abstract value is `units × 10⁻ˢᶜᵃˡᵉ`, under which `-0` and `0` are one value; the same page's ordering section says signed zero is the deliberate refinement. Two halves of one page disagree. **Reaches it:** an `expect_event` payload asserting `0.0` no longer matches an observed `-0.0`; at base `eq` was `f64 ==` and it did. No failing case written — a case here would assert against what the page explicitly chose, so it is a document question, not a code one. |

## 5. Attacked and could not break

* `Serialize` byte preservation for every constructor (`From<i64>/<u32>/<usize>`, `Number::new`, `parse_decimal`, `Node::from_value` on both integer and float tokens) — all still write the same `f64`; `report.rs::quote` goes through `serde_json`, not `Display`.
* `exact_cmp` as a numeric order: `binary` is the correctly-rounded value of `(units, scale)` at every construction site, rounding is monotone, and the whole-part length comparison handles the power-of-ten straddle (`999999999999999999` vs `10^18`). No counterexample found.
* `exact_of_decimal_text` boundaries: `MAX_SCALE`, `i128` overflow via `checked_mul`, `1e300`, 43-digit literals, `"."`, `"-"`, `"+1"`, `"inf"`, `"nan"`, `"1e400"` — every rejection falls back to `Binary64` and every corpus expectation still holds. `normalise`'s `u8` clamp is unreachable, not wrong.
* `is_padded_base64` and `is_canonical_uuid` in Rust: no panic path (`bytes[..len - padding]` is always in range), no multibyte divergence from Go's rune loop, `"===="`/`"A==="`/`"=AAA"`/`"AA==AA=="` all refused by all three.
* `payload_agrees_with_its_shape` vs the JS `payloadAgreesWithShape`: enum, list, map/union and null-skip branches agree; `Primitive::Binary64` cannot reach a shape (`deserialize_primitive:422` refuses it), so the "both silently refuse a legal binary64 payload" theory is dead.
* Witness generation still produces canonical UUIDs (`witness.rs:576`) and padded base64 (`witness.rs:553`), so `payload_agrees_with_its_shape` does not refuse generated suites.
* The node lane fails loudly when `node` is absent — `.expect("the required Node toolchain executes; set ESS_NODE to name it")`, not a skip.

## 6. Paths written outside the worktree

None. All scratch is inside the worktree at `target/review-boundaries-21/scratch/` — `divergence-*` go/js probe trees, `suite-after.log`, `suite-after-nff.log`, `suite-before-nff.log`. Cargo used the assigned `TMPDIR` as instructed. Lease `U1-adversary` acquired and released.

```findings
- file: crates/specify/ess-primitives/src/facts.rs
  line: 90
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'Serialize writes the binary64 while node.rs::from_value reads an integer token exactly, so a Node written and re-read is a different Number and compares unequal to itself.'
- file: crates/specify/ess-primitives/src/facts.rs
  line: 90
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'i64::MAX is admitted as Primitive::Integer on the way in and refused on the way back out of the same serializer, so payload_agrees_with_its_shape refuses a suite this repository itself wrote.'
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 2570
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'the Go paddedBase64 admits = at any of the last two positions rather than only as a suffix, so it admits "AA=A" and "AAAAAA=A" that is_padded_base64, BASE64_PATTERN and the browser regex all refuse.'
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 2585
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'the Go integral guard and the browser primitiveAdmits integer branch compare an already-rounded float64 against 2^63, refusing every integer above 9223372036854775296 including the i64::MAX the design page declares admissible and Rust admits.'
- file: crates/specify/ess-primitives/src/facts.rs
  line: 359
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'PartialEq via total_cmp makes -0.0 unequal to 0.0 for Integer and Decimal facts, which contradicts the design page Decimal row where the value is units x 10^-scale and -0 is 0.'
```
