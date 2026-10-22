---
format: aep.planning-md/3
id: story:an-update-may-fall-back-to-the-stored-value-when-an-optional-input-is-absent
kind: story
status: draft
title: An update may fall back to the stored value when an optional input is absent
summary: 'else: admits only {generated: true}, a literal or an input path, so a partial update cannot keep the held value with else: {subject: <field>} (adopter item 11, 2026-10-22); same on 0.67.0.'
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Need

Reported by an adopter on 2026-10-22 (item 11 of the recheck batch, the second team's limit file L1, relayed by the
orchestrator; adopter names kept out). A partial update takes an optional input and, when the caller leaves it out, keeps the
value the record already holds. The natural spelling is a fallback to the stored field: `sets: {nickname: {input: nickname,
else: {subject: nickname}}}`. `else:` admits `{generated: true}`, a literal (beyond10x/ess#163, closed) or another input path,
not the subject's own field.

Needed: `else: {subject: <field>}` on a branch that holds a row (`updates:`, `moves:`), reading the row as it was before the
branch, or a recorded idiom that says how a "keep when absent" update is written today and what its scenario checks.

## Reproduction

Brand-free fixture, `format: ess/29`, domain `demo.server`: entity `Profile` with `nickname: String`, `Register` creating it,
`Rename(profile_id, nickname: Optional<String>)` with an `updates:` branch setting
`{nickname: {input: nickname, else: {subject: nickname}}}` and an `unknown_instance:` refusal, and an immediate view
`ProfileById`. `ess specify validate --path spec.yaml`, 0.68.0 (integration build, also at `format: ess/30`) and 0.67.0
identical:

```text
spec.yaml was refused:
  - spec.yaml: `else:` admits `{generated: true}`, a literal (format ess/16) or `input.<path>` (format ess/22)
```

Control, both versions: the same model with `else: "anon"` validates and passes 3/3 interpreted scenarios (byte-identical
suites).

## Where it lives

- `crates/specify/ess-domain/src/command.rs:2106-2125` (`PayloadSource::fallback`): admits `Generated`, a literal or scalar,
  and from `ess/22` an `InputField`; every other source, including `Subject`, falls to the refusal at `:2118-2121`.
- Inferred, not traced: a `Subject` fallback needs the held row in `value_expression` validation and in the interpreter's and
  generators' evaluation of `else:`; the refusal is a reader error with no location, which is also worth fixing.
- The same refusal stops a nested `else:` chain in `story:an-input-path-reads-a-list-element-and-a-fallback-chain-tries-several-places`
  (adopter item 20); a change to `fallback` serves both.

## Regression status

Not a 0.68.0 regression: identical refusal on 0.67.0 and 0.68.0. Gap. Does not block 0.68.0.
