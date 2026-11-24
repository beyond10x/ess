---
format: aep.planning-md/3
id: review-result:acceptance-wave-2026-11-23-round-1
kind: review-result
status: active
title: Acceptance critic, round 1, on the wave of 2026-11-23
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

story:every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads — the body says the registry has 70 rows but its listed rows sum to 72 (`git grep` of the static pattern gives 72 at `ff7f8e74fb`, 73 at HEAD), and acceptance 6's `spawn` grep also matches `respawn` in `process.rs` and `process/tests.rs`, so neither check can be decided as written — .engineering/planning/story/every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads.md:509

story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan — acceptance 6's grep path leaves out `synthesize/caller.rs`, where the `Rc<EssIr>` and `thread_local!` sites are (lines 358, 391 and 3532), so it passes while the thing it guards is still there. Test 2's claim that `AddTag` is already mixed between actors is not established; the `NOTES` model at `caller.rs:3541-3600` already gives it to two actors, so the red-today reason is doubtful — .engineering/planning/story/every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan.md:334

story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite — the Outcome claims byte identity for every `*_work.rs` bound (about 15 targets) and every mutant, but acceptance 4 and tests 8-9 cover only a subset, and the mutation proof names no fallback when a mutant does not turn the suite red — .engineering/planning/story/with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite.md:349

story:ess-verify-conform-synthesize-and-run-take-jobs-and-the-bytes-are-the-same-at-every-job-count — no test shows that the `--jobs` flag or `ESS_SYNTH_JOBS` reaches `with_jobs`, because a byte-equality test passes at one job whether or not the flag is wired, so the acceptance cannot tell a wired flag from an ignored one; add a counter or thread-count observation at jobs 2 and 4 — .engineering/planning/story/ess-verify-conform-synthesize-and-run-take-jobs-and-the-bytes-are-the-same-at-every-job-count.md:205

story:cargo-xtask-synth-jobs-measures-one-synthesis-at-one-two-and-four-jobs-and-refuses-a-differing-suite — acceptance 2's 20-120 s wall-time range names no load condition, and `/proc/loadavg` read 64.74 on this host while I judged, above the 60 at which a budget counts; state the 1-minute load at which it holds or decide it by work counters instead — .engineering/planning/story/cargo-xtask-synth-jobs-measures-one-synthesis-at-one-two-and-four-jobs-and-refuses-a-differing-suite.md:207

story:a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries — acceptance 2 rests on probe times nobody has measured (the six probes now run in CI for the first time), and the step-0 measure-then-decide rule names no threshold, so it cannot say when the lane is kept as is and when a probe is split — .engineering/planning/story/a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries.md:153

story:the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane — the 30 s bound is stated for a run that includes compilation, so it passes or fails with the cache state, and acceptances 3 and 4 contradict each other on when a pin is re-stamped; fix the bound to a warm run and make one acceptance own the re-stamp — .engineering/planning/story/the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane.md:110

story:the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane — the 30 s and 45 s thresholds disagree between Outcome and Acceptance, and "or by the reasons printed" leaves the npm/go skip check undecided; name one threshold and one observable (the skip line text) — .engineering/planning/story/the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane.md:116

story:the-generated-view-queries-target-keeps-generation-in-the-test-step-and-compiles-in-the-slow-lane — the 30 s bound has no load condition, so a run at a 1-minute load above 60 decides nothing; state the load at which it is measured — .engineering/planning/story/the-generated-view-queries-target-keeps-generation-in-the-test-step-and-compiles-in-the-slow-lane.md:106

story:three-adversary-targets-run-a-reduced-sweep-in-the-test-step-and-the-full-sweep-in-the-slow-lane — the "stop" branches of the measure-then-decide rule leave a target over 30 s, which contradicts acceptance 1 that every target is within it; the acceptance has to name what happens to such a target (reduce further or mark slow) — .engineering/planning/story/three-adversary-targets-run-a-reduced-sweep-in-the-test-step-and-the-full-sweep-in-the-slow-lane.md:109

story:the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step — acceptance 4 has an escape hatch that lets the step exceed 120 s when the cache is cold, so the 120 s step budget is not decided; make the bound hold on a warm cache and name how the cache state is shown — .engineering/planning/story/the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step.md:97

story:the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests — the exit-124 handler is only matched as text, so the hand check never makes the handler run, and nothing shows a test over 60 s is killed and counted failed; add a case that runs the handler on exit 124 — .engineering/planning/story/the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests.md:125

story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose — decision D8 and Step 3 contradict the Outcome's 0 refusals and test 3, test 6b can pass vacuously on the speculative model when no row is chosen, and acceptances 4 and 5 disagree on whether the committed pins are re-pinned; the parent recommends a different option than B, so name the fallback if B's model does not exercise `fixed` — .engineering/planning/story/a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose.md:154

story:an-ess-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets — acceptance 3 runs through the CLI, which no gate builds, so a gate run cannot decide it; name a test target that holds it — .engineering/planning/story/an-ess-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets.md:123

story:the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route — the 5 % timing bound names no noise margin or load condition and cannot be decided on a shared host; replace it with a count of bytes read — .engineering/planning/story/the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md:146

