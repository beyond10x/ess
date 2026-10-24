---
format: aep.planning-md/3
id: story:a-dotted-variant-name-synthesizes-as-an-identifier
kind: story
status: draft
title: A dotted variant name synthesizes as an identifier
summary: 'Adopter report N8: synthesize --http refuses dotted variant names (ess#662).'
refs:
- provider: github
  reference: beyond10x/ess#662
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

`ess verify conform synthesize --http` refuses an enum variant whose name contains a dot, because the Go target allocates no identifier for it (`crates/generate/ess-synth/src/go/layout.rs:458-480`), while `ess specify validate` accepts the name. Adopters add `{name, wire}` to every such variant. Adopter report N8, ess 0.68.0.

## Acceptance

- `a_dotted_variant_synthesizes`: a variant `call.started` synthesizes on the Go and Rust targets, with an identifier built from its words and the declared spelling as the wire value.
- `two_variants_that_collide_after_conversion_are_refused`: `a.b` and `a_b` in one enum are refused by name.
- `a_declared_wire_keeps_precedence`: `{name: x, wire: call.started}` keeps generating as today.
