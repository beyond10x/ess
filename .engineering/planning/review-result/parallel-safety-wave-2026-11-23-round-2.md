---
format: aep.planning-md/3
id: review-result:parallel-safety-wave-2026-11-23-round-2
kind: review-result
status: active
title: Parallel safety critic, round 2, on the wave of 2026-11-23
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

story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it — p3's slow-lane test 13 carries t1a's marker, and t1a's test `every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked` scans the real tree against `MARKED`, but t1a lists only t2, t3, t4 and t5 as row-adders and p3 neither owns nor mentions `slow_lane.rs`, so `slow_lane` goes red when p3 lands and p3's `ess-conformance`-only Gate does not show it; the row must be owned by one of the two bodies, and p3's typed scope gains the file if it is p3 (cited, semantic: p3 does not edit the file, so a merge hides the red) — .engineering/planning/story/a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it.md:83, :116

story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite — its steps remove every per-item `allow` that p2 placed with `git grep -n 'p3 removes'`, but its Owned files name only `threads.rs` for that removal, while p2's fallback may leave such an allow on an accessor in any of the 24 other files p2 edits; the removal there is outside every stated surface (cited) — .engineering/planning/story/with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite.md:213, :397-406, .engineering/planning/story/every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads.md:535

story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan — p1 and c1 are both round 1, p1's typed scope lists the two model tables and its `## Not yours` says `related_guard.rs` is c1's with "no shared file", yet c1 moves a row of the table whose byte proof p1 runs, and the wave page's "Model tables" bullet says p1, p2 and p3 re-pin tables that their bodies leave untouched; neither body says that the second of p1 and c1 to land reruns the byte-pin targets on the merged tree (cited, semantic only; textually p1 edits no table file) — .engineering/planning/story/every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan.md:294, :318, .engineering/waves/wave-2026-11-23.md:68

Round 1:
1 t2 byte-pin stamp — fixed — `depends_on` edges to t1a, p3, c1, h1 and h4; h2 and h3 left out on the right ground (`ess-synth` and `ess-composition` are not in the eight crates' dependency closure), and no other unit runs the digest test (t2 :95, :126).
2 c1 against p2 — fixed — p2 depends on c1 and p1, and both bodies say p2 classifies `OWN_OVERLAY_WORK`.
3 p4 against w3 — fixed — edge p4 to w3; p4 says it regenerates `cli.md`, never merges it, and keeps both bullets.
4 h3 against the hotfix Go set-up — fixed — h3 owns both `setup-go` steps (`ci.yml` `build-tests` and `release.yml` `hotfix-tests`) and one `ci_lanes` test for both; t7 no longer edits that job.
5 t7 against h2 and h3 on `Taskfile.yml` — fixed — t7 depends on h2 and h3 and names them.
6 h3 `ci_lanes.rs` and `ci.yml` — fixed — t1b, t3, t6, t7 and h3 name each other. t1b "lands before t3" is prose only, but t3 states it does not call the changed `run_gate`, so it is safe.
7 t6 typed `main.rs` — fixed — the typed scope now equals the Owned files.
8 t4 typed `no_test_builds.rs` — fixed — typed scope and Owned files agree, `MARKED` rows included.
9 t1 Owned files — fixed — split into t1a and t1b; the Owned files of each match its typed scope, `main.rs` and `release.yml` included.
10 h4 typed `ess-gen/src/http.rs` — fixed — it is in the typed scope.

The four placements the page keeps side by side, as written:
- `CHANGELOG.md`, 12 writers: safe. Every body says "keep both bullets" and names its heading (the `### Added`, `### Changed` and `### Fixed` headings all exist under `[Unreleased]`). The conflict is textual only, at one insertion point.
- Model tables, c1 against p1 (and p2, p3): safe for tables, because only c1 re-pins and p2 and p3 are ordered after it by edge; finding 3 covers the one open point. Anyone who re-pins a table again regenerates, never merges.
- `ess-synth` HTTP files, h2 and h3 against h4: safe, no edge needed. h4 reads `rust/http.rs` and `go/http.rs` only as pins (h4 :103), and h2 and h3 state that they do not edit the default `413` text (`HTTP` const and `readBody` stay), so the store's overlap is a read pin.
- `Taskfile.yml`, `AGENTS.md`, `ci_lanes.rs` and `ci.yml`, t3 against h2 and h3: safe. t3 edits the `test-slow` task, `AGENTS.md` 144-146 and the `test` job, while h2 and h3 edit `test`, `test-packages`, `test-shard` and `AGENTS.md` 234-237 (distant hunks), and h3 adds a `build-tests` step. The `ci_lanes.rs` tail appends by t1b, t3, t6, h3 and t7 all say "keep both". No edge is missing.

What I read: 21 units (`aep plan artifact show`-equivalent bodies, typed scope and `depends_on` from the store files), the wave page, the brief, the routing decisions, the store's `waves` derivation, and `CHANGELOG.md` and `Taskfile.yml` in the tree. Surfaces: 21 cited, 0 inferred, 0 unplaced. Acceptance reads traced to a producer in the set: 32 recorded by `depends_on` edges and 1 not (p1's byte proof against the table rows c1 moves, finding 3). No cycle and no stranded unit: t1b feeds t7, t6 feeds t7, p4 and p5 follow p3.

What I could not establish: whether the `-D warnings` build stays clean under the single module-level `allow(dead_code)` of p2 (the page lists it as open); whether `ess-composition` can enter the digest's crate closure later (checked from manifests today only).

```findings
- file: .engineering/planning/story/with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite.md
  line: 334
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "p3's slow-lane test 13 carries t1a's marker and t1a's test every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked scans the real tree against MARKED, but t1a lists only t2, t3, t4 and t5 as row-adders and p3 neither owns nor mentions crates/edge/ess-xtask/tests/slow_lane.rs, so slow_lane goes red when p3 lands and p3's ess-conformance-only Gate does not show it (cited, semantic: p3 does not edit the file, so a merge hides the red); one of the two bodies must own the row, and p3's typed scope gains the file if it is p3"
- file: .engineering/planning/story/with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite.md
  line: 213
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its steps remove every per-item allow that p2 placed (git grep -n 'p3 removes'), but its Owned files name only threads.rs for that removal, while p2's fallback may leave such an allow on an accessor in any of the 24 other files p2 edits, so the removal there is outside every stated surface (cited)"
- file: .engineering/planning/story/every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan.md
  line: 318
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "p1 and c1 are both round 1, p1's typed scope lists the two model tables and its Not yours says related_guard.rs is c1's with no shared file, yet c1 moves a row of the table whose byte proof p1 runs and the wave page's Model tables bullet says p1, p2 and p3 re-pin tables their bodies leave untouched; neither body says the second of p1 and c1 to land reruns the byte-pin targets on the merged tree (cited, semantic only; textually p1 edits no table file)"
```
