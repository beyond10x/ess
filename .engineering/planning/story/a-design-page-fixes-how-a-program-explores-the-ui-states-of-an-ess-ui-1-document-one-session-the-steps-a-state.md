---
format: aep.planning-md/3
id: story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state
kind: story
status: draft
title: 'A design page fixes how a program explores the UI states of an ess-ui/1 document: one session, the steps a state offers, forks, and the published state ess-ui-state/1'
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A program outside ESS needs to search every UI state of a page of an `ess-ui/1` document. For each
state it needs the actions available there, and a successor per action, without replaying the whole
path from `open` again for every state. ESS offers neither.

- **No state can be forked.** `ess_ui_tui::App` (`crates/ui/ess-ui-tui/src/app.rs:433-489`) derives
  nothing. It owns `adapter: Box<dyn DataAdapter>` (`:438`) and a `StateStore` that derives only
  `Debug` (`placement.rs:179-188`). The store writes local and session storage as files under
  `Options::state_dir` (`placement.rs:335-351`, `app.rs:27-31`), so two apps over one directory
  share them. `FixtureAdapter` derives `Debug` and `Default` only (`data.rs:82-97`).
- **No list of actions.** The nearest construct is the terminal palette. It lists navigation
  pages, `switch_to` pages, visible header actions and account actions (`app.rs:4473-4545`), and it
  is `pub(crate)`. The test runner turns one step into keys (`crates/ui/ess-ui-test/src/runner.rs:105-170`,
  `Runner` at `:47-53`, `pub(crate)`). It refuses the steps the browser cannot take
  (`parity.rs:97-306`, `website/docs/reference/ess-ui-test.md:412-447`).
- **The only route to a state is replay.** An `ess-ui-test/1` test is replayed from `open`, with a
  full frame drawn after every step (`screen.rs:14-16`, `:62`). Its report holds only pass or fail
  (`ess-ui-test.md:363-385`).

The requester's words (report item 1): "For a UI state of an `ess-ui/1` document: the list of
actions available in it, and a cloneable state (or a replay-free way to reach it)."

## Outcome

`docs/design/ui-state-exploration.md` is written and accepted before any code of
`g01-session-actions` and `g01-state-document`. It fixes the following:

- **The surface.** `ess_ui_test::Session`: start, apply, fork, offered steps, state. It is a public
  Rust API of `ess-ui-test`. A program reaches it by the Git tag of an ESS release, the way generated
  terminal crates reach `ess-ui-tui` (`crates/edge/ess-cli/src/ui.rs:91-95`).
- **Offered steps.** An action is an `ess-ui-test/1` step, of the kinds `open`, `select`, `mark`,
  `type`, `choose`, `act`, `move`, `upload`, `page` and `advance`. A step is listed only where the
  terminal applies it and the parity rules admit it in both renderers. For each kind the page says
  what it is computed from: the drawn regions (`App::regions`, `app.rs:657`), the options a choice
  offers (`App::options`, `:740`), the pager, and the next instant something is due on the virtual
  clock. It also fixes the order of the list.
- **Forks.** A fork shares nothing mutable with its origin. The page says what is cloned and what
  makes a fork refuse: an HTTP adapter, and file-backed storage.
- **`ess-ui-state/1`.** Every field of `App` (`app.rs:434-489`) and of the fixture adapter is
  classified, with the reason, as one of:
  - **state**: it enters the document;
  - **cloned only**;
  - **history**: the command log, exports and counters, which stay out;
  - **drawing**: regions and marks, which stay out.

  State is keyed by node path and row key, never by an index. The canonical bytes and the digest
  follow `g02-document-digest`. The envelope is `{format, document, digest, state}`.
- **The format consequence and decisions D1 to D5,** as recorded below.

## Acceptance

- Before this story, `git grep -l "ess-ui-state/1" -- docs website crates` finds nothing. After it,
  `docs/design/ui-state-exploration.md` exists with these sections: Surface, Session, Offered
  steps, Forks, `ess-ui-state/1`, Format consequence, Out of scope, Decisions.
- In the Offered steps section, a table has one row per step kind. Each row gives the source the
  step is computed from, the parity function that filters it (`inside_host`, `unrendered`,
  `undriven`, `submit`, `closed_group`, `choose` in `parity.rs`), and its place in the order.
- In the `ess-ui-state/1` section, a table has one row per field of `App` (`app.rs:434-489`), plus
  the mutable fields of `FixtureAdapter` (`data.rs:84-96`) and `StateStore` (`placement.rs:181-187`).
  Each row is classified as state, cloned only, history or drawing, and the table leaves no field
  out.
- The page names the three tests the code stories write, each with its bound:
  - every offered step applies to a fork without error;
  - two routes to one state print equal bytes;
  - equal state digests imply equal screens and equal offered steps, also after each offered step.

  The bound is a breadth-first walk over named corpus documents, capped by a state count so that no
  test runs over 60 s and the step stays within 120 s.
- The page states the format consequence: `ess-ui-state/1` is a new published, Serialize-only
  format, and `ess-ui/1` and `ess-ui-test/1` keep their versions.
- The design, scope and acceptance plan critics return `approve` on the page.

## Fit review

Read at tree `bcfc51555` (branch `integration/wave-2026-11-24`, workspace 0.98.0). Nothing under
`crates/ui`, `schemas/ui` or `crates/edge/ess-cli/src/ui.rs` changed since `fad42d26f`. The request
came from a downstream application on 2026-11-24, report item 1.

1. **Need.** A program enumerates UI states. Reproduction: open `examples/partner-portal/ui.yaml`
   in `ess_ui_tui::App::from_path`. Nothing can produce a second app at the state the first one is
   in, short of building a new app and pressing every key again. Nothing says which steps the
   shown page offers.
