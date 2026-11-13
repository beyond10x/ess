---
format: aep.planning-md/3
id: story:a-per-value-channel-script-that-names-no-session-is-reported
kind: story
status: draft
title: A script for a per-value channel that names no session, or a section that cannot match it, is reported
summary: 'With session: {per} on a channel, the headless renderer applies no played event unless the script names the session and a read takes a parameter of that name; nothing says so (a consumer''s UI report, item 4)'
tags:
- backlog
- defect
- ess-ui
- external-request
- from-consumer-report
- severity-medium
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
revision: 1
---
## Need

A channel with `session: {per: <name>}` has one instance per value. For such a channel the headless
terminal renderer applies a played event only when two things hold
(`crates/ui/ess-ui-tui/src/app.rs`, `in_session` and `shows_session`): the fixture script carries
`session: {<name>: <value>}`, and a section of the page shown reads with a parameter called exactly
`<name>` at that value. Otherwise the event belongs to no page and is dropped ("fail closed").

Neither condition is written in the reference: `website/docs/reference/ess-ui.md` documents
`session` as "one channel instance per value", and no page documents a script's `session` key. And
nothing reports a miss: `ess ui check` has no finding, the `play` step of a journey test passes, and
only a later `expect` fails. A consumer who adds `session: {per: …}` to a working channel sees every
played event stop arriving, with no message.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 4 of its collected report.

Reproduction on ess 0.94.0 with `examples/partner-portal` (channel `ticket_chat`,
`session: {per: ticket_id}`, script `fixtures/scripts/ticket_chat.yaml`) and this test:

```yaml
- open: {page: tickets.detail, params: {id: tk-01}}
- expect: {at: pages/tickets.detail/sections/conversation, rows: 5}
- play: {event: tickets.ReplyPosted, with: {id: m-06}}
- advance: 500ms
- expect: {at: pages/tickets.detail/sections/conversation, rows: 6}
```

- As shipped (the script begins `session: {ticket_id: tk-01}`): `1 passed, 0 failed`.
- With that one line removed from the script: `ess ui check` prints no finding about it, steps 1 to
  4 pass, and step 5 fails: `expected 6 rows ...; it shows 5 [m-01, m-02, m-03, m-04, m-05]`.
- With `session: {per: ticket_id}` removed from the channel as well: `1 passed, 0 failed`.

Class: defect. The renderer drops input silently, and the key that decides it is undocumented.

## Acceptance (sketch, to be fixed by the fit review)

- The reference documents a script's `session` key and the rule that ties it to a read parameter of
  the channel's `per` name.
- A fixture script for a channel with `session.per` that names no session value for it is a finding
  of `ess ui check`, by file and channel.
- A channel with `session.per` that feeds a section none of whose reads takes a parameter of that
  name is a finding, by node path.
- A `play` step whose event is applied by no section of the page shown fails at that step and says
  why (no session in the script, or no section in that session), instead of passing.
- The React project's fixture player follows the same rule
  (`crates/ui/ess-ui-react/tests/adversary_pass1.rs`,
  `a_per_param_channel_session_reaches_transport_and_fixture_player`).

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the acceptance is fixed there.
