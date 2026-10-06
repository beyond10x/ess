---
format: aep.planning-md/3
id: story:feature-request-467
kind: story
status: draft
title: A generated Optional event field goes through a generator port
tags:
- ess-0.55.0
refs:
- provider: github
  reference: beyond10x/ess#467
relations:
- decomposes: epic:downstream-reported-gaps
revision: 1
---
# A generated Optional event field goes through a generator port

## Acceptance

`ess generate synthesize --target rust` gives an event payload field declared
`{type: Optional<T>, generated: true}` a generator port returning `Option<T>`, as it gives a
non-optional `{generated: true}` field a `generate_<t>` port; the generated behaviour builds the
event from that port's answer instead of a constant `None`, and a conformance or synthesis test
shows both `Some` and `None` reaching the emitted event.

## Context

beyond10x/ess#467, filed 2026-10-06 against 0.52.0. A downstream consumer modelling a quality
measurement had to leave two computed optional fields off its event and mark them `UNMAPPED:`.

## Fit review

Owed. Run `.agents/skills/assessing-external-requests/SKILL.md` before this story is dispatched;
check whether Go, Web and Clap targets give the same field a port too, and whether a source-format
change is needed (none is expected: the declaration already parses).

## Milestone

`release-plan:ess-055`.
