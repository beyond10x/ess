---
format: aep.planning-md/3
id: story:a-predicate-refusal-policy-is-forced-through-an-arranged-row
kind: story
status: draft
title: A predicate refusal failure policy is forced through an arranged row
summary: 'ESS-SYNTH-010: a per-refusal failure policy on a via plus predicate refusal cannot be forced; 9 / 1, 10 / 0 with external.'
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, neutral repro (ess/25, `demo.*` names, committable as a fixture; the reporter confirmed each "after" on the full adopter specification with no new refusal). Counts are selected scenarios / refusals / of which ESS-SYNTH-010, reproduced here on 0.69.2.

A per-refusal failure policy on a predicate refusal (`via:` plus `predicate:`) cannot be forced: only `external:` and `exists: false` refusals can (`refusal_policy::witness`). 9 / 1 / 1; replacing it by `external:` gives 10 / 0 / 0.

Classification: correct refusal (a limitation); the help leads towards weakening the specification.

## Acceptance

- `a_predicate_refusal_policy_is_forced_through_an_arranged_row`: synthesis arranges a row the predicate refuses, as it arranges a missing row for `exists: false`, and the repro gives 10 / 0.
- Until then the help names the authored-scenario route first and does not suggest declaring a predicate refusal `external:`.
