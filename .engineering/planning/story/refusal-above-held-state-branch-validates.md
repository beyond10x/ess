---
format: aep.planning-md/3
id: story:refusal-above-held-state-branch-validates
kind: story
status: draft
title: A refusal declared above a held-state branch validates or names the ordering rule
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Declaring a credential-check refusal above a held-state branch validates when the ordering rule of
https://github.com/beyond10x/ess/issues/486 does not apply to it, or the refusal names the rule
and the fix.

## Evidence

An adopter on ess 0.56.0 reports ESS-COMMAND-004 when moving a credential check above a
held-state branch. The rule shipped for #486 refuses a held-state branch declared after an
accepting or external branch whose input guard can hold with its own; a refusal branch is
neither, so on its text the reported refusal is not that rule. Not yet reproduced: the
reproducer is owed by the first step.

## Acceptance

- A neutral reproducer under `.engineering/repro/` shows which ESS-COMMAND-004 refusal fires and
  which branch kinds it names.
- If the refusal is the #486 rule applied to a branch that is neither accepting nor external, it
  is fixed with a validation test; otherwise the diagnostic text names the rule and the order that
  validates.
