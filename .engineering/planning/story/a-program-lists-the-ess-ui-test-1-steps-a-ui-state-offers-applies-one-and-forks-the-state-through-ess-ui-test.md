---
format: aep.planning-md/3
id: story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test
kind: story
status: draft
title: A program lists the ess-ui-test/1 steps a UI state offers, applies one, and forks the state, through ess_ui_test::Session
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state
- depends_on: story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in
revision: 1
---
## Need

Report item 1: for a UI state, a program needs the list of actions available in it and a state it
can clone. The runner already knows how to take one step in the terminal and which steps the
browser cannot take, but all of it is crate-private. Nothing lists the steps a state offers.

- The runner, `Runner` (`crates/ui/ess-ui-test/src/runner.rs:47-53`), is `pub(crate)`. It holds
  the app beside `Rc<RefCell<FixtureAdapter>>` (`:29-45`), so that `expect_command` can read the
  command log (`:1154`).
- Steps are applied in `Runner::step` (`:105-170`). Paths are resolved and checked by
  `acting`/`resolve` (`:185-211`) against the parity rules
  (`parity.rs:97-306`; `website/docs/reference/ess-ui-test.md:412-447`).
- Every step keyword is listed in `STEPS` (`lib.rs:41-56`). There is no writer of a `Step`, only
  parsers (`spec.rs:297`, `:343`).
- The nearest list of actions is the palette: pages, `switch_to`, visible header actions and
  account actions (`crates/ui/ess-ui-tui/src/app.rs:4473-4545`, `pub(crate)`). It knows nothing of
  rows, overlays, fields or the clock.
- The terminal's player keeps when each script entry is due (`live.rs:93-140`), but `App` does not
  say when the next thing is due.

## Outcome

`ess-ui-test` exports a `Session`, the design of `g01-state-design`.

- `Session::start(document: &Document, base: &Path, test: &Test) -> Result<Session, TestError>`
  opens the document with the test's fixtures, as `Runner::start` does (`runner.rs:57-102`). It
  uses the in-memory store from `g01-app-forks`, so it needs no directory.
- `apply(&mut self, step: &Step) -> Result<(), String>` takes one step with exactly the effect and
  the failure message `ess ui test` gives at that step.
- `fork(&self) -> Session` copies the fixture adapter into a new `Rc` and calls
  `App::fork_with` with it. The fork and its origin then share nothing mutable.
- `offered(&self) -> Vec<Offered>` lists the steps the state offers.
  - The enum is `Offered::Step(Step)`, `Offered::Type { at }` or `Offered::Upload { at }`. The last
    two carry no value; the caller supplies the text or the file (P0-D4).
  - The list holds every step of the kinds `open`, `select`, `mark`, `type`, `choose`, `act`,
    `move`, `upload`, `page` and `advance` that `apply` takes in this state and that the parity
    rules admit in both renderers.
  - The list is ordered by region drawing order, then by the keyword order of `STEPS`.
- `page()`, `regions()` and the screen text of the state are exposed for the caller's own
  measures.
- `ess_ui_tui::App::next_due(&self) -> Option<Duration>` gives the next instant at which a script
  entry plays, a read is answered, a poll runs or a coalesced batch is delivered. An `advance`
  step is offered to that instant.

## Acceptance

New test file `crates/ui/ess-ui-test/tests/session.rs`. The walks are bounded by a state count
chosen so each test runs under 30 s. Time every test: none may run over 60 s, and the file's step
stays within 120 s. A test over 5 s is named with its time in the report.

1. `every_offered_step_applies_on_a_fork`. Red today: there is no `Session`.
   - Walk breadth-first from the home page of `examples/partner-portal/ui.yaml`, and from the
     first page of each of three documents of `crates/ui/ess-ui-test/tests/`, to depth 2. Stop at
     500 states per document.
   - Apply each offered step to a fork, with `x` as the text of a `Type` and a fixture file of the
     test as the file of an `Upload`.
   - Every step returns `Ok`.
2. `a_step_that_is_not_offered_is_refused`. None of these is offered, and `apply` refuses each
   with the runner's message:
   - an action whose `visible` is false;
   - a field of a collapsed form group (`parity::closed_group`);
   - a path below a host slot (`parity::inside_host`);
   - `act` at a form with `save: on_change` (`parity::submit`).
