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
- gap-p1
- gap-reports-2026-11-24
- severity-medium
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
revision: 3
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

## Need, as the fit review reads the source

A channel with `session: {per: <param>}` has one instance per value of `<param>`. A fixture script
for it says which instance it plays with a script-level `session: {<param>: <value>}`
(`crates/ui/ess-ui-tui/src/live.rs:20-21`, read at `:83`). What happens to a script that names no
session differs by renderer:

- Terminal (`ess ui test` and `ess ui run`): `deliver` applies an event to a section only if
  `in_session` holds (`crates/ui/ess-ui-tui/src/app.rs:2184-2186`, nested readers `:2231`), and
  `in_session` returns `false` when the script's session does not name `<param>`, with the comment
  "Fail closed: a session that does not name the `per` param belongs to no page"
  (`app.rs:2335-2353`). A script with no `session` at all takes the same branch. The event is
  dropped with no message.
- React (`fixturePlayer`): `recorded` is `undefined` when the script has no `session` or its
  session lacks `<param>`, and the script then plays into every instance
  (`crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl:44-56`).
- `play` finds the event in the scripts without looking at sessions
  (`crates/ui/ess-ui-test/src/clock.rs:47-60`) and passes; only a later `expect` fails.
- `ess ui check` checks only that a script exists per channel (`script_per_channel`,
  `crates/ui/ess-ui-check/src/rules.rs:1110-1114`).
- The reference says a script "may say `channel`, `session`" and nothing more
  (`website/docs/reference/ess-ui.md:2188`); `scope: param` is "which events reach the actor"
  and "Scope is decided by the ESS model" (`ess-ui.md:2252`, `:2263`; `schemas/ui/ess-ui.schema.yaml:1374`, `:1385`).
  No renderer reads `scope` in fixture mode: the terminal never reads it, React copies it into the
  channel spec (`crates/ui/ess-ui-react/src/emit.rs:3576`) and no runtime code reads it back.

So one test file gets two verdicts: the terminal fails it, the browser passes it. The reference
promises the opposite: "A test file means the same in the terminal and in the browser … The two
renderers never give one file different verdicts without saying so"
(`website/docs/reference/ess-ui-test.md:412-417`). That is a defect.

The failing test, generic names (fresh, not the reporter's files; not run here, the result follows
from the lines above):

`ui.yaml`

```yaml
format: ess-ui/1
app: orders
model: orders.system
placement_profile: fat
fixtures: {dir: fixtures, index: fixtures/index.yaml}
types: {OrderId: string}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: orders.detail
  sections: [{name: all, pages: [orders.detail]}]
channels:
  lines:
    carries: {events: [orders.LineAdded]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
    scope: param
    session: {per: order_id}
pages:
  orders.detail:
    kind: detail_page
    title: Order
    params: {order_id: OrderId}
    sections:
      - name: lines
        component: collection
        reads: {placeholder: orders.Lines, fixture: lines.yaml, key: line_id, params: {order_id: params.order_id}}
        columns: [line_id, qty]
        live: {channel: lines, effect: insert_or_patch}
```

`fixtures/index.yaml`: `scripts: {lines: scripts/lines.yaml}`

`fixtures/lines.yaml`:

```yaml
view: orders.Lines
rows:
  - {line_id: l-1, order_id: o-1, qty: 1}
  - {line_id: l-2, order_id: o-1, qty: 2}
```

`fixtures/scripts/lines.yaml` (no `session`):

```yaml
channel: lines
events:
  - {at: 1s, event: orders.LineAdded, payload: {line_id: l-3, order_id: o-1, qty: 3}}
```

`tests/lines.yaml`:

```yaml
format: ess-ui-test/1
document: ../ui.yaml
tests:
  - name: a played line reaches the order it belongs to
    steps:
      - open: {page: orders.detail, params: {order_id: o-1}}
      - expect: {at: pages/orders.detail/sections/lines, rows: 2}
      - play: {event: orders.LineAdded, with: {line_id: l-3}}
      - expect: {at: pages/orders.detail/sections/lines, rows: 3}
```

