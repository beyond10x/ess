---
format: aep.planning-md/3
id: story:feature-request-301
kind: story
status: implemented
title: synthesize is 20-30x slower since 0.40.0 on one specification (8 s to 4-7 min, scenarios +15%)
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#301
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T19:35:43Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T19:35:43Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:19Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Synthesis time is proportional to the scenarios it writes again.

## Acceptance

- The #301 reference specification synthesizes within 2x of 0.40.0's time.
- A synthesis timing guard runs in CI.

## Origin

beyond10x/ess#301, reported downstream (8.3 s on 0.40.0 vs 4–7 min on main).

## Fit review

- Class: defect (performance regression). No surface. **accept**: bisect 0.40.0..0.46.1 with a debug-symbol build on the reporter's copy, then fix. Priority 1.

## Released performance repair; timing acceptance remains qualified

Release0.51.0 CHANGELOG records151s to6s with byte-identical suites. Current synthesize/caller.rs:1180-1340 contains deterministic work-count guards, including a_caller_reading_command_costs_no_whole_synthesis_of_its_own, and focused-vs-whole suite comparison; witness_memo.rs caches per-model witness answers. The current three-package run passes these unit tests. A work-count guard is not literally the promised elapsed-time CI guard, and this audit has not recovered the original reference-model timing receipt. Keep the release claim and actual deterministic protection distinct from complete acceptance evidence; do not rebuild the optimization or declare this whole story completed from the changelog alone.
