---
format: aep.planning-md/3
id: story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test
kind: story
status: active
title: A program lists the ess-ui-test/1 steps a UI state offers, applies one, and forks the state, through ess_ui_test::Session
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state
- depends_on: story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in
- depends_on: story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
- depends_on: story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:19:04Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T01:19:06Z", actor: "human:timo", revision: 4}
---
## Need

Report item 1: for a UI state, a program needs the list of actions available in it and a state it
can clone. The runner already knows how to take one step in the terminal and which steps the
browser cannot take, but all of it is crate-private. Nothing lists the steps a state offers.

- The runner, `Runner` (`crates/ui/ess-ui-test/src/runner.rs:47-53`), is `pub(crate)`. It holds the
  app beside `Rc<RefCell<FixtureAdapter>>` (`:29-45`), so that `expect_command` can read the
  command log (`:1154`). It keeps `scripts` and the virtual clock `now` outside `App`.
- Steps are applied in `Runner::step` (`:105-170`). Paths are resolved and checked by
  `acting`/`resolve` (`:185-211`) against the parity rules (`parity.rs:97-306`;
  `website/docs/reference/ess-ui-test.md:412-447`).
- Every step keyword is listed in `STEPS` (`lib.rs:41-56`). There is no writer of a `Step`, only
  parsers (`spec.rs:297`, `:343`).
- The nearest list of actions is the palette: pages, `switch_to`, visible header actions and
  account actions (`crates/ui/ess-ui-tui/src/app.rs:4473-4545`, `pub(crate)`). It knows nothing of
  rows, overlays, fields or the clock.
- A model's fixture effects reach the adapter only inside `with_fixture_effects`, around the load
  (`lib.rs:216-218`, `crates/ui/ess-ui-tui/src/data.rs:100-118`, `:249`).

The caller (`g01-state-design`, Caller) is a Rust program that searches thousands to millions of
states with paths of 5 to 30 steps. It needs offered steps, apply and fork in memory, at a cost per
state far below the about 190 ms per stage it measured through the binary.

## Outcome

`ess-ui-test` exports a `Session`, as designed in `g01-state-design`.

- `Session::start(loaded: Rc<Loaded>, model: Option<&dyn ModelEnums>, base: &Path, test:
  Option<&Test>) -> Result<Session, TestError>` opens the document as `Runner::start` does
  (`runner.rs:57-102`).
  - It holds the filled document (`Loaded`, from `g03-loader-fills-defaults`).
  - It loads the adapter inside `with_fixture_effects` with the model's fixture effects, as
    `ess ui test --model` does.
  - It uses the test's fixtures, or, without a test, those the document's `fixtures:` index
    names.
  - It uses the in-memory store of `g01-app-forks`, so it needs no directory.
- `apply(&mut self, step: &Step) -> Result<(), String>` takes one step, with exactly the effect
  and the failure message that `ess ui test` gives at that step.
- `fork(&self) -> Session` copies the fixture adapter into a new `Rc`, copies `scripts` and `now`,
  and calls `App::fork_with`. The fork and its origin then share nothing mutable. They share the
  `Rc<Loaded>`.
- `offered(&self) -> Vec<Offered>` lists the steps the state offers.
  - The enum is `Offered::Step(Step)`, `Offered::Type { at }` or `Offered::Upload { at }`. The
    last two carry no value: the caller supplies the text or the file (P0-D4).
  - The list holds every step of the kinds `open`, `select`, `mark`, `type`, `choose`, `act`,
    `move`, `upload`, `page` and `advance` that `apply` takes in this state and that the parity
    rules admit in both renderers.
  - `advance` is offered to `App::next_due` (`g01-app-forks`).
  - `open` offers the navigation and `switch_to` entries, each with its own params.
  - The list is ordered by region drawing order, then by the keyword order of `STEPS`.
- `page()`, `regions()` and `screen()` (the screen text) are exposed for the caller's own
  measures.

## Acceptance

