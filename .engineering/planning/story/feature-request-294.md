---
format: aep.planning-md/3
id: story:feature-request-294
kind: story
status: draft
title: 'mutate: no way to declare a known-failing baseline scenario; one failure refuses the whole audit (ESS-MUTATE-001)'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#294
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 2
---
## Outcome

Resolve beyond10x/ess#294: mutate: no way to declare a known-failing baseline scenario; one failure refuses the whole audit (ESS-MUTATE-001).

## Origin

beyond10x/ess#294, from a downstream hardening run on ess 0.48.0; reproduced minimally (triage item 6a, `~/.cache/ess-gaps/triage-cb/`).

## Fit review

Confirmed capability request on current source, not a failing implementation of the present default. mutate.rs Ruler::new at1890 rejects every baseline Failed/Error before constructing the unscored set; only Unsupported/Skipped enter that set. Both direct audit and collect share this ruler; collect at2724 preserves the same refusal. CLI Mutate atmain.rs655-724 exposes no known-failure input. Current refusal is intentional and scientifically necessary without a declared exception contract.

Group294 with296 around one reviewed known-failure assessment contract. They are related but not duplicates:294 specifies mutation scoring/exclusion, while296 covers run/report/strict interpretation. A source-external declaration keyed by exact suite/scenario identity is a promising way to preserve the specification's intended truth without relabeling a failed check as passed; that choice is a proposal, not an implemented or accepted format. Before implementation settle digest/target binding, duplicate/unknown IDs, reasons/tracking references, stale pass/error/unsupported entries, empty eligible sets and versioned reporting. Unlisted failure still refuses; a mutant affecting excluded known-failing cells must be inconclusive unless an independently eligible witness kills it. Test default behavior, mixed known/unknown failures, stale declarations, exclusion-only mutants and full target/collect parity.

This does not authorize treating a failed baseline as a skipped success or silently changing report/2. New generated-runtime status-profile work for389 is a separate correction and does not implement known failures. Source audit only; own new test/build executions0. Binding design and named red-capable acceptance remain the next deliverables.
