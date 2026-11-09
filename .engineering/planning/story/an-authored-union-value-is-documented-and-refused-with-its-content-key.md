---
format: aep.planning-md/3
id: story:an-authored-union-value-is-documented-and-refused-with-its-content-key
kind: story
status: draft
title: An authored union value's form is documented, and its refusals name it
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#764
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/authored_union_value.rs
- confidence: cited
  path: website/docs/guides/verify/author-scenarios.md
revision: 4
---
## Need

An authored scenario writes a union value in its wire form: the tag, and the variant's payload
under the content key, `{kind: Agent, value: {instance: i-1, kind: worker}}`
(`docs/design/union-unit-variants.md:56-63`). The scenario guide does not say so
(`website/docs/guides/verify/author-scenarios.md` names unions only to refuse a dotted path through
one), and neither refusal of a flat form names it. An author whose variant has a field named like
the tag writes the flat form and gets a YAML duplicate-key refusal. One whose variant does not gets
"missing union payload" (`repro/run.txt`, `fixture/*`):

```text
input: {actor: {kind: Agent, instance: i-1, kind: worker}}
-> refusal[ESS-AUTHOR-047]: timeline[0].input.actor: duplicate entry with key "kind"
input: {actor: {kind: Human, operator: ada}}
-> refusal[ESS-AUTHOR-015]: ... `actor` is outside its declared type example.store.Actor: missing union payload
input: {actor: {kind: Agent, value: {instance: i-1, kind: worker}}}
-> 1 authored scenario(s) from 1 file(s), 0 refusal(s)
```

## Acceptance

- `website/docs/guides/verify/author-scenarios.md` gains a short section on writing a union value:
  the tag, the payload under the content key (`value`; `content` where the tag is `value`; the
  declared content key once `story:projections-use-a-unions-declared-content-key` lands), a unit
  variant as the tag alone. A `yaml ess-check` example is checked by the docs gate.
- `ESS-AUTHOR-015`'s "missing union payload" names the expected form in its hint:
  `write {kind: Human, value: {…}}`, with the union's tag and content key.
- `ESS-AUTHOR-047` on a key that is a union's tag, inside a value of that union, adds the same hint.
- The agent plugin's syntax reference lives in `beyond10x/agentplugins` (`plugins/ess/`,
  `AGENTS.md` § "Agent plugin"), so its one line is a change there. This story's description names
  the line to add.

## Scope

- cited: website/docs/guides/verify/author-scenarios.md
- inferred: crates/verify/ess-conformance/src/authored.rs
- inferred: crates/verify/ess-conformance/tests/authored_union_value.rs
confidence: medium

## Fit review

1. **Need.** An author can find the fixture form of a union value. Requester's ask (theirs): "a line
   in `references/syntax.md` would save the probe".
2. **Class.** Documentation gap and diagnostic gap. The form works (`run.txt`, `fixture/nested`),
   the guide is silent, and the two refusals do not name it.
3. **Already expressible?** Yes. This is the idiom, documented.
4. **Fit.** The fixture form is the wire form (`union-unit-variants.md:56-63`), so nothing new is
   designed. A flat form is not added: it would collide with a variant field named like the tag,
   which is exactly the reported case.
5. **Second adopter.** Any author with a union input, such as a payment `method` union in a scenario.
6. **Cost.** Documentation and two hints. No format.
7. **Alternatives.** (a) Accept a flat form: refused, it is ambiguous where a variant has a field
   named like the tag. (b) Document the form and name it in the refusals: chosen.

## Decisions

accept, redesigned. The requester asked for one line in the plugin's syntax reference. The form
`{<tag>: <Variant>, <content key>: {…}}` already works (`run.txt`), so nothing is added to the
language. It is documented in this repository's scenario guide and named in the two refusals an
author meets, and the plugin line follows in `beyond10x/agentplugins`.

## Source

github:beyond10x/ess#764
