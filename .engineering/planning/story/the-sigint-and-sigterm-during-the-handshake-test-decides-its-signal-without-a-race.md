---
format: aep.planning-md/3
id: story:the-sigint-and-sigterm-during-the-handshake-test-decides-its-signal-without-a-race
kind: story
status: draft
title: The test of a SIGINT and a SIGTERM during the handshake decides the signal it asserts without a race
summary: 'A flake: a_sigint_and_a_sigterm_during_the_handshake_die_of_the_signal_and_kill_the_target got SIGTERM where it asserted SIGINT once in CI on 2026-11-25 (PT-D1)'
tags:
- defect
- flaky-test
- test-budget
relations:
- serves: vision:O2
- informed_by: story:a-killed-childs-outcome-says-which-signal-ended-it
revision: 3
---
## Defect

`a_sigint_and_a_sigterm_during_the_handshake_die_of_the_signal_and_kill_the_target`
(`crates/edge/ess-cli/tests/adversary_process_target_pass2.rs:246` at `d7833ee4af`) failed once in CI on 2026-11-25. It sends `SIGINT` and
`SIGTERM` back to back and asserts that `ess` died of `SIGINT`; it got `SIGTERM`.

## Reported case

CI run 45522328346, job `Test 6/20` (the feature-off lane), https://github.com/beyond10x/ess/actions/runs/45522328346/job/138906417378:

```
assertion `left == right` failed: ExitStatus(unix_wait_status(15))
  left: Some(15)
 right: Some(2)
```

in 0.151 s. The head's earlier run (45502204802) and its base run (45477616807) passed the test with the same `ess-cli` sources: a
flake, not a regression of the product. No other run is known to fail it.

What the code and the design say:

- The test's own comment claims "the first one decides the status" (`adversary_process_target_pass2.rs:242-244`); it calls
  `run.signal(Signal::INT)` and then `run.signal(Signal::TERM)` with no wait between them (`:264-265`) and asserts `Some(2)` (`:271`).
- `ess` waits for the signals in one thread: `signals.forever().next()` of a `signal_hook::iterator::Signals` over `SIGINT`, `SIGTERM`
  and `SIGHUP` (`crates/edge/ess-cli/src/target_process.rs:52-66`), and ends by the signal that call returns (`end_on`, `:82`). Hypothesis, not verified:
  when both signals are pending when the thread wakes, the iterator does not return them in arrival order, so either may decide.
- The design says "On one it kills the target's process group ... Then `ess` ends by that same signal"
  (`docs/design/conformance-target-process.md:172-182`); it does not say which of two signals decides.

## Fit review

1. **Need.** A test that decides its expectation by a race fails a gate at random and is retried by hand; the test budget rules ask
   for no retry setting and no `#[ignore]` (`AGENTS.md` § Tests stay fast).
2. **Class.** A defect of a test (a flake), pending a decision on whether the product should order two signals.
3. **Existing stories.** None covers this test or flaky process-target tests (a search of the store for the test, the file and
   "flaky" finds only the test-budget stories that list the slowness of the same file). `story:a-killed-childs-outcome-says-which-signal-ended-it`
   borders it (signals of a killed child; not the signals `ess` receives).
4. **Release train:** no release of its own: a test change, carried by the next release.

## Decisions

- PT-D1, what the test asserts. A: what holds for either order: `ess` dies of `SIGINT` or of `SIGTERM`, writes no report, and the
  target's group is gone; the design promises one of them. B: `end_on` decides by arrival order (the first signal received), a
  product change, and the test keeps asserting `SIGINT`; needs the design page to say first-wins. Recommended A. Open until answered;
  taken on default if unanswered.

- PT-D1: A (the test asserts what holds for either order of the two signals), taken by the coordinator on 2026-11-25; the race's cause is not verified, the implementor states it before the fix.

## Acceptance

- the_test_decides_without_a_race: the test passes 200 consecutive runs in a row (one `cargo nextest run -p ess-cli --test
  adversary_process_target_pass2 --retries 0` loop) with the assertion of PT-D1 A (or B's change), and no retry setting and no
  `#[ignore]` are added.
- the_other_signal_tests_are_unchanged: `a_sigterm_to_ess_mid_run_leaves_no_process_of_the_targets_group_behind` and its `SIGINT` and
  `SIGHUP` siblings keep their assertions.

## Scope

- cited: `crates/edge/ess-cli/tests/adversary_process_target_pass2.rs` (269-296), `crates/edge/ess-cli/src/target_process.rs` (50-92, only
  under PT-D1 B)
- inferred: `docs/design/conformance-target-process.md` (172-182, only under PT-D1 B)

## Delivery

Fixed in 0.99.8 (2026-11-27), and not by a change to the test: the cause was in `ess`. Every thread of `ess` took the ending signals, so the SIGINT's and the SIGTERM's handlers could run on two threads at once, and the SIGTERM's was recorded first when the SIGINT's thread was preempted; this replaces the hypothesis above about the order of a pending set in the iterator. Only the thread that watches for SIGINT, SIGTERM and SIGHUP takes them now, so they are handled one at a time in the order the kernel delivers them. `ess-cli` depends on the `nix` crate (=0.31.4, feature `signal`) for the thread's signal mask, which the workspace's ban on `unsafe` allows. CI shard 12 failed the test with signal 15 in three runs of pull request 843 before the fix. The fix is part of the hotfix 0.99.8, pull request 843 (https://github.com/beyond10x/ess/pull/843), merged into `main` on 2026-11-27 09:23 CET as `e29c029fab`; the tag 0.99.8 is on `e29c029fab` (2026-11-27 09:26 CET); release run 45965734005 pending when this was written.
