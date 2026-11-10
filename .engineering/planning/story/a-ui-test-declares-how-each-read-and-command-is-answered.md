---
format: aep.planning-md/3
id: story:a-ui-test-declares-how-each-read-and-command-is-answered
kind: story
status: active
title: A ui test declares how each read and command is answered
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#722
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/ui/ess-ui-test/src/parity.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/src/playwright.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/runner.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/spec.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/fixture_answers.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/data.rs
- confidence: cited
  path: website/docs/reference/ess-ui-test.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:42:16Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:42:17Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

The fixture adapter a journey test runs against answers every read the same way and accepts every
command, so a test cannot reach the states a real backend produces (beyond10x/ess#722 items 27 to
31, on 0.84.0; reproduced on 0.92.0), and one fact it relies on is not documented (item 26):

- every command is accepted (`crates/ui/ess-ui-tui/src/data.rs:279-283`) and no read can fail,
  so a refusal message and a section's `failed` state cannot be tested (item 27); the test file has
  no key for either (`crates/ui/ess-ui-test/src/spec.rs:383-404` knows `views` and `scripts` only);
- a read is narrowed only by a row field named like a param (`data.rs:235-269`); a param with
  another name (`order: params.id` against a row's `id`, `q` against a field, `max`) narrows
  nothing, so a record section shows the first fixture row whatever its id param says (items 28,
  29); `by_params` records one param set per view and empties every other
  (`data.rs:224-234`);
- a read declared as a `placeholder` with a `fixture:` file ignores a test's `fixtures.views` for
  that name (item 30): the placeholder's file is read directly (`data.rs:213-219`), while the
  runner replaces only the index's views (`crates/ui/ess-ui-test/src/runner.rs:1562-1607`);
- a test has one latency for every read (`spec.rs:360-363`), so it cannot show one section loading
  while another is ready (item 31);
- in a fixture run the signed-in actor is the first row of the first view the home page's shell
  preloads (`crates/ui/ess-ui-tui/src/app.rs:4300-4310`, `:4496-4516`); with no preload,
  `actor.account_id` is null (item 26). The reference pages do not say so.

Minimal reproduction on 0.92.0:

```yaml
# ui.yaml, page orders.detail (params: {id: string})
- {name: other, component: record, reads: {view: example.order.Page, params: {order: params.id}}, fields: [id, customer]}
# page orders.draft
- {name: list, component: collection, reads: {placeholder: example.draft.Page, fixture: drafts.yaml}, columns: [id, name]}
```

```
failed  item 28 ... expected "Customer 3" at pages/orders.detail/sections/other; it shows: id o-1, customer Customer 1
failed  item 30 ... expected rows [d-9] at pages/orders.draft/sections/list; it shows [d-1, d-2]
tests27.yaml: tests[1] (...): fixtures: unknown key `refuse`; expected one of views, scripts
tests31.yaml: tests[1] (...): `example.order.Page: 2s` is not a duration (500ms, 12s, 5m, 1h)
```

## Acceptance

Tests in `crates/ui/ess-ui-test/tests/`, each red on 0.92.0:

- `fixtures.views.<view>` takes a list of answers, each `{by_params?, rows, total?}`; a read is
  answered by the first whose `by_params` all equal the params asked, else by the one without
  `by_params`, else with no rows. A record reading `{order: o-3}` against answers keyed
  `by_params: {order: o-3}` shows o-3; the map form of 0.92.0 reads as before.
- `fixtures.views.<view>: {fails: <message>}` makes every read of the view fail: the section
  reading it shows `state: failed` and the message.
- `fixtures.commands.<command>: {refuses: <error code>, payload?: {…}}` answers the command with
  that refusal: the command is still logged for `expect_command`, and the refusal shows where the
  user acted, as a served surface's refusal shows.
- `tests[].latency` takes a duration, as today, or a map of view to duration; a view not named
  answers at once. With `{example.order.Page: 2s}` one section is `loading` and another `ready`
  until `advance: 2s`.
- `fixtures.views.<name>` replaces the rows of a placeholder read named `<name>`, as it does a
  view's.
- An unknown key under `fixtures` is still refused with the file, test and key.
- The Playwright spec marks a test that uses any of these keys `test.fixme`, with one reason read
  from the parity rules, until its routes answer them.
- `website/docs/reference/ess-ui-test.md` § Fixtures documents each key and states where the fixture
  actor comes from: the first row of the first view the home page's shell preloads, signed in for
  `actor: from_session`.

## Scope

- cited: crates/ui/ess-ui-test/src/spec.rs
- cited: crates/ui/ess-ui-test/src/runner.rs
- cited: crates/ui/ess-ui-tui/src/data.rs
- inferred: crates/ui/ess-ui-test/src/parity.rs
- inferred: crates/ui/ess-ui-test/src/playwright.rs
- inferred: crates/ui/ess-ui-test/tests/fixture_answers.rs
- cited: website/docs/reference/ess-ui-test.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A test must declare the backend's answer per read and per command: rows per param
   set, a failed read, a refused command, a slow read. Reproduction in § Need. Requester's words:
   "A per-test way to refuse a command or fail a read is missing" (#722 item 27); no syntax
   proposed.
2. **Class.** Gap for items 27, 28 and 31: the document declares refusals (`ess-ui.md` § Action:
   "its typed errors are shown where the action happened") and `failed` states
   (`ess-ui-test.md` § expect, `state`), but no test can reach them. Defect for item 30: the
   reference says `fixtures.views` "replaces a view's data" and a placeholder names a view
   (`ess-ui.md` § Reads). Item 29 is item 28 with an id param. Item 26 is a documentation gap: the
   behaviour exists (`app.rs:4300-4310`).
3. **Already expressible?** Item 26: yes, a shell `preload` of the session view; the triage
   reproduction's test "item 26 idiom" passes with `actor.account_id` read from it. Item 28: only for
   one param set per view (`by_params`) or a param named like the row field. Items 27, 30, 31: no.
4. **Fit.** Extends the two constructs that already describe answers: the fixture view entry
   (`rows`, `total`, `by_params`, `ess-ui.md` § FixtureIndex, View file) becomes a list of such
   entries, and the test's `latency` takes a map; refusals reuse the command answer the data
   adapter already returns (`ess_ui::binding::Answer::Refused`,
   `crates/ui/ess-ui/src/binding.rs:230-241`). No new step. The language grids do not apply.
5. **Second adopter.** A payments console whose "Refund" is refused with `already_refunded` and
   whose ledger section fails while the balance section shows.
6. **Cost.** Three optional keys under `tests[].fixtures`, a map form of `latency`: additive to
   `ess-ui-test/1`, whose unknown keys were refused before, so no format version
   (`website/docs/reference/formats.md:198`). No change to `ess-ui/1` fixture files.
7. **Considered.** (a) Change nothing: rejected, refusal and failure paths stay untestable. (b)
   Steps that refuse or fail mid-test (`refuse_next: <command>`): rejected, the answer is backend
   state, which the reference keeps declared per test, not played by steps (`ess-ui-test.md` §
   Design notes, "Backend state and time are declared"). (c) Fixture files gain the same keys:
   deferred to a later change, the test file is where per-test answers live, and the document's
   fixture files serve the React fixture adapter too. (d) Chosen: per-test answers under
   `fixtures`.

## Decisions

Accept, redesigned (triage of beyond10x/ess#722, 2026-11-09): items 27, 28, 29, 30 and 31 become one
per-test answer declaration under `tests[].fixtures` and `latency`. Item 26 is declined, with the
idiom (a shell preload of the session view), and the idiom is documented here.

## Source

github:beyond10x/ess#722
