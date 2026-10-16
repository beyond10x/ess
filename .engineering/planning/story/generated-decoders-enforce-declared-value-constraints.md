---
format: aep.planning-md/3
id: story:generated-decoders-enforce-declared-value-constraints
kind: story
status: draft
title: Every generated decoder enforces the value constraints its type publishes, or the generator refuses the type
tags:
- adopter-report
- defect
refs:
- provider: github
  reference: beyond10x/ess#579
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
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
