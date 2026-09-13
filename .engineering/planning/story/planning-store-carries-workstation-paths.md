---
format: aep.planning-md/1
id: story:planning-store-carries-workstation-paths
kind: story
status: active
title: 60 tracked files under .engineering/ carry home-directory paths the host-path lane does not scan
relations:
- serves: vision:O2
- informed_by: story:fixtures-carry-workstation-paths
scope:
- confidence: inferred
  path: .engineering/planning
- confidence: cited
  path: crates/edge/ess-xtask/tests/host_paths.rs
revision: 5
---
## Finding

Measured by `aep-drive:adversary` pass 1 in ESS wave 22
(`review-result:adversary-wave22-unit2-pass-1`).

`story:fixtures-carry-workstation-paths` accepts: *"No tracked file under `crates/`, `docs/`,
`website/` or `models/` contains a path under a user's home directory, and a gate lane refuses one."*
That sentence is now true, and the repository is still not clean.

```
git grep -acE '/(home|Users|root)/[A-Za-z0-9._+@-]' -- .engineering/
60 files, at ab02915b and at base c8023067 alike
```

The two largest: `.engineering/planning/journal.jsonl` (87 lines) and
`.engineering/planning/review-result/authored-discovery-source-pass1.md` (28,730 lines).

## Why it is not that story's defect

Its acceptance names four trees and `.engineering/` is not among them, so the unit that satisfied it
was right to stop at the boundary it was given. The gap is between the accepted sentence and what a
reader takes it to mean.

## What makes this different from the four trees

The journal is **append-only**. A path already written into it cannot be corrected the way a
document can — rewriting it is rewriting the store's own event history, which its validator reports
as forgery. So the remedy is not the same remedy, and that is the thing to decide before anything is
scrubbed.

## Acceptance

One of these, decided rather than drifted into:

- the planning store is in scope, the documents under it are scrubbed, and the journal gets whatever
  treatment its append-only contract actually permits — which may be *nothing*, stated;
- or the store is out of scope, and `crates/edge/ess-xtask/tests/host_paths.rs` says so in its module
  doc beside the bound it already states for non-standard home roots, so the next reader of the lane
  knows what it does not cover.

Either way the lane's documentation and the repository's state agree afterwards.
