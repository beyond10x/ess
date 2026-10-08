---
format: aep.planning-md/3
id: story:test-shards-leave-room-for-the-download-retry
kind: story
status: draft
title: The CI test job's time limit leaves room for the artifact download retry
relations:
- serves: vision:O2
revision: 1
---
## Finding

Main CI run 37615777010 (attempt 2, 2026-10-07): shards 19/20 and 20/20 were cancelled at the test
job's 45-minute limit (`.github/workflows/ci.yml:261`) near the end of their feature-off tests
(231/256 and 209/256). Their three `actions/download-artifact` steps took 1,512 s and 1,730 s, the
first one reaching its 12-minute timeout and retrying; the tests themselves took 18.4 and 14.1
minutes. On green run 37556433936 shard 19's tests took 770 s + 356 s.

## Acceptance

- The test job's `timeout-minutes` covers the download retry budget (each archive's timeout and
  one retry) plus the slowest shard's measured test time with margin; the value and its arithmetic
  are stated where `crates/edge/ess-xtask/tests/ci_lanes.rs` checks the workflow, and that check
  fails if the limit drops below the download budget plus test time.
