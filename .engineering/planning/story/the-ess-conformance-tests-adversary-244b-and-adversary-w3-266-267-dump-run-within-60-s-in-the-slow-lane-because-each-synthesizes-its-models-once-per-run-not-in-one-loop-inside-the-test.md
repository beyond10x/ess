---
format: aep.planning-md/3
id: story:the-ess-conformance-tests-adversary-244b-and-adversary-w3-266-267-dump-run-within-60-s-in-the-slow-lane-because-each-synthesizes-its-models-once-per-run-not-in-one-loop-inside-the-test
kind: story
status: draft
title: The ess-conformance tests adversary_244b and adversary_w3_266_267_dump run within 60 s in the slow lane, because each synthesizes its models once per run and not in one loop inside the test
summary: 'Two slow-lane tests measured 231 s and 312 s on 2026-11-26, over the 60 s bound: each loops over every model of the repository inside one test body'
tags:
- defect
- issues-2026-11-26
- test-budget
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- informed_by: story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it
- informed_by: story:the-slow-nextest-profile-kills-a-test-at-1200-s-while-adv-e-u6-byte-identity-probe-is-marked-over-1318-s-and
- informed_by: story:the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane
revision: 1
---
## Need

Two ess-conformance tests ran over the 60 s bound of one test in a slow lane on 2026-11-26, and no story of the store plans their time:

| test | file | measured in the slow lane |
|---|---|---|
| `adversary_244b_every_window_free_model_keeps_its_bytes_against_3b1684d1a` | `crates/verify/ess-conformance/tests/adversary_244b_window_free_bytes.rs` | 231 s (over 120 s in a second run) |
| `adv_every_model_synthesizes_one_document_and_dumps_for_base_comparison` | `crates/verify/ess-conformance/tests/adversary_w3_266_267_dump.rs` | 312 s |

The slow-lane stories mark both as slow (`story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it`) and name them in a list, with the markers "over 180 s at load 83" and "over 180 s at load 118". None plans to bring either within 60 s: `story:the-slow-nextest-profile-kills-a-test-at-1200-s-while-adv-e-u6-byte-identity-probe-is-marked-over-1318-s-and` does that for the two byte-identity probes of `ess-synth`, and `story:tests-that-run-over-60-s-on-main-without-a-branch-of-the-week-run-within-60-s-or-move-to-the-slow-lane` names other targets.

This is a defect of the test budget the test step of the gate keeps: no single test runs longer than 60 s, and an expensive product is made once, never inside a test; a step over the budget is fixed before the next feature, and raising a timeout is not a fix.

## Cause

Both tests are one test body that loops over every model of the repository and synthesizes each one inside the loop. The first compares, per model, the canonical IR, the synthesized suite, the refusals and the mutant list with a pinned digest, as written and relabelled `ess/22`. The second synthesizes every single-file fixture, the committed examples and the toolchain model twice each and, with `ADV_DUMP_DIR` set, writes each canonical suite for a comparison with a base commit. The cost is a loop of independent, equal-shaped items inside one test, the shape the rule "an expensive product is made once or split" names.

## Outcome

Each of the two tests runs within 60 s in its lane. The synthesis of a model is made once per run, not inside the test: either the loop is split into one case per model (or per group of models) so that no case passes 60 s, or the per-model products are read from a shared fixture built once per test run, in a cache on disk keyed by the digest of the sources, as `story:the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane` does for the byte tables. Both keep their assertions, and the slow-lane list still names them.

## Acceptance

1. The measure is taken on the build node, one target at a time, before and after, with the node's load written beside it; the time of every test of the two targets is recorded.
2. No test of the two targets is over 60 s in the ten slowest of the slow lane, and the runner's kill does not end any of them.
3. A case split out of the loop runs the same assertions on the same models; the models covered before and after are listed, and none is dropped.
4. `cargo xtask slow-list` still names the tests, and `cargo test -p ess-xtask --test slow_lane` is green with their rows.
5. A model added to the repository is read by one of the cases without a hand edit of a list.

## Fit review

1. **Need.** The slow lane must meet the 60 s bound too: one test over 60 s kills the lane's own purpose, which is to run in a bounded time.
2. **Class.** A defect of the test budget, not a feature.
3. **Already expressible?** Yes, by the two accepted forms of the rule (a committed generated file with a digest, or one shared fixture built once per run), and by splitting a loop into cases.
4. **Fit.** The same shape as the other budget stories; this one only names two more tests.
5. **Second adopter.** Any test that loops over a corpus inside one body.
6. **Cost.** A split of two test files into cases, or a cache on disk keyed by the digest of the sources.
7. **Considered.** (a) Raise the slow profile's timeout: refused by the rule. (b) Leave the tests as they are because they run only in the lane: the rule has no such exemption. (c) Split or share (chosen).

## Decisions

- None open. One story for the two tests, because one cause is plain: a loop over every model inside one test body (the coordinator, 2026-11-26).

## Scope

Cited: `crates/verify/ess-conformance/tests/adversary_244b_window_free_bytes.rs`, `crates/verify/ess-conformance/tests/adversary_w3_266_267_dump.rs`, `crates/edge/ess-xtask/tests/slow_lane.rs`, `.config/nextest.toml`.
