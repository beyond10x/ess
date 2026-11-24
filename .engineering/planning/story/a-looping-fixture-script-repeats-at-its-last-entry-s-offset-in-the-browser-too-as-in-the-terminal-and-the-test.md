---
format: aep.planning-md/3
id: story:a-looping-fixture-script-repeats-at-its-last-entry-s-offset-in-the-browser-too-as-in-the-terminal-and-the-test
kind: story
status: draft
title: A looping fixture script repeats at its last entry's offset in the browser too, as in the terminal and the test clock
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 2
---
## Need

A fixture script with `loop: true` repeats. The terminal, the test runner's clock and the
Playwright spec emitter put the next cycle at the last entry's `at`. The React fixture player
puts it 1 s later, and never sooner than 2 s. So the same test file passes in the terminal and
fails in the browser once a looping script reaches its second cycle.

Reported by a downstream application on 0.98.0 (third list, item 3).

Both sides:

- Terminal: "A looping script restarts when its last entry has played: its cycle is the offset of
  that last entry" (`crates/ui/ess-ui-tui/src/live.rs:5-6`). `Script::cycle` returns the last
  entry's `at` (`:88-90`). `Player::due` plays entry `i` of cycle `n` at `n * cycle + at`
  (`:121-145`), and does not loop a script whose cycle is zero (`:137`). Its unit test pins
  `[4, 4, 8, 8]` for entries at 0 s and 4 s (`:221-232`).
- Test clock: `advance` counts cycles by the last entry's `at`
  (`crates/ui/ess-ui-test/src/clock.rs:25-38`). `next`, which turns `play` into a time, uses the
  same rule ("`ess_ui_tui::live::Player`", `:54-66`). The Playwright spec turns that time into
  `page.clock.runFor(…)` (`crates/ui/ess-ui-test/src/playwright.rs:729-748`), so the browser is
  moved by the terminal's period.
- React: `fixturePlayer` uses `const period = Math.max(1000, ...script.events.map((entry) =>
  entry.at)) + 1000;` (`crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl:58`) and starts
  the next round after `period` (`:76`). A script whose entries are all at 0 s loops every 2 s in
  React and plays once in the terminal.

The reference: a script "may say … `loop: true`, which repeats the script from the start after
its last entry (default: it plays once)" (`website/docs/reference/ess-ui.md:2188`, from
`schemas/ui/ess-ui.schema.yaml:1343`). "Both renderers play every channel's fixture script on
one clock … Looping scripts repeat" (`website/docs/reference/ess-ui-test.md:357-361`). "The two
renderers never give one file different verdicts without saying so" (`ess-ui-test.md:412-417`).
"After its last entry" names no gap. The terminal, the test clock and the emitter agree on none,
and React adds 1 s that nothing documents.

**Which side is the defect: React.** It disagrees with the clock its own Playwright spec is
driven by. The reference is silent on the exact period, so the rule every renderer keeps is
written down here: the cycle is the last entry's `at`, and a script whose cycle is zero plays
once.

Reproduction (fresh, generic names, not run here; the result follows from the lines above):

`ui.yaml`

```yaml
format: ess-ui/1
app: orders
model: orders.system
placement_profile: fat
fixtures: {dir: fixtures, index: fixtures/index.yaml}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: orders.list
  sections: [{name: all, pages: [orders.list]}]
channels:
  lines:
    carries: {events: [orders.LineChanged]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  orders.list:
    kind: list_page
    title: Lines
    sections:
      - name: lines
        component: collection
        reads: {placeholder: orders.Lines, fixture: lines.yaml, key: line_id}
        columns: [line_id, qty]
        live: {channel: lines, effect: patch_row}
```

`fixtures/index.yaml`: `scripts: {lines: scripts/lines.yaml}`;
`fixtures/lines.yaml`: `{view: orders.Lines, rows: [{line_id: l-1, qty: 1}]}`;
`fixtures/scripts/lines.yaml`:

```yaml
channel: lines
loop: true
events:
  - {at: 1s, event: orders.LineChanged, payload: {line_id: l-1, qty: 2}}
  - {at: 4s, event: orders.LineChanged, payload: {line_id: l-1, qty: 3}}
```

`tests/loop.yaml`:

```yaml
format: ess-ui-test/1
document: ../ui.yaml
tests:
  - name: the second cycle starts at the last entry
    steps:
      - open: {page: orders.list}
      - advance: 5s
      - expect: {at: pages/orders.list/sections/lines/rows/l-1/columns/qty, text: "2"}
```

