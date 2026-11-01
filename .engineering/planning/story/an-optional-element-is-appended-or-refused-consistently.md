---
format: aep.planning-md/3
id: story:an-optional-element-is-appended-or-refused-consistently
kind: story
status: draft
title: An optional element is appended or refused, consistently across domain and interpreter
summary: 'The domain admits {append: input.<Optional<T>>} over List<Optional<T>> and generated Go appends nil, but the interpreter refuses the request as Unsupported when the element is absent or null.'
tags:
- ess-0.70.0
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command/append_list.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/determined.rs
revision: 5
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

Derived 2026-11-01 by `aep:story-scoper` for wave D350 on dc60f4532. Every line is **cited** (read from the story or the tree) or **inferred**.

- **Primary surface:** `crates/specify/ess-domain` (command/append_list) and `crates/verify/ess-conformance` (interpret) — cited
- **Domain:** `crates/specify/ess-domain/src/command/append_list.rs:199-238` `check_element` (type equality :220); one call site `command/value_expression.rs:1440` — cited
- **Interpreter:** `crates/verify/ess-conformance/src/interpret/execute/determined.rs:528-552` `append` (absent/Null element is NoValue :539-541); `interpret/execute.rs:2665-2672` dispatch — cited
- **Generators:** `ess-synth/src/go/behaviour.rs:3725-3742`, `ess-synth/src/rust/behaviour.rs:2882-2897` — cited for the arms; nil/None behaviour — inferred
- **Documents:** `ess-domain/src/system.rs:335` format-history text — cited
- **Tests:** `ess-domain/tests/append_set.rs:103-114`, `ess-conformance/tests/append_set.rs:148,170` — cited; no test covers an Optional element append — inferred
- **Confidence:** medium
- **Would collide with:** `command/append_list.rs`, `command/value_expression.rs`, `interpret/execute/determined.rs`
