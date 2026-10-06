---
format: aep.planning-md/3
id: review-result:ess-054-w3-2-adversary-1
kind: review-result
status: active
title: Adversary pass 1, unit W3-2 binding literals and enum attributes
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-445
- reviews: story:feature-request-450
revision: 1
---
unit: W3-2 literals-variant-attributes, commit c576c9a21 (range 97271a43d..c576c9a21) plus my 4 uncommitted test files
verdict: NEEDS-CHANGE
cases: executed 20→30, red 6
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 locations (scratch, build dir, Go cache), listed in part 6
needs-coordinator: finding 5 contradicts the unit's own assertion at `crates/generate/ess-gen/tests/binding_literal_scalar.rs:122`, which pins a `Decimal` constant as a JSON number. You need to decide whether a `Decimal` constant is written as a string or a number.

## 1. What I touched

`git --no-pager diff --stat` is empty: every addition is a new, untracked test file. From `git status --short`:

```
?? crates/generate/ess-gen/tests/adversary_w3_2_pass1.rs        (131 lines)
?? crates/generate/ess-synth/tests/adversary_w3_2_pass1.rs      (185 lines)
?? crates/generate/schema-contract/tests/adversary_w3_2_pass1.rs (153 lines)
?? crates/specify/ess-domain/tests/adversary_w3_2_pass1.rs      (254 lines)
```

All four are test files. I edited no implementation file.

## 2. Cases added

Each case was run alone first, before any suite run. Red output is quoted as captured; the line numbers are from before I ran `rustfmt` on my own files.

| File | Case | Asserts | Now |
|---|---|---|---|
| ess-domain | `adv_w3_2_not_equal_and_negated_equal_agree_on_unfilled_optional_attribute` | `operator.arity != 2` and `not: operator.arity == 2` give the same answer for every variant | red |
| ess-domain | `adv_w3_2_enum_attribute_literal_naming_no_variant_is_refused` | `operator.family == Numerik` is refused, as `family == Numerik` is (control passes) | red |
| ess-domain | `adv_w3_2_truthiness_of_text_attribute_agrees_with_the_evaluator` | `operator.label` holds where the evaluator holds `>` truthy | red |
| ess-domain | `adv_w3_2_ordering_a_boolean_attribute_is_refused_as_for_a_boolean_fact` | `operator.takes_number < true` is refused, as `flag < true` is (control passes) | red |
| ess-domain | `adv_w3_2_ess22_verdict_unchanged_after_ess23_in_one_thread` | an ess/22 document gets the same verdict before and after an ess/23 assembly in the same thread | green |
| schema-contract | `adv_w3_2_enum_typed_attribute_accessor_answers_the_wire_spelling` | the accessor answers `numeric`, the wire spelling of `Family` | red |
| schema-contract | `adv_w3_2_single_variant_enum_gets_its_accessor` | Rust, Go and TypeScript each emit an accessor for a one-variant enum | green |
| schema-contract | `adv_w3_2_typescript_accessors_compile_and_answer` | `tsc --strict` compiles the accessors and node runs them with the declared values | green |
| ess-gen | `adv_w3_2_asyncapi_decimal_constant_is_a_value_of_its_input_schema` | the AsyncAPI `share` constant is `"0.5"`, matching the decimal-string schema in the same document | red |
| ess-synth | `adv_w3_2_typed_binding_constants_compile_in_rust_and_go` | the generated Rust passes `cargo check` and the generated Go passes `go vet`, covering Boolean/Integer/Decimal constants, newtypes, `-3`, and a YAML integer over `Decimal` | green |

