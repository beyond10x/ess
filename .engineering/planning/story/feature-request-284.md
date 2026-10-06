---
format: aep.planning-md/3
id: story:feature-request-284
kind: story
status: implemented
title: ess ui check does not check that a page actor is granted the commands it binds
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#284
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T14:01:05Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-04T14:01:06Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:09Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Outcome

`ess ui check` refuses a page binding to a command its actor is not granted.

## Acceptance

- The #284 reduction fails `ess ui check`, and the error names the page, actor and command.

## Origin

beyond10x/ess#284, reported downstream on 0.48.0.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (coordinator, 2026-10-01).

- Need: `ess ui check` refuses a page binding a command its actor is not granted. Class: defect (a check the tool already implies). No authored surface changes.

## Decisions

- **accept as proposed.**

## Reconciliation

Backlog reconciliation (coordinator, 2026-10-01).

- UI spec: lower priority than every base-spec story (operator, 2026-10-01).

## Current source reconciliation (2026-10-02)

Still open; do not mark implemented or dismiss the consumer's grant-checking need. Current source at 482609 differs from the report's syntax premise: crates/ui/ess-ui/src/model.rs:239 ActorSource admits FromSession, Anonymous and Unmapped only, and Page at :543 has no named actor field. The published UI reference line152 likewise lists from_session and anonymous. ess-ui-check/src/model.rs:280 checks that each referenced command exists, but :307-312 performs no actor-specific command-grant comparison; its view-readability check at :190 pools grants from every actor. A named-page-actor interpretation therefore requires an explicit authored-contract decision, not merely adding a comparison to the current checker. The earlier accept decision remains recorded, but its implementation scope must be reconciled against these concrete types before dispatch. No current red regression has yet established the report's claimed successful admission of that exact named-actor document.
