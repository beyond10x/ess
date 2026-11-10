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
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/authored_union_value.rs
- confidence: cited
  path: website/docs/guides/verify/author-scenarios.md
revision: 7
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

- `website/docs/guides/verify/author-scenarios.md` has a section "Write a union value" stating the
  tag, the payload under the union's content key (`value`, or `content` where the tag is named
  `value`), and a unit variant as the tag alone, with one YAML example; that example, as a fixture
  in `cargo test -p ess-conformance --locked --test authored_union_value`, compiles with 0
  refusals.
- On the input `{actor: {kind: Human, operator: ada}}`, the `ESS-AUTHOR-015` refusal ends with the
  hint `write {kind: Human, value: {…}}`, its tag and content key those of the union (a test in
  `--test authored_union_value`).
- On the input `{actor: {kind: Agent, instance: i-1, kind: worker}}`, where `kind` is the union's
  tag, the `ESS-AUTHOR-047` refusal ends with the hint `write {kind: Agent, value: {…}}` (a test in
  `--test authored_union_value`); a repeated key that is no union's tag keeps today's message
  unchanged (a test in the same target).

## Scope

- cited: website/docs/guides/verify/author-scenarios.md
- cited: crates/verify/ess-conformance/src/authored.rs
- cited: crates/verify/ess-conformance/src/input.rs
- inferred: crates/verify/ess-conformance/tests/authored_union_value.rs
confidence: medium

Shared surfaces: proposes its `CHANGELOG.md` sentence; moves no byte-pin row.

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
author meets.

The plugin's syntax reference lives in `beyond10x/agentplugins` (`plugins/ess/`, `AGENTS.md`
§ "Agent plugin"), outside this repository, so it is not an acceptance bullet here. The line to add
there: "A union value is its tag and the payload under the content key, `{kind: Agent, value:
{…}}`; a unit variant is the tag alone."

The hint reads the content key from the same place the reader takes it
(`crates/verify/ess-conformance/src/input.rs:1050`), so the guide documents today's rule
(`value`, or `content`) and the hint follows a declared key without a change here if
`story:projections-use-a-unions-declared-content-key` lands.

The `ess-check` docs gate covers `website/docs/reference/predicates.md` only
(`crates/edge/ess-cli/tests/predicate_reference_page.rs`), so the guide's example is checked as a
fixture of this story's test target instead.

## Source

github:beyond10x/ess#764
