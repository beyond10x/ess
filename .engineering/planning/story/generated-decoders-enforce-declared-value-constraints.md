---
format: aep.planning-md/3
id: story:generated-decoders-enforce-declared-value-constraints
kind: story
status: active
title: Every generated decoder enforces the value constraints its type publishes, or the generator refuses the type
tags:
- adopter-report
- defect
refs:
- provider: github
  reference: beyond10x/ess#579
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/invariant.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/items.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/runtime/invariant.go
- confidence: inferred
  path: crates/generate/ess-synth/tests/invariant_check/value_constraints.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize.rs
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T10:20:19Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-16T10:20:19Z", actor: "human:timo", revision: 3}
---
## Outcome

Every generated decoder refuses a value outside the constraints its type declares, naming the
path: the Rust model crate's wire decoder, the Rust types target's serde path, and the Go and
TypeScript decoders. Where a constraint is published (`x-ess-alphabet`, a length, a bound, a prefix,
an invariant the predicate language evaluates) it is enforced at the decoder, or the generator
refuses the type by name. A published constraint is never left to the application.

Findings covered: https://github.com/beyond10x/ess/issues/579, part 2 (a downstream adversarial
review on ess 0.60.0, `ess/25`).

## Fit review

1. **Need.** A String newtype declares `alphabet:`; the projected schema publishes it, but the
   generated model decoder only checks that the value is a string and the types target is
   `#[serde(transparent)]` over `String`. A value with a space, a control character, a non-ASCII
   character or the empty string reaches the application through a generated route. Minimal
   reproduction to write first: `demo.units.Unit`, newtype of String with `alphabet:` of printable
   ASCII without space and `length: {min: 1}`, a command `Record {unit}` and a served component;
   decode `"m s"`, `"\u007f"`, `"µm"` and `""`.
   Requester's expectation (theirs): both decoders refuse a value outside `alphabet` and `length`,
   naming the path.
2. **Class.** Defect: the projection publishes a constraint the generated code does not hold, and
   the decoder sits behind the generated `POST` routes. The constraint was designed to be a type
   fact (`story:string-newtype-declares-its-alphabet`, implemented).
3. **Already expressible?** The constraint is expressible today; its enforcement is not, and the
   consumer cannot fix generated code.
4. **Fit.** One rule for every decoder and every constraint kind, not an alphabet patch in one
   target. The same constraints are already evaluated by the conformance observers
   (`story:response-string-newtype-constraints-checked-not-refused`, active) and by Entity Runtime
   lowering for alphabet and text count (`story:entity-runtime-lowers-alphabet-and-text-count`,
   implemented); `story:entity-runtime-lowering-enforces-input-type-invariants` and
   `story:cli-type-invariants` ask the same of two other edges. This story is the generated-code
   edge, sharing the one predicate evaluator each runtime already carries.
5. **Second adopter.** A currency code newtype (`alphabet: A-Z`, length 3) on a payment API; a slug
   newtype on a content API. Both publish the rule and expect the generated server to hold it.
6. **Cost.** Generated Rust, Go and TypeScript code changes; a client that sends a value outside
   the declared alphabet starts receiving a decode refusal (breaking for that client, listed in the
   CHANGELOG). No source or suite format change. A constraint a target cannot evaluate is refused at
   generation by name.
7. **Alternatives.** (a) Change nothing and document that decoders check shape only: the schema
   keeps promising what the server does not hold. (b) Alphabet only, Rust only: the request as
   filed, one cell of the grid. (c) Every declared constraint in every generated decoder, or a named
   refusal (chosen).

## Decisions

Accept, redesigned from "enforce alphabet in two Rust decoders" to "every generated decoder enforces
every constraint its type publishes, or the generator refuses the type by name".

## Acceptance

- `model_decoder_refuses_a_value_outside_the_alphabet`: the reproduction's Rust model decoder
  refuses each of the four values with a decode error naming the path; a value inside the alphabet
  decodes.
- `types_target_refuses_a_value_outside_the_alphabet`: deserializing the same values into the
  `generate types --target rust` newtype fails, naming the constraint.
- `go_and_typescript_decoders_agree`: the Go and TypeScript decoders give the same verdict on the
  same values.
- `every_published_constraint_is_enforced_or_refused`: a corpus test walks every constraint kind the
  schema projection publishes and requires, per target, an enforcing decoder or a named generation
  refusal.

## Scope (inferred)

`crates/generate/ess-synth/src/rust/` (wire decoders), `crates/generate/ess-gen/src/types.rs`
(Rust types target), `crates/generate/ess-synth/src/go/`, the TypeScript generator, CHANGELOG.

## State

2026-11-02 (store hygiene H0 of wave W1, reconcile roadmap section 2d; decision D9 option A, kept active with a dated note): Rust half done; Go and TypeScript half remains; scheduled W3.

## Scope

Derived 2026-11-07 by `story-scoper`. Every line is **cited** (read from the story or the tree) or
**inferred** (a reading that could be wrong).

- **Already delivered (0.62.0, beyond10x/ess#579):**
  - the Rust model decoder (`453bde9ad`), acceptance test
    `model_decoder_refuses_a_value_outside_the_alphabet`;
  - Rust types alphabet and length (`1d94e6977`), acceptance test
    `rust_refuses_a_value_outside_a_declared_alphabet_or_length`.

  Both are cited.
- **Primary surface, remaining part A:** the generated Go server decoder in
  `crates/generate/ess-synth/src/go/` — cited.
  - `go/http.rs` (`decode<Type>` for `ResolvedBody::Newtype` returns the ctor with no constraint
    check) — cited.
  - `go/items.rs` (`newtype`, `alphabet_doc`, `prefix_doc`: documented "and does not check it") —
    cited.
  - `go/invariant.rs` (`preflight`, entity invariants only today) and
    `go/runtime/invariant.go` — inferred.
- **Remaining part B:**
  - the Go and TypeScript types targets, and Rust prefix and invariants:
    `crates/generate/schema-contract/src/realize.rs` (the `model_invariants` and `model_alphabet`
    obligations) and `realize/{go,ts,rust}.rs` — cited.
  - The TypeScript types target emits declarations only, so "enforce it or refuse the type" there
    is a design decision: a new runtime decoder, or refusing every constrained type.
- **Symbols:** `broken_constraint`, the Rust shape to mirror, in `ess-synth/src/rust/wire.rs` and
  `rust/items.rs` — cited.
- **Tests:** `crates/generate/ess-synth/tests/invariant_check/value_constraints.rs` and
  `crates/generate/schema-contract/tests/model_ergonomics.rs` — inferred.
- **Documents:** `CHANGELOG.md`, breaking for a client that sends a value outside a declared
  constraint — cited.
- **Confidence:** medium. The Go decoder site is pinned by the tree; the TypeScript leg is a design
  reading.
- **Safety fact:** a newtype with no alphabet, prefix or invariant keeps its exact Go and
  types-target bytes; the check is emitted only where a constraint is declared (unproven).
- **Split:** part A, the Go server decoder, is one unit. Part B waits on the TypeScript decision.
  `every_published_constraint_is_enforced_or_refused` lands with or after part B.
