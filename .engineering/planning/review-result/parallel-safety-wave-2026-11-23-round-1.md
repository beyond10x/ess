---
format: aep.planning-md/3
id: review-result:parallel-safety-wave-2026-11-23-round-1
kind: review-result
status: active
title: Parallel safety critic, round 1, on the wave of 2026-11-23
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

story:the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane — its stamp hashes `src/` of eight crates and every `format: ess/` model, so p1, p2, p3, c1 (new `.yaml` fixtures), h1 (`ess-http`) and h4 (`ess-gen`) each turn its fast digest test red and each would re-write the same one-line file, and only this body (not those six, not the page) says the integrator re-stamps once (cited, textual conflict on the stamp line and semantic, since the second must regenerate) — .engineering/planning/story/the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane.md:75, :130

story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose — c1 adds the static `OWN_OVERLAY_WORK` (D4), which p2's test `every_static_of_the_crate_is_classified` reports unless p2's `threads.rs` classifies it (and, as a counter, adds it to `Work` with two accessors); p2 says "whichever lands second classifies" but c1's body, the page and the round-1 table give no order and c1 does not own `threads.rs` (cited, semantic: textually the hunks are disjoint) — .engineering/planning/story/a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose.md:171, .engineering/planning/story/every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads.md:454-465

story:ess-verify-conform-synthesize-and-run-take-jobs-and-the-bytes-are-the-same-at-every-job-count — p4 and w3 are both round 3 and both edit `ess-cli/src/main.rs` (different lines), regenerate `website/docs/reference/cli.md` and add a `CHANGELOG.md` bullet, yet p4 names only `m1` (not in the wave) and the page says "no conflict arises here" (cited, semantic on the generated `cli.md`: the second must rerun `cargo xtask cli-reference`, textual on `CHANGELOG.md`) — .engineering/planning/story/ess-verify-conform-synthesize-and-run-take-jobs-and-the-bytes-are-the-same-at-every-job-count.md:177-181, .engineering/planning/story/the-documents-and-help-describe-the-writer-true-conformance-entry-and-ess-composition-6.md:166-167, .engineering/waves/wave-2026-11-23.md:74

story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route — h3 wires `body-cap-go-server` into `test-packages`, which release.yml's `hotfix-tests` runs and which has no `actions/setup-go`, while h3's new `ci_lanes.rs` test and step cover `ci.yml` jobs only and t7 (round 3, no edge from h3) owns that job's setup steps, so a hotfix gate can run the Go task without Go; the body does not say who adds the step or in what order (cited) — .engineering/planning/story/the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md:65-68, .engineering/planning/story/the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests.md:59, :94

story:the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests — t7 puts the `timeout` prefixes and `STEP_SECONDS` on `test`, `test-packages` and `test-shard` in `Taskfile.yml` and says it lands last, but h2 and h3 (round 2, other group) add `- task:` lines and `env:` entries to the same three tasks, and neither t7's Shared list nor the page's cap-group list names the other (cited, textual in adjacent lines of the same task bodies; the `AGENTS.md` 234-237 lines are clean beside t1/t2/t3) — .engineering/planning/story/the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests.md:57, :111, .engineering/planning/story/the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md:61, :123

story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route — h3's one `ci_lanes.rs` test and its `build-tests` step share `ci_lanes.rs` and `ci.yml` with t1, t3, t6 and t7, all of which append tests at the end of `ci_lanes.rs`, and none of the five bodies names h3 (cited, textual and trivially resolved by keeping both, but unnamed) — .engineering/planning/story/the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md:67-68, .engineering/planning/story/a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries.md:136

story:the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step — its typed scope lists `crates/edge/ess-xtask/src/main.rs`, but the body edits nothing there (it only notes `mod docs;` at line 7), which makes the store report false collisions with t1 and p5 on that file (typed scope against body, cited) — .engineering/planning/story/the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step.md:20, :47, :77