Red output, verbatim:
```
`operator.arity != 2` lowered to Never and `not: operator.arity == 2` lowered to Not(AnyOf { path: FactPath(operator), values: [Text("GreaterThan"), Text("LessThan")] }) disagree for `Contains`
  left: false
 right: true
`operator.family == Numerik` names no variant of `demo.rules.Family` and was admitted, lowered to Never
`operator.label` was admitted and lowered to Never, which does not hold for `GreaterThan`, whose label `>` the evaluator holds truthy
`operator.takes_number < true` orders a Boolean and was admitted, lowered to Never
Operator::family() for `gt` answers Family's wire spelling: ... Self::V1 => "Numeric",   (ProbeRulesFamily: #[serde(rename = "numeric")] V0)
the `Decimal` constant is stated as a value its input's decimal-string schema admits
  left: Number(0.5)
 right: String("0.5")
```

I also wrote and then removed one of my own cases, `input.operator.takes_number` partition. It failed for a reason older than this unit: the control `input.family == Numeric` / `Textual` is also refused with `non_exhaustive_branches`. So `input.`-namespaced guards never got a coverage proof in the first place.

## 3. Suite run, after the cases existed

```
cargo test --no-fail-fast -p ess-domain --locked --offline --test adversary_w3_2_pass1 --test enum_attributes --test binding_mapping_scalar --test binding_stored_read_idiom   exit=101
  adversary_w3_2_pass1 FAILED 1 passed; 4 failed | binding_mapping_scalar ok 5 | binding_stored_read_idiom ok 2 | enum_attributes ok 7
cargo test --no-fail-fast -p schema-contract ... --test adversary_w3_2_pass1 --test enum_attributes   exit=101
  adversary_w3_2_pass1 FAILED 2 passed; 1 failed | enum_attributes ok 3
cargo test --no-fail-fast -p ess-gen ... --test adversary_w3_2_pass1 --test binding_literal_scalar --test enum_attributes   exit=101
  adversary_w3_2_pass1 FAILED 0 passed; 1 failed | binding_literal_scalar ok 1 | enum_attributes ok 1
cargo test -p ess-synth ... --test adversary_w3_2_pass1 --test binding_literal_scalar   exit=0
  adversary_w3_2_pass1 ok 1 | binding_literal_scalar ok 1
```

The before-count of 20 comes from the implementor's `final-*.log` for the same 7 targets.

## 4. Findings (all cover c576c9a21)

| # | file:line | What breaks | Case | Verdict / origin |
|---|---|---|---|---|
| 1 | `crates/specify/ess-domain/src/expression/attributes.rs:47` | `Not` is lowered two-valued over a membership. For a variant that leaves an `Optional` attribute empty, `not: a == v` holds while `a != v` does not. The evaluator answers `Unknown` for both, so this breaks the module doc's "agree by construction". **Reached by:** any ess/23 guard, invariant or filter that negates a comparison on an `Optional` attribute; the guide's own model uses `Optional<Integer>`. **Fix:** lower `not` over an attribute read three-valued, or leave it as written. | `adv_w3_2_not_equal_…` | NEEDS-CHANGE / introduced |
| 2 | `crates/specify/ess-domain/src/expression/attributes.rs:193` | The lowering runs before the checker and admits any literal whose scalar kind matches. Two cases: a word that names no variant of an enum-typed attribute, and an ordering over a `Boolean` attribute. Both become `never` with no diagnostic, so a typo silently turns an outcome off. **Reached by:** any guard over an enum-typed or `Boolean` attribute. **Fix:** in `Read::admits`, check enum-typed literals against the attribute enum's variants and leave orderings over `Bool` unlowered, so the checker refuses them. | `adv_w3_2_enum_attribute_literal_…`, `adv_w3_2_ordering_…` | NEEDS-CHANGE / introduced |
| 3 | `crates/specify/ess-domain/src/expression/attributes.rs:59` | Truthiness of a non-`Boolean` attribute is lowered as `== true`, giving `never`. The checker admits truthiness over any scalar, and the evaluator holds non-empty text and non-zero numbers truthy. **Reached by:** `when: plan.label`-style guards; uncommon. **Fix:** use `is_truthy`, or leave non-`Boolean` truthiness unlowered. | `adv_w3_2_truthiness_…` | NEEDS-CHANGE / introduced |
| 4 | `crates/generate/ess-gen/src/types.rs:1314` | `x-ess-attributes` writes an enum-typed attribute's value as the authored variant name. The Rust, Go and TypeScript accessors therefore return `"Numeric"`, which the generated `Family` type (`rename = "numeric"`) cannot read. **Reached by:** an enum-typed attribute whose enum declares `wire:`. **Fix:** write the attribute enum's variant wire spelling. | `adv_w3_2_enum_typed_attribute_…` | NEEDS-CHANGE / introduced |
| 5 | `crates/generate/ess-gen/src/asyncapi.rs:1570` | The AsyncAPI reaction states a `Decimal` constant as the JSON number `0.5`. The same document types `Decimal` as a string with format `decimal`, and the unit's own `x-ess-attributes` writes it as "the decimal string". The unit's test pins the number at `binding_literal_scalar.rs:122`. **Reached by:** any `Decimal` binding constant. **Fix:** return a string for `Decimal` in `literal_value`, and change the pinned assertion (your call). | `adv_w3_2_asyncapi_decimal_…` | NEEDS-CHANGE / introduced |

