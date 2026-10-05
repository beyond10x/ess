---
format: aep.planning-md/3
id: story:feature-request-445
kind: story
status: draft
title: A binding mapping cannot write a Boolean or number literal
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#445
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-426b
- depends_on: story:feature-request-448
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/asyncapi.rs
- confidence: cited
  path: crates/generate/ess-gen/src/docs.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/system.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/system.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/binding_literal_scalar.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/binding_mapping_scalar.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/bindings.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/delivery_context.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/binding_literal_scalar.rs
- confidence: cited
  path: docs/design/typed-literals-and-unknown-instances.md
- confidence: cited
  path: website/docs/guides/specify/bindings-and-components.md
revision: 8
---
## Outcome
Resolve beyond10x/ess#445: A binding mapping cannot write a Boolean or number literal.

## Origin
beyond10x/ess#445, filed 2026-10-05; reported from an adopter's specification, where three bindings pass a constant flag into the command they invoke.

## Fit review
1. Need: a binding must be able to fill a `Boolean`, `Integer` or `Decimal` command input with a constant, as an outcome's `sets:`/`payload:` already can. The requester's proposed syntax is `is_bridged: true` in `mapping:`, "typed against the target input, as command `sets:` already allows". Minimal reproduction: `<fit-review scratch>/probe-445/` (installed ess 0.52.0; 0.53.0 head has the same code, cited below):
   - `is_bridged: true` gives `components.yaml: invalid type: boolean 'true', expected an existing mapping string or closed {selection, path} object`;
   - `is_bridged: "true"` / `weight: "3"` give `[type_mismatch] … is 'Boolean', which is 'Boolean' underneath, and a literal in a binding is text` (`ESS-BINDING-002`);
   - the same model's outcome `payload:` writes `is_bridged: true` and `weight: 3` and validates.