Today: `ess ui test --path ui.yaml tests/lines.yaml` fails at step 4 ("expected 3 rows …; it shows
2 [l-1, l-2]"); the Playwright spec of the same file passes; `ess ui check` reports nothing about
the script. With `session: {order_id: o-1}` added to the script, both pass. That one line is the
adopter's workaround today, instead of `only_if: matches(params)` on every live section.

## Outcome

A script for a `session: {per}` channel names its session, or the tools say it does not, in both
renderers alike: `ess ui check` reports such a script, a `play` step whose event reaches no reader
on screen fails at that step and says why, and the React fixture player follows the terminal's rule.
The reference documents a script's `session`, its tie to a read parameter named `<param>`, and that
`scope` is not read in fixture mode.

## Acceptance

- First: the test above, committed under `crates/ui/ess-ui-test/tests/` with its document and
  fixtures. Before: step 4 fails in the terminal and the emitted spec has no `fixme`. After (P1-D5
  A): `ess ui check` reports `script_session` (warning) at `fixtures.scripts.lines`, naming
  `session: {order_id: <value>}`; step 3 fails in the terminal with "the fixture script of `lines`
  names no `session: {order_id: …}`, so its events reach no instance of the per-`order_id`
  channel"; the Playwright spec marks the test `test.fixme` with the same text, from
  `ess-ui-test`'s parity rules.
- With `session: {order_id: o-1}` in the script, the test passes in the terminal and the spec has no
  `fixme`.
- With `session: {order_id: o-2}` and the page opened at `o-1`, step 3 fails in both, naming the
  session `o-2` and the value `o-1` the page reads with.
- A channel with `session.per` that feeds a section none of whose reads takes a param of that name
  is a finding of `ess ui check` at the section's path (React already refuses to generate it,
  `emit.rs:2233-2240`; the terminal drops silently).
- React's `fixturePlayer` plays a script into an instance only when the script names the instance's
  value; a script without one plays into none (`live.ts.tmpl:51-53`); a test in
  `crates/ui/ess-ui-react/tests/` pins it beside
  `a_per_param_channel_session_reaches_transport_and_fixture_player`.
- `adv2_a_script_session_without_the_per_param_does_not_reach_every_ticket`
  (`crates/ui/ess-ui-tui/tests/adversary_pass2_live.rs:280-303`) stays green.
- `ess-ui.md` (generated from the schema's `FixtureIndex.doc` and `Channel.doc`) documents the
  script's `session`; `ess-ui-test.md` lists the new refusal; `CHANGELOG.md` names the fix.

## Fit review

Tree `bcfc51555`, read-only. Report item 7 of a downstream application's report of 2026-11-24;
the same defect as item 4 of a consumer report of 2026-11-13 recorded in this story's revision 1.

1. **Need.** A scripted event on a channel that has one instance per param value reaches the
   instance the page shows, or the author is told why not. Reproduction above.
2. **Class.** *Defect*: the two renderers give one test file two verdicts without saying so, against
   `ess-ui-test.md:412-417`; the terminal drops input silently.
3. **Already expressible?** Yes, the script's `session` key, but undocumented; the idiom does not
   answer the defect (two verdicts, silence).
4. **Fit.** No new key in `ess-ui/1` or `ess-ui-test/1`: one new check id, one refusal in the
   parity rules, one behaviour change in React's fixture player. The rule is the terminal's,
   already defended by an adversary test.
5. **Second adopter.** Every document whose channel is `session: {per}`: the shipped
   `examples/partner-portal` `ticket_chat` (its script names `ticket_id: tk-01`).
6. **Cost.** A new warning id (`script_session`) in `ess-ui-check/1` findings; no format version
   (findings' `check` values grow, `formats.md:219`). React fixture mode changes for a script without
   a session: it played into every instance and plays into none, which `ess ui check` reports.
7. **Considered.** P1-D5 (B), React's rule in both, fails open, which the terminal's adversary pass
   refused (`adversary-ui-tui-pass-1` finding 1); (C) the loader refuses the fixture index: a
   document in design, before its scripts are finished, would not load. Chosen (A).

## Decisions

**Accept (defect).** No construct added.

- **P1-D5. What a script without a session does on a `session: {per}` channel.** (A) Plays into no
  instance in both renderers; `ess ui check` warns; `play` fails at once with the reason.
  (B) Plays into every instance in both renderers (React's rule today). (C) The loader refuses the
  fixture index. **Recommended: (A).**

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs` (`in_session`, `shows_session`, `deliver`)
- cited: `crates/ui/ess-ui-test/src/runner.rs` (`play`), `crates/ui/ess-ui-test/src/clock.rs`,
  `crates/ui/ess-ui-test/src/parity.rs`, `crates/ui/ess-ui-test/src/playwright.rs`
- cited: `crates/ui/ess-ui-check/src/rules.rs` (beside `script_per_channel`),
  `crates/ui/ess-ui-check/src/lib.rs:126` and `schemas/ui/ess-ui.schema.yaml:1769` (the check list)
- cited: `crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl` (`fixturePlayer`)
- cited: `schemas/ui/ess-ui.schema.yaml` (`FixtureIndex.doc`, `Channel.doc`, `checks.list`),
  `website/docs/reference/ess-ui.md` (regenerated), `website/docs/reference/ess-ui-test.md`,
  `CHANGELOG.md`
- inferred: new test fixtures under `crates/ui/ess-ui-test/tests/fixtures/`
- Size: M (about 350 changed lines). Hotfix material once reproduced (`AGENTS.md` § Hotfix).

## Depends on

- None.
