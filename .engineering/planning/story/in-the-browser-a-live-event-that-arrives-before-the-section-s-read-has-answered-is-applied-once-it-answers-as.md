---
format: aep.planning-md/3
id: story:in-the-browser-a-live-event-that-arrives-before-the-section-s-read-has-answered-is-applied-once-it-answers-as
kind: story
status: draft
title: In the browser, a live event that arrives before the section's read has answered is applied once it answers, as in the terminal
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-live-insert-of-a-key-the-section-already-holds-shows-one-row-in-both-renderers
revision: 1
---
## Need

A scripted event at `0s` reaches a collection in the terminal and is lost in the browser. In a
section with `coalesce` (the reporter saw it in a chart) the browser draws it. The cause is
general: the React runtime applies an event that arrives while the section's read is
outstanding to the empty rows of the loading read, then throws the result away when the read
answers. A section with `coalesce` longer than the read's latency applies its batch after the
answer, so it keeps the event.

Reported by a downstream application on 0.98.0 (third list, item 4).

Both sides:

- Terminal: the test runner plays everything due at zero before the first step
  (`crates/ui/ess-ui-test/src/runner.rs:91-92`). `apply_live` holds an event whose section's read
  is outstanding ("A read outstanding … apply once it has answered",
  `crates/ui/ess-ui-tui/src/app.rs:2475-2483`; nested readers `:2255-2261`). `pump` replays the
  held events when reads answer (`:1437-1440`, `apply_deferred` `:2538-2544`). A coalesced batch
  goes through the same path (`flush`, `:2314-2330`). The terminal draws the event in a
  collection and in a chart alike. It once dropped such events too: finding 4 of
  `.engineering/planning/review-result/adversary-ui-tui-pass-2.md:120` ("a list without coalesce
  loses it every time"), fixed by the deferral.
- React: the fixture player fires a 0 s entry on a 0 ms timer as soon as the channel is taken
  (`crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl:59-80`). A section without `coalesce`
  calls `flush` at once (`:526-531`), which applies the event to `current ?? base.current`
  (`:508`). `base.current` is the loading read's rows, `[]` (`runtime/data.ts.tmpl:836-839`). A
  fixture read answers after `LATENCY_MS = 120` (`data.ts.tmpl:70-73`). When it answers,
  `readKey` changes and the effect at `live.ts.tmpl:444-451` sets `rows` back to `null`, so the
  read's rows show without the event. With `coalesce: 500ms` the batch waits in `pending`, which
  that effect keeps for the same source (`:447`), and is applied to the answered rows.

The reference: "Anything due at zero has played before the first step"
(`website/docs/reference/ess-ui-test.md:357-358`), and one verdict in both renderers
(`:412-417`). Neither page says what happens to an event during an outstanding read. The
terminal's rule is written only in its code.

**Which side is the defect: React.** It loses an event the terminal applies, and its own
`coalesce` path keeps it, so its two paths disagree. The rule every renderer keeps, written down
here: an event that arrives while the section's read is outstanding waits, and is applied in
arrival order to the rows the read answers. A `refetch` effect arriving then is answered by that
read. In served mode the same race drops a real event that arrives during the first read.

Reproduction (fresh, generic names, not run here). The `ui.yaml` of `sp3-loop-period`, with
`live: {channel: lines, effect: insert_or_patch}`, and:

`fixtures/lines.yaml`: `{view: orders.Lines, rows: [{line_id: l-1, qty: 1}]}`;
`fixtures/scripts/lines.yaml`:

```yaml
channel: lines
events:
  - {at: 0s, event: orders.LineChanged, payload: {line_id: l-2, qty: 2}}
```

`tests/zero.yaml`:

```yaml
format: ess-ui-test/1
document: ../ui.yaml
tests:
  - name: an event at zero is on screen before the first step
    steps:
      - open: {page: orders.list}
      - expect: {at: pages/orders.list/sections/lines, rows: 2}
```

Terminal: the test passes with rows `[l-2, l-1]`. Browser (the Playwright spec of the same file):
the event lands on `[]` at about 0 ms, and the read's answer at 120 ms replaces it, so the section
shows 1 row and the test fails. With `coalesce: 500ms` on the `live` block and `advance: 500ms`
before the `expect`, both pass.

## Outcome

The React runtime holds a live event that arrives while the section's read (or a nested reader's)
is outstanding, and applies it in arrival order to the rows the read answers. This holds with
and without `coalesce`, on the filtered (`keep`) path and the plain one. The schema's `Live`
text and `ess-ui-test.md` § Time and live events say so.

