---
format: aep.planning-md/3
id: review-result:acceptance-wave-2026-11-23-round-2
kind: review-result
status: active
title: Acceptance critic, round 2, on the wave of 2026-11-23
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

story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite — test 13 adds a test marked `slow lane` to `synthesis_suite_bytes_table.rs`, but the body never adds its `MARKED` row or lists `slow_lane.rs` as owned or in scope, so t1a's `every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked` goes red from p3 on, and again in t2's acceptance 7; add the row to Owned files, Steps and Acceptance — .engineering/planning/story/with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite.md:458

story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it — step 0 keeps a probe in the test step at up to 60 s while t3, t4 and t5 hold their targets to 30 s, so a probe measured at 55 s passes acceptance 4 and still takes half of the 120 s step; use the 30 s threshold, or state the step's total for the kept probes — .engineering/planning/story/a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it.md:93

story:the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane — acceptance 4 says `task test-slow` "is green with `ESS_TUTORIAL_NETWORK=1` in its environment" and "fails ... without it", but the task sets the variable itself (Outcome, `env:` of `test-slow`), so "without it" can only be checked by running the ignored test directly; name that command (`cargo nextest run -p ess-cli --test tutorial_page --run-ignored only` with the variable unset) — .engineering/planning/story/the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane.md:122

story:the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step — the step-0 decision rule ends with "the acceptance in item 4 below is then not met and says so in the report", which lets the unit land with its one time acceptance unmet; say instead that the unit does not land and what is reported for the coordinator, as c1 D8 does — .engineering/planning/story/the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step.md:57

Round 1:
1 p2 — fixed — the registry has 74 rows (73 at `80794bbd698` plus `OWN_OVERLAY_WORK`) and the spawn grep uses `-w`; I ran it at the tree and it prints nothing (every-piece-of-thread-state…:540, :550).
2 p1 — fixed — acceptance 6 names both `caller.rs` and `caller/` and prints nothing at the tree. Acceptance 5 covers all of `src/`, including the `made.rs` and `witness_memo.rs` sites that Steps change. Test 2 asserts its own premise (every-caller-reading…:353, :357).
3 p3 — fixed in part — the Outcome is narrowed to what is tested and acceptance 6 names the 14 `*_work.rs` targets, which I counted at 14. The mutation proof now names the green-mutant case. Test 13 adds a marked test without its `MARKED` row (finding 1 above).
4 p4 — fixed — tests 1 and 2 count readings off the calling thread at 2 and 4 jobs (above 0) and at 1 job (0), and read the job count back through `current_jobs` (ess-verify-conform…:254).
5 p5 — fixed — acceptance 2 is decided by counts and identical suites. The 20-120 s range is stated at a load of 60 or less, and the step that raises `--flags` until it holds is at line 163 (cargo-xtask-synth-jobs…:228, :231).
6 t1 (slow lane) — fixed — step 0 names a 60 s threshold, and each half has a command-decided acceptance. The 60 s threshold itself is finding 2.
7 t2 — fixed — one stamp owner, the 30 s figure is for a warm run, and acceptances 3 and 4 no longer contradict each other (the-synthesis-byte-pin…:115-117).
8 t3 — fixed — one 30 s threshold and exact skip-line text, plus a `Toolchains::Skipped` unit test. Acceptance 4 still has the ambiguity in finding 3 (the-tutorial-walkthrough…:119-121).
9 t4 — fixed — the bound states warm build, load of 60 or less and the runner's `junit.xml` figure (the-generated-view-queries…:108).
10 t5 — fixed — the closing rule marks the slowest test until the target is within 30 s, and acceptance 1 says so (three-adversary-targets…:113).
11 t6 — fixed in part — the cold-cache escape is gone and warm is shown by the first command printing no `Compiling` line. The step-0 stop clause is finding 4.
12 t7 — fixed — `the_step_handler_exits_124_with_a_message_naming_the_task_and_passes_every_other_status_on` runs the handler on `( exit 124 )`, on `( exit 3 )` and on a real `timeout 1 sleep 5` (the-runner-kills-a-test…:124).
13 c1 — fixed — one outcome statement (0 refusals, no leg left over). Test 6b asserts `asked_at_the_bound >= 1` before the refusals, and fails if no model asks at depth 2. Acceptances 4 and 5 agree: only the three tables are re-pinned. Step 4a states the fallback for B (the unit does not land, and A is an operator decision) (a-setup-run…:159-163, :164).
14 h1 — fixed — acceptance 3 names `a_limit_is_a_whole_number_of_bytes_from_one_to_the_ceiling` in `route_refusals_and_view_answers`, and the CLI is no longer an acceptance (an-ess-http-binding…:137).
15 h2 — fixed — the 5 % timing bound is replaced by socket-level cases that need no clock (0 body bytes read, 65 against 64, 2 MiB route cap). Acceptance 6 still says "at the load of the Gate" and does not give a load, but the unit's only time check is a report line (the-generated-rust-server…:163, :167).
16 h3 — fixed — the same, plus the parity case 3c and the `ci_lanes` test over both Go set-up steps (the-generated-go-server…:223, :225, :227).
17 h4 — fixed — acceptance 7 counts projected output bytes and `413` responses instead of timing (the-openapi-projection…:137).

