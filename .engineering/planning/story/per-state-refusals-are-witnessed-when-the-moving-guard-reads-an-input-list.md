---
format: aep.planning-md/3
id: story:per-state-refusals-are-witnessed-when-the-moving-guard-reads-an-input-list
kind: story
status: draft
title: Per-state refusals are witnessed when the moving branch's guard reads an input list
summary: wrong_state witnesses are refused ESS-SYNTH-003 when the moving branch's when_subject reads an input list, though the outcome scenario solves it (beyond10x/ess#726)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#726
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

A command whose moving branch carries a when_subject predicate over an input list loses its per-state refusal scenarios: synthesis tries one candidate input and does not solve the guard from its literals, although it solves the same guard for the branch's own outcome scenario.

Reported in https://github.com/beyond10x/ess/issues/726 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept: a synthesis defect; the per-state witness should reuse the outcome scenario's solved input. Pairs with #727 and #734 (list guards).