I checked the origin for all five by reading the base: before this unit, attributes did not exist and `Decimal` binding constants were refused.

## 5. Attacked and could not break

- **Byte identity for #445 (focus 1):** old text and enum literals take the unchanged `Text` / `Variants` arms. `literal_mapping` in the IR is unchanged, and `literal_primitive` returns `None` for String and enum targets, so the interpreter, synthesis, AsyncAPI and ess-synth paths emit the same bytes as before (read, not run).
- **`mapping:` vs `sets:` parity:** both route through the same `scalar_representation` / `literal_representation`. I found no literal that one admits and the other refuses.
- **Typed constants:** the generated Rust and Go adapters compile.
- **TypeScript accessors:** they compile under `--strict` and answer the declared values. Single-variant enums get accessors in all three lanes.
- **Thread-local ess/23 flag:** no leak between two assemblies in one thread. There is no `catch_unwind` in production code, so the flag is always restored.
- **View-filter and invariant lowering:** the compiler takes the domain's already-lowered filter (`resolve.rs:3814`).
- **Ordering and `defined()` on `Optional` attributes:** both agree with the evaluator.

## 6. Paths written outside the worktree

- `~/.cache/ess-054-wave/W3-2/adv1/`: `env.sh`, `red-domain.log`, `red-domain-namespace.log`, `red-domain-ordering.log`, `red-schema-contract.log`, `red-gen.log`, `run-synth.log`, `suite-domain.log`, `suite-schema.log`, `suite-gen.log`, `suite-synth.log`, `tmp/`
- `/dev/shm/ess-054/W3-2`: build dir, already removed with `cargo clean` (1.3 GiB)
- `~/.cache/b10x-go-cache/W3-2`: entries from `go vet`

## 7. Findings block

```findings
- file: crates/specify/ess-domain/src/expression/attributes.rs
  line: 47
  category: property
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'not over an attribute comparison is lowered two-valued, so not: a == v holds for a variant leaving an Optional attribute unfilled while a != v and the evaluator do not'
- file: crates/specify/ess-domain/src/expression/attributes.rs
  line: 193
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'attribute comparisons are lowered before the checker, so a non-variant word against an enum-typed attribute and an ordering over a Boolean attribute become never instead of being refused'
- file: crates/specify/ess-domain/src/expression/attributes.rs
  line: 59
  category: property
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'truthiness of a non-Boolean attribute is lowered as equality with true, giving never where the evaluator holds non-empty text and non-zero numbers truthy'
- file: crates/generate/ess-gen/src/types.rs
  line: 1314
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'x-ess-attributes and the Rust, Go and TypeScript accessors answer an enum-typed attribute with the authored variant name, which the attribute enum generated type does not accept when the enum declares wire spellings'
- file: crates/generate/ess-gen/src/asyncapi.rs
  line: 1570
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'the AsyncAPI reaction states a Decimal binding constant as a JSON number while the same document types Decimal as a decimal string, and the unit test at binding_literal_scalar.rs:122 pins the number'
```
