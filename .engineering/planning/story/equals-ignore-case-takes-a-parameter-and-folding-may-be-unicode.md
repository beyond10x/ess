---
format: aep.planning-md/3
id: story:equals-ignore-case-takes-a-parameter-and-folding-may-be-unicode
kind: story
status: draft
title: equals_ignore_case takes a parameter operand, and a comparison may fold case by a Unicode rule
summary: 'equals_ignore_case: {param: q} and a Unicode fold are ESS-SPEC-012 on 0.93.0; folding is ASCII only (adopter report)'
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- depends_on: story:text-shape-operators-ignore-case
revision: 1
---
## Need

A downstream adopter's service compares texts without regard to case using its platform's Unicode
case mapping, and compares against a request parameter. ESS folds ASCII only
(`generic-language-features.md`, "Text", answer 6), and `equals_ignore_case` takes a literal only.
`story:text-shape-operators-ignore-case` adds `contains_ignore_case` with a `{param: ...}` operand
and `story:a-text-operator-reads-a-binder-or-a-field-as-its-operand` adds `{fact: ...}` on
`equals_ignore_case`; neither adds `{param: ...}` on `equals_ignore_case` or a Unicode fold. The
adopter counts 17 open notes on case-insensitive comparison in all.

Refusals on 0.93.0 (`format: ess/37`, view filter over a String field `label`, optional String
parameter `q`):

```yaml
- label: {equals_ignore_case: {param: q}}
```

```
error[ESS-SPEC-012]: cannot parse predicate "label: {equals_ignore_case: {param: q}}": a comparison operand must be a scalar
```

```yaml
- label: {equals_ignore_case: vip, fold: unicode}
```

```
error[ESS-SPEC-012]: cannot parse predicate "label: {fold: …}": unknown operator "fold"; expected one of eq, ne, lt, lte, gt, gte, any_of, none_of, exists, truthy, starts_with, ends_with, contains, equals_ignore_case, in_ignore_case, trimmed
```

`label: {equals_ignore_case: vip}` validates.

## Acceptance

- At a new source format `equals_ignore_case` and `in_ignore_case` take `{param: ...}` and
  `{input: ...}` operands in the positions they validate in.
- A specification may state that an ignore-case comparison folds by a named Unicode rule (simple
  case folding of a pinned Unicode version, or another rule the scoper names) in place of ASCII;
  the spelling is per operator or per specification, stated in `generic-language-features.md`
  with the reason answer 6 is extended.
- The shared evaluator and the generated Go and TypeScript targets fold alike; a test compares
  them over a pinned table of code points. Synthesis witnesses a pair equal under the Unicode
  rule and unequal under ASCII folding; a target that folds ASCII only fails that scenario.
- Without the new spelling every comparison keeps ASCII folding and its bytes.

## Evidence

Both refusals and the validating form were rerun for this story on `ess 0.93.0` and are quoted
verbatim. The count of notes is the adopter's and covers the forms the two existing stories add
as well.
