---
format: aep.planning-md/3
id: story:an-integer-equality-guard-pair-is-exhaustive
kind: story
status: active
title: An integer equality guard and its negation are exhaustive
summary: 'ESS-COMMAND-005: guards == 0 and != 0 over one integer are not proved exhaustive.'
tags:
- adopter-report
- defect
- ess-0.71.0
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/generate/ess-synth/tests/finite_literal.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/finite.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/subject_fact.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/finite_literal.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/subject_guard_input.rs
- confidence: cited
  path: docs/design/closed-enum-outcome-coverage.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:39:59Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:40:00Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":2}}}
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, from a downstream repository specification (which quarantines 9 scenarios for gaps 1 and 2).

ESS-COMMAND-005: two branches guarded `n == 0` and `n != 0` over one integer are not proved exhaustive, so the command is refused although every value takes exactly one branch. Related: story:defined-and-not-defined-branches-are-exhaustive (the same refusal for `defined` / `not defined`, implemented).

Reproduced on 0.92.0 (`ess/35`). A creating command with input `n: Integer` and no default:

```yaml
outcomes:
- name: created-a
  when: n == 0
  creates: gap.demo.Parent
  # instance, sets, emits, payload as usual
- name: created-b
  when: n != 0
  creates: gap.demo.Parent
  # same
```

```
error[ESS-COMMAND-005]: every outcome of `gap.demo.CreateParent` is conditional, so there is input the specification says nothing about
```

The same refusal on 0.92.0 for `n == 7` / `n != 7`, for `name == "a"` / `name != "a"` over a
`String` input, and for a stored field (`when_subject: {predicate: n == 0}` /
`{predicate: n != 0}`, "`n` has no closed set of values; declare a genuine default"). Cause: the
finite coverage proof admits only closed domains (`docs/design/closed-enum-outcome-coverage.md`,
"open scalar domains ... cannot establish this proof"; `crates/specify/ess-domain/src/command/finite.rs`
`input_paths`). An equality and its negation against one literal partition any domain into
{k, not k}, so no closed domain is needed.

Workaround today: drop the guard from one branch (make it the default).

## Acceptance

- `an_integer_equality_and_its_negation_are_exhaustive`: `== k` beside `!= k` over one integer
  input validates (exit 0) for any literal k; `== 0` beside `> 0` alone still draws ESS-COMMAND-005.
- The same holds for a `String` input against one text literal, and for a stored field under
  `when_subject:` (the stored-field partition), each with a test case.
- Two literals (`n == 0`, `n == 1`, `n != 0`) still draw ESS-COMMAND-005 unless the branches cover
  every cell of the {0, 1, other} partition exactly once; an overlap is still reported as one.
- `docs/design/closed-enum-outcome-coverage.md` states the literal-partition rule.

## Scope

- cited: crates/specify/ess-domain/src/command/finite.rs
- cited: docs/design/closed-enum-outcome-coverage.md
- cited: crates/specify/ess-domain/tests/subject_guard_input.rs (the stored-field case, beside `defined_and_not_defined_over_one_stored_optional_field_are_exhaustive`)
- inferred: crates/specify/ess-domain/src/command/subject_fact.rs
- inferred: crates/specify/ess-domain/tests/finite_literal.rs (new: the input cases)
- inferred: crates/generate/ess-synth/tests/finite_literal.rs (new: the partition reaches generated targets, as `tests/finite_presence.rs` does for `defined`)

confidence: medium

Shared surfaces: none; no format row, no byte-pin row. The scenarios per partition cell come from
`crates/verify/ess-conformance/src/witness.rs`, which reads `finite::analyze` and is expected
unchanged.
