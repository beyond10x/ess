---
format: aep.planning-md/3
id: story:ess-ui-test-model-refuses-to-start-on-mistyped-document-fixtures-and-fails-a-test-whose-own-fixtures
kind: story
status: draft
title: ess ui test --model refuses to start on mistyped document fixtures and fails a test whose own fixtures contradict the model, in both renderers
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:with-model-ess-ui-check-refuses-a-fixture-row-or-a-script-payload-whose-value-contradicts-the-type-the-model
revision: 1
---
## Need

`ess ui test --model` runs a document's tests headless against the terminal renderer, answering
reads from the document's fixtures and from each test's own `fixtures:` (views, command refusals,
scripts; `website/docs/reference/ess-ui-test.md:283-333`). With the model it lists model enums and
applies the commands' fixture effects (`crates/ui/ess-ui-test/src/lib.rs:189-226`), but it types
no fixture value against the model: a test passes against rows the served view cannot answer, and
its Playwright spec passes in the browser the same way.

Reported by a downstream application on 0.98.0 (sixth list, item G15, rank P1), together with the
check side (`sx15-fixture-row-types`): `ess ui test --model` passed fixtures whose rows
contradicted the view types.

What is read today (tree `bcfc51555`, read-only):

- `ess-ui-test` holds the model only as `&dyn ess_ui::binding::ModelEnums` and never compiles one
  (`lib.rs:338-343`); it depends on `ess-ui`, `ess-ui-tui` and `ess-ui-react`, and on
  `ess-ui-check` only as a dev-dependency (`crates/ui/ess-ui-test/Cargo.toml:11-24`).
- `Runner::start` loads the document's fixtures through `FixtureAdapter::load`
  (`crates/ui/ess-ui-tui/src/data.rs:242-292`), then applies the test's `fixtures.views`
  (`replace_view`), `fixtures.commands` (`refuse`, with an optional `payload`) and
  `fixtures.scripts` (`crates/ui/ess-ui-test/src/runner.rs:57-79`). None of them is typed.
- A setup failure fails the test without a step (`lib.rs:262-266`; reference
  `ess-ui-test.md:384`). The Playwright branch loads the document and writes the spec with no
  fixture reading at all (`lib.rs:351-361`).

## Outcome

With `--model`, `ess ui test` types fixtures with the walk `sx15-fixture-row-types` adds
(`ess_ui::fixture_types` over `ModelEnums::row_types`), per SX-D3 (default A):

- **The document's fixtures.** Before any test runs, every answer of a model view and every script
  payload is typed. An error finding refuses the run: exit 1, no test is run and no spec is
  written, the message listing the findings as `ess ui check --model` prints them, and naming that
  command. A warning finding (SX-D1 C: an identity's prefix or alphabet) is printed to standard
  error and the run starts.
- **A test's own fixtures.** Each `fixtures.views.<view>` answer (one, or a list of `by_params`
  answers), each `fixtures.scripts.<channel>` payload, and each `fixtures.commands.<command>`
  refusal `payload` against the fields of the command's error with that code, are typed when the
  test starts. An error fails that test at setup, with no step, as a fixture that does not load
  does today: "setup: fixtures: views.orders.Lines: row 2: `qty` is a text, and the view declares
  an Integer". Other tests run.
- **The browser.** With `--model` and `--playwright`, the same document refusal stops the spec,
  and a test whose own fixtures fail is marked `test.fixme` with the same text, so one test file
  keeps one verdict in both renderers (`ess-ui-test.md:412-417`).
- Without `--model` nothing changes.
- `RowTypes` gains the payload fields of each error a document's command can answer, which this
  story needs for `refuses:` payloads.

## Acceptance

- First: a case in the existing `--model` target `crates/ui/ess-ui-test/tests/model_enums.rs`
  (no new test binary; model and fixtures under `tests/fixtures/fixture-types/`) whose document fixture holds `qty: "two"` for an Integer
  field. Today `ess ui test --model` runs the tests and they pass. After: exit 1 before any test,
  the message naming `fixtures/views/orders.Lines`, row 1, `qty`, and `ess ui check --model`.
