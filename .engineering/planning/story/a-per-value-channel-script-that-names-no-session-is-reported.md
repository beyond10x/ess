---
format: aep.planning-md/3
id: story:a-per-value-channel-script-that-names-no-session-is-reported
kind: story
status: active
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
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T00:38:58Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-24T00:39:00Z", actor: "human:timo", revision: 6}
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

For a `session: {per}` channel, the session cases get one verdict in both renderers, and the tools
say when a scripted event reaches no instance. Two cases, one code path in the terminal
(`in_session`, `app.rs:2335-2353`: a session without `<per>` returns `false` at `:2345-2348`, a
session whose `<per>` value differs from the read's at `:2349-2352`):

- a script that names no session for `<per>`;
- a script whose session names a value that no reader on screen reads with.

`ess ui check` reports a script of the first case. A `play` step whose event comes only from such a
script (either case, judged against the page the last `open` showed) fails at that step in the
terminal and says why; the Playwright spec marks the test `test.fixme` with the same text. Every
other `play` keeps its meaning: a `play` before any `open`, or one whose event feeds only another
page, still just runs the clock (`ess-ui-test.md:228-241`). React's fixture mode stops playing a
session-less script into every instance. The reference documents a script's `session`, its tie to
a read or page parameter named `<per>`, and that `scope` is not read in fixture mode.

## Acceptance

- First: a case in `crates/ui/ess-ui-test/tests/parity.rs`, fixtures under
  `crates/ui/ess-ui-test/tests/fixtures/session-script/` (the document and files above). Before:
  step 4 fails in the terminal and the emitted spec has no `fixme`. After (P1-D5 A): `ess ui check`
  reports `script_session` (warning) at `fixtures/scripts/lines`, naming
  `session: {order_id: <value>}`; step 3 fails in the terminal with "the fixture script of `lines`
  names no `session: {order_id: …}`, so its events reach no instance of the per-`order_id`
  channel"; the spec marks the test `test.fixme` with the same text.
- With `session: {order_id: o-1}` in the script, the test passes in the terminal and the spec has no
  `fixme`.
- With `session: {order_id: o-2}` and the page opened at `o-1`: step 3 fails in the terminal naming
  the session `o-2` and the value `o-1`; the spec marks the test `test.fixme` with the same text,
  from a parity rule that compares the script's session with the params of the last `open` (a
  browser `play` cannot fail at run time: the spec refuses only statically, `playwright.rs:729-741`).
- A `play` of an event of a non-session channel, a `play` before any `open`, and a `play` whose
  event feeds only a section of another page pass as today; a case pins each.
- `session_param` (error), a finding of `ess ui check` at the section's path: a section fed by the
  channel whose read passes no `<per>` param while its page declares no `<per>` param either. This
  is React's existing generation refusal (`crates/ui/ess-ui-react/src/emit.rs:2215-2240`: the read's
  param, else the page param). The terminal adopts the same rule: `in_session` and `shows_session`
  fall back to the page param when the read passes none, where today they compare the read's params
  only (`app.rs:2349-2352`, `:2374-2379`); a case pins a page param with no read param in both.
- React's `fixturePlayer` plays a script into an instance only when the script names the instance's
  value; a script without one plays into none (`live.ts.tmpl:51-53`); a case in
  `crates/ui/ess-ui-react/tests/adversary_pass1.rs` beside
  `a_per_param_channel_session_reaches_transport_and_fixture_player`, one compile.
- `adv2_a_script_session_without_the_per_param_does_not_reach_every_ticket`
  (`crates/ui/ess-ui-tui/tests/adversary_pass2_live.rs:280-303`) stays green; no committed tree
  moves (the one per-value script, `examples/partner-portal/fixtures/scripts/ticket_chat.yaml:2`,
  names `ticket_id: tk-01`).
- `ess-ui.md` (generated from the schema's `FixtureIndex.doc` and `Channel.doc`) documents the
  script's `session`; `ess-ui-test.md` lists the new refusal under "One verdict in both renderers".
  `CHANGELOG.md`, under `### Fixed` (no `**Breaking**`): the defect, and the two moved behaviours:
  React fixture mode plays a session-less script of a `session: {per}` channel into no instance,
  and `ess ui test` fails such a `play` where it passed. The new warning does not change
  `ess ui check`'s exit status (`crates/ui/ess-ui-check/src/lib.rs:584`).

## Gate

`cargo clippy` and `cargo test` for `ess-ui-tui`, `ess-ui-test`, `ess-ui-check`, `ess-ui-react`;
`task fmt-check`. Each test 60 s or less, the test step 120 s or less, each build-pool job 10
minutes or less.

## Fit review

Tree `bcfc51555`, read-only. Report item 7 of a downstream application's report of 2026-11-24;
the same defect as item 4 of a consumer report of 2026-11-13 recorded in this story's revision 1.

1. **Need.** A scripted event on a channel that has one instance per param value reaches the
   instance the page shows, or the author is told why not. Reproduction above.
2. **Class.** *Defect*: the two renderers give one test file two verdicts without saying so, against
   `ess-ui-test.md:412-417`; the terminal drops input silently. The reference does not say which
   side's rule is right; each is written only in a code comment (`live.ts.tmpl:45-46`,
   `app.rs:2347`). P1-D5 picks one.
