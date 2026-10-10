---
format: aep.planning-md/3
id: story:synthesis-and-validation-refusals-reproduced
kind: story
status: draft
title: Four reported synthesis and validation refusals are reproduced and answered
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Each of four synthesis and validation refusals an adopter reported is reproduced on `main` with a
neutral specification, then either fixed (its own story) or answered with the rule it follows.

## Evidence

An adopter hardening OAuth specifications on ess 0.56.0 reported, without reproducers readable
here:

1. An `example:` on an input narrows synthesis to that one value, and ESS-SYNTH-003 then blames a
   related row. Possibly related: f5994b7804 (invariant-bound inputs grounded).
2. Any `external:` sibling triggers ESS-SYNTH-003.
3. ESS-COMMAND-005 refuses mutants that compare an open String field.
4. ESS-COMMAND-012 forbids a command whose outcomes are all `external:`.

Answered by design on `main`, recorded here so the adopter gets one reply:

- ESS-COMMAND-004 depends on whether a sibling compares a String field: a guard the prover
  declines counts as overlapping (`docs/design/cross-record-and-stored-field-guards.md:763`).
- `mutate --class from-drop` needs at least two `from` states and a `wrong_state` sibling
  (`crates/verify/ess-conformance/src/mutate.rs:1299-1307`).
- ESS-AUTHOR-041 does not see a `wrong_state` step a `when_subject` sibling answers first:
  held-state guards claim nothing (`src/authored.rs:1251-1258`).

## Acceptance

- A reproducer per numbered item, run against `main`; each item ends as a draft story naming the
  reproducer, or as a documented rule cited on the reference page.
