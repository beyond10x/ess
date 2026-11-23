---
format: aep.planning-md/3
id: review-result:scope-wave-2026-11-23-round-1
kind: review-result
status: active
title: Scope critic, round 1, on the wave of 2026-11-23
relations:
- reviews: story:a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries
- reviews: story:the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step
- reviews: story:the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane
- reviews: story:the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane
- reviews: story:the-generated-view-queries-target-keeps-generation-in-the-test-step-and-compiles-in-the-slow-lane
- reviews: story:three-adversary-targets-run-a-reduced-sweep-in-the-test-step-and-the-full-sweep-in-the-slow-lane
- reviews: story:the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests
- reviews: story:a-writer-true-conformance-entry-under-ess-composition-6-decides-presence-by-what-the-sender-may-write
- reviews: story:a-writer-true-conformance-entry-admits-a-senders-narrowing-of-types-and-value-constraints
- reviews: story:the-documents-and-help-describe-the-writer-true-conformance-entry-and-ess-composition-6
- reviews: story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan
- reviews: story:every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads
- reviews: story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite
- reviews: story:ess-verify-conform-synthesize-and-run-take-jobs-and-the-bytes-are-the-same-at-every-job-count
- reviews: story:cargo-xtask-synth-jobs-measures-one-synthesis-at-one-two-and-four-jobs-and-refuses-a-differing-suite
- reviews: story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose
- reviews: story:an-ess-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
- reviews: story:the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route
- reviews: story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route
- reviews: story:the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets
revision: 1
---
needs-revision

Not findings, left out on purpose: the test-budget parent being wider than its seven units, the threads D2 C limit, c1's D1 option B, and the diagnostics page edit the body-cap units drop.

