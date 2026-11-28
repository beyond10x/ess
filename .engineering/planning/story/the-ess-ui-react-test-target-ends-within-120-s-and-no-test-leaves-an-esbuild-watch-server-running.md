---
format: aep.planning-md/3
id: story:the-ess-ui-react-test-target-ends-within-120-s-and-no-test-leaves-an-esbuild-watch-server-running
kind: story
status: draft
title: The ess-ui-react test target ends within 120 s and no test leaves an esbuild watch server running
summary: 'A test-budget defect: the target takes 264 to 337 s as a whole in one batch''s gate logs and its tests leave esbuild --watch=forever servers under target/tmp that block worktree cache discards'
tags:
- defect
- test-budget
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
revision: 1
---
## Summary

The `ess-ui-react` test target runs 264 to 337 s as a whole in the gate logs of one batch (job
`cargo test -p ess-ui-react --locked --no-fail-fast`, build included, at a 1-minute load of 32 to
38), over the 120 s step budget, and its tests leave `esbuild --watch=forever` servers running
under `target/tmp` after the run: the dev and preview cases of `adversary_react_plain` and the
bundle dev case left 3 at one point, and the batch ended 6 by PID over its run. A leftover server
holds the worktree: the worktree tool refuses to discard the build cache while an `esbuild`
process has its working directory or an open file in the tree (`worktree-in-use`, repeated across
the batch's cache discards).

## Acceptance

1. The `ess-ui-react` test step ends within 120 s on a warm build at a 1-minute load of 16 or
   less, measured by the shell's `time` and the ten slowest tests the step prints; none of its
   tests runs over 60 s.
2. No process a test started outlives the test: after `cargo test -p ess-ui-react --locked`,
   `pgrep` finds no `esbuild` whose working directory or open files are in the checkout, and a test
   in the target fails when a server it started is still running at its end (the server is owned by
   a guard that kills it on drop, also on a panic).
3. A test that cannot meet 1 moves to the `slow` lane with its reason.

## Fit review

1. **Class.** A test-budget defect and a leak, one target.
2. **Not covered.** `story:ui-react-plain` (implemented) built the plain React output with
   one esbuild step; no story names the target's runtime or the servers it leaves.
3. **Borders.** The parent `story:the-test-step-meets-the-test-budget`; the tutorial story
   `story:the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane`
   (the same shape: a tool the step starts moves to the slow lane).

## Decisions

- TB-D2, default A: the watch-mode cases (`dev`, `preview`) A keep in the step with a guard that
  kills the server on drop and a bounded wait; B move them to the `slow` lane and leave the
  one-shot bundle cases in the step. Recommended A if the guarded cases meet 60 s, else B for
  those that do not. Not architecture and not a security boundary; taken on default if unanswered.

## Scope

- `crates/ui/ess-ui-react/tests/` (`adversary_react_plain_pass1.rs`, `adversary_react_plain_pass2.rs` and the bundle test file),
  its test helpers, `Taskfile.yml` only if a step is split.
