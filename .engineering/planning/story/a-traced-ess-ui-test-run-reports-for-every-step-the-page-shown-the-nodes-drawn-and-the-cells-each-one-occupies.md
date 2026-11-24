---
format: aep.planning-md/3
id: story:a-traced-ess-ui-test-run-reports-for-every-step-the-page-shown-the-nodes-drawn-and-the-cells-each-one-occupies
kind: story
status: active
title: A traced `ess ui test` run reports, for every step, the page shown, the nodes drawn and the cells each one occupies
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-per-value-channel-script-that-names-no-session-is-reported
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T00:39:02Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T00:39:04Z", actor: "human:timo", revision: 4}
---
## Need

A tool that uses `ess ui test` as its evidence gets one record per test and nothing about the steps
inside it. `ess-ui-test-report/1` is `{format, document, tests}`, each test
`{name, status, step?, message?}` (`crates/ui/ess-ui-test/src/lib.rs:83-106`,
`website/docs/reference/ess-ui-test.md:363-384`, `website/docs/reference/formats.md:221`). A passed
test says nothing about what it reached: which page, which nodes were on screen, where they were
drawn.

The terminal renderer already knows all three after every frame:

- the page shown (`Runner::current_page`, used at `crates/ui/ess-ui-test/src/lib.rs:274`);
- every node drawn on cells of its own, by canonical node path, with its row key and its
  `Rect` (`ess_ui_tui::Region`, `crates/ui/ess-ui-tui/src/app.rs:55-74`, `App::regions` at
  `:655-659`);
- the run draws a frame only where a step needs the screen (`Runner::screen`,
  `crates/ui/ess-ui-test/src/runner.rs:213-215`, a 160 x 240 test backend,
  `crates/ui/ess-ui-test/src/screen.rs:14-16`).

Minimal reproduction, any example: `ess ui test --path examples/partner-portal/ui.yaml <tests>
--format json` prints `{"name": …, "status": "passed"}` for each test and no step.

The requester proposed, in their words: "an `ess-ui-test` report with one record per step: the
state reached, the nodes shown, the geometry read."

## Outcome

`ess ui test --format json --trace` writes, in each test of `ess-ui-test-report/1`, an optional
`steps` list: one record per step that ran, in order, the failing step last. Each record names the
step (number from 1 and keyword), the page shown after it, the open overlay if any, and the nodes
the frame after it drew, each with its path, its row key where it has one, and its cells
`{x, y, width, height}`. Without `--trace` the report is byte for byte what it is today. The
"state reached" joins the record in `g04-step-trace-state`, once the UI state form of report
items 1 and 2 exists.

## Acceptance

- First: a case in `crates/ui/ess-ui-test/tests/cli.rs` runs `examples/partner-portal/ui.yaml` with one
  three-step test (open a page, select a row, expect a text) under `--trace` and asserts three
  records with `step` 1, 2, 3, `keyword` `open`, `select`, `expect`, the page name, and a node list
  that holds the selected row's path with its row key and a non-empty area. It fails today: the
  flag does not exist.
- A test that fails at step 2 of 3 has two records; the second is the failing step, drawn after
  the step's attempt.
- A test whose setup fails has `steps: []`.
- Without `--trace`, every committed JSON report fixture and the bytes of the reference example
  (`ess-ui-test.md:367-381`) are unchanged; a test pins this on one passing and one failing test.
- The node list of a record is exactly `App::regions` of the frame drawn after the step, in drawing
  order, with no node the frame did not draw. The reference says that a node drawn without cells
  of its own (cards, a list or a tree with an `item`: `app.rs:55-60`) is not listed.
- The area is in terminal cells of the run's backend, and the reference says so. The report carries
  the backend size once (`terminal: {width, height}`).
- `--trace` with `--format text` or with `--playwright` is refused with one line naming
  `--format json`.
- `website/docs/reference/ess-ui-test.md` § The report documents `steps` and `terminal`;
  `formats.md:221` adds "From <version> an optional `steps` and `terminal`, written only with
  `--trace`"; `CHANGELOG.md` names it.
- An older reader: a test deserialises a traced report with a reader that knows only `{format,
  document, tests}` per report and `{name, status, step, message}` per test and ignores the rest,
  and a test shows the untraced bytes did not move. No `ess-ui-test-report/2`.
- With `--trace`, every test's `status`, `step` and `message` equal those of the untraced run of
  the same file: the extra draw per step moves no verdict. A case runs the partner-portal tests
  both ways and compares.

## Fit review

Tree `bcfc51555` (`integration/wave-2026-11-24`, workspace 0.98.0 and later commits), read-only.

1. **Need.** A run's evidence names only the failing step; a tool that counts what a suite reached
   (pages, nodes, geometry at each step) has nothing to read. Reproduction above. The requester's
   proposal is quoted under Need. Report item 4.
