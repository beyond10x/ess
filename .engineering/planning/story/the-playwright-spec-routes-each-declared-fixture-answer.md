---
format: aep.planning-md/3
id: story:the-playwright-spec-routes-each-declared-fixture-answer
kind: story
status: active
title: The Playwright spec routes each declared fixture answer
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#722
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-ui-test-declares-how-each-read-and-command-is-answered
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/ui/ess-ui-test/src/parity.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/playwright.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/playwright_answers.rs
- confidence: inferred
  path: website/docs/reference/ess-ui-test.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:47:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:52Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

story:a-ui-test-declares-how-each-read-and-command-is-answered gives a journey test per-test
answers (rows per param set, a failed read, a refused command, a latency per view) and marks every
test using them `test.fixme` in the Playwright spec, so the browser gives such a test no verdict
(beyond10x/ess#722 items 27 to 31). The spec already routes `/views/<view>` and `/commands/<command>`
for replaced fixtures and expected commands (`website/docs/reference/ess-ui-test.md` § In the
browser); it can answer the new declarations the same way, so the two renderers give one verdict.

Minimal reproduction: a test with `fixtures: {commands: {example.order.Create: {refuses:
duplicate_order}}}` written with `ess ui test --path ui.yaml t.yaml --playwright ui.spec.ts`
emits `test.fixme` once that story lands.

## Acceptance

Tests in `crates/ui/ess-ui-test/tests/` over the emitted spec text:

- A view with a list of answers is routed by one handler that answers the first entry whose
  `by_params` equal the request's query params, else the entry without `by_params`, else `{rows: []}`.
- `{fails: <message>}` is routed as status 500 with the message; `{refuses: <code>, payload?}` on
  a command as the refusal body the HTTP data adapter reads as `Answer::Refused`.
- A latency map delays each named view's route by its duration on the page clock.
- None of these tests is `test.fixme` any longer; a test using no new key emits byte for byte the
  spec it emitted before.

## Scope

- cited: crates/ui/ess-ui-test/src/playwright.rs
- inferred: crates/ui/ess-ui-test/src/parity.rs
- inferred: crates/ui/ess-ui-test/tests/playwright_answers.rs
- inferred: website/docs/reference/ess-ui-test.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A test file has one verdict in both renderers (`ess-ui-test.md` § One verdict in both
   renderers); a test that declares answers must run in the browser too. Reproduction in § Need.
2. **Class.** Gap, completing the per-test answers in the second target.
3. **Already expressible?** No: the spec routes replaced rows and captures commands only
   (`crates/ui/ess-ui-test/src/playwright.rs:271-276` comments out a latency).
4. **Fit.** Reuses the spec's existing `/views/` and `/commands/` routes and the HTTP data
   adapter's answer shapes; no authored surface beyond the predecessor story's. The language grids
   do not apply.
5. **Second adopter.** The payments console of the predecessor story, run in CI in a browser.
6. **Cost.** Spec emission only; no format change.
7. **Considered.** (a) Keep `test.fixme`: rejected as the end state, the browser would never test
   refusals. (b) Answer in the generated React fixture adapter: rejected, the spec runs against the
   HTTP adapter (`ess-ui-test.md` § In the browser). (c) Chosen: route each answer.

## Decisions

Accept, redesigned (triage of beyond10x/ess#722, 2026-11-09): the browser half of the per-test
answers.

## Source

github:beyond10x/ess#722
