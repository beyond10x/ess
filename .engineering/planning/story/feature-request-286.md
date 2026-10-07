---
format: aep.planning-md/3
id: story:feature-request-286
kind: story
status: draft
title: A view declares which actors may read it
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#286
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
---
## Outcome

A specification declares which actors may read a view, and conformance checks it.

## Acceptance

- A view readable by one of two actors validates.
- Served components refuse the other actor's read.
- A synthesized scenario witnesses that refusal.

## Origin

beyond10x/ess#286, reported downstream on 0.48.0. Read-side counterpart of #265.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (coordinator, 2026-10-01).

- Need: declare which actors may read a view. Class: gap. Existing idiom: `may:` grants commands; #265 enforces command grants on served components and documents views as open. Fit: one grant table, so views belong in the same `may:` rather than a second `readable_by:` on the view.

## Decisions

- **accept, redesigned:** an actor's `may:` may name views from ess/21. A view no actor names stays open to every caller, so existing documents keep their meaning. Served components refuse an ungranted read with the same 403 body as #265, and synthesis adds `<view>/grant/denied`. Scheduled after #265 merges, reusing its `Caller` and `admit`.
