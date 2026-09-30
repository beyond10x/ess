---
format: aep.planning-md/3
id: story:related-guard-vocabulary-aligns
kind: story
status: draft
title: '`when_related` uses the vocabulary of its siblings'
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
revision: 1
---
## Outcome

`when_related` names its link and its existence test the way `{related:}` and quantifiers do (review S5).

## Acceptance

- `when_related` `via` accepts a subject field as `{related:}` does, or the refusal names why not
- the four meanings of `exists` are documented in one table in PR, and the row-existence form is renamed or aliased if it clashes
- a guide section for `when_related` exists; a missing related row's status (409 vs 404) is stated with its reason

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (related-guard-vocabulary-aligns). Drafted, not scheduled; a fit review precedes dispatch.