2. **Class.** This is a *gap*. No page documents a fork or an action list, and `App` is documented
   only as "A running `ess-ui/1` document" (`app.rs:433`).
3. **Already expressible?** Only by replay: a state costs its path length times one full frame.
   The actions are not expressible. `Document::nodes` (`crates/ui/ess-ui/src/path.rs:108-110`)
   lists every node, but not which are drawn, visible to the actor, or refused by parity. Only
   `App::regions` and `parity.rs` know that.
4. **Fit.**
   - **Grids.** The grids cover the specification language, so none applies
     (`docs/design/language-grids.md:15`, `:61-63`).
   - **Authored keys.** There is no new key in `ess-ui/1` or `ess-ui-test/1`.
   - **Reused pieces.**
     - The step vocabulary is reused for actions, so a path a search finds is a test file that
       both renderers run.
     - Node paths and row keys identify state.
     - The digest scheme of `InfraIR` is reused (`website/docs/reference/formats.md:29-30`).
   - **Boundaries.** `crates/ui/ess-ui` keeps its single dependency, `ess-yaml` (`AGENTS.md` §
     Crate tree). The state machine stays the terminal renderer's own. It is exposed through
     `ess-ui-test`, which already owns step meaning and parity, so no second runtime appears.
   - **Targets.** React holds no in-process state that could be forked. Only steps the parity
     rules admit in both renderers are offered, so a found path also runs as a Playwright spec.
5. **Second adopter.**
   - A test-planning tool writes `ess-ui-test/1` journeys until every offered step of every page
     has been taken once.
   - An accessibility audit visits every overlay state of a document.
6. **Cost.**
   - **Format.** One new published, Serialize-only format, `ess-ui-state/1`. It needs a
     `FORMAT_RELEASES` row (`crates/edge/ess-xtask/src/docs.rs:396-399`), a row in `formats.md`
     and a golden file.
   - **Rust API.** New public API in `ess-ui-test` and `ess-ui-tui`. The crates are
     `publish = false` (`Cargo.toml:61`), so a Rust consumer reaches them only by Git tag.
   - **Coupling.** `ess-ui-state/1` follows what the terminal renderer holds as state, so a
     refactor that changes which state exists needs `/2`. That is why every field is classified
     here before any code is written.
7. **Considered.**
   - (a) Change nothing and replay. Rejected: the cost is paid per state, across every state.
   - (b) A state machine in `ess-ui`. Rejected: that crate has no renderer, may depend only on
     `ess-yaml`, and the machine would be a second runtime.
   - (c) `App::actions` returning keys. Rejected: keys exist only in the terminal, and a path of
     keys is not a test.
   - (d) A new crate, `ess-ui-explore`. This is D1 B. It adds a crate for code that needs the
     runner's internals.
   - (e) Chosen: `Session` in `ess-ui-test`.
   - From the request: the action list and the cloneable state are taken. The "replay-free way to
     reach it" is answered by an in-process fork, not by restoring from a document (D2).

## Decisions

- **D1 (P0-D1): Where does the exploration surface live?**
  - (A) `ess_ui_test::Session`, next to the steps and parity it reuses.
  - (B) A new crate, `crates/ui/ess-ui-explore`, over `ess-ui-test`.
  - (C) `ess-ui`. This breaks its dependency rule and makes it a second runtime.
  - (D) Decline. The program replays tests.

  **Recommended: (A).**
- **D2 (P0-D2): How is a state reached without replay?**
  - (A) An in-process fork. `ess-ui-state/1` describes a state and carries its digest, but nothing
    restores an app from it.
  - (B) Restore from `ess-ui-state/1`. This publishes every internal field of the terminal renderer
    as format surface, size XL.
  - (C) The document carries its step path, and a restore replays that path.

  **Recommended: (A).** The requester asked for a cloneable state *or* a replay-free route, and
  the fork is the cloneable state.
- **D3 (P0-D3): Is there a command line for it?**
  - (A) `ess ui state`, last in order (`g01-state-cli`). It replays a list of steps and prints the
    state and the steps it offers.
  - (B) No command in P0, the library only.

  **Recommended: (A).** It gives other languages and a golden test a route, and it costs M. The
  requester also states, for item 3, that the ESS crates are unpublished and that linking them is
  no alternative. For a program that cannot take a Git-tag dependency, this verb is the only
  route, so it stays in P0.
- **D4 (P0-D4): How are steps that take a free value offered?**
  - (A) By target only. `type` and `upload` are listed with the text or the file left to the
    caller.
  - (B) ESS picks representative texts, such as blank, a hit and a miss.
  - (C) They are left out.

  **Recommended: (A).** Choosing values is the search's policy, not ESS's.
- **D5 (P0-D5): Which part of the clock is part of a state's identity?**
  - (A) The absolute virtual time and each script's position.
  - (B) Only the durations until each pending event is due. Two states that differ only by whole
    cycles of a looping script would then be one state.

  **Recommended: (A),** which is simple and exact. (B) is the follow-up if a looping script makes
  a walk endless.

## Scope

- new: `docs/design/ui-state-exploration.md`
- read: `crates/ui/ess-ui-tui/src/app.rs` (434-489, 600-800, 4473-4545), `placement.rs` (179-352),
  `data.rs` (53-97), `live.rs` (93-140); `crates/ui/ess-ui-test/src/runner.rs` (29-216),
  `parity.rs`, `screen.rs`, `spec.rs` (58-140), `lib.rs` (36-56)
- Size: S, about 250 lines of prose and no code.

## Depends on

Nothing. `g01-session-actions` and `g01-state-document` depend on this story.
