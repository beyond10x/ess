---
format: aep.planning-md/3
id: review-result:adversary-wave-20261007b-u2-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 2026-10-07b unit 2 (exact-numbers crate feature)
relations:
- reviews: story:generated-rust-types-make-exact-numbers-a-crate-feature
revision: 1
---
unit: story:generated-rust-types-make-exact-numbers-a-crate-feature (worktree ess-wave-20261007b-u2 @ 00dcb91eff + one untracked test file)
verdict: red
cases: executed 200→207, red 1
origin: introduced 3, pre-existing 0, undecided 0
wrote-outside-worktree: none
needs-coordinator: yes

No silent precision loss in the default build was found: every number position is detected, named by its own pointer and exact by default. The one red case is the opposite direction: a union that holds no number still gets the feature.

Cases added in `crates/generate/schema-contract/tests/adversary_exact_numbers_pass1.rs`:

| case | asserts | now |
|---|---|---|
| `a_union_whose_alternatives_hold_no_number_holds_no_exact_number` | event = String + tagged union (String/Boolean): no feature, no `rust_exact_numbers` entry | red |
| `every_bundle_number_position_is_named_by_its_own_pointer` | exact 11-pointer set: map value, array items, tuple slot 1, nullable via `type` and via `anyOf`, `["integer","string"]`, union and its variant, `true`, open record, recursive `Tree` | green |
| `every_model_number_position_is_named_and_native_widths_are_not` | Newtype Integer, Map value, `List<Optional<Integer>>`, Optional field, Json, Integer variant of the `Value` union and of the raw (Binary64) union named; `i32` and Binary64 not | green |
| `event_roots_and_type_roots_reach_the_same_detection` | event root reaches `/$defs/probe.n.Count`; a root holding only a bounded integer gets no feature | green |
| `the_feature_is_declared_exactly_when_the_declarations_name_serde_json_numbers` | 9 selections (each root alone and all together): feature declared ⇔ code names `serde_json` `Number` or `Value` | green |
| `a_{bundle,model}_crate_is_exact_by_default_and_compiles_without_default_features` | default build round-trips `2.20`, `40000000000000000000001` byte-exact at every position; a consumer with `default-features = false` compiles and gets binary64 | green |

Red output (`cargo test -p schema-contract --locked --test adversary_exact_numbers_pass1 -- a_union_whose…`, exit 101):

```
panicked at …/adversary_exact_numbers_pass1.rs:319:5:
assertion `left == right` failed: named as holding JSON numbers although no alternative holds one
  left: {"/$defs/probe.n.Words"}
 right: {}
```

Suite: `cargo test -p schema-contract --locked --no-fail-fast` exit 101, 207 executed, 206 passed, 1 failed (the case above).

Findings:

- `rust.rs:312`: every `Value`-decoded union is marked, so a text-only union still forces `arbitrary_precision` (the #483 symptom) and the report says it holds JSON numbers. Reached by `ess generate types --root <event with a string-only Union>`.
- `docs/design/types-only-realizations.md:190`, `:196`: still says the Rust library uses `serde_json` with arbitrary-precision numbers and that union decoding preserves number precision; true only with the default feature.
- `rust.rs:126`: the `integer` obligation says `serde_json::Number` retains exact JSON numbers, false with `default-features = false` (the off run shows `4e+22`), while `rust_exact_numbers` at the same pointer says the opposite.

Attacked without a break: nested map, array, tuple, Optional and union positions; recursive refs; `additionalProperties`; numeric `Shape::Literal` (refused before emission); bounded and unbounded integers; event root versus type root; all roots versus one; bundle versus model; compiling every shape without default features; the publisher manifest rewrite; normalization manifests.

```findings
- file: crates/generate/schema-contract/src/realize/rust.rs
  line: 312
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: every Value-decoded union is marked rust_exact_numbers and turns the feature on even when no alternative can hold a number, so a text-only event model with a union still forces arbitrary_precision and the report claims numbers it cannot hold (red case adversary_exact_numbers_pass1.rs:323)
- file: docs/design/types-only-realizations.md
  line: 190
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the binding design still says the Rust library uses serde_json with arbitrary-precision numbers and that union decoding preserves number precision, which is now true only with the default exact-numbers feature
- file: crates/generate/schema-contract/src/realize/rust.rs
  line: 126
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the integer obligation says serde_json::Number retains exact JSON numbers, false under default-features = false, while rust_exact_numbers at the same pointer says the opposite
```
