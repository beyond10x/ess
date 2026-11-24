---
format: aep.planning-md/3
id: story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state
kind: story
status: active
title: 'A design page fixes how a program explores the UI states of an ess-ui/1 document: one session, the steps a state offers, forks, and the published state ess-ui-state/1'
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:18:50Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T01:18:52Z", actor: "human:timo", revision: 4}
---
## Need

A program outside ESS needs to search every UI state of a page of an `ess-ui/1` document. For each
state it needs the actions available there, and a successor per action, without replaying the whole
path from `open` again for every state. ESS offers neither.

- **No state can be forked.**
  - `ess_ui_tui::App` (`crates/ui/ess-ui-tui/src/app.rs:433-489`) derives nothing. It owns
    `doc: Document` by value (`:435`) and `adapter: Box<dyn DataAdapter>` (`:438`).
  - Its `StateStore` derives only `Debug` (`placement.rs:179-188`). The store writes local and
    session storage as files under `Options::state_dir` (`placement.rs:335-351`, `app.rs:27-31`),
    so two apps over one directory share them.
  - `FixtureAdapter` derives only `Debug` and `Default` (`data.rs:82-97`).
- **No list of actions.**
  - The nearest construct is the terminal palette. It lists navigation pages, `switch_to` pages,
    visible header actions and account actions (`app.rs:4473-4545`), and it is `pub(crate)`.
  - The test runner turns one step into keys (`crates/ui/ess-ui-test/src/runner.rs:105-170`).
    `Runner`, at `:47-53`, is `pub(crate)`.
  - The runner refuses the steps the browser cannot take (`parity.rs:97-306`,
    `website/docs/reference/ess-ui-test.md:412-447`).
- **The only route to a state is replay.** An `ess-ui-test/1` test is replayed from `open`, with a
  full frame drawn after every step (`screen.rs:14-16`, `:62`). Its report holds only pass or fail
  (`ess-ui-test.md:363-385`).

The requester's words (report item 1): "For a UI state of an `ess-ui/1` document: the list of
actions available in it, and a cloneable state (or a replay-free way to reach it)."

**The caller, settled by the requester on 2026-11-24.**
- It is a Rust program that takes ESS as a Git dependency at a tag. It needs the forkable session
  in memory (offered steps, apply, fork) and the `ess-ui-state/1` digest to de-duplicate states.
- Paths are 5 to 30 steps long. A journey has thousands to low millions of states.
- The cost per state decides their design. Replay per call is quadratic and unusable for search.
- `ess ui state --steps`, which replays, is an inspection aid only. No long-running process is
  wanted.
- Measured by the requester: one variant's first stage costs about 190 ms through the `ess` binary
  (check and test as processes), against about 5 ms in process over the crates (check 2.6 ms,
  open 1.2 ms, draw 0.4 ms). So the in-process surface is required.

## Outcome

`docs/design/ui-state-exploration.md` is written and accepted before any code of
`g01-session-actions` and `g01-state-document`. It fixes the following.

- **The surface.** `ess_ui_test::Session`: start, apply, fork, offered steps and state. It is a
  public Rust API of `ess-ui-test`, which a program reaches by the Git tag of an ESS release, the
  way generated terminal crates reach `ess-ui-tui` (`crates/edge/ess-cli/src/ui.rs:91-95`).
- **What a program at a tag may rely on across releases.** The page lists these items:
  - `Session` (`start`, `apply`, `fork`, `offered`, `state`, `page`, `regions`, `screen`);
  - `Offered`, `Step`, `Test`, `parse_steps`, `write_step`, `parse_offers` and `UiState`;
  - `ess_ui::Loaded`, `load_resolved` and `canonical_json`.

  A change to the signature or meaning of any listed item is named in `CHANGELOG.md` and in the
  release notes as `**Breaking**`. Every other public item of these crates may change in any
  release. `g01-session-actions` holds the list in a compile test, `the_stable_surface_compiles`.
  The test grows by unit: it starts with the items that exist in round 3, and
  `g02-document-digest` (round 5), `g01-state-document` (round 7) and `g01-state-cli` (round 8)
  each add theirs.
- **The session holds the filled document and takes the model.**
  - `Session::start(loaded: Rc<Loaded>, model: Option<&dyn ModelEnums>, base: &Path, test:
    Option<&Test>)`.
  - The model's fixture effects reach the adapter as they do in `ess ui test --model`
    (`crates/ui/ess-ui-test/src/lib.rs:216-218`).
  - Without a test, the fixtures are those the document's `fixtures:` index names.
  - The state document takes its document digest from `loaded`.
