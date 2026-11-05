---
format: aep.planning-md/3
id: story:ci-go-task-install-survives-a-release-download-error
kind: story
status: draft
title: A CI job's go-task install survives a transient release download error
summary: 'A 504 on the go-task release download failed two 0.87.0 test shards with task: command not found and no test run; retry or cache the install'
tags:
- from-finding
- next-wave-first
relations:
- serves: vision:O2
revision: 1
---
## Need

Every CI job that runs `task` first downloads go-task from its GitHub release with `curl`. In the
0.87.0 queue run 42043226338 that download returned HTTP 504 on two of twenty test shards
(Test 14/20 and Test 16/20, group b). `task` was then missing, the shard's test steps exited 127
with `task: command not found`, and no test ran; the failed jobs passed on a re-run of the same
head. One transient error on a third-party download should not fail a shard.

## Acceptance

- The go-task install retries the download with backoff before it gives up (for example
  `curl --retry 5 --retry-all-errors`), or restores the binary from the Actions cache keyed on
  `TASK_VERSION` and downloads it only on a cache miss.
- An install that still fails stops the job at the install step with its error, never later as
  `task: command not found`.
- `ci_lanes.rs` asserts the retry or the cache on every job that installs go-task, and a mutation
  test shows that removing it turns `ci_lanes` red.