2. **Class.** *Gap*. Nothing in the report or the CLI writes per-step data (`lib.rs:83-106`,
   `TestArgs` at `:119-144`). Not a defect: the reference promises one record per test.
3. **Already expressible?** No. A test can assert a node's text or rows (`spec.rs:195-208`), not
   read its place. `ess-ui-tui`'s `App::regions` is a Rust API, reachable only from Rust code that
   links the renderer, with no format.
4. **Fit.** The language grids do not cover `ess-ui-test` (`docs/design/language-grids.md:15`).
   - Vocabulary: `step` already means "the failing step, from 1" in the report; a record reuses it.
     Node paths are the canonical paths every UI tool reports (`ess-ui-test.md:59-63`). The area
     reuses `Region`'s fields.
   - Siblings: the Playwright spec writes no report (`ess-ui-test.md:450-453`); the trace is the
     terminal runner's, and the reference says so. A browser trace is a separate request if a
     second need appears.
   - Composes with every step: a record is written after each step, whatever its keyword.
   - The state form is not designed here: "state reached" is the serialised UI state of report
     items 1 and 2 (P0 group). `g04-step-trace-state` adds it; this story leaves room for one
     `state` key and adds nothing that a second state form could fill.
5. **Second adopter.** A team gates merges on "every page of the document is opened by some test"
   and "no row action is untested": it counts distinct paths over all records. A documentation
   generator places a callout on the cells of the node a tutorial step acts on.
6. **Cost.** One CLI flag and two optional keys in `ess-ui-test-report/1`, written only with the
   flag, so an untraced report keeps its bytes and its meaning. Precedent: `ess-ui-check/1` took
   optional `backlog` (0.90.0) and `hosts` and `coverage` (0.91.1) inside `/1`, each left out when
   empty so existing output did not move (`formats.md:219`). No new diagnostic, no `ess-ui/1` or
   `ess-ui-test/1` change. Run time: one extra draw per step; the runner already draws on every
   `expect`.
7. **Considered.**
   - (a) Change nothing: refused, no idiom (question 3).
   - (b) A separate format, `ess-ui-test-trace/1`, written to a file: keeps the report small, but
     adds a published format for one flag's output. Kept as option B of decision D1.
   - (c) `ess-ui-test-report/2` with steps always: every reader and every report changes for a need
     only tracing runs have. Refused.
   - (d) Chosen: optional `steps` behind `--trace`. The requester's three parts are kept; "state
     reached" is deferred to the state form of items 1 and 2 rather than invented here.

## Decisions

**Accept, redesigned.** What changed from the request: the trace is opt-in, terminal-only and in
cells; "the state reached" waits for the P0 state form (follow-up story).

- **P1-D1. Where the trace goes.** (A) Optional `steps` per test and `terminal` in
  `ess-ui-test-report/1`, written only with `--trace`. (B) A new format `ess-ui-test-trace/1`
  written to `--trace <file>`, report unchanged. (C) `ess-ui-test-report/2`, steps always.
  **Recommended: (A).** Same precedent as `ess-ui-check/1`; one document per run.
- **P1-D2. What "the state reached" holds** (decided here, built in `g04-step-trace-state`). (A)
  The digest of the serialised UI state of items 1 and 2, the full state only with `--trace full`.
  (B) The full state in every record. (C) Nothing; the application replays. **Recommended: (A).**

## Scope

- cited: `crates/ui/ess-ui-test/src/lib.rs` (`Outcome`, `Report`, `TestArgs`, `run_test`,
  `run_to_with`)
- cited: `crates/ui/ess-ui-test/src/runner.rs` (`step`, `screen`, `current_page`)
- cited: `crates/ui/ess-ui-test/src/screen.rs` (`Screen::capture`, regions kept per frame)
- inferred: `crates/edge/ess-cli/src/ui.rs` (the flag reaches `TestArgs` through clap)
- inferred: cases in `crates/ui/ess-ui-test/tests/cli.rs` (no new test binary)
- cited: `website/docs/reference/ess-ui-test.md` § The report; `website/docs/reference/formats.md:221`;
  `CHANGELOG.md`
- Overlap: `story:the-terminal-addresses-every-node-the-browser-addresses` (active) changes which
  regions the terminal records; land either first, the trace lists whatever `regions` holds.
- Size: M (about 350 changed lines).

## Depends on

- Lands after `g07-session-script` (shares `runner.rs` and `ess-ui-test.md`; g07 changes `play`
  and § One verdict, this story `step`, `screen` and § The report). `g04-step-trace-state` depends
  on this story and on the UI state form of report
  items 1 and 2 (P0 group).
