---
format: aep.planning-md/3
id: story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the
kind: story
status: active
title: A UI state prints as ess-ui-state/1 with a digest, and two routes to one state print the same bytes and the same digest
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state
- depends_on: story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test
- depends_on: story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling
- depends_on: story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:19:18Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T01:19:20Z", actor: "human:timo", revision: 4}
---
## Need

Report item 2 asks for "a normative canonical byte form and digest … of a UI state". Report item 1
asks for a state that can be told apart from another without replay. A program that walks UI states
merges two routes that reach one state, and caches what it computed per state under the state's
digest. The requester's caller de-duplicates thousands to millions of states per journey by this
digest. Today a UI state exists only as private fields of `ess_ui_tui::App`
(`crates/ui/ess-ui-tui/src/app.rs:434-489`), of the fixture adapter (`data.rs:84-96`) and of the
runner (`crates/ui/ess-ui-test/src/runner.rs:47-53`).

- Nothing prints a state. The `ess-ui-test-report/1` report holds only pass or fail
  (`website/docs/reference/ess-ui-test.md:363-385`).
- No UI crate computes a digest of a state.
- Some fields are history, not state, and must stay out of a state's identity: `commands_sent`
  (`app.rs:479`), `exports` (`:481`), the cache's `generations` (`:468`) and the adapter's command
  log (`data.rs:96`).
- Others are indexes that differ between two routes to one state: the per-view `cursor`
  (`app.rs:319`) and `item` (`:326`) hold a position, not a row key.

## Outcome

- `Session::state(&self) -> UiState` (`ess-ui-test`) builds the state from three sources: a new
  `ess_ui_tui::App::snapshot()`, the fixture adapter's data, and the session's `scripts` and
  `now`. It prints as one JSON object `{format: "ess-ui-state/1", document, model, digest, state}`:
  - `document`: the session's `Loaded::digest` (`g02-document-digest`). The session holds
    `Rc<Loaded>` (`g01-session-actions`), so the digest comes from the filled tree, not from the
    typed `Document`.
  - `model`: `true` when the session was started with a model, because the document digest
    differs with one.
  - `state`: the fields that the design page (`g01-state-design`) classifies as state.
  - `digest`: the digest of `state` alone, by the scheme of `g02-document-digest`, through
    `ess_ui::canonical_json`. It leaves the envelope out, as the `InfraIR` digest does
    (`website/docs/reference/formats.md:29`). So two documents can share a state digest, and a
    caller that mixes documents keys by `document` and `digest` together.
- Rows and positions are named by node path and row key, never by an index.
- History (the command log, exports, counters) and drawing (regions, marks) stay out.
- `snapshot` destructures `App`, `FixtureAdapter` and `StateStore` without `..`. A field added
  later fails to compile until it is classified.
- The format is versioned, Serialize-only, and has a row in the formats reference. `offers`, which
  `g01-state-cli` adds in the same release, is an optional member outside the digest. That story
  records it in the row and in an old-reader case.

## Acceptance

New test file `crates/ui/ess-ui-test/tests/state_document.rs`. Time every test: none may run over
60 s, the crate's test step stays within 120 s, and a test over 5 s is named with its time in the
report. Before fixing a cap, the implementer times a walk of 100 states. The cap is then a named
const set so that the test finishes in 20 s, and never below 200 states; the report records the
time. The file joins the crate's single test binary of
`story:the-test-step-meets-the-test-budget` if that has landed; otherwise it is a new file, which
that story moves.

1. `two_routes_to_one_state_print_the_same_bytes`. Red today: there is no `Session::state`. On
   `examples/partner-portal/ui.yaml`, each pair below ends in byte-equal documents with equal
   digests. Every pair takes no read on either route, and the test first asserts that the two
   routes' read logs are equal, so a pair that reads fails as a wrong pair, not as a digest:
   - selecting the second row of the partner list directly, and selecting the first row and then
     the second;
   - opening the partner list's `delete` confirm dialog (`ui.yaml:142`, which reads nothing) and
     cancelling it, and never opening it, with the back stack equal in both routes.

   And one pair that must differ: paging to page 2 and back to page 1 gives a different digest
   from staying on page 1, because the page-2 entry of the read cache is state (`g01-state-design`,
   every cache entry is state, keyed by read request).
2. `equal_digests_mean_equal_behaviour`.
   - Walk breadth-first from the home page to depth 3, up to the cap.
   - The walk finds at least 2 digests shared by two or more states, among them the pairs of
     test 1. The count found at the first run is pinned.
   - For every two states with one digest: their screen text, their regions and their offered
     steps are equal.
   - After applying each offered step to a fork of each, the two digests are again equal.
3. `a_different_state_has_a_different_digest`. A marked row, a typed filter text, a moved clock
   and an open overlay each change the digest of the state they are applied to.
4. `the_state_prints_its_golden_bytes`. The home page state of `examples/partner-portal/ui.yaml`
   prints exactly the live golden
   `crates/ui/ess-ui-test/tests/golden/partner-portal-home.state.json`.
   - Printing twice gives equal bytes.
   - The bytes hold no absolute path and no time of day.
   - The file is written by `ESS_UI_GOLDEN=write cargo test -p ess-ui-test --test state_document`,
     and a later story that moves it names that command in its Scope.
5. `the_digest_is_recomputable`. The test recomputes the digest from the golden file's `state`
   member with `sha2` and `serde_json` alone, sorting keys itself. The result equals the printed
   `digest`.
6. `an_old_reader_still_reads`. The frozen golden
   `crates/ui/ess-ui-test/tests/golden/partner-portal-home.state.first-release.json` is a byte
   copy of the live golden as this unit writes it, and no later story edits it. Every key it holds
   is still printed, with the same meaning, for the same input.
