---
format: aep.planning-md/3
id: review-result:model-ergonomics-offline-dependencies
kind: review-result
status: active
title: Offline timestamp dependency correction review
relations:
- reviews: story:feature-request-406
revision: 1
---
Review verdict: **green**, pending fresh-CI confirmation.

- All observed shard failures are the same missing `time-macros` offline dependency.
- Scope is limited to `crates/generate/schema-contract/Cargo.toml` and `Cargo.lock`.
- The lock adds only `schema-contract → time` and `time-macros 0.2.32` with `num-conv` and `time-core`.
- CI runs `cargo fetch --locked` before building test archives, so fresh runners will fetch the complete closure before generated crates build offline.
- Generated manifests, runtime behavior, and generator source are unchanged.
- No finding. No local build or mutation performed.