3. **Already expressible?** Yes, the script's `session` key, but undocumented; the idiom does not
   answer the defect (two verdicts, silence).
4. **Fit.** No new key in `ess-ui/1` or `ess-ui-test/1`: two check ids, one parity refusal, one
   behaviour change in React's fixture player, one fallback in the terminal's session match. The
   script rule is the terminal's, already defended by an adversary test; the param rule is React's,
   already enforced at generation.
5. **Second adopter.** Every document whose channel is `session: {per}`: the shipped
   `examples/partner-portal` `ticket_chat` (its script names `ticket_id: tk-01`).
6. **Cost.** Two finding ids (`script_session` warning, `session_param` error) in `ess-ui-check/1`;
   no format version (findings' `check` values grow, `formats.md:219`). `session_param` refuses
   nothing React generated before: React already refuses those documents.
7. **Considered.** P1-D5 (B), React's rule in both, fails open, which the terminal's adversary pass
   refused (`adversary-ui-tui-pass-1` finding 1); (C) the loader refuses the fixture index: a
   document in design, before its scripts are finished, would not load. Chosen (A). For the param
   rule, React adopting the terminal's read-only rule was not taken: React would then refuse
   documents it generates today, a `**Breaking**` change.

## Decisions

**Accept (defect).** No construct added.

- **P1-D5. What a script without a session does on a `session: {per}` channel.** (A) Plays into no
  instance in both renderers; `ess ui check` warns; a `play` of its event, and of a script whose
  session matches no reader on the page last opened, fails in the terminal and is `fixme` in the
  browser. (B) Plays into every instance in both renderers (React's rule today). (C) The loader
  refuses the fixture index. **Recommended: (A).**

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs`: `in_session` (2335-2353) and `shows_session`
  (2357-2380) only, the page-param fallback; `shows_session` becomes `pub` for the runner. `deliver`,
  `nested_targets`, `Options` and the `App` struct are not touched.
- cited: `crates/ui/ess-ui-test/src/runner.rs` (`play`), `crates/ui/ess-ui-test/src/clock.rs`
  (`next` returns the script that plays the beat), `crates/ui/ess-ui-test/src/parity.rs` (the
  session rule against the last `open`), `crates/ui/ess-ui-test/src/playwright.rs` (`Step::Play`,
  729-741)
- cited: `crates/ui/ess-ui-check/src/rules.rs`: `absorb` (564) reads each script's `session`
  beside its name (522); the two checks beside `script_per_channel` (1110-1114);
  `crates/ui/ess-ui-check/src/lib.rs:126` and `schemas/ui/ess-ui.schema.yaml:1769` (the check list)
- cited: `crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl` (`fixturePlayer`, 44-56)
- cited: `schemas/ui/ess-ui.schema.yaml` (`FixtureIndex.doc`, `Channel.doc`, `checks.list`),
  `website/docs/reference/ess-ui.md` (regenerated), `website/docs/reference/ess-ui-test.md`,
  `CHANGELOG.md`
- cited: tests `crates/ui/ess-ui-test/tests/parity.rs` with
  `tests/fixtures/session-script/`, `crates/ui/ess-ui-react/tests/adversary_pass1.rs`,
  `crates/ui/ess-ui-tui/tests/adversary_pass2_live.rs`
- Size: M (about 400 changed lines). Hotfix material once reproduced (`AGENTS.md` § Hotfix).

## Depends on

- None. Lands first of P1 and before `g01-app-forks` (P0), which edits `app.rs` `Options`, `App`
  and a new `fork_with`; this story touches only `in_session` and `shows_session` there, so that
  unit rebases onto it. `g04-step-trace` (shares `runner.rs`, `ess-ui-test.md`) and
  `g06b-chart-rows` (shares `runner.rs`, `playwright.rs`, `parity.rs`) land after it.