story:the-generated-view-queries-target-keeps-generation-in-the-test-step-and-compiles-in-the-slow-lane — its typed scope lists `crates/edge/ess-xtask/tests/no_test_builds.rs` though step 4 says to leave that file unchanged, and its Owned files omit the `MARKED` row in `slow_lane.rs` that its Shared line says it adds (typed scope against body, cited) — .engineering/planning/story/the-generated-view-queries-target-keeps-generation-in-the-test-step-and-compiles-in-the-slow-lane.md:17, :80

story:a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries — its Owned files omit `crates/edge/ess-xtask/src/main.rs` (the `SlowList` variant), `release.yml` (comment), `AGENTS.md` 220-222 and `CHANGELOG.md`, all of which its Where and Steps edit and which p5, t2 and t7 depend on (Owned files against Where, cited) — .engineering/planning/story/a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries.md:128, :67, :73, :78

story:the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets — its Owned files name `crates/generate/ess-gen/src/http.rs` (the one constant) but its typed scope does not, so the store cannot see that surface (body against typed scope, cited) — .engineering/planning/story/the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets.md:86

What I read: all 20 `wave-2026-11-23` drafts (full `## Where`, `## Owned files`, `## Not yours` and shared-files text, plus typed `scope` and `depends_on` from the frontmatter), the wave page, and `aep plan artifact waves --kind story --status draft --format json`, filtered to the 20 for collisions and unassessed items. Surfaces: 20 cited, 0 inferred, 0 unplaceable (none of the 20 is in the store's `unassessed`). Acceptance reads traced to a producer in the set: 27, of which 18 have a `depends_on` edge (t2 to t5 on t1, t7 on t2 to t6, p3 on p1 and p2, p4 and p5 on p3, w2 on w1, w3 on w2, h2 to h4 on h1) and 9 have none (t2 reads the sources p1, p2, p3, c1, h1 and h4 change; p3 runs the byte-pin target t2 changes; p2's static census reads the statics c1 and p3 add).

Collisions that are real but covered by the stated order or by "keep both", so not findings:
- `CHANGELOG.md`, 13 units: textual, one bullet each.
- `slow_lane.rs` `MARKED`, t2 to t5 in round 2: textual only if the sorted rows are adjacent.
- `ci_lanes.rs` tails.
- t1 before p5 on `Taskfile.yml`.
- t1, t2 and t3 on `AGENTS.md`: different paragraphs.
- Writer chain on `conformance.rs`, `lib.rs` and `references`: strictly serial by edge.
- h1 then h4 on `openapi.rs` and the design page: edge, disjoint hunks.
- h2 then h3 on `Taskfile.yml`, `AGENTS.md` 234-237 and the `http_binding_rust.rs` module line: textual and certain, ordered in both bodies but by prose only, with no h3-to-h2 edge in the store.

Collisions that merge cleanly and are unnamed, so left out of the findings:
- `formats.md` for h1 (line 228), w1 (after 195) and w3.
- `docs.rs` for w1 (one row) and t6 (`mod tests`).
- p2 against c1 and p3 in `synthesize.rs` and `related_guard.rs` (different functions).

The only table re-pin in the set is c1's three tables, so no two units re-pin one table. No round must move wholesale. If an order is chosen, the pairs that cannot both land blind in their round are c1 and p2 (round 1), t2 with p3 and h4 (round 2) and p4 with w3 (round 3).

What I could not establish:
- whether `hotfix-tests` ends up with Go through the hosted image's own toolchain; I only read that no `setup-go` step is named.
- whether p2's static census regex matches c1's `OWN_OVERLAY_WORK`, which the c1 body does not show as `thread_local!` or a plain static; I read p2's census rule only.
- t2's step 2 `cargo tree` check may add crates to the eight, which would widen the list of units that move the stamp.
- the correction unit `unit/w1122-s3fix`, which c1 waits on, is outside the set and I did not assess it.
- p2's 26 typed files each get only new accessors per its body, so I judged that narrowed rather than too wide. Whether the split is right is outside my lane.

