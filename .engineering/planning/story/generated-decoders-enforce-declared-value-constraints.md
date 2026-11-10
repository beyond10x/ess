---
format: aep.planning-md/3
id: story:generated-decoders-enforce-declared-value-constraints
kind: story
status: active
title: Every generated decoder enforces the value constraints its type publishes, or the generator refuses the type
tags:
- adopter-report
- defect
- severity-medium
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#579
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/schema_bundle.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/invariant.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/runtime/invariant.go
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/go.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/go_support.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/rust.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/ts.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/model_ergonomics.rs
revision: 21
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T10:20:19Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-16T10:20:19Z", actor: "human:timo", revision: 3}
---
## Outcome

Every generated decoder refuses a value outside the constraints its type declares, naming the
path: the Rust model crate's wire decoder, the generated Go server decoder, and the Rust, Go and
TypeScript `ess generate types` targets. Where a constraint is published (`x-ess-alphabet`, a
length, a bound, a prefix, an invariant the predicate language evaluates) it is enforced at the
decoder, or the generator refuses the type by name. A published constraint is never left to the
application.

Findings covered: https://github.com/beyond10x/ess/issues/579, part 2 (a downstream adversarial
review on ess 0.60.0, `ess/25`).

## Problem

Delivered so far:

- Rust model decoder (0.62.0, `453bde9ad`), test `model_decoder_refuses_a_value_outside_the_alphabet`.
- Rust types target alphabet and length (0.62.0, `1d94e6977`), test
  `rust_refuses_a_value_outside_a_declared_alphabet_or_length`.
- Generated Go server decoder: alphabet, prefix and invariants through `BrokenConstraint()`
  (0.91.0, CHANGELOG "Breaking for a client that sends a value outside a declared constraint").

Still open on 0.92.0, the `ess generate types` targets. Reproduction (`format: ess/15`):

```yaml
types:
  - {name: demo.units.Unit,  kind: newtype, of: String, alphabet: "abcdefghijklmnopqrstuvwxyz"}
  - {name: demo.units.Code,  kind: newtype, of: String, prefix: "u-"}
  - {name: demo.units.Label, kind: newtype, of: String, invariants: ["value.count >= 1"]}
```

```
ess generate types --path spec --all-types --target rust --package demo_units --out out-rust
ess generate types --path spec --all-types --target go --package demo_units --module example.com/demo --out out-go
ess generate types --path spec --all-types --target typescript --out out-typescript
```

All three exit 0. `types-report.json` obligations:

- rust: `pattern /$defs/demo.units.Code/pattern`, `model_invariants /$defs/demo.units.Label/x-ess-invariants`
  (the prefix is not checked: `DemoUnitsCode` is `#[serde(transparent)]` over `String`; the Label
  invariant is checked by `broken_constraint` yet still reported as an obligation).
- go: `pattern` (Code), `model_invariants` (Label), `model_alphabet` (Unit); every
  `UnmarshalJSON` only decodes a string and assigns it.
- typescript: the same three, beside `typescript_nominal_identity`; the target emits declarations
  only.

## Fit review

1. **Need.** A String newtype declares `alphabet:`; the projected schema publishes it, but a
   generated decoder that only checks that the value is a string lets a value with a space, a
   control character, a non-ASCII character or the empty string reach the application.
   Requester's expectation (theirs): every decoder refuses a value outside `alphabet` and `length`,
   naming the path.
2. **Class.** Defect: the projection publishes a constraint the generated code does not hold. The
   constraint was designed to be a type fact (`story:string-newtype-declares-its-alphabet`,
   implemented).
3. **Already expressible?** The constraint is expressible today; its enforcement is not, and the
   consumer cannot fix generated code.
4. **Fit.** One rule for every decoder and every constraint kind, not an alphabet patch in one
   target. The same constraints are already evaluated by the conformance observers
   (`story:response-string-newtype-constraints-checked-not-refused`) and by Entity Runtime
   lowering (`story:entity-runtime-lowers-alphabet-and-text-count`);
   `story:entity-runtime-lowering-enforces-input-type-invariants` and `story:cli-type-invariants`
   ask the same of two other edges. This story is the generated-code edge, sharing the one
   predicate evaluator each runtime already carries.
