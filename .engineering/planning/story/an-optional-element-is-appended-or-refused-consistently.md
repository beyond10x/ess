---
format: aep.planning-md/3
id: story:an-optional-element-is-appended-or-refused-consistently
kind: story
status: active
title: An optional element is appended or refused, consistently across domain and interpreter
summary: 'The domain admits {append: input.<Optional<T>>} over List<Optional<T>> and generated Go appends nil, but the interpreter refuses the request as Unsupported when the element is absent or null.'
tags:
- ess-0.70.0
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command/append_list.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/determined.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/append_set.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T07:12:04Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-02T07:12:04Z", actor: "human:timo", revision: 10}
---
## Finding

Adversary pass on U10 (story:an-outcome-may-append-to-a-stored-list) before 0.69.0, on integrate/ess-batch-0690 f1197bcd4. Severity warning, origin introduced by U10; recorded for 0.70.0 under approval-record:ess-0690-cut-after-gu19.

- `check_element` in `crates/specify/ess-domain/src/append_list.rs` compares element and read types for equality, so `{append: input.label}` with `label: Optional<String>` over a `List<Optional<String>>` field is admitted. The module comment says an Optional read has nothing to append.
- Generated Rust pushes `None`; generated Go appends `nil` and builds.
- The interpreter (`append` in `crates/verify/ess-conformance/src/execute/values.rs`) turns an absent or null element into NoValue and refuses with `Unsupported { why: "... its type has no shape the interpreter can mint" }`.
- No adopter model is known to use this combination.

## Acceptance

One of the two, chosen in the fit of this story:

- the domain refuses an Optional element read with `type_mismatch`, as the module comment says; or
- the interpreter appends `null` when the element type is Optional, and a view over the list after two such appends from `["a"]` reads `["a", null, null]`.

The adversary case `an_optional_element_the_domain_admits_is_appended_by_the_interpreter` (ess-conformance) is the red test for the second option.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave D368 on 6437975cb, replacing the D350 scope of 2026-11-01. Every line is **cited** (read from the story or the tree) or **inferred**.

- **Primary surface:** `crates/specify/ess-domain` (command/append_list) and `crates/verify/ess-conformance` (interpret) — cited
- **Domain:** `command/append_list.rs:199-238` `check_element`, type equality `:220`, call site `command/value_expression.rs:1440`; module comment `:6-7` ("an `Optional` read has nothing to append"), hint `:231-233` — cited
- **Interpreter:** `interpret/execute/determined.rs:528-552` `append`; an absent or `Node::Null` element becomes NoValue at `:539-541`; dispatch `interpret/execute.rs:2665-2672`; NoValue `:294-298`, `is_capability_gap` `:331-340` — cited
- **Generators today:** Go `ess-synth/src/go/behaviour.rs:3725-3744` appends `input_value(source)` (`:3729`, `:3731-3738`), a pointer, so `nil`; Rust `ess-synth/src/rust/behaviour.rs:2882-2897` pushes `input_value(source)` (`:2886`, `:2890`, `:2894`), an `Option<T>`, so `None` — cited lines, inferred values. No TypeScript behaviour emitter exists — cited
- **Suite synthesis:** `ess-conformance/src/synthesize.rs:10542-10561` `appended_at` returns no expectation for an absent or Null element (`:10553-10554`); caller `:10637-10639` — cited
- **Documents:** `ess-domain/src/system.rs:335` (ess/31 history text); `website/docs/guides/specify/values-and-views.md:47` — cited
- **Committed specs with append over `List<Optional<T>>`:** none under `examples/`, `suites/`, `models/`, `generated/`; no append fixture declares an Optional element — cited (grep)
- **Red test** `an_optional_element_the_domain_admits_is_appended_by_the_interpreter` does not exist on origin/main — cited
- **Option A (domain refuses):** `append_list.rs:220` plus `ess-domain/tests/append_set.rs`; refusing a specification that validates today is stricter and by the `stricter:` precedent (`system.rs:173`, `:224`) needs a new format major — inferred
- **Option B (interpreter appends null):** `determined.rs:525-552` appends `Node::Null` for an Optional element type (`validate_typed_value` accepts Null for Optional, `ess-conformance/src/input.rs:842-844` — cited); `synthesize.rs:10542-10561` expects the null; the comments at `append_list.rs:6-7` and `system.rs:335` are corrected; the red test goes in `ess-conformance/tests/append_set.rs`; no generator edit and no format bump — inferred. About 60 changed lines
- **Confidence:** medium (arms and greps read; JSON encoding of an appended null not run)
- **Would collide with:** `interpret/execute/determined.rs`, the `appended_at` region of `synthesize.rs`, `command/append_list.rs`, `system.rs` format history

## Decisions

Wave D368 (`approval-record:ess-d368-wave`, 2026-11-02) takes the second option: the interpreter
appends `null` when the element type is Optional and the element is absent or null, and synthesis
expects it. The domain has admitted the read since ess/31 and the generated Go and Rust already
append `nil`/`None` (see Scope), so this moves the interpreter to shipped behaviour without a format
major; refusing it in the domain would be a stricter rule for a specification that validates today,
which needs a new major. The module comment in `command/append_list.rs` and the ess/31 history
text in `system.rs` are corrected to say an Optional element of a `List<Optional<T>>` appends null.
