---
format: aep.planning-md/3
id: story:ci-test-shards-each-extract-half-the-test-binaries
kind: story
status: active
title: CI test shards each extract half the test binaries
summary: The 46.8G of extracted test binaries does not fit 52G hosted runners; split the workspace archive in two by binary, shards in two groups, extract then delete the archive
tags:
- from-finding
- next-wave-first
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T10:37:39Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T10:37:39Z", actor: "human:timo", revision: 3}
---
## Need

The queue runs of 0.87.0 (41984160096, 41997613644) stopped test shards on hosted runners that
have 52G free on `/` after the cleanup step and no `/mnt`. The disk watch stopped `cargo-nextest`
at 4902M free while it extracted the workspace archive (Test 11/20, job 127150909089). `main`'s
scheduled run 41816926906 of 2026-11-04 failed the same way in `Build workspace test archive`
(`No space left on device`). A passing shard on a larger runner (Test 4/20, run 41997613644)
measured it: `target/` held 10.9G after the workspace archive download, 17.1G after the
feature-off one, and 63.9G once nextest had extracted the 1817 workspace binaries, which are
already built with `debug = 0` and `strip = symbols`. The extracted binaries take 46.8G, and the
tests add 1.2G. Extracting the archive and deleting it before the run is not enough on its own:
46.8G plus the test output leaves 4G on 52G.

## Acceptance

- The workspace test archive is built as two archives from one build, split by test binary with
  `cargo nextest archive -E <filterset>`, together holding every test binary exactly once.
- The test shards run as two groups of ten, each running `slice:m/10` of its own archive; every
  partition of both archives is run.
- A shard downloads its archive, extracts it into the checkout, deletes the archive, and runs from
  the extracted `target/nextest/binaries-metadata.json` and `cargo-metadata.json` without
  compiling anything.
- `crates/edge/ess-xtask/tests/ci_lanes.rs` holds the two archives, the two groups' complete
  partitions and the extract-delete-run order.
- A queue run on runners with 52G free completes every shard, and its disk watch log shows each
  shard's peak with at least 5G free.

## Schedule

The first unit of the next ESS wave (0.88.0).