5. **Second adopter.** A currency code newtype (`alphabet: A-Z`, length 3) on a payment API; a slug
   newtype on a content API. Both publish the rule and expect the generated code to hold it.
6. **Cost.** Generated Rust, Go and TypeScript code changes; a client that sends a value outside
   the declared constraint starts receiving a decode refusal (breaking for that client, listed in
   the CHANGELOG). No source or suite format change. A constraint a target cannot evaluate is
   refused at generation by name.
7. **Alternatives.** (a) Change nothing and document that decoders check shape only: the schema
   keeps promising what the code does not hold. (b) Alphabet only, Rust only: the request as filed,
   one cell of the grid. (c) Every declared constraint in every generated decoder, or a named
   refusal (chosen).

## Decisions

Accept, redesigned from "enforce alphabet in two Rust decoders" to "every generated decoder enforces
every constraint its type publishes, or the generator refuses the type by name".

- TypeScript: `--target typescript` refuses a type with a published constraint by name instead of
  gaining a runtime decoder, because the target emits declarations only and already reports "these
  declarations do not validate runtime JSON" (`typescript_compiler_options`), and issue #579 asks
  for no TypeScript decoder; the break is listed in the CHANGELOG.

## Acceptance

- `cargo test -p schema-contract --locked --test model_ergonomics` passes
  `rust_types_refuse_a_value_outside_a_declared_prefix_or_invariant`: for the reproduction,
  deserializing `"x-1"` into the Rust `DemoUnitsCode` fails naming the prefix; `"u-1"` decodes; the
  Rust `types-report.json` lists no `pattern` obligation for `Code` and no `model_invariants`
  obligation for `Label`.
- The same target passes `go_types_refuse_a_value_outside_a_declared_constraint`: `json.Unmarshal`
  of `"m s"`, `"µm"` and `""` into the Go `DemoUnitsUnit`, of `"x-1"` into `DemoUnitsCode` and of
  `""` into `DemoUnitsLabel` returns an error naming the type's schema pointer and the constraint;
  `"ms"`, `"u-1"` and `"a"` decode; the Go report lists none of the three obligations.
- The same target passes `typescript_types_refuse_a_declared_constraint`: the reproduction's
  TypeScript realization is `Refused` with one finding each at `/$defs/demo.units.Unit`,
  `/$defs/demo.units.Code` and `/$defs/demo.units.Label`, each detail naming its constraint; a
  plan whose newtypes declare no constraint realizes with unchanged bytes. On the reproduction,
  `ess generate types --path spec --all-types --target typescript --out out-typescript` exits
  non-zero, prints those three `<pointer>: <rule>: <detail>` lines and writes no `types.ts`.
- The same target passes `every_published_constraint_is_enforced_or_refused`: a corpus test walks
  every constraint kind the schema projection publishes and requires, per target, an enforcing
  decoder or a named generation refusal; no target reports `model_alphabet`, `model_invariants` or
  a prefix `pattern` as an obligation.

## Scope

- cited: crates/generate/schema-contract/src/realize.rs (`Plan::typescript`, the `model_alphabet`, `model_invariants` and `pattern` obligations)
- cited: crates/generate/schema-contract/src/realize/rust.rs (`#[serde(transparent)]` newtype, `broken_constraint`)
- cited: crates/generate/schema-contract/src/realize/go.rs (newtype `UnmarshalJSON`)
- inferred: crates/generate/schema-contract/src/realize/go_support.go.txt (a shared constraint helper)
- cited: crates/generate/schema-contract/src/realize/ts.rs (`emit`)
- cited: crates/edge/ess-cli/src/schema_bundle.rs (`plan.typescript()` call site)
- inferred: crates/generate/ess-synth/src/go/invariant.rs (the Go predicate emission the Go types leg mirrors)
- inferred: crates/generate/ess-synth/src/go/runtime/invariant.go
- inferred: crates/generate/schema-contract/tests/model_ergonomics.rs

confidence: medium

The Go and Rust legs mirror `broken_constraint`. A newtype with no alphabet, prefix or invariant
keeps its exact bytes in every target. The story moves `CHANGELOG.md` (its breaking sentence) and
any generated-types byte-pin rows its constrained fixtures claim.