- **Offered steps.**
  - An action is an `ess-ui-test/1` step, of the kinds `open`, `select`, `mark`, `type`,
    `choose`, `act`, `move`, `upload`, `page` and `advance`.
  - A step is offered only where the terminal applies it and the parity rules admit it in both
    renderers.
  - For each kind, the page says what it is computed from: the drawn regions (`App::regions`,
    `app.rs:657`), the options a choice offers (`App::options`, `:740`), the pager, or the next
    instant something is due on the virtual clock (`App::next_due`, from `g01-app-forks`).
  - The page fixes the order of the list.
- **Forks.**
  - A fork shares nothing mutable with its origin.
  - What a fork shares by reference count: the document (`App::doc` becomes `Rc<Document>`), the
    plan and the options.
  - What a fork copies: every other field of `App`, the fixture adapter's rows and command log,
    the in-memory storage maps, and the runner's `scripts` and `now`.
  - So the cost of a fork grows with the state held, not with the document.
  - A fork refuses an HTTP adapter and file-backed storage, naming why.
- **`ess-ui-state/1`.** Every field is classified, with its reason, as one of:
  - **state**: it enters the document;
  - **cloned only**;
  - **history**: the command log, exports and counters, which stay out;
  - **drawing**: regions and marks, which stay out.

  The fields come from `App` (`app.rs:434-489`), the fixture adapter, the store, and `Runner`
  (`runner.rs:47-53`: `scripts`, `now`). State is keyed by node path and row key, never by an
  index. The canonical bytes and the digest follow `g02-document-digest`. The envelope is
  `{format, document, model, digest, state}`, and `ess ui state` adds the optional member
  `offers`.
- **The read cache is state, every entry of it.** `App::cache` keeps the answer of every read
  taken on the page, on screen or not (`app.rs:1385-1395`), and drops one only on a page change
  (`:2769-2773`) or `invalidate` (`:1537-1544`). A cached answer decides what a later step shows,
  with its `answered` time, which stale marks read. So every entry is state, keyed by read
  request (view, parameters, page), never by screen position. This makes "equal digests imply
  equal screens and offered steps after each step" hold. Its price: paging to page 2 and back is
  a different state from staying on page 1, because the page-2 entry stays.
- **The format consequence and decisions D1 to D5,** as recorded below.

## Acceptance

- Before this story, `git grep -l "ess-ui-state/1" -- docs website crates` finds nothing. After
  it, `docs/design/ui-state-exploration.md` exists with these sections: Caller, Surface, Stable
  surface, Session, Offered steps, Forks, `ess-ui-state/1`, Cost, Format consequence, Out of
  scope, Decisions.
- **Offered steps:** a table has one row per step kind. Each row gives the source the step is
  computed from, the parity function that filters it (`inside_host`, `unrendered`, `undriven`,
  `submit`, `closed_group`, `choose` in `parity.rs`), and its place in the order.
- **`ess-ui-state/1`:** a table has one row per field of `App` (`app.rs:434-489`), of
  `FixtureAdapter` (`data.rs:84-96`), of `StateStore` (`placement.rs:181-187`) and of `Runner`
  (`runner.rs:47-53`). The table leaves no field out. The memory-mode fields that `g01-app-forks`
  adds to `StateStore` are classified by reference to that story, so the page can be checked
  whichever of the two lands first. Each row is classified as state, cloned only, history or
  drawing. The `cache` row says that every entry is state, keyed by read request, including the
  entries of reads no longer on screen.
- **Forks:** the section names, field by field, what a fork shares and what it copies, as listed
  above.
- **Cost:** the section names the six costs that `g01-app-forks`, `g01-session-actions` and
  `g01-state-document` measure: check, open, step, fork, digest and draw. It names the document
  they are measured on, `examples/partner-portal/ui.yaml`, and quotes the requester's measurement.
- **Out of scope** names:
  - restoring an app from `ess-ui-state/1`;
  - forks over an HTTP adapter or file storage;
  - steps only one renderer takes;
  - a replay-free route for a program that does not link the crates. The caller is a Rust program
    that takes ESS as a Git dependency at a tag and needs the forkable session in memory (offered
    steps, apply, fork) plus the `ess-ui-state/1` digest to de-duplicate states. `ess ui state
    --steps` replays and is an inspection aid only, and no long-running process is wanted.
- The page names the three tests the code stories write, each with its bound:
  - every offered step applies to a fork without error;
  - two routes to one state print equal bytes;
  - equal state digests imply equal screens and equal offered steps, also after each offered step.

  Each bound is a breadth-first walk capped by a named state count, set from a timed run so that
  no test runs over 60 s and the step stays within 120 s.
