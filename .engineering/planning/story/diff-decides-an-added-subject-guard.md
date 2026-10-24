---
format: aep.planning-md/3
id: story:diff-decides-an-added-subject-guard
kind: story
status: draft
title: ess verify diff decides an added when_subject refusal
summary: An added when_subject refusal that refuses a subject state and input the earlier revision accepted is classified RefusalAdded instead of left undecided.
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

`ess verify diff` classifies an added `when_subject:` refusal as `RefusalAdded` where some stored subject state and input that the earlier revision accepted are refused by the later revision, instead of leaving it undecided.

## Evidence

`crates/verify/ess-diff/src/compatibility.rs` (`narrowing`) leaves every outcome that is not a plain `when:` or `otherwise` branch undecided, `when_subject:` included. `story:guard-tests-list-membership` asked for an added membership guard to be called breaking; that unit (0.69.0) left the line out.

## Acceptance

- Adding a `when_subject` refusal that refuses a reachable subject state the earlier revision accepted is classified `RefusalAdded`.
- Adding one that refuses only states the earlier revision already refused stays unclassified.