7. `every_field_is_classified`. A test lists each field of `App`, `FixtureAdapter`, `StateStore`
   and the session's runner part under state, cloned only, history or drawing, matching the
   design page's table. With the `..`-free destructuring in `snapshot`, a new field fails to
   compile or fails this test.
8. `the_cost_of_a_state_digest`. On the partner portal's home page, the test times 200
   `Session::state` calls with their digest and prints `cost: digest=<µs>`, the median. The first
   measurement on this host is pinned as `DIGEST_CEILING_US`, three times the measured median,
   and the report names the value. The test fails above the ceiling and ends within 20 s.
9. `ess ui test` output is unchanged for every test file of the partner portal, and the tests of
   `g01-session-actions` still pass.
10. `the_stable_surface_compiles` (`crates/ui/ess-ui-test/tests/session.rs`) adds `Session::state`
    and `UiState`.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui-test -p ess-ui-tui --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui-test -p ess-ui-tui --locked`;
- `task test-xtask`;
- `task projection-check` (the formats row and `FORMAT_RELEASES`, through `cargo xtask docs`).

Also:
- `website/docs/reference/formats.md` gets a row, "UI state: `format: ess-ui-state/1`". It says:
  - the release that added the format, and that it is Serialize-only;
  - the members, and what the digest covers;
  - what keeps the version and what changes it.

  The section "Which digest is this?" (`formats.md:20-31`) gets a row for the state digest.
- `crates/edge/ess-xtask/src/docs.rs`, `FORMAT_RELEASES` (`:396-399`), gets
  `("ess-ui-state", 1, Some("<release>"))`.
- `CHANGELOG.md` has an entry under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
items 1 and 2.

1. **Need.** A state's identity, so that two routes merge and work done per state is cached. The
   reproduction is above: no state can be printed or compared. The caller uses the digest to
   de-duplicate states.
2. **Class.** It is a *gap*.
3. **Already expressible?** No. A program could hash the screen text, but two states that draw
   alike can differ in what they hold off screen: rows of a page not shown, a pending live event,
   a draft in session storage.
4. **Fit.**
   - It reuses node paths and row keys as identity, the same keys that `ess-ui-test/1` steps and
     the regions use (`app.rs:61-73`).
   - It reuses the digest scheme of `g02-document-digest`, through its one writer, so ESS keeps one
     algorithm for the UI area.
   - It adds no authored key.
5. **Second adopter.** A visual-regression tool keys each screenshot by state digest and document
   digest, so a rerun compares like with like.
6. **Cost.**
   - One new published format, `ess-ui-state/1`, with two golden files, an old-reader test and a
     formats row.
   - A change to which state the terminal renderer holds is a change of this format, which is the
     coupling `g01-state-design` accepted. The destructuring makes that change visible at compile
     time.
   - A new public `App::snapshot` in `ess-ui-tui`.
   - No `ess-ui/1` or `ess-ui-test/1` version moves.
7. **Considered.**
   - (a) A digest of the screen. Rejected, see question 3.
   - (b) A digest of the step path. Rejected: two routes would give two digests.
   - (c) A digest over the envelope, document digest included. Rejected: the caller walks one
     document at a time, and `InfraIR` keeps its envelope out too. The envelope carries
     `document` for a caller that mixes documents.
   - (d) Chosen: a digest of the classified state.

## Decisions

This story follows P0-D2 (A), P0-D5 (A) and P0-D6 (A), and the design page's rule that every read
cache entry is state, keyed by read request, on screen or not. One question is left open: do the
read cache's rows enter `state` inline, or as one digest per read? The default is inline, because the
document is evidence and fixture data is small. If the home page's golden file goes over 64 KiB,
the implementer switches to digests and says so.

## Scope

- `crates/ui/ess-ui-tui/src/app.rs`: a public `snapshot` and its serialisable type, beside
  `regions` at 657. It reads `Ui` (317-337), `OpenOverlay` (351-362) and the fields at 434-489.
- `crates/ui/ess-ui-tui/src/placement.rs`: an accessor for the store's values by path.
- `crates/ui/ess-ui-tui/src/live.rs`: script positions, 93-140.
- `crates/ui/ess-ui-tui/src/data.rs`: an accessor for the fixture adapter's mutable data.
- `crates/ui/ess-ui-tui/src/lib.rs`: exports.
- `crates/ui/ess-ui-test/src/session.rs`: `state`, and `UiState`.
- `crates/ui/ess-ui-test/src/lib.rs`: exports.
- read only: `crates/ui/ess-ui/src/canonical.rs`.
- new:
  - `crates/ui/ess-ui-test/tests/state_document.rs`;
  - `crates/ui/ess-ui-test/tests/golden/partner-portal-home.state.json`;
  - `crates/ui/ess-ui-test/tests/golden/partner-portal-home.state.first-release.json`.
- `crates/ui/ess-ui-test/tests/session.rs`: two names added to `the_stable_surface_compiles`.
- `website/docs/reference/formats.md`, `crates/edge/ess-xtask/src/docs.rs`.
- `CHANGELOG.md`.
- Size: L, about 590 changed lines: snapshot about 220, state and print about 120, tests about 250.

## Depends on

- `g01-state-design`
- `g01-session-actions`: `Session` and `Rc<Loaded>`.
- `g02-document-digest`: `canonical_json` and `Loaded::digest`.
- `g03-resolved-json`: `formats.md` and `docs.rs`, in round 6.
- Every fill unit, through `g02-document-digest`. The `document` member of the golden does not
  move after this unit.

Round 7, beside `g03-resolved-binding`, with which it shares no file.