3. `apply_reaches_what_ess_ui_test_reaches`. For each test in the `ess-ui-test` corpus files named
   in the test, applying its steps one by one through `Session::apply` gives, after every step,
   the screen text the runner's own run shows.
4. `a_fork_moves_alone`. After `fork`, applying steps to one session leaves the other's screen
   text and `offered()` unchanged.
5. `offered_is_deterministic`. Two calls give equal lists, and two sessions started alike give
   equal lists.
6. `advance_is_offered_to_the_next_due_instant`. With a channel script whose next entry is at 2 s,
   the offered `advance` is exactly 2 s. Applying it plays that entry. With no script, no pending
   read and no poll, no `advance` is offered.

Also:

- `cargo clippy` and `cargo test` pass for `-p ess-ui-test` and `-p ess-ui-tui`, and `ess ui test`
  output is unchanged for every corpus file.
- `website/docs/reference/ess-ui-test.md` gains a short section, "From a program", naming
  `Session` and saying that the offered steps are the steps both renderers take.
  `CHANGELOG.md` gets an entry under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 1. The full review is in `g01-state-design`.

1. **Need.** The actions of a state, as steps a program can take, plus a fork. The reproduction is
   above: no public API lists or applies a step outside a whole test file.
2. **Class.** This is a *gap*.
3. **Already expressible?** No. A program can write a one-step test file per candidate and run
   `ess ui test`. That replays from `open` each time, and the program still has to guess the
   candidates.
4. **Fit.** It reuses three things and adds no new vocabulary:
   - the `ess-ui-test/1` steps as the action alphabet;
   - the parity rules, so a path the search finds runs in both renderers;
   - the regions the terminal already records.
   No format moves.
5. **Second adopter.** A coverage tool counts, for every page, the offered steps that no committed
   test takes.
6. **Cost.**
   - New public API in `ess-ui-test` (`Session`, `Offered`) and in `ess-ui-tui` (`next_due`).
   - The runner's private structure gets a second caller, so a change to how a step is taken is
     now visible through the API.
   - No format, diagnostic or generated byte changes.
7. **Considered.**
   - (a) Offering keys from `ess-ui-tui`. Rejected: keys are not steps.
   - (b) Offering every node path from `Document::nodes`. Rejected: it offers undrawn, invisible
     and parity-refused steps, and a search would waste every one of them.
   - (c) Chosen: offered steps computed from the drawn regions and filtered by parity.

## Decisions

This story follows P0-D1 (A) and P0-D4 (A). Two questions are left to the implementer, within the
design page:

- whether `Session` borrows the document or holds its own copy (the runner borrows it,
  `runner.rs:48`);
- whether `open` offers every page, the way a typed URL can, or only the pages the navigation and
  `switch_to` list. The design page decides this; the default is navigation plus `switch_to`, with
  each entry's own params.

## Scope

- cited: `crates/ui/ess-ui-test/src/runner.rs` (`Runner` 47-216; a `fork`; `start` 57-102 takes
  the memory store)
- new: `crates/ui/ess-ui-test/src/session.rs` (`Session`, `Offered`, the offered-step computation,
  about 300 lines)
- cited: `crates/ui/ess-ui-test/src/lib.rs` (exports, 32-36), `screen.rs` (regions, 110-146),
  `parity.rs` (the filters, read only)
- cited: `crates/ui/ess-ui-tui/src/app.rs` (a `next_due` beside `advance`, 798), `live.rs` (a
  `Player::next_at`, 93-140)
- new: `crates/ui/ess-ui-test/tests/session.rs`
- `website/docs/reference/ess-ui-test.md`, `CHANGELOG.md`
- Overlap: active stories edit `runner.rs` and `parity.rs`, among them
  `story:the-ui-test-runner-acts-on-an-action-inside-an-overlay`,
  `story:the-terminal-addresses-every-node-the-browser-addresses`,
  `story:terminal-header-toggles-show-their-state` and
  `story:the-terminal-tags-input-offers-suggestions`. This unit adds a function to `runner.rs` and
  reads `parity.rs` without changing it.
- Size: L, about 550 changed lines (code about 330, tests about 220).

## Depends on

- `g01-state-design` (accepted).
- `g01-app-forks` (`App::fork_with`, memory store, `FixtureAdapter: Clone`).
