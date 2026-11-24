---
format: aep.planning-md/3
id: story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the
kind: story
status: draft
title: A UI state prints as ess-ui-state/1 with a digest, and two routes to one state print the same bytes and the same digest
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state
- depends_on: story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test
- depends_on: story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling
revision: 1
---
## Need

Report item 2 asks for "a normative canonical byte form and digest … of a UI state". Report item 1
asks for a state that can be told apart from another without replay. A program that walks UI states
merges two routes that reach one state, and caches what it computed per state under the state's
digest. Today a UI state exists only as private fields of `ess_ui_tui::App`
(`crates/ui/ess-ui-tui/src/app.rs:434-489`) and of the fixture adapter (`data.rs:84-96`).

- Nothing prints a state. The `ess-ui-test-report/1` report holds only pass or fail
  (`website/docs/reference/ess-ui-test.md:363-385`).
- No UI crate computes a digest (`git grep -n "digest\|sha2" -- crates/ui/*/src crates/ui/*/Cargo.toml`
  finds nothing).
- Some fields are history, not state, and must stay out of a state's identity: `commands_sent`
  (`app.rs:479`), `exports` (`:481`), the cache's `generations` (`:468`) and the adapter's command
  log (`data.rs:96`). Others are indexes that differ between two routes to one state: the per-view
  `cursor` (`app.rs:319`) and `item` (`:326`) hold a position, not a row key.

## Outcome

- `Session::state(&self) -> UiState` (`ess-ui-test`) builds the state from a new
  `ess_ui_tui::App::snapshot()` and the fixture adapter's data. It prints as one JSON object
  `{format: "ess-ui-state/1", document, digest, state}`:
  - `document`: the `ess-ui-resolved/1` digest of the document the state belongs to
    (`g02-document-digest`);
  - `state`: the fields the design page (`g01-state-design`) classifies as state;
  - `digest`: the digest of `state` alone, by the scheme of `g02-document-digest` (compact,
    key-sorted JSON, bare SHA-256, no trailing newline). It leaves out the envelope, as the
    `InfraIR` digest leaves out its envelope (`website/docs/reference/formats.md:29`).
- Rows and positions are named by node path and row key, never by an index.
- History (the command log, exports, counters) and drawing (regions, marks) stay out.
- The format is versioned, Serialize-only and has a row in the formats reference.

## Acceptance

New test file `crates/ui/ess-ui-test/tests/state_document.rs`. The walks are bounded by a state
count chosen so each test runs under 30 s. Time every test: none may run over 60 s, and the file's
step stays within 120 s.

1. `two_routes_to_one_state_print_the_same_bytes`. Red today: there is no `Session::state`. On
   `examples/partner-portal/ui.yaml`, each pair below ends in byte-equal documents with equal
   digests:
   - selecting the second row of the partner list directly, and selecting the first row and then
     the second;
   - paging to page 2 and back to page 1, and staying on page 1;
   - opening an overlay and cancelling it, and never opening it (with the back stack equal in both
     routes).
2. `equal_digests_mean_equal_behaviour`.
   - Walk breadth-first from the home page to depth 3, stopping at 1,000 states.
   - For every two states with one digest: their screen text, their regions and their offered
     steps are equal.
   - After applying each offered step to a fork of each, the two digests are again equal.
3. `a_different_state_has_a_different_digest`. A marked row, a typed filter text, a moved clock
   and an open overlay each change the digest of the state they are applied to.
4. `the_state_prints_its_golden_bytes`. The home page state of `examples/partner-portal/ui.yaml`
   prints exactly `crates/ui/ess-ui-test/tests/golden/partner-portal-home.state.json`. Printing
   twice gives equal bytes. The bytes hold no absolute path and no time of day.
5. `the_digest_is_recomputable`. The test recomputes the digest from the golden file's `state`
   member with `sha2` and `serde_json` alone, sorting keys itself. The result equals the printed
   `digest`.
6. `ess ui test` output is unchanged for every corpus file. The tests of `g01-session-actions`
   still pass.

Also:

- `website/docs/reference/formats.md` gets a row `UI state: format: ess-ui-state/1`, saying the
  release that added it, Serialize-only, what the digest covers, and what keeps or changes the
  version. The section "Which digest is this?" (`formats.md:20-31`) gets a row for the state
  digest.
- `crates/edge/ess-xtask/src/docs.rs` `FORMAT_RELEASES` (`:396-399`) gets `("ess-ui-state", 1,
  Some("<release>"))`, and `task test-xtask` passes.
- The golden file of the first release that writes the format stays in the tree. An old-reader
  test checks that every key it holds is still printed with the same meaning for the same input.
- `CHANGELOG.md` has an entry under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
items 1 and 2.

1. **Need.** A state's identity, so that two routes merge and work done per state is cached. The
   reproduction is above: no state can be printed or compared.
2. **Class.** This is a *gap*.
3. **Already expressible?** No. A program could hash the screen text, but two states that draw
   alike can differ in what they hold off screen: rows of a page not shown, a pending live event,
   a draft in session storage.
4. **Fit.**
   - It reuses node paths and row keys as identity, the same keys `ess-ui-test/1` steps and the
     regions use (`app.rs:61-73`).
   - It reuses the digest scheme already published for `InfraIR` and `InfraSpec`
     (`formats.md:29-30`), through the one canonical writer of `g02-document-digest`, so ESS
     keeps one algorithm for the UI area.
   - It adds no authored key.
5. **Second adopter.** A visual-regression tool keys each screenshot by state digest and document
   digest, so a rerun compares like with like.
6. **Cost.**
   - One new published format, `ess-ui-state/1`, with a golden file, an old-reader test and a
     formats row. A change to which state the terminal renderer holds is a change of this format,
     which is the coupling `g01-state-design` accepted.
   - A new public `App::snapshot` in `ess-ui-tui`.
   - No `ess-ui/1` or `ess-ui-test/1` version moves.
7. **Considered.**
   - (a) A digest of the screen. Rejected, see question 3.
   - (b) A digest of the step path. Rejected: two routes would give two digests.
   - (c) Chosen: a digest of the classified state.

## Decisions

This story follows P0-D2 (A), P0-D5 (A) and P0-D6 (A). It also leaves open whether the read
cache's rows enter `state` inline or as one digest per read. The default is inline, because the
document is evidence and fixture data is small. If the golden file of the home page goes over
64 KiB, the implementer switches to digests and says so.

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs` (a public `snapshot` and its serialisable type, beside
  `regions` at 657; reads `Ui` 317-337, `OpenOverlay` 351-362, the fields at 434-489)
- cited: `crates/ui/ess-ui-tui/src/placement.rs` (the store's values per path), `live.rs` (script
  positions, 93-140), `data.rs` (the fixture adapter's mutable data)
- cited: `crates/ui/ess-ui-test/src/session.rs` (from `g01-session-actions`: `state`)
- cited: `crates/ui/ess-ui/src/` (the canonical writer from `g02-document-digest`, read only)
- new: `crates/ui/ess-ui-test/tests/state_document.rs`, `crates/ui/ess-ui-test/tests/golden/`
- `website/docs/reference/formats.md`, `crates/edge/ess-xtask/src/docs.rs`, `CHANGELOG.md`
- Size: L, about 560 changed lines (snapshot about 220, state and print about 120, tests about 220).

## Depends on

- `g01-state-design`
- `g01-session-actions`
- `g02-document-digest` (the canonical writer and the document digest)
