---
format: aep.planning-md/3
id: story:a-text-length-is-counted-in-utf16-units-and-over-the-trimmed-value
kind: story
status: active
title: A text length may be counted in UTF-16 code units, and over the trimmed value
summary: 'label.utf16_count, length(trim(label)) and trimmed: {count_gt} are refused in a guard on 0.93.0 (adopter report)'
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
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:54Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:55Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Need

A downstream adopter's service refuses a text longer than a limit, where its platform counts
UTF-16 code units and counts after trimming. `label.count > 64` validates on 0.93.0 and counts
characters of the untrimmed value. `story:text-functions-compute-values` adds `len` (characters),
`utf8_bytes` and `trim` as value expressions; it has no UTF-16 count and does not admit a function
of a function on the left side of a guard comparison. The adopter counts 13 open notes on it.

Refusals on 0.93.0 (`format: ess/37`, command `demo.t.Make` with a String input `label`, outcome
`refused` with `error: demo.t.Bad`):

```yaml
when: label.utf16_count > 64
```

```
error[ESS-COMMAND-003]: `label.utf16_count` cannot select `utf16_count` from `String` (resolved prefix type `String`)
```

```yaml
when: length(trim(label)) > 64
```

```
error[ESS-COMMAND-002]: `length(trim(label)) > …`: the left side of a comparison is a fact path or a function of exactly one fact, such as `coalesce(retries, 0)`; `length(trim(label))` is neither
```

```yaml
when: {label: {trimmed: {count_gt: 64}}}
```

```
error[ESS-SPEC-012]: cannot parse predicate "label: {trimmed: {count_gt: 64}}": `trimmed` takes a mapping of exactly one text operator: equals, in, equals_ignore_case, in_ignore_case, starts_with, ends_with or contains
```

## Acceptance

- At a new source format a guard and an invariant may compare the length of a text in UTF-16
  code units, and the length (in characters, UTF-8 bytes or UTF-16 units) of the trimmed value;
  one spelling of each is stated in `generic-language-features.md`.
- The shared evaluator computes the UTF-16 count; a character outside the Basic Multilingual
  Plane counts 2. Synthesis witnesses the boundary with a text at the limit and one over it, and
  one text whose character count is within the limit and whose UTF-16 count is over it; a target
  that counts characters fails that scenario.
- For the trimmed length synthesis witnesses a text over the limit only by its surrounding
  whitespace, and expects it accepted.
- JSON Schema writes `maxLength` only where its unit matches the declared one, and says in the
  projection report where it does not.
- Below the new format the three spellings keep today's refusals.

## Evidence

The three refusals and the validating `label.count > 64` were rerun for this story on
`ess 0.93.0` and are quoted verbatim. The count of notes is the adopter's.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).
