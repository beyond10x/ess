---
format: aep.planning-md/3
id: story:a-synthesized-binding-scenario-reads-a-left-out-awaited-input-from-the-answer
kind: story
status: draft
title: A synthesized binding scenario reads a left-out awaited input from the answer
summary: The binding scenario for an awaited Optional input passes against a target that leaves it out, without a hand edit.
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

Found while landing `story:an-optional-input-a-system-command-awaits-is-compared-only-when-issued`
(0.69.0). For an awaited system command whose `Optional` input is copied into the row and published
in an event, the binding scenario synthesis generates reads that input from the observed event
(`Finished`), not from the awaited answer. When the target leaves the input out, which the
specification allows, the scenario errors with ESS-CF-SUITE "`jobs.job.Finished` carried no field
`note`" instead of comparing the row as empty. The unit's test rewrote that one value to read the
answer so the invocation path is exercised.

## Acceptance

- The synthesized binding scenario for that fixture passes against a target that leaves the
  `Optional` input out and against one that issues it, with no hand edit of the suite.
- A target that publishes a value it never issued still fails.
- The Rust, Go and TypeScript runners give the same verdicts; one test replays the suite through all
  three.
