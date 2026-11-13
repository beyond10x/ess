---
format: aep.planning-md/3
id: story:a-wrapped-answer-carries-literal-members-and-paging-facts
kind: story
status: active
title: A wrapped answer may carry literal members and paging facts beside its rows
summary: An answer object of several members ({success, items, pagination}) has no ess-http/1 form (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
- depends_on: story:an-answer-may-be-wrapped-in-one-named-member
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:45Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:46Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Need

A downstream adopter's reads answer an object of several members: a constant flag, the rows under
a named member, and the paging facts under another (`{"success": true, "items": [...],
"pagination": {...}}`), or the flag and one row (`{"success": true, "item": {...}}`).
`story:an-answer-may-be-wrapped-in-one-named-member` admits one member (`body: {items: array}`)
and refuses an object of two members as `ESS-HTTP-024`; a paged view is refused in every wrapped
form. The adopter counts about 32 open notes on it.

## Acceptance

- An `ess-http/1` view route may declare a body object holding one member with a view form
  (`single`, `array`), any number of members with a literal value, and, on a paged view, one
  member holding the paging facts (page, size, total) under names the binding states.
- A command route's outcome may declare the same shape around `response`.
- OpenAPI describes the object with each member; conformance over the binding reads rows and
  paging facts from their members and compares each literal member.
- Two members with a view form, and a paging member on an unpaged view, are refused as
  `ESS-HTTP-024` naming the member.
- A binding without such a body keeps its bytes.

## Evidence

The one-member rule and its refusal of two members are read from the acceptance of
`story:an-answer-may-be-wrapped-in-one-named-member`. The adopter's routes were observed by the
adopter's session and are not rerun here; the scoper reruns one before the story is proposed.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).
