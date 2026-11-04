---
format: aep.planning-md/3
id: story:an-author-key-named-action-is-not-an-action
kind: story
status: draft
title: An author key named action is not read as an action
summary: A record field or export param keyed action with a map value is read as an Action and misnames the refusal (UI-adopter wave 1 U4 pass-2 note, pre-existing, backlog)
tags:
- backlog
- from-finding
- review-finding
relations:
- informed_by: review-result:w086-u4-adversary-pass-2
revision: 1
---
## Need

The loader still reads a map key `action` that is author data as an Action whenever its value is a map (`crates/ui/ess-ui/src/locate.rs:154`). This predates UI-adopter wave 1 (base 83391ebaeb). The wave 1 U4 adversary pass 2 found it (review-result:w086-u4-adversary-pass-2, note, pre-existing).

- A state type's record with a field named `action` shows the problem. A refusal inside that field is reported as the Action's `unknown field` at `.../type/record/action`.
- A nested export param keyed `action` is refused at the right path but with the Action's message.

## Acceptance

- A refused node under an author-named key `action` is reported with its own refusal at its own path.
- A valid record field or export param named `action` hides no later refusal.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
