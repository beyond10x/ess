---
format: aep.planning-md/3
id: story:a-text-is-split-at-a-separator-and-its-parts-are-read
kind: story
status: active
title: A text may be split at a separator, one part read, and a predicate asked of each part
summary: split(label, ".")[0] and any_of over a split parameter are ESS-SPEC-012 on 0.93.0 (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- depends_on: story:text-functions-compute-values
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:51Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:52Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Need

A downstream adopter's service splits a text at a separator and reads one part (the first label of
a dotted name), and tokenises a search string so that a row matches when its field contains any
token. `story:text-functions-compute-values` adds `len`, `prefix`, `suffix`, `repeat`, `lower`,
`upper` and `trim`; it has no split, no index and no predicate over the parts. The adopter counts
20 open notes on it.

Refusals on 0.93.0 (`format: ess/37`, view `demo.t.Items` over `demo.t.Item` with a String field
`label` and an optional String parameter `q`):

```yaml
fields:
  - {name: head, type: String, expr: "split(label, \".\")[0]"}
```

```
error[ESS-SPEC-012]: `split(label, ".")[0]`: `[` is no part of an expression; it is written with paths, literals, `+ - * /`, parentheses and function calls
```

```yaml
filter:
  any:
    - "not defined(param.q)"
    - label: {any_of: {split: {param: q, by: " "}}}
```

```
error[ESS-SPEC-012]: predicate value list: expected a scalar, found a mapping
```

## Acceptance

- At a new source format an expression may split a text at a literal separator and read one part
  by position (the first, the last, the n-th); the spelling, the value when the position is past
  the parts, and the rule for an empty separator and for adjacent separators are stated in
  `generic-language-features.md`.
- A predicate may ask a text operator of each part of a split parameter or input ("the field
  contains any token of `q`", "every token"), in the positions `contains` validates in.
- The shared evaluator computes both; synthesis witnesses a text with two parts, a text without
  the separator, and for the predicate a row matching one token and a row matching none.
  `ess verify conform run --target interpreted` passes; a target that reads the whole text fails.
- JSON Schema and OpenAPI keep the field's declared type; generated Go and TypeScript targets
  compute the same parts as the interpreter.
- Below the new format both spellings keep today's refusals.

## Evidence

Both refusals were rerun for this story on `ess 0.93.0` and are quoted verbatim. The count of
notes is the adopter's.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).
