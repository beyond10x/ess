---
format: aep.planning-md/3
id: review-result:scope-wave-2026-11-23-round-2
kind: review-result
status: active
title: Scope critic, round 2, on the wave of 2026-11-23
relations:
- reviews: story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it
- reviews: story:the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step
- reviews: story:a-slow-ci-job-runs-task-test-slow-on-every-run-that-can-count-for-a-release-and-gate-carries-it
- reviews: story:the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane
- reviews: story:the-generated-view-queries-target-keeps-generation-in-the-test-step-and-compiles-in-the-slow-lane
- reviews: story:three-adversary-targets-run-a-reduced-sweep-in-the-test-step-and-the-full-sweep-in-the-slow-lane
- reviews: story:the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane
- reviews: story:the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests
- reviews: story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan
- reviews: story:every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads
- reviews: story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite
- reviews: story:ess-verify-conform-synthesize-and-run-take-jobs-and-the-bytes-are-the-same-at-every-job-count
- reviews: story:cargo-xtask-synth-jobs-measures-one-synthesis-at-one-two-and-four-jobs-and-refuses-a-differing-suite
- reviews: story:a-writer-true-conformance-entry-under-ess-composition-6-decides-presence-by-what-the-sender-may-write
- reviews: story:a-writer-true-conformance-entry-admits-a-senders-narrowing-of-types-and-value-constraints
- reviews: story:the-documents-and-help-describe-the-writer-true-conformance-entry-and-ess-composition-6
- reviews: story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose
- reviews: story:an-ess-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
- reviews: story:the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route
- reviews: story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route
- reviews: story:the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets
revision: 1
---
needs-revision

- `story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite` — test 13 is marked with t1a's marker, but the row for it in `MARKED` in `crates/edge/ess-xtask/tests/slow_lane.rs` is claimed by no unit. That file is not in p3's typed scope, "Shared files" lists only `synthesis_suite_bytes_table.rs` and `threads.rs`, and p3's Gate never runs `ess-xtask --test slow_lane`. t1a's acceptance 1 makes the test `every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked` fail on a marker without a row. The marker lands in p3 and the red test shows up at merge. Fix: add `slow_lane.rs` to p3's scope with the row for `every_repository_model_and_component_writes_its_bytes_at_one_two_and_seven_jobs`, and add `cargo test -p ess-xtask --test slow_lane` to p3's Gate. — `.engineering/planning/story/with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite.md:334`
- `story:the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests` — the parent promises that `test-xtask` and `test-doc` "run under the same limit" (a test that sleeps 61 s fails its step) and that every test-running step prints its ten slowest tests. t7 gives these two tasks only a whole-step `timeout` ceiling. They run `cargo test`, which has no per-test kill and no JUnit report for `test-slowest`. Neither the unit's Decisions nor the follow-ups say this narrowing is taken. The test step is not shortened by it. Fix: add one Decisions line that names both as not built here, and add them to the follow-ups, or move the two tasks to nextest. — `.engineering/planning/story/the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests.md:42`
- `story:the-test-step-meets-the-test-budget` — the parent's end-state promises are neither built by a unit nor listed as follow-ups. They are: the test step ends within 120 s on a `main` push (the run id recorded in the parent, or D2 naming what is counted) and D2, the one-integration-binary-per-crate story `story:one-integration-test-binary-per-large-crate`. t7 sets the ceilings at about 25 minutes, and its Decisions say 120 s is "the end state, one follow-up per crate". The parent's `## Decisions taken` and the wave page list the per-crate stories but not the 120 s recording or D2. The wave page row (line 23) still states the 2-minute end state as the parent's outcome. Fix: add one follow-up line to both places. — `.engineering/waves/wave-2026-11-23.md:23`

