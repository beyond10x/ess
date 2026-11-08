---
format: aep.planning-md/3
id: story:a-host-slot-in-a-reused-template-is-one-registration
kind: story
status: draft
title: A host slot in a reused template is one registration with one identity per instance path
summary: 'Reused template holding a slot is refused declared twice in 0.90.0 (U2 adversary; coordinator: draft for 0.91.0)'
tags:
- backlog
- from-finding
- queue-0.91.0
- rank-10
relations:
- derived_from: design:ui-host-slots
- depends_on: story:a-ui-document-declares-a-host-slot
- serves: vision:O2
- depends_on: story:the-react-project-registers-each-declared-host-slot
- depends_on: story:the-terminal-draws-a-host-slot-as-a-labelled-region
revision: 2
---
## Need

Found by the U2 adversary of wave-2026-11-07-ess (ESS 0.90.0). In 0.90.0, a host slot inside a
widget or page template that a document uses twice is refused as "declared twice". This is the
fail-closed reading of the ADR's section 1, where a host name is unique per document. The
coordinator kept that refusal for 0.90.0 and asked for this story.

A reused template is the natural home for a bespoke part repeated across pages. The refusal stops
an adopter from declaring such a part once.

## Acceptance

- A template that holds one host slot, used on two pages, is accepted.
- The host registers one component for that name.
- Each use is its own instance path, named in the text and JSON reports. Coverage and the
  generators address each instance by that path.
- Two slots with one name that come from different declarations are still refused.

## Scope

- inferred: `crates/ui/ess-ui/src/expand.rs` (template expansion), `crates/ui/ess-ui-check/src/`
  (the duplicate-name rule, reports), design:ui-host-slots (a section on templates).