## Acceptance

- First: the rule is a pure function exported from `runtime/live.ts`: the events waiting, the
  read's answer, and the spec give the rows. A test in `crates/ui/ess-ui-react/tests/` runs it
  under `node` after `tsc` (harness of `overlay_params.rs:120-190`; the stub React there does not
  run effects, so `useLive` itself is not driven). On 0.98.0 the equivalent path loses the event.
  After the change, an `insert_or_patch` of `l-2` received before the answer `[l-1]` gives
  `[l-2, l-1]`.
- An `insert_top` of a key the answer already holds gives one row, which needs
  `sp1-held-key-insert`.
- A `refetch` event received before the answer starts no second read.
- The reproduction above is committed as a terminal case under `crates/ui/ess-ui-test/tests/`
  (green before and after). The Playwright spec it emits is run in the local browser lane
  (`task test-browser-product` or the Playwright run the lane names), not in CI.
- `CHANGELOG.md` names the fix under `### Fixed`.

## Fit review

Verdict: **adopt (defect)**. Tree `bcfc51555`, read-only; the lines cited are unchanged since
`0.98.0`.

1. Need: a scripted or served event is not lost because it came before the first answer.
2. Class: defect. One test file gets two verdicts (`ess-ui-test.md:412-417`).
3. Already expressible: `coalesce` longer than the read's latency hides it. That is the
   workaround.
4. Fit: no new key. The React runtime takes the terminal's rule.
5. Second adopter: the shipped `examples/partner-portal` activity script plays `ev-10` at `0s`
   (`fixtures/scripts/activity.yaml`) into the `feed` section, which has no `coalesce`
   (`ui.yaml:219`). The browser loses it.
6. Cost: React behaviour changes only where it disagreed with the terminal. No format version,
   and no generated tree pins `runtime/live.ts`.
7. Considered: see SP-D4.

Covers report item 4 of the third list. The generic feature is "a live event waits for the
read it applies to".

## Decisions

- **SP-D4. What React does with an event that arrives during an outstanding read.** (A) It holds
  the event and applies it when the read answers, for every outstanding read: the first, a
  `refetch`, and new params of the same source. This is the terminal's rule
  (`app.rs:2475-2483`). (B) Only during the first read; a refetch still resets applied events, as
  today. That leaves a smaller disagreement. (C) Make the terminal drop the event too. That
  reopens adversary finding 4 and loses events in both. **Recommended: (A).**

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl` (`useLive`, `:431-552`)
- cited: `schemas/ui/ess-ui.schema.yaml` (`Live` doc), `website/docs/reference/ess-ui.md` (regenerated), `website/docs/reference/ess-ui-test.md`, `CHANGELOG.md`
- inferred: a new test in `crates/ui/ess-ui-react/tests/` and a terminal case in `crates/ui/ess-ui-test/tests/`
- Size: S to M (about 150 changed lines). Hotfix-sized: yes, after 0.98.1. It changes no
  verdict except where the renderers disagreed. To reproduce: the Playwright spec of the
  reproduction against `ess generate ui --target react` (needs `npm` and a browser). The pure
  function's node test needs `tsc` and `node` only.

## Depends on

- `sp1-held-key-insert` (0.98.1). A held event re-applied after an answer that already holds its
  key must not duplicate the row.
- Same file as `sp3-loop-period` (`fixturePlayer`, a different function): one patch release can
  carry both, as two commits.
