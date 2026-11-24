---
format: aep.planning-md/3
id: review-result:design-wave-2026-11-23-round-1
kind: review-result
status: active
title: Design critic, round 1, on the wave of 2026-11-23
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
story:the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane — its stamp hashes the `src/` of ess-conformance, ess-gen, ess-http, ess-compiler, ess-domain and every model file, which p1, p2, p3, c1, h1 and h4 also change, yet line 95 exempts only t5 and t7 and says both "the integrator re-stamps once" and "a unit that changes one of those crates re-stamps in its own change"; pick one rule (an edge from this unit to the last of those units, or one re-stamp step in the wave page and in each unit's Gate) — .engineering/planning/story/the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane.md:95
story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose — step 7 re-pins `synthesis-suite-bytes-table.tsv` with `SYNTHESIS_SUITE_BYTES_TABLE_WRITE ... cargo test --test synthesis_suite_bytes_table`, which writes nothing once t2 marks those two tests `#[ignore]`, and its new fixture also moves t2's model records; add `-- --ignored` to the command, or an edge from t2 to this unit so t2 stamps the tree that holds the new rows — .engineering/planning/story/a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose.md:99, .engineering/planning/story/the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane.md:80
story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose — D4 adds the thread-local counter `OWN_OVERLAY_WORK` but names no `STATE` row or `Work` field, so p2's `every_static_of_the_crate_is_classified` fails on it when p2 lands first and a `*_work` bound reads 0 at four jobs; add a step that classifies it as a Counter, and an edge to p2 or the wave page's order — .engineering/planning/story/a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose.md:171, .engineering/planning/story/every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads.md:455
story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan — p1 is declared behaviour-free and testable alone, but `read_all<R: Send>` and its `const _` Sync guard compile only if `Synthesis: Send`, `AdmittedSeeds: Sync` and `EssIr: Sync` hold, which p2 asserts and where a failure would be fixed in the same types; drop the `Send`/`Sync` bounds here and let p3 add them, or add `depends_on` p2 — .engineering/planning/story/every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan.md:185
story:every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads — the design page it owns describes `readings::plan`, the lanes and the arranged-callers stages that only p1 creates (p1 D7 hands the page to p2), so p2 cannot finish its page beside p1 without an edge; add `depends_on` p1 (p1 then carries no p2 dependency) — .engineering/planning/story/every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads.md:375
story:every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads — `Context`, `Work`, `enter`, `capture` and the new accessors have no caller outside tests until p3, so the unit's own `clippy -D warnings` gate fails on `dead_code` and the body names no `cfg(test)` or `allow` that p3 removes; either say so, or move the consumer-less half into p3 — .engineering/planning/story/every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads.md:498
story:a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries — it is two units: the local mechanism (scanner, `[profile.slow]`, `test-slow`, six markers, `MARKED`) that t2 to t5 need, and the CI wiring (`diff` skip step, `slow` job, `Gate` and `release.yml`, six `ci_lanes` tests) that none of them needs; split at that seam so t2 to t5 depend on the first half only — .engineering/planning/story/a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries.md:106
story:a-writer-true-conformance-entry-admits-a-senders-narrowing-of-types-and-value-constraints — `omittable` restates the "imported field reads a missing key" half of w1's `writes_within` table, so the presence rule lives in two units and can drift; derive `omittable` from `writes_within` (or have w1 expose the read side) — .engineering/planning/story/a-writer-true-conformance-entry-admits-a-senders-narrowing-of-types-and-value-constraints.md:52, .engineering/planning/story/a-writer-true-conformance-entry-under-ess-composition-6-decides-presence-by-what-the-sender-may-write.md:94

**What you read**: 20 stories in full (`aep plan artifact show` was replaced by reading each body file under `.engineering/planning/story/`), plus the wave page, `aep plan artifact relations`, `aep plan artifact graph` and `aep plan artifact validate` (reports valid). I walked the 18 `depends_on` edges inside the set and the 20 `derived_from` edges to the five parents, and I read the whole-store graph for edges leaving the set. There is no cycle. About 26 acceptance and gate reads traced to a producer in the set. 18 are recorded by an edge. The 8 unrecorded are the p1/p2 pair (2), c1 to t2 and c1 to p2 (2), p1/p2/p3 reading the byte-pin tests t2 changes (3, handled by "if t2 has landed" text in both orders), and t2's stamp reading the sources of six other units (1).

**What you could not establish**:
- Whether `Synthesis` is `Send` and `AdmittedSeeds` is `Sync`: both units say "believed yes" and I did not build.
- Whether an unused `pub(super)` item under `-D warnings` is a dead-code error in this crate: I read the lints and found no `allow`, but I did not compile.
- Whether the Taskfile and CHANGELOG sharing among t1, t2, t3, t6, t7, p4, p5, w3, c1, h2, h3 and h4 can run at once. That belongs to parallel-safety, and the wave page does record the order.
- Out of my lane: c1 (`chosen_values_arranged_in_bounded_work.rs`) and h1 (`request_limits.rs`) each add a test binary against the organization rule of one integration binary per crate. p3's proof that the corpus is byte-identical at four jobs runs by hand in the slow lane, and CI's slow job never sets `ESS_SYNTH_JOBS`.
- The body-cap units add optional keys with no format version, and the writer units open `ess-composition/6`. Both agree with AGENTS.md § Determinism and formats, since h1 cites the precedent and the `deny_unknown_fields` refusal, and w1 D1 cites the new-major rule. I found no generic registry or property bag. p2's `STATE` is a table of the crate's own statics, not an ESS facet registry.

```findings
- file: .engineering/planning/story/the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane.md
  line: 95
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its stamp hashes the src of ess-conformance, ess-gen, ess-http, ess-compiler, ess-domain and every model file, which p1, p2, p3, c1, h1 and h4 also change, yet line 95 exempts only t5 and t7 and says both that the integrator re-stamps once and that each changing unit re-stamps in its own change; pick one rule (an edge from this unit to the last of those units, or one re-stamp step in the wave page and in each unit's Gate)"
- file: .engineering/planning/story/a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose.md
  line: 99
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "step 7 re-pins synthesis-suite-bytes-table.tsv with SYNTHESIS_SUITE_BYTES_TABLE_WRITE on cargo test --test synthesis_suite_bytes_table, which writes nothing once t2 marks those two tests ignored, and its new fixture also moves t2's model records; add -- --ignored to the command, or an edge from t2 to this unit so t2 stamps the tree that holds the new rows"
- file: .engineering/planning/story/a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose.md
  line: 171
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "D4 adds the thread-local counter OWN_OVERLAY_WORK but names no STATE row or Work field, so p2's every_static_of_the_crate_is_classified fails on it when p2 lands first and a work bound reads 0 at four jobs; add a step that classifies it as a Counter, and an edge to p2 or the wave page's order"
- file: .engineering/planning/story/every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan.md
  line: 185
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "p1 is declared behaviour-free and testable alone, but read_all with an R: Send bound and its const Sync guard compile only if Synthesis is Send, AdmittedSeeds is Sync and EssIr is Sync, which p2 asserts and where a failure would be fixed in the same types; drop the Send and Sync bounds here and let p3 add them, or add depends_on p2"
- file: .engineering/planning/story/every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads.md
  line: 375
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the design page it owns describes readings::plan, the lanes and the arranged-callers stages that only p1 creates (p1 D7 hands the page to p2), so p2 cannot finish its page beside p1 without an edge; add depends_on p1 (p1 then carries no p2 dependency)"
- file: .engineering/planning/story/every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads.md
  line: 498
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Context, Work, enter, capture and the new accessors have no caller outside tests until p3, so the unit's own clippy -D warnings gate fails on dead_code and the body names no cfg(test) or allow that p3 removes; either say so, or move the consumer-less half into p3"
- file: .engineering/planning/story/a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries.md
  line: 106
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "it is two units: the local mechanism (scanner, profile slow, test-slow, six markers, MARKED) that t2 to t5 need, and the CI wiring (diff skip step, slow job, Gate and release.yml, six ci_lanes tests) that none of them needs; split at that seam so t2 to t5 depend on the first half only"
- file: .engineering/planning/story/a-writer-true-conformance-entry-admits-a-senders-narrowing-of-types-and-value-constraints.md
  line: 52
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "omittable restates the imported-field-reads-a-missing-key half of w1's writes_within table, so the presence rule lives in two units and can drift; derive omittable from writes_within (or have w1 expose the read side)"
```
