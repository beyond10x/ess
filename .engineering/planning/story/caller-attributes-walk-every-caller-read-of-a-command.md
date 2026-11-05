---
format: aep.planning-md/3
id: story:caller-attributes-walk-every-caller-read-of-a-command
kind: story
status: draft
title: caller_attributes walks every caller read of a command
summary: ess-gen http::caller_attributes misses caller reads in an external branch's when_subject and in an error payload (beyond10x/ess#740)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#740
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

ess_gen::http::caller_attributes documents that it collects every caller read of a command, but does not walk an external: branch's when_subject predicate and input guard, nor {caller: ...} in an error payload; by reading, also not related-row conditions.

Reported in https://github.com/beyond10x/ess/issues/740 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept: a small ess-gen fix; the issue is from reading the code, so the failing test comes first.