Round 1:
1 test-budget byte-pin ci_lanes pin — fixed — now follow-up (a) on the wave page (lines 129-130) and in the parent's `## Decisions taken`; t2 states "4 of the 13" in its Why.
2 browser-product and the five harness tasks — fixed — follow-up (b) is listed; t1a "Not yours" and Decisions say they stay as they are.
3 prose-only `*.md` pull request — fixed — follow-up (c) is listed on the page and in the parent.
4 t3 network pin — fixed — t3 Decisions (line 102) says the pin covers `ESS_TUTORIAL_NETWORK` only; follow-up (d) names the wider pin and `transport-network`.
5 t6 consumer-check and `test*` pin — fixed — both cut. t6's typed scope is `CHANGELOG.md`, `Taskfile.yml`, `docs.rs` and `ci_lanes.rs`; the `main.rs` edit is gone and the draft story is named.
6 t7 JUnit upload and hotfix slow lane — fixed — both cut, and the page lists them. t7 edits no workflow file (acceptance 5). h3 owns the hotfix Go setup through one `ci_lanes` test over both jobs.
7 p3 corpus test — fixed in part — test 13 exists as a slow-lane test and p3 depends on t1a. The `MARKED` row is unowned (Round-2 finding 1).
8 p5 measurement tool — fixed — the parent records the xtask-on-a-generated-model decision and the 45 % verdict as the coordinator's. p5's acceptance 2 is decided by counts and identical suites, and acceptance 3 is stated at a load of 60 or less.
9 `PARENT-THREADS` placeholder — fixed — grep over all 21 bodies finds no `PARENT-` token; the five Why lines name the parent id.
10 c1 D3 departure — fixed — c1 D3 and the page's "Departures from a parent" list it, and the parent's note records it. The outcome is one statement, "0 refusals, no leg left". Test 6b and D8 are consistent with it.
11 h2 2 MiB cap lookup — fixed — test 3b parses the generated `body_limit` table and asserts `POST /bulk` returns 2097152, above `FIXED_REQUEST_BYTES` read from the crate. Derived and no-body rows return `None`, so "derived routes keep the fixed bound" holds, and no 2 MiB request is sent.
12 h3 2 MiB cap lookup — fixed — the same test 3b over `bodyLimitParcel`; the socket case sends 1,200,000 bytes to `/bulk` and 1,048,577 to the derived route.
13 h2 server build — fixed — the harness task `body-cap-rust-server` and its env var are recorded as parent D8. h2's Decisions explain why the new server replaces "no new server is built".
14 h3 Go set-up in CI — fixed — `ci_lanes` test 6 covers `build-tests` and `hotfix-tests`; D11 is recorded.
15 h3 capped parity — fixed — test 5b sends the same raw requests to both servers, with h3 depending on h2 and h1.
16 h4 precedence, per-route OpenAPI form, guide — fixed — step 1a with no renumbering; the stage-1 form is kept and `x-ess-outcomes` is not added. `ess-synth/src/{go,rust}/http.rs` are named only as pinned reads, and the guide subsection is D9 in the parent.

What you read:
- The two slow-lane units and t2 to t7 in full, apart from their Where and Steps sections.
- The h1 to h4 bodies (h1 Outcome, Not yours and Acceptance; h2 and h3 in full; h4 Outcome, Not yours and Acceptance).
- The c1, p3, p4 and p5 Outcome and Acceptance sections.
- p3 tests 13 and the mutation proof.
- The typed scope paths and `depends_on` edges of all 21 units.
- The five parents' decision notes.
- The revised wave page.
- The test-budget parent's acceptance and decisions.

What you could not establish:
- Whether the 28 `docs::` tests of `ess-xtask` fit 120 s (t6 step 0 decides).
- Whether p1, p2, w1, w2 and w3 were revised as their authors claim. I checked their typed scope and edges only, not their bodies.
- Whether t2's eight-crate hash list is complete. I read the manifest reasoning in its body, not the manifests.

```findings
- file: ".engineering/planning/story/with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite.md"
  line: 334
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "Test 13 is marked with t1a's slow-lane marker, but its row in MARKED (crates/edge/ess-xtask/tests/slow_lane.rs) is claimed by no unit: the file is not in p3's typed scope or Shared files, and p3's Gate does not run ess-xtask --test slow_lane. t1a's test that markers and MARKED agree is red from the moment p3 lands. Add slow_lane.rs to p3's scope with the row, and the slow_lane test to its Gate."
- file: ".engineering/planning/story/the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests.md"
  line: 42
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The parent promises that test-xtask and test-doc run under the same 60 s per-test limit and that every test-running step prints its ten slowest tests. t7 gives those two cargo-test tasks only a whole-step timeout ceiling, and nothing records that the per-test limit and the ten-slowest report are not built for them. Add a Decisions line and follow-ups, or move the two tasks to nextest."
- file: ".engineering/waves/wave-2026-11-23.md"
  line: 23
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The parent's end-state promises (the test step within 120 s on a main push with the run id recorded, and D2, the one-integration-binary-per-crate story) are neither built by a unit nor listed as follow-ups here or in the parent's Decisions taken. t7 sets ceilings near 25 minutes. Add one follow-up line to the page and the parent, and align the table row's statement of the parent's outcome."
```
