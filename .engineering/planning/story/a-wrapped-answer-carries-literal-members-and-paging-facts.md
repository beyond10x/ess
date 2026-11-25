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
revision: 6
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

- HB-D4: A (a count member beside the rows of an unpaged view), taken by the coordinator on 2026-11-25 (D118); reversible until the release that carries it

- A release that carries this key names this pick in its pull-request line; the operator session merges that release only after the owner has read the list.

## Related (2026-11-25)

A second adopter reports the shape on 0.99.0 (twentieth gap report, shape 5): services wrap answers as
`{success, item}` and `{success, items, total}`, and a consumer-side adapter adds the envelope. Read at
tag 0.99.1, what is released and what is not:

- Released with 0.99.0 (beyond10x/ess#755): one bare form under one named member, `body: {item: single}`
  or `body: {items: array}` on a view and `{<member>: response}` or `{<member>: response.<field>}` on
  a command outcome (`CHANGELOG.md`, 0.99.0, the `ess-http/1` route's body entry;
  `docs/design/http-presentation-binding.md`, the rows `{<member>: single}` to `{<member>: response.<field>}`).
  `body: {item: single}` therefore answers `{"item": {...}}`.
- Not released, this story: the constant sibling `success: true`, because "an object of two members" is
  refused `ESS-HTTP-024` and "a body holds one form: fixed members beside the wrapped value
  (`{"success": true, "app": …}`) are not a body" (the design page, after the table); and `total`,
  because a paged view is refused in every wrapped form (`ESS-HTTP-024`, `-030`, `-031`) and no member
  holds a count on an unpaged one.
- A literal body member is read only for a standard refusal: "`{written}` is a literal body, which only
  a standard refusal answers" (`crates/specify/ess-http/src/lib.rs:773-775`). The error side of the same
  services, `{success: false, message, errors}`, is
  `story:an-error-outcome-may-answer-its-error-s-fields-as-the-whole-json-body-with-no-member-naming-the-error`.

Decision HB-D4 (shape 5, `total` on a view that is not paged): what a member beside the rows holds
when the service sends `{success, items, total}` and pages nothing.

- A. `total` names the number of rows the answer holds: a member `{<member>: count}` beside the wrapped
  `array`, on a view that is not paged. It adds no model fact, since the number is the length of the
  rows, and both generated servers and the conformance runner can check it.
- B. Only the paging facts of a paged view carry a count (the story's acceptance as written), so an
  unpaged `{success, items, total}` is not expressible and the consumer adapter adds `total`.
- C. Decline any count member.
- Recommended A: the service's `total` on an unpaged list is the length of its list, which the suite can
  check; B leaves the commonest list envelope outside the binding. The accepted acceptance (a paging
  member on an unpaged view is refused) changes under A, so the owner or the operator confirms before
  the story is dispatched.
