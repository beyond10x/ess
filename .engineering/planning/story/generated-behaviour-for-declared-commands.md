---
format: aep.planning-md/3
id: story:generated-behaviour-for-declared-commands
kind: story
status: implemented
title: A fully declared command's behaviour is generated over storage and context ports
relations:
- serves: vision:O2
- decomposes: epic:generated-determined-behaviour
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T20:20:01Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T20:20:01Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T21:25:27Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

For a command whose every outcome the generator can express, `--target rust` emits its behaviour
instead of a `…Behavior` obligation, written against generated ports; the plan lists it as
generated.

## Expressible (generated)

- `when:` input guards, `when_subject:` predicates over stored fields, `unknown_instance:`,
  `wrong_state:`, default branches, `external:` branches (answer supplied by the context port);
- `creates:`, `moves:` (through the generated typestate), `updates:`, `deletes:`;
- `sets:` from input, literal, `{increment: n}`, `{cleared}`, caller attributes, `{generated: true}`
  (identity minted by the context port);
- event and error payloads from input, subject fields, caller and generated values.

## Stays an obligation (whole command)

Anything else, named in the plan with the reason: `{related:}` sets or guards, `when_related:`,
`when_subject_state:` mixed with `external:`, subject predicates that choose between a move and an
update, aggregates, filtered or multi-record effects, `instances:`/`affects:` outcomes — unless the
unit implements one with the interpreter's semantics and a test.

## Ports

- Storage: one trait per entity, get/put/delete by identity over the generated snapshot; ess
  generates the trait, never an implementation.
- Context: caller identity and attributes, identity minting, and the answer to an `external:`
  branch (forced or decided).

## Acceptance

- Order of evaluation matches the conformance interpreter and the precedence order in
  `docs/design/cross-record-and-stored-field-guards.md` (input-guarded refusals first declared;
  existence; held state; accepting and external branches in declaration order; a forced external
  answers before the subject is read).
- A fixture model's generated workspace builds with `-D warnings`, and its synthesized conformance
  suite passes against the generated behaviours with an in-memory port implementation written in
  the test, with zero hand-written behaviour.
- The billing example's committed generated trees stay byte-identical, or every changed byte is
  named and the regeneration committed.
- Generated behaviours call the entity invariant check (story 2) before writing.