- story:the-test-step-meets-the-test-budget — "A `slow` job exists … `ci_lanes.rs` pins its triggers and that every byte-pin target `AGENTS.md` lists runs in it": no unit claims it, and the page's follow-up list omits it. t1 pins no such list, and t2 moves only 4 of the 13 listed targets (the rest: `adversary_related_values_all_bytes`, `external_beside_held_guard`, `served_codec_names`, `go`, `feasibility`, `adversary_related_guard_behaviour_pass1`, `http_binding_rust`, `adversary_ess_issues_2_u6`, `string_alphabet_and_length`). Either t1 takes the pin and each target moves, or the page lists it as a follow-up — .engineering/planning/story/the-test-step-meets-the-test-budget.md:242
- story:the-test-step-meets-the-test-budget — the Out of scope line "The browser-product tests beyond moving them to `slow`" puts moving them into scope, and step 5 puts "the five harness tasks" in the lane. t1 states `task test-browser-product` "stays as it is" and moves no harness task, and the page's follow-ups do not name them. t1 would take it — .engineering/planning/story/the-test-step-meets-the-test-budget.md:375
- story:the-test-step-meets-the-test-budget — the last Acceptance line, "A pull request whose diff is only prose `*.md` files outside `website/` passes its required checks without a test job … (the second change of design step 2)", is claimed by no unit and appears in no follow-up on the page — .engineering/planning/story/the-test-step-meets-the-test-budget.md:256
- story:the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane — the parent promises "The test step … runs no network command; `ci_lanes.rs` fails when it does", and "`transport-network` is named a network lane outside the test step". The unit's only pin is on `ESS_TUTORIAL_NETWORK`, and nothing names `transport-network`, so the promise narrows with no decision saying so — .engineering/planning/story/the-test-step-meets-the-test-budget.md:240
- story:the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step — the parent asks only for the `docs::` tests to run within 120 s. The unit also edits the parked `consumer-check` task and adds a `ci_lanes` pin over every `test*` task for wrappers and `-j`, which the parent never mentions. Drop both, or record them as a separate story — .engineering/planning/story/the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step.md:29
- story:the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests — two additions are not traceable to the parent: the hotfix gate running `test-slow` (with new `setup-go` and TypeScript steps in `release.yml`), and `upload-artifact` steps for JUnIt reports in every shard. The parent names the JUnit report but no upload and no hotfix change. Cut them, or put them in the parent's Decisions first — .engineering/planning/story/the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests.md:42
- story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite — the parent promises "one byte-identity test over the committed model corpus … synthesized at 1, 2 and 7 threads … compared with `cmp`". The unit tests five fixtures, and D6 proves the corpus with a one-off `ESS_SYNTH_JOBS=4` run over the tables, which is no committed test and does not cover 2 or 7. Nothing records the narrowing — .engineering/planning/story/the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes.md:175
- story:cargo-xtask-synth-jobs-measures-one-synthesis-at-one-two-and-four-jobs-and-refuses-a-differing-suite — the parent asks for N = 1, 2, 4 "on the downstream specification … reported in the story", with N = 4 at most 45 % of N = 1. The unit adds a permanent xtask subcommand, a `synth-jobs-profile` task and a model generator, measures a generated model, and states no verdict on the 45 %. Either the tool is named in the parent's Decisions, or the measurement is a scratch script (allowed) plus a recorded verdict — .engineering/planning/story/the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes.md:185
- story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan — the Why of p1 to p5 cites the placeholder `PARENT-THREADS` instead of the parent's id, so the prose link to the parent is broken in all five units — .engineering/planning/story/every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan.md:46
- story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose — the parent's D3 option A (recommended) says to measure whether the added CPU comes from the new scenarios or from failed fallbacks, and to bound the latter with a memo. The unit's D3 declines both and leaves the repeated arrangement to a draft story. The page lists D1 as a veto choice but not this one — .engineering/planning/story/a-setup-run-of-a-creating-branch-that-reads-several-related-rows-creates-its-row-with-the-field-values-another-scenario-chose.md:131
- story:the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route — the Acceptance "A route with a 2 MiB cap accepts a body the default 1 MiB refuses; checked by a unit test of the cap lookup in each server" is claimed by no unit. The socket cases stop at a 256-byte route cap against a 64-byte binding cap, so a cap above 1 MiB, the consumer's actual need, is never exercised. h2 and h3 each need one lookup test — .engineering/planning/story/an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets.md:282
- story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route — same gap: no Go cap-lookup test for a route cap above the fixed 1 MiB (the 256 and 64 byte cases only), so the unit never exercises the 2 MiB route case — .engineering/planning/story/an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets.md:282
- story:the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route — the parent says the test shares "the fixture's one server build" and "no new server is built". The unit adds a Taskfile task `body-cap-rust-server`, the env var `ESS_BODY_CAP_RUST_SERVER`, a fixture copy and an `AGENTS.md` harness line. That may be right under the test budget, but no parent decision records it — .engineering/planning/story/an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets.md:245
- story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route — the unit adds a `setup-go` step to `build-tests` in `ci.yml`, a new `ci_lanes` test, the task `body-cap-go-server` and the env var `ESS_BODY_CAP_GO_SERVER`. The parent names none of these and says no new server is built; the departure needs a parent decision — .engineering/planning/story/the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md:67
- story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route — the parent's unit 3 is "the same, with `LimitReader` bound by route, and the Rust-equals-Go parity case". The unit only reads the existing uncapped parity test and adds no capped-binding parity case, with nothing saying it was dropped — .engineering/planning/story/an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets.md:259
- story:the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets — three additions the parent did not ask for. (1) A new subsection in `website/docs/guides/http-routes.md`; the parent names the design page, `formats.md` and `diagnostics.md`. (2) Renumbering the precedence list (new step 2, old 2 to 8 become 3 to 9), where the parent says the page "names it as step 1a". (3) Putting every response of a bounded route in the `x-ess-outcomes` form, where the parent asks only for a `413` response and `x-ess-request-max-bytes` — .engineering/planning/story/the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets.md:57

What I read: 5 parents and all 20 units in full, plus the wave page; `aep plan artifact list --kind story --status draft --format json`; the wave tag found by `grep -l wave-2026-11-23` over `.engineering/planning/story`; `aep plan artifact graph` (header only); `Read` on every file. Promises extracted and traced to a unit, or to a drafter-named omission or the page's follow-up list:

| parent | extracted | traced |
|---|---|---|
| test budget | 19 | 14 |
| threads | 9 | 7 |
| writer | 12 | 12 |
| chosen values | 7 | 6 |
| body cap | 14 | 12 |

The writer set is complete in both directions. The mismatch with the parent's "no existing assertion changed" at `owner_types.rs:174` is a supported-formats list edit that w1 names, so I did not count it.

What I could not establish:
- Whether the parent's "generated comments name the 1 MiB default" (cap D5) conflicts with "bytes unchanged". h2 and h3 satisfy the byte-pin side only, and the contradiction is the parent's own.
- t1's inverted slow-lane trigger (run unless every changed path is skippable, against the parent's "only when sources changed") looks like design, so I left it out of scope.
- c1's D8 hedge, that a still-refused leg may stay refused, is an acceptance-checkability matter and not scope.
- h4's `x-ess-outcomes` choice may also be design, not scope. I raised it only for traceability.
- c1's new test binary conflicts with the one-binary-per-crate rule; that belongs to the design critic.
- Which of the 172 spawning files the page's per-crate follow-ups cover, since I did not count them.

