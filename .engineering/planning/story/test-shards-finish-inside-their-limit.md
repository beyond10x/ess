---
format: aep.planning-md/3
id: story:test-shards-finish-inside-their-limit
kind: story
status: implemented
title: Every CI test shard finishes inside its 30-minute limit
tags:
- ci
- ess-0.54.0
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T17:50:02Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-06T17:50:03Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:50:04Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
# Every CI test shard finishes inside its 30-minute limit

## Acceptance

On a `main` push run, every `Test <m>/20` job of `.github/workflows/ci.yml` completes inside its
`timeout-minutes`, with the slowest shard's duration recorded; `crates/edge/ess-xtask/tests/ci_lanes.rs`
still holds that the shards leave no partition unrun.

## Evidence

- `main` `CI` run 37331001433 at 0.53.0 (`a81a8729d`), job `Test 14/20` (check run
  111838645483): "The job has exceeded the maximum execution time of 30m0s" at test 373 of 457,
  with `ess-cli::explore_concurrent::an_unsupported_command_is_left_out_and_fails_the_check_unless_accepted`
  reported SLOW past 60 s.
- Every other shard of that run finished; shard 12 failed on a separate defect
  (`story:ts-prerequisite-runtime-race`).

Which tests make shard 14 slow is not established; the log names one SLOW test only.

## Milestone

`release-plan:ess-054`.