Terminal (`ess ui test`): the entries play at 1 s, 4 s and 5 s, so the cell reads `2` and the
test passes. Browser (`ess ui test --playwright`, `runFor(5000)`): the period is 5 s, the second
cycle's first entry plays at 6 s, so the cell still reads `3` and the test fails.

Carried by patch release 0.98.1: the `hotfix/0.98.1` tree holds the `fixturePlayer` change, the schema sentence, the `CHANGELOG.md` line and the node test `a_looping_fixture_script_repeats_at_its_last_entry_s_time`, SP-D3 taken as A; the terminal pin case under `crates/ui/ess-ui-test/tests/` is the one acceptance line not yet in it.

## Outcome

The React fixture player repeats a looping script at its last entry's `at`, and plays a script
whose last entry is at 0 s once, as the terminal does. The schema's `FixtureIndex` text, which
`ess-ui.md` is generated from, says the cycle is the last entry's offset, so the next cycle's
first entry plays at the same moment as the last one, after it.

## Acceptance

- First: a test in `crates/ui/ess-ui-react/tests/` compiles the generated `runtime/live.ts` with
  `tsc` and runs it under `node` with a recording `setTimeout` (the harness of
  `crates/ui/ess-ui-react/tests/overlay_params.rs:120-190`; it needs `tsc` and `node`, and no
  browser or network). It holds `fixturePlayer`'s second round for the script above at 4000 ms. On
  0.98.0 it fails with 5000.
- The same test holds a script with one entry at 0 s and `loop: true` to one round.
- `live.rs`'s `a_looping_script_plays_up_to_the_clock_and_restarts` stays green unchanged.
- The reproduction above is committed under `crates/ui/ess-ui-test/tests/` as a terminal case
  that passes before and after: it pins the rule on the terminal side.
- `CHANGELOG.md` names the browser's period under `### Fixed`.

## Fit review

Verdict: **adopt (defect)**. Tree `bcfc51555`, read-only; the lines cited are unchanged since
`0.98.0`.

1. Need: one looping script plays at the same moments in both renderers. Reproduction above.
2. Class: defect. It breaks "one verdict in both renderers" (`ess-ui-test.md:412-417`), and
   the Playwright emitter already assumes the terminal's period.
3. Already expressible: a script can avoid looping, which is a workaround and not a fix.
4. Fit: a one-line runtime change and one sentence of the schema text. No new key.
5. Second adopter: the shipped `examples/partner-portal` loops four scripts
   (`fixtures/scripts/activity.yaml`, `metrics.yaml`, `ticket_chat.yaml`, `tickets.yaml`).
6. Cost: in the generated app's fixture mode, the gap between cycles shrinks by 1 s, and a
   last entry and the next first entry can play together. No generated tree pins `runtime/live.ts`
   (`git grep -l runtime/live.ts -- generated examples website` is empty). No format version.
7. Considered: see SP-D3. Changing the terminal and the clock would move accepted terminal
   verdicts, while React's period is pinned by no test.

Covers report item 3 of the third list. The generic feature is "one fixture clock for every
renderer". Not covered: the React player starts when a section first takes the channel
(`acquire`, `live.ts.tmpl:249-316`) and the terminal starts at application start
(`clock.rs:3-4`). That shows only for a channel first used on a later page. It is named here and
not designed.

## Decisions

- **SP-D3. What the loop period is.** (A) The last entry's `at` in every renderer: React drops
  the `+ 1000` and the 1 s floor, and a zero-length script plays once. Terminal and test-clock
  verdicts are unchanged. (B) A fixed 1 s gap after the last entry everywhere: this changes
  `live.rs`, `clock.rs` and every accepted terminal test that crosses a cycle. (C) (A) now, plus
  a script key `period:` later for an author who wants a gap. That is a new key in the fixture
  script, so it waits for a request. **Recommended: (A).**

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl` (`fixturePlayer`, `:47-84`)
- cited: `schemas/ui/ess-ui.schema.yaml` (`FixtureIndex` doc, `:1343`), `website/docs/reference/ess-ui.md` (regenerated), `CHANGELOG.md`
- inferred: a new test in `crates/ui/ess-ui-react/tests/` and a terminal case in `crates/ui/ess-ui-test/tests/`
- Size: S (about 100 changed lines, most of it the tests). Hotfix-sized: yes. It changes no
  verdict except where the renderers disagreed. To reproduce: the node test above (`tsc`, `node`),
  or the Playwright spec of the reproduction against `ess generate ui --target react` (needs `npm`
  and a browser).

## Depends on

- None. It touches `fixturePlayer`, which `story:a-per-value-channel-script-that-names-no-session-is-reported`
  (item 7's story) also changes (`live.ts.tmpl:44-56`): land one after the other, or in one unit.