```findings
- file: .engineering/planning/story/the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane.md
  line: 130
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its stamp hashes src/ of eight crates and every format: ess/ model, so p1, p2, p3, c1 (new yaml fixtures), h1 (ess-http) and h4 (ess-gen) each turn its fast digest test red and each would re-write the same one-line file, and only this body (not those six, not the page) says the integrator re-stamps once (cited, textual conflict on the stamp line and semantic, since the second must regenerate)"
- file: .engineering/planning/story/a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose.md
  line: 171
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "c1 adds the static OWN_OVERLAY_WORK (D4), which p2's test every_static_of_the_crate_is_classified reports unless p2's threads.rs classifies it and adds it to Work with two accessors; p2 says whichever lands second classifies but c1's body, the page and the round-1 table give no order and c1 does not own threads.rs (cited, semantic: textually the hunks are disjoint)"
- file: .engineering/planning/story/ess-verify-conform-synthesize-and-run-take-jobs-and-the-bytes-are-the-same-at-every-job-count.md
  line: 177
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "p4 and w3 are both round 3 and both edit ess-cli/src/main.rs (different lines), regenerate website/docs/reference/cli.md and add a CHANGELOG.md bullet, yet p4 names only m1 (not in the wave) and the page says no conflict arises here (cited, semantic on the generated cli.md since the second must rerun cargo xtask cli-reference, textual on CHANGELOG.md)"
- file: .engineering/planning/story/the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md
  line: 65
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "h3 wires body-cap-go-server into test-packages, which release.yml's hotfix-tests runs and which has no actions/setup-go, while h3's new ci_lanes test and step cover ci.yml jobs only and t7 (round 3, no edge from h3) owns that job's setup steps, so a hotfix gate can run the Go task without Go; the body does not say who adds the step or in what order (cited)"
- file: .engineering/planning/story/the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests.md
  line: 111
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "t7 puts timeout prefixes and STEP_SECONDS on test, test-packages and test-shard in Taskfile.yml and says it lands last, but h2 and h3 (round 2, other group) add task lines and env entries to the same three tasks and neither t7's Shared list nor the page's cap-group list names the other (cited, textual in adjacent lines of the same task bodies)"
- file: .engineering/planning/story/the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md
  line: 67
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "h3's one ci_lanes.rs test and its build-tests step share ci_lanes.rs and ci.yml with t1, t3, t6 and t7, which append tests at the end of ci_lanes.rs, and none of the five bodies names h3 (cited, textual and trivially resolved by keeping both, but unnamed)"
- file: .engineering/planning/story/the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step.md
  line: 20
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its typed scope lists crates/edge/ess-xtask/src/main.rs but the body edits nothing there (it only notes mod docs; at line 7), which makes the store report false collisions with t1 and p5 on that file (typed scope against body, cited)"
- file: .engineering/planning/story/the-generated-view-queries-target-keeps-generation-in-the-test-step-and-compiles-in-the-slow-lane.md
  line: 17
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its typed scope lists crates/edge/ess-xtask/tests/no_test_builds.rs though step 4 says to leave that file unchanged, and its Owned files omit the MARKED row in slow_lane.rs that its Shared line says it adds (typed scope against body, cited)"
- file: .engineering/planning/story/a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries.md
  line: 128
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its Owned files omit crates/edge/ess-xtask/src/main.rs (the SlowList variant), release.yml (comment), AGENTS.md 220-222 and CHANGELOG.md, all of which its Where and Steps edit and which p5, t2 and t7 depend on (Owned files against Where, cited)"
- file: .engineering/planning/story/the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets.md
  line: 86
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its Owned files name crates/generate/ess-gen/src/http.rs (the one constant) but its typed scope does not, so the store cannot see that surface (body against typed scope, cited)"
```
