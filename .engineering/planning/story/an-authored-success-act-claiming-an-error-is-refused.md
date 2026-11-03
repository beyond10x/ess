---
format: aep.planning-md/3
id: story:an-authored-success-act-claiming-an-error-is-refused
kind: story
status: draft
title: An authored success act that also claims an error is refused
summary: 'outcome: <success> plus error: compiles to a step no target can pass; refuse it at compile (W1 RUN1 review finding, pre-existing, backlog)'
tags:
- backlog
- review-finding
relations:
- informed_by: review-result:adversary-w1-run1-pass-1
- informed_by: review-result:security-w1-run1-pass-1
revision: 2
---
## Need

An authored act may name a success outcome and also claim an `error:`. `authored.rs` accepts the
combination (`error()` at `crates/verify/ess-conformance/src/authored.rs:4261` refuses only undeclared
names; nothing refuses an error on a branch that declares none). Since wave W1 such an act lowers to
`expect_outcome`, `expect_no_error`, `expect_error`, which no target can pass; before W1 it passed only a
target that answered the success branch with that error. Found by both W1 RUN1 review passes
(review-result:adversary-w1-run1-pass-1, review-result:security-w1-run1-pass-1); pre-existing at
b81deac78; no document in the tree writes one.

## Acceptance

- An authored act that names a branch declaring no error and also claims `error:` is refused at
  compile, naming the act, the outcome and the claimed error.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.

## Repro

Two failing cases were written by the W1 RUN1 reviewers and left out of the committed test files, because the
finding is out of W1's scope: `a_success_act_that_also_claims_an_error_is_refused` (adversary pass; compiles
an `outcome: accepted` act with `error: billing.invoice.InvalidAmount` against the coverage-producer billing
model and asserts `!compiled.is_complete()`) and `w1run1_security_a_success_act_claiming_an_error_is_refused`
(security pass; the same act on the `billing` example). Either becomes this story's first failing test.