- The page states the format consequence: `ess-ui-state/1` is a new published, Serialize-only
  format, and `ess-ui/1` and `ess-ui-test/1` keep their versions.
- `task test-xtask` passes with the page in the tree, and the Decisions section records the
  answers to P0-D1 to P0-D5.

Gate: `task test-xtask` (`docs/` changes).

## Fit review

Read at tree `bcfc51555` (branch `integration/wave-2026-11-24`, workspace 0.98.0). Nothing under
`crates/ui`, `schemas/ui` or `crates/edge/ess-cli/src/ui.rs` changed since `fad42d26f`. The request
came from a downstream application on 2026-11-24, report item 1.

1. **Need.** A program enumerates UI states.
   - Reproduction: open `examples/partner-portal/ui.yaml` in `ess_ui_tui::App::from_path`.
     Nothing can produce a second app at the state the first one is in, short of building a new
     app and pressing every key again. Nothing says which steps the shown page offers.
   - Evidence: about 190 ms per stage through the binary against about 5 ms in process, and
     journeys of thousands to millions of states.
2. **Class.** It is a *gap*. No page documents a fork or an action list, and `App` is documented
   only as "A running `ess-ui/1` document" (`app.rs:433`).
3. **Already expressible?** Only by replay: a state costs its path length times one full frame.
   The actions are not expressible. `Document::nodes` (`crates/ui/ess-ui/src/path.rs:108-110`)
   lists every node, but not which are drawn, visible to the actor, or refused by parity. Only
   `App::regions` and `parity.rs` know that.
4. **Fit.**
   - **Grids.** The grids cover the specification language, so none applies
     (`docs/design/language-grids.md:15`, `:61-63`).
   - **Authored keys.** No new key in `ess-ui/1` or `ess-ui-test/1`.
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
     and golden files.
   - **Rust API.** New public API in `ess-ui-test` and `ess-ui-tui`. The crates are
     `publish = false` (`Cargo.toml:61`), so a Rust consumer reaches them only by Git tag. The
     stable list above says which items it may rely on.
   - **Coupling.** `ess-ui-state/1` follows what the terminal renderer holds as state, so a
     refactor that changes which state exists needs `/2`. That is why every field is classified
     here before any code is written.
7. **Considered.**
   - (a) Change nothing and replay. Rejected: the cost is paid per state, across every state, and
     the requester measured it.
   - (b) A state machine in `ess-ui`. Rejected: that crate has no renderer, may depend only on
     `ess-yaml`, and the machine would be a second runtime.
   - (c) `App::actions` returning keys. Rejected: keys exist only in the terminal, and a path of
     keys is not a test.
   - (d) A new crate, `ess-ui-explore` (D1 B). It adds a crate for code that needs the runner's
     internals.
   - (e) Chosen: `Session` in `ess-ui-test`.
   - From the request: the action list and the cloneable state are taken. The "replay-free way to
     reach it" is answered by an in-process fork, not by restoring from a document (D2). That fits
     the caller, which links the crates by tag.

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
  - (B) Restore from `ess-ui-state/1`. This publishes every internal field of the terminal
    renderer as format surface, size XL.
  - (C) The document carries its step path, and a restore replays that path.

  **Recommended: (A).** The requester's caller links the crates and wants the fork in memory.
- **D3 (P0-D3): Is there a command line for it?**
  - (A) `ess ui state`, last in order (`g01-state-cli`). It replays a list of steps and prints the
    state and the steps it offers. It is an inspection aid and a route for golden tests, not the
    search path.
  - (B) No command in P0, the library only.

  **Recommended: (A).** The requester names `ess ui state --steps` as an inspection aid, and it
  costs M.
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

- new: `docs/design/ui-state-exploration.md`.
- read:
  - `crates/ui/ess-ui-tui/src/app.rs` (434-489, 600-800, 4473-4545), `placement.rs` (179-352),
    `data.rs` (53-97), `live.rs` (93-140);
  - `crates/ui/ess-ui-test/src/runner.rs` (29-216), `parity.rs`, `screen.rs`, `spec.rs`
    (58-140), `lib.rs` (36-56, 205-225).
- Size: S, about 290 lines of prose and no code.

## Depends on

Nothing. Round 1, beside `g03-loader-fills-defaults` and `g01-app-forks`. It classifies the
memory-mode fields of `g01-app-forks` by reference, so either may land first.
`g01-session-actions` and `g01-state-document` depend on this story.
