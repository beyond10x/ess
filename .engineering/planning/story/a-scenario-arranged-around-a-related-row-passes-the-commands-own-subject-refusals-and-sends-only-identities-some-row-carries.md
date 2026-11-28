---
format: aep.planning-md/3
id: story:a-scenario-arranged-around-a-related-row-passes-the-commands-own-subject-refusals-and-sends-only-identities-some-row-carries
kind: story
status: draft
title: A scenario arranged around a related row passes the command's own subject refusals, and an arranging step sends only identities some row carries
summary: 'Three causes of 12 failing scenarios a downstream suite shows on 0.99.9: the related-row strategy ignores subject refusals, an arranging step sends an identity no row carries, and two new failures on 0.99.9'
tags:
- defect
- gap-reports-2026-11-28
- hotfix-0.99.11
relations:
- serves: vision:O2
- informed_by: story:an-unknown-instance-scenario-is-sent-an-identity-no-row-carries-where-a-related-guard-reads-the-addressed-row
- informed_by: story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent
revision: 3
---
## Need

A downstream specification of reusable building blocks (ess 0.99.2, checked again on 0.99.9 and on
`main` at `298790bd0`) lists 15 scenarios of its synthesized suite that fail on the interpreted
target. In every one the interpreter gives the answer the model requires; the scenario is wrong.
Three causes remain after the refusals of 0.99.4 took five of them out of the suite (those five
belong to `story:a-synthesized-scenario-whose-setup-its-own-model-refuses-is-written-with-the-rows-its-guards-relate-or-refused-by-synthesis`).
The downstream gate treats exactly those scenarios as known failures, so a real defect among them
would be hidden; none of the 15 is a specification defect.

First cause, 6 of the 15: the related-row strategy ignores the command's own `when_subject:`
refusals. In `crates/verify/ess-conformance/src/synthesize/related_guard.rs`, `selects` drops the
subject-predicate, subject-field and subject-state branches ("the arrangement leaves them
unselected"), and `steering_for` grounds the input only on the `when_related:` refusals. The
addressed row is never steered past the subject refusals that answer before the related ones
(the one precedence rule: unknown identity, then subject guards, then related rows, then the
accepting branch). Shape: a command that guards its subject with a comparison against an input
(`when_subject: {predicate: target != input.expected}`) and then on a row its input names
(`when_related: {via: input.by, exists: false}`). The scenario for the accepting branch, and the
one for the later related refusal, create the subject with one value and send the other, so the
subject refusal answers. Four of the 6 are one family (a merge-like command whose subject guards
include "not yet approved"; the scenario sends the command on a row that never reached the guarded
state), two are a compare-and-set.

Second cause, 4 of the 15: an arranging step sends an identity no row carries, so that step's own
`when_related: {via: input.<field>, exists: false}` refusal answers instead of the step the
scenario needs. It shows in the held-state family (`run_state_refusal` calling `arrange_first` in
`synthesize.rs`: the arranging command is sent at its plain witness for the related identity,
where the creator of the related row is not run first) and in a creating command placed beside an
invariant check (a related identity sent that no earlier step created). One scenario of the same
entity registers the related row and passes, so the machinery exists. The exact site of the
arranging run is not pinned yet. A neutral reduction of the held-state shape (a lot claimed by a
bidder that must exist, a withdraw refused in the claimed state) passes on 0.99.9, so the cause
needs the downstream's extra ingredient: the reduction is part of this story's work.

Third cause, 2 failures new on 0.99.9 and not in the downstream's list (0.99.2 passes them): two
scenarios of a command that sets a list of related identities on an entity, and of the invariant
scenario built on it, are answered by an error branch of the command's own, where the suite
requires the accepting outcome. The cause is being found.

Neutral reproduction of the first cause on 0.99.9: `.engineering/repro/a-scenario-arranged-around-a-related-row-passes-the-commands-own-subject-refusals-and-sends-only-identities-some-row-carries/system.yaml`
(a pin moves by compare and set by a member that exists). `ess verify conform synthesize` writes
11 scenarios, 0 refusals; `ess verify conform run --target interpreted --report-format 2` ends
`11 scenarios: 9 passed, 2 failed`, the failures being `MovePin/outcome/moved` (the suite creates
the pin at `target` and sends `expected`) and `MovePin/outcome/unknown-member` (the same send
answers the subject refusal first).

The first, second and third causes fail the same scenarios on 0.99.9 and on `main`; the first two
are older than 0.99.8 (the downstream's 15 are the same on 0.99.2).

## Outcome

A scenario synthesis writes for a branch below a command's subject refusals, and a scenario whose
arranging steps are other commands, is written so the interpreter answers each step as the scenario
says: the subject is created at the value the input compares against (or the input is sent the
value the subject holds), and every identity an arranging step sends names a row an earlier step
created. Where no arrangement exists the branch is refused by synthesis and says why. A suite
never holds a scenario its own model answers differently.

## Acceptance

- On the reproduction above, `ess verify conform run --target interpreted` passes all 11
  scenarios; before the fix 2 fail.
- A red test for the first cause fails on 0.99.9 and passes with the fix; one for the second cause
  on a reduction the fix's author finds (the neutral held-state shape passes today and is not the
  red test); one for the third once its cause is named.
- On the downstream specification, the 6 scenarios of the first cause and the 4 of the second
  pass, and the 2 new failures of 0.99.9 pass; the 5 refused scenarios stay refused. This is
  checked by the downstream's own run, not by a committed test.
- A scenario written before this change for a branch of another shape keeps its bytes; the
  committed byte tables move only by the rows of the new fixtures.
- D4: when an outcome reads a related row through `{related: {via, field}}`, the expectation names the
  row the command addresses, not the row of that entity created first. Reproduced on 0.99.9 by a
  second downstream (92 scenarios, 1 failed). It is in hotfix 0.99.11, in progress.

## Fit review

1. Need: a suite written by synthesis must pass the model it was written from. Reproduction above.
2. Class: defect. Synthesis writes a scenario the model's own interpreter answers differently. The
   sibling defects of 0.99.8 (`story:an-unknown-instance-scenario-is-sent-an-identity-no-row-carries-where-a-related-guard-reads-the-addressed-row`)
   were the same class one precedence step earlier.
3. Already expressible: no author action avoids it; an authored scenario per branch replaces the
   generated one.
4. Fit: the arrangement the related-guard path already builds, steered by the subject refusals it
   leaves out today.
5. Second adopter: any command with a compare-and-set or a state guard on its subject beside a
   related-row refusal.
6. Cost: scenarios gained or corrected in suites of such models; no format key, no new diagnostic.
7. Considered: change nothing (the downstream keeps a list of known failures); refuse the branches
   (loses 10 scenarios); steer the arrangement (this story).

## Decisions

- None open. Taken as a defect, to ship as a hotfix.

## Status

hotfix 0.99.11 in progress (branch `hotfix/0.99.11`). The first, second and third causes are the
three groups of the work; the third is not yet understood.

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/tests/related_guard.rs
- inferred: CHANGELOG.md