New test file `crates/ui/ess-ui-test/tests/session.rs`. Time every test: none may run over 60 s,
the crate's test step stays within 120 s, and a test over 5 s is named with its time in the
report. The file joins the crate's single test binary of
`story:the-test-step-meets-the-test-budget` if that has landed; otherwise it is a new file, which
that story moves.

The walk documents are:
- `examples/partner-portal/ui.yaml`;
- `crates/ui/ess-ui-test/tests/fixtures/fixture-command-effect/ui/desk.ui.yaml`;
- `crates/ui/ess-ui-test/tests/fixtures/adversary-t3-fixture-command-effect/ui/desk.ui.yaml`;
- `crates/ui/ess-ui-test/tests/fixtures/adversary-t3-fixture-command-effect/ui/desk-derived.ui.yaml`.

Before fixing a cap, the implementer times a walk of 100 states. Each cap is then a named const
(`WALK_CAP`), set so that the test finishes in 20 s, and never below 100 states per document. The
report records the time.

1. `every_offered_step_applies_on_a_fork`. Red today: there is no `Session`.
   - Walk breadth-first from the first page of each walk document, to depth 2, up to `WALK_CAP`
     states per document.
   - Apply each offered step to a fork, with `x` as the text of a `Type` and a fixture file of the
     document's directory as the file of an `Upload`.
   - Every step returns `Ok`, and the walk visits at least 100 states in total.
2. `the_home_page_offers_every_kind_it_shows`. On the partner portal's home page, `offered()`
   holds at least one `select`, one `act`, one `Type` and one `page` step. The counts per kind,
   taken at the first run, are pinned in the test.
3. `every_step_a_test_takes_was_offered`. For each test in `examples/partner-portal/tests/live.yaml`,
   `partners-list.yaml` and `stale.yaml`, the steps are applied one by one through
   `Session::apply`.
   - Before each step that is not an `expect`, that step is in `offered()`; a `type` or `upload`
     is matched by its target.
   - After every step, the screen text equals the runner's own run.
4. `a_step_that_is_not_offered_is_refused`. None of these is offered, and `apply` refuses each
   with the runner's message:
   - an action whose `visible` is false;
   - a field of a collapsed form group (`parity::closed_group`);
   - a path below a host slot (`parity::inside_host`);
   - `act` at a form with `save: on_change` (`parity::submit`).
5. `a_fork_moves_alone`. After `fork`, applying steps to one session leaves the other's screen
   text and `offered()` unchanged.
6. `offered_is_deterministic`. Two calls give equal lists, and two sessions started alike give
   equal lists.
7. `advance_is_offered_to_the_next_due_instant`. With a channel script whose next entry is at 2 s,
   the offered `advance` is exactly 2 s, and applying it plays that entry. With no script, no
   pending read and no poll, no `advance` is offered.
8. `the_model_reaches_the_fixtures`. `fixture-command-effect/ui/desk.ui.yaml`, started with its
   model, shows after an `act` the row the model's effect changed, as `ess ui test --model` shows.
   Started without the model, it does not.
9. `a_session_without_a_test_uses_the_documents_fixtures`. `Session::start` with `test: None` on
   the partner portal shows the rows of its `fixtures:` index.
10. `the_cost_of_check_open_step_fork_and_draw`. On `examples/partner-portal/ui.yaml`, the test
    times 200 of each and prints one line,
    `cost: check=<µs> open=<µs> step=<µs> fork=<µs> offered=<µs> draw=<µs>`, with the median of
    each:
    - check: `load_resolved`;
    - open: `Session::start`;
    - step: one `select` through `apply`;
    - fork: `Session::fork`;
    - offered: `offered()`;
    - draw: one frame.

    The first measurement on this host is pinned as a ceiling per cost, three times the measured
    median, and the report names the measured values beside the requester's (check 2.6 ms, open
    1.2 ms, draw 0.4 ms). The test fails when a median exceeds its ceiling, and it ends within
    20 s.