```findings
- file: .engineering/planning/story/the-test-step-meets-the-test-budget.md
  line: 242
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Acceptance that ci_lanes.rs pins that every byte-pin target AGENTS.md lists runs in the slow job is claimed by no unit and is not in the page's follow-ups; t2 moves 4 of the 13 listed targets and t1 pins no list (t1 would take the pin)"
- file: .engineering/planning/story/the-test-step-meets-the-test-budget.md
  line: 375
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the parent puts moving the browser-product tests and the five harness tasks into the slow lane in scope (Out of scope line, design step 5); t1 says test-browser-product stays as it is and moves no harness task, and the page lists no follow-up for it"
- file: .engineering/planning/story/the-test-step-meets-the-test-budget.md
  line: 256
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Acceptance that a prose-only *.md pull request passes without a test job once test-xtask, tutorial_page and site_navigation run in one small job is claimed by no unit and appears in no follow-up on the page"
- file: .engineering/planning/story/the-test-step-meets-the-test-budget.md
  line: 240
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the parent promises the test step runs no network command (ci_lanes.rs fails when it does) and that transport-network is named a network lane; t3 pins only ESS_TUTORIAL_NETWORK and nothing names transport-network, a narrowing no decision records"
- file: .engineering/planning/story/the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step.md
  line: 29
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the unit edits the parked consumer-check task and adds a ci_lanes pin over every test* task for wrappers and -j, neither of which the parent asks for (it asks only that the docs:: tests run within 120 s); drop both or record them as a separate story"
- file: .engineering/planning/story/the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests.md
  line: 42
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the JUnit upload steps in every shard and the hotfix gate running test-slow with new setup-go and TypeScript steps in release.yml are named nowhere in the parent; cut them or put them in the parent's Decisions first"
- file: .engineering/planning/story/the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes.md
  line: 175
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the promised committed byte-identity test over the model corpus at 1, 2 and 7 threads is claimed by no unit: p3 tests five fixtures and D6 proves the corpus with a one-off ESS_SYNTH_JOBS=4 run, which is no committed test and does not cover 2 or 7; p3 would take it or record the narrowing"
- file: .engineering/planning/story/the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes.md
  line: 185
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "p5 adds a permanent xtask subcommand, a Taskfile task and a model generator the parent never names, measures a generated model instead of the downstream specification, and gives no verdict on the parent's 'N = 4 at most 45 % of N = 1'; name the tool in the parent's Decisions or use a scratch script and record the verdict"
- file: .engineering/planning/story/every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan.md
  line: 46
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Why of p1 to p5 cites the literal placeholder PARENT-THREADS instead of the parent's id, so the prose link to the parent is broken in all five units"
- file: .engineering/planning/story/a-setup-run-of-a-creating-branch-that-reads-several-related-rows-creates-its-row-with-the-field-values-another-scenario-chose.md
  line: 131
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the parent's D3 option A (recommended) says to measure whether the added CPU comes from the new scenarios or from failed fallbacks and to bound the latter by a memo; c1's D3 declines both and defers to a draft story, a departure the wave page does not list among the choices critics may veto"
- file: .engineering/planning/story/an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets.md
  line: 282
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the Acceptance 'A route with a 2 MiB cap accepts a body the default 1 MiB refuses, checked by a unit test of the cap lookup in each server' is claimed by no unit: h2's socket cases stop at a 256-byte route cap, so a cap above 1 MiB (the consumer's real need) is never exercised; h2 and h3 each need a lookup test"
- file: .engineering/planning/story/the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md
  line: 33
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "h3 has no test of a route cap above the fixed 1 MiB (its cases use 64 and 256 bytes), so the parent's 2 MiB cap-lookup acceptance is unclaimed on the Go side as well"
- file: .engineering/planning/story/an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets.md
  line: 245
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "h2 adds the task body-cap-rust-server, the env var ESS_BODY_CAP_RUST_SERVER, a fixture copy and an AGENTS.md harness line although the parent says the tests share the fixture's one server build and that no new server is built; the departure needs a parent decision"
- file: .engineering/planning/story/the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md
  line: 67
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "h3 adds a setup-go step to build-tests in ci.yml, a new ci_lanes test, the task body-cap-go-server and ESS_BODY_CAP_GO_SERVER, none named by the parent, which says no new server is built; record the departure in the parent's Decisions or remove it"
- file: .engineering/planning/story/an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets.md
  line: 259
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the parent's unit 3 includes 'the Rust-equals-Go parity case'; h3 only reads the existing uncapped parity test and adds no capped-binding parity case, with nothing saying it was dropped"
- file: .engineering/planning/story/the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets.md
  line: 57
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "h4 adds a guide subsection in website/docs/guides/http-routes.md, renumbers the precedence list where the parent says the page 'names it as step 1a', and moves every response of a bounded route into the x-ess-outcomes form where the parent asks only for a 413 response and x-ess-request-max-bytes; none is traceable to the parent"
```