- The same document with a correct fixture and a test whose `fixtures.views.orders.Lines` holds a
  row without the required `status`: that test fails at setup with no `step`, naming
  `views.orders.Lines`, the row and `status`; a second test of the file passes.
- A test whose `fixtures.commands.orders.Approve` is `{refuses: line_locked, payload: {line_id: 7}}`
  where the error declares `line_id: LineId` (a text newtype) fails at setup naming `payload.line_id`.
- A test-local script whose event payload misses a required event field fails at setup.
- A fixture whose only finding is a prefix warning runs, prints one warning line to standard
  error, and the test passes.
- `--playwright` with `--model`: the document refusal writes no spec and exits 1; a test whose own
  fixtures fail is emitted as `test.fixme` with the setup text; a case in
  `crates/ui/ess-ui-test/tests/` reads the emitted spec.
- Without `--model` the same files run as today: a case pins the passing run.
- `ess-ui-test.md` documents both refusals under "Fixtures" and under "One verdict in both
  renderers"; `CHANGELOG.md` names under Changed that `ess ui test --model` refuses mistyped
  fixtures it ran before.
- Each new test is timed and expected under 5 s; none may run over 60 s, the crate's test step stays
  within 120 s, and each build-pool job within 10 minutes.

## Fit review

Verdict: **adopt (gap)**, the runner half of the feature "fixtures and live declarations are typed
against the bound model". Tree `bcfc51555`, read-only.

1. **Need.** A test is a claim about the served surface; one run on rows that surface cannot
   answer claims nothing. Reproduction above.
2. **Class.** Gap.
3. **Already expressible?** Only by running `ess ui check --model` first, which does not see a
   test file's own fixtures.
4. **Fit.** No key in `ess-ui-test/1`. The runner reads the types through the trait it already
   receives (`ModelEnums`, as `fixture_effects`), so it still compiles no model. The document
   refusal follows the existing pattern of a run refused before tests (`check_documents`,
   `lib.rs:296`); the per-test failure follows "a test whose fixtures do not load fails without
   one" (`ess-ui-test.md:384`); the browser marking follows the parity rule
   (`crates/ui/ess-ui-test/src/parity.rs:1-27`).
5. **Second adopter.** Any suite of UI tests that overrides fixtures per test, as the reference's
   own example does (`ess-ui-test.md:291-307`).
6. **Cost.** No format version. `ess ui test --model` exits 1 where it ran and passed before, for
   mistyped fixtures only; named in `CHANGELOG.md`.
7. **Considered.** SX-D3 (B): no run refusal, every test fails at setup for a document fixture;
   one cause reported N times. SX-D3 (C): print the findings and run; the requester's gate would
   still pass. Chosen (A).

Covers sixth list, item G15 (with `sx15-fixture-row-types`).

## Decisions

- **SX-D3. What `ess ui test --model` does with a mistyped fixture.** (A) A document fixture error
  refuses the run (no test, no spec); a test's own fixture error fails that test at setup and is
  `test.fixme` in the spec; warnings print and run. (B) Every error fails each test it reaches at
  setup; no run refusal. (C) Print the findings and run every test. **Recommended: (A).**

## Scope

- cited: `crates/ui/ess-ui-test/src/lib.rs` (`execute_with` `:194-236`, the document typing before
  `run_test`; the `--playwright` branch `:351-361`)
- cited: `crates/ui/ess-ui-test/src/runner.rs` (`Runner::start` `:57-79`, test-local fixtures)
- cited: `crates/ui/ess-ui-test/src/playwright.rs` (a setup-failing test as `test.fixme`)
- cited: `crates/ui/ess-ui/src/binding.rs` (`RowTypes` gains error payload fields)
- cited: `crates/ui/ess-ui-check/src/model.rs` (fills them from the IR's errors)
- cited: `website/docs/reference/ess-ui-test.md`, `CHANGELOG.md`
- inferred: cases in `crates/ui/ess-ui-test/tests/model_enums.rs` and `playwright.rs`,
  `tests/fixtures/fixture-types/`
- Size: M, about 350 changed lines.

## Depends on

- `sx15-fixture-row-types` (the types, the walk and `ModelEnums::row_types`).
- Lands after `g07-session-script`, which edits `runner.rs` (`play`), `parity.rs` and
  `playwright.rs`.