11. `the_stable_surface_compiles`. One function names, with its signature, as typed function
    pointers and type uses, each item of the stable list of `g01-state-design` that exists by
    round 3: `Session::start`, `apply`, `fork`, `offered`, `page`, `regions` and `screen`;
    `Offered`, `Step`, `Test` and `parse_steps`; `Loaded` and `load_resolved`. A change to a
    listed signature fails to compile. The units that add the other items add them to this
    function: `g02-document-digest` (round 5), `g01-state-document` (round 7) and `g01-state-cli`
    (round 8).

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui-test --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui-test -p ess-ui-tui --locked`. `ess ui test` output is unchanged for every
  test file of the walk documents and of the partner portal.
- `task test-xtask` (`website/` changes).

Also:
- `website/docs/reference/ess-ui-test.md` gains a short section, "From a program". It names
  `Session`, says that the offered steps are the steps both renderers take, and links the stable
  list of the design page.
- `CHANGELOG.md` gets an entry under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 1. The full review is in `g01-state-design`.

1. **Need.** The actions of a state as steps a program can take, plus a fork, in process. The
   reproduction is above: no public API lists or applies a step outside a whole test file. The
   requester measured about 190 ms per stage through the binary.
2. **Class.** It is a *gap*.
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
   - New public API in `ess-ui-test` (`Session`, `Offered`), with a stable list held by a compile
     test.
   - The runner's private structure gets a second caller, so a change to how a step is taken is
     now visible through the API.
   - No format, diagnostic or generated byte changes.
7. **Considered.**
   - (a) Offering keys from `ess-ui-tui`. Rejected: keys are not steps.
   - (b) Offering every node path from `Document::nodes`. Rejected: it offers undrawn, invisible
     and parity-refused steps, and a search would waste every one of them.
   - (c) A `test: &Test` that is required. Rejected: a document without a test file could not be
     explored, and `ess ui state` makes the test optional.
   - (d) Chosen: offered steps computed from the drawn regions and filtered by parity, over a
     session that takes the model and an optional test.

## Decisions

This story follows P0-D1 (A) and P0-D4 (A). The design page fixes the rest: that the session
holds `Rc<Loaded>`, and that `open` offers navigation plus `switch_to`.

## Scope

- `crates/ui/ess-ui-test/src/runner.rs`: `Runner` 47-216; a `fork`; `start` 57-102 takes the
  memory store, the model and an optional test.
- new: `crates/ui/ess-ui-test/src/session.rs`: `Session`, `Offered` and the offered-step
  computation, about 300 lines.
- `crates/ui/ess-ui-test/src/lib.rs`: exports, 32-36.
- read only: `screen.rs` (regions, 110-146), `parity.rs`, and `crates/ui/ess-ui-tui/src/app.rs`.
- `crates/ui/ess-ui-tui/src/app.rs`: one public accessor for the navigation targets (`App::nav`, `switch_targets` are `pub(crate)` today); the design page `docs/design/ui-state-exploration.md` names it.
- new: `crates/ui/ess-ui-test/tests/session.rs`.
- `website/docs/reference/ess-ui-test.md`.
- `CHANGELOG.md`.
- Overlap: active stories edit `runner.rs` and `parity.rs`, among them
  `story:the-ui-test-runner-acts-on-an-action-inside-an-overlay`,
  `story:the-terminal-addresses-every-node-the-browser-addresses`,
  `story:terminal-header-toggles-show-their-state` and
  `story:the-terminal-tags-input-offers-suggestions`. This unit adds a function to `runner.rs` and
  reads `parity.rs` without changing it.
- Size: L, about 570 changed lines: code about 320, tests about 250.

## Depends on

- `g01-state-design` (accepted).
- `g01-app-forks`: `App::fork_with`, `next_due`, the memory store and `FixtureAdapter: Clone`.
- `g03-loader-fills-defaults`: `Loaded`.
- `g03-actor-from-session`, which edits `ess-ui-test.md` in round 2.
- `g03-model-defaults`, which edits `crates/ui/ess-ui-test/src/lib.rs` in round 2.

Round 3, beside `g03-placement-in-loader`, with which it shares no file.
