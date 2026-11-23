---
format: aep.planning-md/3
id: review-result:design-wave-2026-11-23-round-2
kind: review-result
status: active
title: Design critic, round 2, on the wave of 2026-11-23
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

- story:every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads — p2 types the chosen-values counter `OWN_OVERLAY_WORK` as `Cell<u64>` with `taken_own_overlay_work`/`add_own_overlay_work`, and c1 defines it as `Cell<(u64, u64)>`, a pair `(built, asked_at_the_bound)`. An implementor rebuilds the registry row, the accessor pair and the `Work` field(s) on the wrong shape, or merges the pair into one sum and loses what test 6b reads. The "74 rows" and the "single counter" wording (line 272) depend on which shape wins — p2 lines 179 and 272, c1 `story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose` D4 line 174 and line 95.
- story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite — p3 adds test 13 under the slow marker, but `MARKED` gets no row for its file, and `crates/edge/ess-xtask/tests/slow_lane.rs` is not in p3's scope. t1a's `every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked` goes red when the marker lands, and t1a names only t2 to t5 as adding rows. Test 13 also keeps a branch on "if t1a defines a per-test bound in the slow lane", which t1a has already settled: the slow profile kills at 20 minutes and the 60 s bound does not apply. Drop the branch — p3 lines 336-351 and scope lines 15-40, t1a `story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it` lines 116 and 152.
- story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route — Test 5b starts h2's Rust server and the Go server and compares their answers. It needs the two fixture copies to be the same bytes (lines 153 and 171), and nothing pins that. The reason given for two copies, "the two units are built at the same time", is false now that h3 depends on h2. A drifted copy makes the parity case fail on a model mismatch instead of a server difference. Read h2's pair from h3, or add an assertion that the two copies are byte-equal — h3 lines 153, 171 and 239, h2 `story:the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route` line 179.

Round 1:
1. t2 stamp edge and the four hashed-source units — fixed. t2's edges are h1, c1, t1a, h4 and p3, and the writer-side units sit in a crate (`ess-composition`) that the stamp does not hash. t2 line 95 gives one stamp rule, line 126 says `ess-synth` is not hashed, and the store's `depends_on` graph for all 21 units has no cycle and no unneeded edge.
2. c1 re-pin — fixed. c1 lines 127-128 and 155 touch no stamp and re-pin with today's commands.
3. c1 counter classification — fixed in part. p2 now classifies it (p2 line 99), but its shape disagrees with c1 D4; see the first finding.
4. p1 `Send`/`Sync` bounds — fixed. p1 lines 41, 195 and 202 add no bound; p3 adds them (p3 lines 123-134).
5. p2 design page and edge — fixed. p2 depends on c1 and p1, and p1 D7 hands the page to p2.
6. p2 dead code — fixed. The `allow(dead_code)` is a named allow marked "p3 removes", with a fallback and an acceptance grep; p3 lines 213-216 and 473 remove it.
7. t1 split — fixed. t1a is the local mechanism and changes no workflow file. t1b adds the `slow` job, the `check` step and `Gate`'s edit, and depends on t1a only. t1b's `skippable` and `needed` lists fail safe, and a skipped `slow` run cannot stand in for a release.
8. w2 `omittable` — fixed. `omittable` is derived from `absent_read`, and acceptance 8 greps that the diff adds no line naming `Presence::` (w2 lines 54 and 216).

What you read: the wave page; the store's `depends_on` edges of all 21 units, read from the story files with shell tools rather than `aep plan artifact show`; the bodies of t1a, t1b, t2, p1, p2, p3 (selected parts), c1, w1, w2, h1, h2, h3 and h4; the scope and edges of p4, p5, t3 to t7 and w3; the coordinator's routing note and the authors' answers. The OpenAPI unit h4 is judged only on its form: it keeps the stage-1 default `413` in `responses()` and the bound form through `bound_responses`, with `x-ess-request-max-bytes`, and that form is coherent. The presence-rule ownership is one: w1 owns `absent_written`, `absent_read` and `writes_within`, and w2 derives from them. The request-cap lookup is shared only through h1's accessors.

What you could not establish: I did not read the bodies of t3 to t7, p4 and p5 line by line, so a shared-file order gap inside them could remain (only their edges and scope were checked). The `Cargo.toml` dependency lists I read confirm the eight hashed crates do not depend on `ess-composition`, but I could not confirm that no other path feeds the digest. Whether the `slow` job fits its 30 minutes is unknowable without a first CI run.

```findings
- file: ".engineering/planning/story/every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads.md"
  line: 179
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "OWN_OVERLAY_WORK is typed Cell<u64> with taken_own_overlay_work/add_own_overlay_work here (lines 179, 272) but is Cell<(u64, u64)> returning (built, asked_at_the_bound) in c1 D4 (line 174). Registry row, accessors, Work field(s) and the 74-row count must follow c1's pair."
- file: ".engineering/planning/story/with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite.md"
  line: 350
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "Test 13 is marked slow but p3 adds no MARKED row and does not list crates/edge/ess-xtask/tests/slow_lane.rs in scope, so t1a's marker-equals-table test goes red; t1a names only t2 to t5 as adding rows. Add the row and the scope entry, and drop the 'if t1a defines a per-test bound' branch: t1a's slow profile kills at 20 minutes and the 60 s bound does not apply."
- file: ".engineering/planning/story/the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route.md"
  line: 171
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Test 5b compares the Go server with h2's Rust server and requires a byte-identical fixture pair, but h3 keeps its own copy (lines 153, 239) for a stale reason ('built at the same time'; h3 depends on h2) and nothing pins the two copies equal. Read h2's pair from h3 or assert byte equality; update h2 line 179 likewise."
```