2. Class: gap, and a sibling inconsistency. The rule is deliberate and documented (`crates/specify/ess-domain/src/binding.rs:95-118`, table row "anything else | refused: … an `Integer`"; refusal at `binding.rs:2151-2157`), so this is not a defect. But `sets:`/`payload:` admit the same literal through one shared rule (`docs/design/typed-literals-and-unknown-instances.md:14-31`, beyond10x/ess#113; `Decimal`, beyond10x/ess#135). The skill's red flag "works in one construct and leaves a sibling construct unable to say the same thing" applies.
3. Existing idiom: none that keeps one command. Workarounds are a dedicated command per constant (`MarkBridged`, whose outcome sets `is_bridged: true`), which splits the command surface, or a field the event never carries, which the guide calls a wire misdescription (`website/docs/guides/specify/bindings-and-components.md:192-194`).
4. Fit, redesigned onto the existing rule rather than a binding-local one:
   - Reader: `AuthoredMappingSource` reads only `visit_str`/`visit_map` (`binding.rs:852-879`). Add `visit_bool`/`visit_i64`/`visit_u64`/`visit_f64` producing the same typed-scalar value `RawPayloadSource` reads (`crates/specify/ess-domain/src/command.rs:1487`, `:1793-1815`), and add the scalar arm to the published `AuthoredMappingSchema` (`binding.rs:847-850`).
   - Check: `check_literal` (`binding.rs:2073-2170`) delegates the `Primitive` arm to `primitive_literal`/`scalar_representation`/`literal_representation` (`command.rs:4339`, `:4389`, `:4426`). Quoted `'true'` over `Boolean` is then admitted exactly as in `sets:`, and an unquoted scalar over text or an enum is refused with the same "quote it" repair.
   - IR: `ResolvedMappingValue::Literal { value }` keeps its shape and canonical text (`crates/specify/ess-compiler/src/ir.rs:2165-2178`, doc updated). The value is read against `target_type`, as `ResolvedPayloadValue::Literal` already is.
   - Targets that read it as text today must read it typed: the interpreter, `Node::Text` at `crates/verify/ess-conformance/src/interpret/bindings.rs:330` → `interpret/execute.rs:2066` `literal(ir, target, text)`; synthesis at `synthesize/binding_effects.rs:1490` and `synthesize/delivery_context.rs:118`; AsyncAPI at `crates/generate/ess-gen/src/asyncapi.rs:1435`; docs at `ess-gen/src/docs.rs:2112`.
   - `ess-synth` plans only text literals (`crates/generate/ess-synth/src/plan.rs:1479-1489`); any other target becomes the named obligation at `:1554`. Extend the plan to typed scalars for Rust and Go, or keep the obligation by name. It is not silent either way.
   - `ess verify diff` displays the literal (`crates/verify/ess-diff/src/diff.rs:1267`); unchanged.
   - Entity Runtime does not lower event bindings (no binding row in `website/docs/reference/entity-runtime-lowering.md`); inferred unaffected.
5. Second adopter: an `OrderPlaced` event causes `CreateShipment` with `requires_signature: false` and `priority: 1`, with constants fixed by the binding and not carried by the order.
6. Cost: no new keyword and no new diagnostic code; it reuses `type_mismatch` and the `sets:` hints. No source format bump, following #113 (`typed-literals-and-unknown-instances.md:3`). The only change is that previously refused documents are admitted. No admitted document changes meaning, and the IR bytes of every existing model are unchanged. No suite format: a typed value in `expect_invocation` input is a JSON scalar every runner already compares. Generated Rust/Go binding adapters gain typed constants where planned.
7. Alternatives: (a) change nothing and keep the documented refusal: the sibling inconsistency stays. (b) A binding-local literal syntax (`{literal: true, as: Boolean}`) would be a second spelling of one idea. Refused. (c) Chosen: the `sets:` typed-scalar rule, applied unchanged to `mapping:`. This takes the requester's proposal, with its checking routed through the existing function.

## Decisions
Accept, redesigned: a binding `mapping:` value may be an unquoted YAML Boolean, integer or decimal, or the quoted text of one. It is checked against the target input by the same `literal_representation`/`scalar_representation` rule as `sets:` and `payload:`, and the interpreter, synthesis, AsyncAPI, docs and `ess-synth` read it typed against `target_type`. No format bump (ess/22, ess-conformance/43 unchanged). Generated Rust and Go adapters pass the typed constant: `ess-synth`'s plan admits a scalar literal over a `Boolean`, `Integer` or `Decimal` target, wrapped or not, as a determined input (`plan.rs:1479-1489`), so the "is filled from the literal …, and no reading of it as … is declared" obligation (`plan.rs:1554`) is no longer raised for those targets. Update the binding module table (`binding.rs:95-118`), the guide (new section `## Fill a command input with a constant` in `bindings-and-components.md`, after `## A binding says what happens when it fails`) and the #113 positions table (`typed-literals-and-unknown-instances.md:44-52`, a `mapping:` row). Depends on #448 (edge recorded): both edit `command.rs` and `resolve.rs` in disjoint functions, and #448 moves predicate parsing out of serde across ess-domain first. Depends on #426b (edge recorded): #426b rewrites the hint inside `scalar_representation`, and this story routes `mapping:` through that function after it. In `command.rs` this story only raises `primitive_literal` (4339), `scalar_representation` (4389) and `literal_representation` (4426) to `pub(crate)` so `binding.rs` can call them; their bodies are #426b's (hint) and #450's (literal rule) to change.

## Acceptance
- binding_mapping_unquoted_boolean_fills_boolean_input: `is_bridged: true` validates; IR `{"kind":"literal","value":"true"}` with `Boolean` target.
- binding_mapping_quoted_scalar_matches_sets_rule: `'true'`, `'3'` and `'0.5'` over Boolean, Integer and Decimal are admitted exactly where `sets:` admits them.
- binding_mapping_scalar_over_text_says_quote_it: `template: 3` over a String target is `type_mismatch` with hint `quote it`.
- binding_mapping_scalar_type_mismatch_named: `weight: true` over Integer and `flag: 1` over Boolean are refused with the `sets:` hints.
- binding_literal_invocation_carries_typed_value: the interpreter and synthesized suites carry `true`/`3` as JSON scalars, not text; a runner sending `"true"` fails.
- binding_literal_existing_text_and_enum_bytes_unchanged: billing `template: invoice-created` IR, suite and generated bytes unchanged.
- binding_literal_generated_rust_go_pass_typed_constant: for `is_bridged: true` over `Boolean`, `weight: 3` over `Integer` and `share: 0.5` over `Decimal`, the generated Rust and Go adapters pass `true`, `3` and `0.5` as typed constants, never text, and the plan raises no "is filled from the literal" obligation for them. A text literal over a `String` target keeps its bytes. Cases of that name in `crates/generate/ess-synth/tests/binding_literal_scalar.rs`.
- binding_literal_docs_state_the_typed_rule: the module table in `crates/specify/ess-domain/src/binding.rs` has an `anything else` row that no longer names `Integer`, and a row admitting a `Boolean`, `Integer` or `Decimal` literal "as `sets:` does". The `### Positions` table of `docs/design/typed-literals-and-unknown-instances.md` has a `mapping:` row. `website/docs/guides/specify/bindings-and-components.md` has the heading `## Fill a command input with a constant`, whose section contains "`is_bridged: true`", "as `sets:`" and "quote it". A case of that name in `crates/specify/ess-domain/tests/binding_mapping_scalar.rs` reads the three files and fails naming the missing row, heading or phrase.

## Scope
- crates/specify/ess-domain/src/binding.rs  cited — reader 847-879, rule table 95-118, `check_literal` 2073-2170
- crates/specify/ess-domain/src/command.rs  cited — visibility only: `primitive_literal` (4339), `scalar_representation` (4389), `literal_representation` (4426) become `pub(crate)`; the scalar reader `Source` visitor (1467-1503: `visit_bool`, `visit_i64`, `visit_u64`, `visit_f64`) is the model the binding reader copies, unchanged
- crates/specify/ess-compiler/src/ir.rs  cited — `ResolvedMappingValue::Literal` doc 2165-2178
- crates/specify/ess-compiler/src/resolve.rs  cited — literal mapping resolution 4388, 4584, 5108
- crates/verify/ess-conformance/src/interpret/bindings.rs  cited — `Node::Text` at 330
- crates/verify/ess-conformance/src/synthesize/binding_effects.rs  cited — literal value at 1490
- crates/verify/ess-conformance/src/synthesize/delivery_context.rs  cited — literal value at 118
- crates/generate/ess-synth/src/plan.rs  cited — text-only plan 1479-1489 admits typed scalars; obligation 1554 no longer raised for them
- crates/generate/ess-synth/src/rust/system.rs  cited — `DeterminedInput::Literal` emission (570, 605) writes a typed constant
- crates/generate/ess-synth/src/go/system.rs  cited — `DeterminedInput::Literal` emission (544) writes a typed constant
- crates/generate/ess-synth/tests/binding_literal_scalar.rs  inferred — generated Rust and Go typed-constant cases
- crates/generate/ess-gen/src/asyncapi.rs  cited — mapped literal 1435
- crates/generate/ess-gen/src/docs.rs  cited — mapped literal 2112
- docs/design/typed-literals-and-unknown-instances.md  cited — positions table gains `mapping:`
- website/docs/guides/specify/bindings-and-components.md  cited — new section `## Fill a command input with a constant` after `## A binding says what happens when it fails` (37-76)
- crates/specify/ess-domain/tests/binding_mapping_scalar.rs  inferred — validation scenarios
- crates/verify/ess-conformance/tests/binding_literal_scalar.rs  inferred — typed invocation scenarios