story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route — the 5 % timing bound names no noise margin or load condition and cannot be decided on a shared host; replace it with a count of bytes read — .engineering/planning/story/the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md:197

story:the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets — the 5 % timing bound on the projection run names no noise margin or load condition; decide it by the projected output bytes instead — .engineering/planning/story/the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets.md:115

What I read: all 20 stories (t1-t7, p1-p5, w1-w3, c1, h1-h4) in full, plus the wave page `.engineering/waves/wave-2026-11-23.md`, through `aep plan artifact list` and `show` and file reads, and I ran `git grep` against the tree for the symbols, paths and counts they name.

What I could not establish:
- p1's claim that `AddTag` is mixed between actors.
- p3's count of 50.
- Whether the inline writer models in w1 and w2 validate.
- c1's measured numbers.
- The host load at the moment of each acceptance's measurement. `/proc/loadavg` read 64.74 / 77.82 / 92.70 at judgment time.
- Citation line numbers are the `## Acceptance` heading lines of each file, not the exact sentence.
- w1, w2 and w3 raised no finding that survived my checks (w1 and w3 table counts and the w3 grep and line numbers matched the tree).

Out of lane, not setting the verdict: t2's digest stamp touches `ess-conformance/src`, which other units in the wave also change, so that is parallel-safety. t7's ceilings sit far above the 120 s step budget, so that is scope.

```findings
- file: .engineering/planning/story/every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads.md
  line: 509
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the body says the registry has 70 rows but its listed rows sum to 72, and acceptance 6's spawn grep also matches respawn in process.rs, so neither check can be decided as written"
- file: .engineering/planning/story/every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan.md
  line: 334
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "acceptance 6's grep path leaves out synthesize/caller.rs where the Rc<EssIr> and thread_local! sites are, and test 2's claim that AddTag is mixed between actors is not established because the NOTES model already gives it to two actors"
- file: .engineering/planning/story/with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite.md
  line: 349
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the Outcome claims byte identity for every *_work.rs bound and every mutant but acceptance 4 and tests 8-9 cover a subset, and the mutation proof names no fallback when a mutant leaves the suite green"
- file: .engineering/planning/story/ess-verify-conform-synthesize-and-run-take-jobs-and-the-bytes-are-the-same-at-every-job-count.md
  line: 205
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "no test shows that --jobs or ESS_SYNTH_JOBS reaches with_jobs, because byte equality passes at one job whether or not the flag is wired; add a thread-count or counter observation at jobs 2 and 4"
- file: .engineering/planning/story/cargo-xtask-synth-jobs-measures-one-synthesis-at-one-two-and-four-jobs-and-refuses-a-differing-suite.md
  line: 207
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "acceptance 2's 20-120 s wall-time range names no 1-minute load at which it holds, and the host read 64.74 at judgment time, above the 60 at which a budget counts"
- file: .engineering/planning/story/a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries.md
  line: 153
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "acceptance 2 rests on unmeasured probe times and the step-0 measure-then-decide rule names no threshold, so it cannot say when the lane stays as is and when a probe is split"
- file: .engineering/planning/story/the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane.md
  line: 110
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the 30 s bound includes compilation so it depends on cache state, and acceptances 3 and 4 contradict each other on when a pin is re-stamped"
- file: .engineering/planning/story/the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane.md
  line: 116
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the 30 s and 45 s thresholds disagree and 'or by the reasons printed' leaves the npm and go skip check undecided; name one threshold and the skip line text"
- file: .engineering/planning/story/the-generated-view-queries-target-keeps-generation-in-the-test-step-and-compiles-in-the-slow-lane.md
  line: 106
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the 30 s bound has no load condition, so a run at a 1-minute load above 60 decides nothing"
- file: .engineering/planning/story/three-adversary-targets-run-a-reduced-sweep-in-the-test-step-and-the-full-sweep-in-the-slow-lane.md
  line: 109
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the stop branches of the measure-then-decide rule leave a target over 30 s, which contradicts acceptance 1; name what happens to such a target"
- file: .engineering/planning/story/the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step.md
  line: 97
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "acceptance 4 has an escape hatch that lets the step pass over 120 s on a cold cache, so the step budget is not decided"
- file: .engineering/planning/story/the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests.md
  line: 125
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the exit-124 handler is only matched as text, so the hand check never runs it; add a case that runs the handler on exit 124"
- file: .engineering/planning/story/a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose.md
  line: 154
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "D8 and Step 3 contradict the 0-refusals outcome and test 3, test 6b can pass vacuously, and acceptances 4 and 5 disagree on re-pinning; name the fallback when option B's model does not exercise fixed"
- file: .engineering/planning/story/an-ess-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets.md
  line: 123
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "acceptance 3 runs through the CLI, which no gate builds, so a gate run cannot decide it; name a test target that holds it"
- file: .engineering/planning/story/the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md
  line: 146
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the 5 % timing bound names no noise margin or load condition; replace it with a count of bytes read"
- file: .engineering/planning/story/the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md
  line: 197
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the 5 % timing bound names no noise margin or load condition; replace it with a count of bytes read"
- file: .engineering/planning/story/the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets.md
  line: 115
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the 5 % timing bound on the projection run names no noise margin or load condition; decide it by the projected output bytes"
```
