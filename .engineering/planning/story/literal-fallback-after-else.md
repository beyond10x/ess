---
format: aep.planning-md/3
id: story:literal-fallback-after-else
kind: story
status: draft
title: 'A payload or sets: fallback after else: can be a literal'
refs:
- provider: github
  reference: beyond10x/ess#163
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

`{input: f, else: <literal>}` in a payload or `sets:` value, under a new source format. Extends E4 of `docs/design/value-expressions.md`, which admits only `{generated: true}` after `else:`.

## Acceptance

A scenario that omits the optional input asserts the literal; a mutant storing a different default is killed; the literal is type-checked against the target field.
