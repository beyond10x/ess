---
format: aep.planning-md/3
id: story:response-newtype-witness-lets-prefix-and-suffix-overlap
kind: story
status: draft
title: A response newtype witness lets a prefix and a suffix overlap
summary: The model interpreter's TextRules::candidates (interpret/response.rs, 262c66ca8) never overlaps starts_with and ends_with, so count <= 3, starts_with AB, ends_with BC has the witness ABC and is still interpreted as Unsupported. Never a wrong verdict. (adversary pass 2026-10-22, F1).
tags:
- defect
- severity-low
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
# Story: a response newtype witness lets a prefix and a suffix overlap

## Outcome

The model interpreter returns a value for a String newtype response field whose only witnesses
share characters between the `starts_with` and the `ends_with` literal, so the interpreted target
passes the outcome scenario instead of answering Unsupported.

## Problem

`TextRules::candidates` (`crates/verify/ess-conformance/src/interpret/response.rs:353`, added in
262c66ca8) lays the prefix, the contained literals and the suffix side by side
(`{prefix}{contained}{core}{suffix}` and two siblings), so its shortest text is the prefix and the
suffix end to end. When a `value.count` upper bound is shorter than that, every candidate breaks
the bound, although a text where the two literals overlap meets every rule. Found by the adversary
pass of 2026-10-22 (F1). The interpreter reports it as Unsupported, never as a wrong verdict.

Reproduction (`overlap.yaml`):

```yaml
format: ess/23
system: catalog
version: v1
domain: catalog.items
components:
  - component: catalog-service
    owns: {domains: [catalog.items]}
    accepts: {commands: [catalog.items.Register]}
    reached_by: network
types:
  - name: catalog.items.Glyph
    kind: newtype
    of: String
    invariants:
      - 'value.count <= 3'
      - value: {starts_with: AB}
      - value: {ends_with: BC}
commands:
  - name: catalog.items.Register
    input:
      - {name: label, type: String}
    response:
      - {name: glyph, type: catalog.items.Glyph}
    outcomes:
      - {name: registered, returns: true}
```

`ess verify conform run --target interpreted --path overlap.yaml --report-format 2` on 0.92.0
(exit 1):

```text
catalog v1 against interpreted 0.92.0 — failed
  unsupported catalog.items.Register/outcome/registered
      ESS-CF-TARGET: cannot expose invoking `catalog.items.Register`: a bounded actual command response is not interpreted yet
  1 scenarios: 0 passed, 0 failed, 0 error, 1 unsupported
```

`ABC` meets all three rules. With `value.count <= 4` the same model passes (1 passed).

## Acceptance

- The reproduction above passes on the interpreted target (exit 0, 1 passed), with a returned
  `glyph` of at most three characters that starts with `AB` and ends with `BC`.
- A test beside
  `adversary_e1_reachability::the_model_interpreter_returns_a_value_under_an_upper_bound_with_a_text_match`
  covers the overlap case, and a contained literal that overlaps the prefix or the suffix.
- `a_count_range_no_text_meets_is_refused_by_name` still holds: a rule set no text meets stays
  Unsupported by name.

## Scope

- cited: `crates/verify/ess-conformance/src/interpret/response.rs` (`TextRules::candidates`)
- cited: `crates/verify/ess-conformance/tests/adversary_e1_reachability.rs`

confidence: high

## Out of Scope

Synthesis of the type's own scenario (ESS-SYNTH-013 when no view publishes the type) is unrelated.
