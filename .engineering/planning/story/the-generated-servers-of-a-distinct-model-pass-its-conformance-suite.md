---
format: aep.planning-md/3
id: story:the-generated-servers-of-a-distinct-model-pass-its-conformance-suite
kind: story
status: draft
title: The generated servers of a distinct model pass its conformance suite
summary: 'Follow-up of story:generated-rust-and-go-decoders-check-a-structs-distinct-invariant: its acceptance 2 asks for a conformance run of the generated Rust and Go servers on a model with every struct distinct form; no generic generated-server harness exists, so that clause has no evidence (adversary pass 1, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

Acceptance 2 of story:generated-rust-and-go-decoders-check-a-structs-distinct-invariant asks that
the generated Rust and Go servers pass the conformance suite of a model holding each `distinct:`
form in a struct, value-invariant scenarios included. That change delivered decoder tests for
every form on both targets but no conformance run: each ess-synth test that runs a synthesized
suite against a generated server (`integer_spellings`, `append_set`, `typed_text_operands`,
`declared_behaviour` and others) carries a storage-port harness, a build task and a
`ConformanceTarget` adapter bound to its own model, and none runs an arbitrary model's generated
server. The regression model also declares no entity or view, so its suite has no command to send
a list through.

## Acceptance

1. A model holding each struct `distinct:` form (scalar list; `by` a text member, an enum member,
   an Optional member; a list nested in a struct of a list) behind a command that takes the struct
   synthesizes a conformance suite with value-invariant scenarios that send a duplicate key and a
   distinct list.
2. The generated Rust and Go servers of that model pass the suite in a test under
   `crates/generate/ess-synth/tests`, and a server whose decoder skips the `distinct:` check fails
   it.
