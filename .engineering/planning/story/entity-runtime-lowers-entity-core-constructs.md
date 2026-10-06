---
format: aep.planning-md/3
id: story:entity-runtime-lowers-entity-core-constructs
kind: story
status: draft
title: Entity Runtime lowering covers the five constructs entity-core gains
tags:
- upstream-gated
refs:
- provider: github
  reference: beyond10x/entity-runtime#54
relations:
- depends_on: story:feature-request-231
revision: 1
---
# Entity Runtime lowering covers the five constructs entity-core gains

## Acceptance

Once entity-core ships the features beyond10x/entity-runtime#54 asks for, `ess-entity-runtime`
lowers `sets: {field: {increment: n}}`, a String type's declared alphabet, `<text>.count` in a
guard, `{cleared: true}` and an Optional-to-Optional `updates:`, each removed from the generated
lowerable-subset table and executed by an Entity Runtime conformance run.

## Context

0.53.0 (beyond10x/ess#231) made the lowering refuse each of these by name and publish its
lowerable subset; that story is implemented. The remaining work waits on entity-core, tracked
upstream as beyond10x/entity-runtime#54 (open, filed 2026-10-05). The open blocker
`decision-blocker:entity-core-lowering-features` moves from #231 to this story.

## Milestone

None until beyond10x/entity-runtime#54 ships; then the next ESS minor.
