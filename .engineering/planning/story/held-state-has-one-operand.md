---
format: aep.planning-md/3
id: story:held-state-has-one-operand
kind: story
status: draft
title: Held state is selected by one operand
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
revision: 1
---
## Outcome

`state` is the one operand for held state: `when_subject_state` is sugar for `state in [..]` and combines with `when_subject` (review S3/S4, worst five #3).

## Acceptance

- `when_subject_state: S` beside `when_subject:` validates and means `{all: [state in [S], <predicate>]}`; the "one selection authority" refusal (`ess-domain/src/command.rs:5240-5253`) is gone
- synthesis and the interpreter take one code path for held-state selection
- GP:139-140 is corrected; the guide lists one way and names the sugar

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (held-state-has-one-operand). Drafted, not scheduled; a fit review precedes dispatch.