What I read: all 21 draft units of the wave, the Outcome, Tests first, Steps, Owned files and Acceptance sections through `aep plan artifact show`, `aep plan artifact waves` and file reads. I read the brief, my round-1 report, the routing file and the five author answers, and checked the typed `depends_on` edges of every unit: no cycle and no stranded unit. I ran `git grep` on p1's and p2's greps and on `Rc<EssIr>`, counted the `*_work.rs` targets (14), and checked `enum_presence_guard` and the Taskfile wrapper lines. The archived slow-lane draft and the two mutation-audit drafts were not judged.

What I could not establish:
- Whether p2's module-level `allow(dead_code)` keeps its accessors live under `-D warnings`.
- The measured times: the six probes (t1a), `docs::` (t6), the ceilings (t7) and test 13's runtime (p3).
- Whether c1's test-6b model asks a chosen-values run at depth 2; the body gives its own stop for that case.
- Whether the inline writer models of w1 and w2 validate; no build was allowed.
- The host's 1-minute load for any acceptance that names one.

Out of lane, not setting the verdict:
- t7's ceilings (longest seen plus 20 %, or 1500 s for `test-shard`) sit far above the 120 s step end state its Outcome names: scope.
- t1b's `slow` job compiles its own targets on a hosted runner within `timeout-minutes: 30`, with no measure of that build: design.

```findings
- file: .engineering/planning/story/with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite.md
  line: 458
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "test 13 adds a test marked slow lane to synthesis_suite_bytes_table.rs but the body never adds its MARKED row nor lists slow_lane.rs as owned, so t1a's every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked goes red from this unit on; add the row to Owned files, Steps and Acceptance"
- file: .engineering/planning/story/a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it.md
  line: 93
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "step 0 keeps a probe in the test step at up to 60 s while t3, t4 and t5 hold their targets to 30 s, so a probe measured at 55 s passes acceptance 4 and still takes half of the 120 s step; use the 30 s threshold or state the total the kept probes add to the step"
- file: .engineering/planning/story/the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane.md
  line: 122
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "acceptance 4 says task test-slow fails without ESS_TUTORIAL_NETWORK=1 but the task sets the variable itself, so the failure can only be seen by running the ignored test directly; name that command with the variable unset"
- file: .engineering/planning/story/the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step.md
  line: 57
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the step-0 decision rule ends with acceptance 4 being not met and saying so in the report, which lets the unit land with its one time acceptance unmet; say that the unit does not land and what it reports for the coordinator, as c1 D8 does"
```
