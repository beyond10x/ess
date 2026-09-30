---
format: aep.planning-md/3
id: review-result:adversary-served-events-pass-2
kind: review-result
status: active
title: Adversary pass 2, generated server published events
relations:
- reviews: story:generated-server-publishes-and-reads-headers
revision: 1
---
unit: story:generated-server-publishes-and-reads-headers, tree ess-sf-http (after correction 1)
verdict: NEEDS-CHANGE, red 8
cases: executed 572→580, red 8
origin: introduced 3

| # | file:line | severity | message |
|---|---|---|---|
| 1 | ess-synth/src/rust/system.rs:1005 (Go system.rs:977) | blocker | an event that keeps failing blocks the pump; every later served command answers a false 501, the log grows per request, the binding beside the failed one never receives it |
| 2 | ess-synth/src/rust/system.rs:16 | blocker | redelivering the whole event runs an at_most_once binding twice |
| 3 | ess-synth/src/web/bridge.rs:332 | warning | the web bridge inherits finding 1 |

Tests: adversary_served_pass2.rs, adversary_served_pass2_go.rs, adversary_served_pass2_at_most_once.rs.

## Correction 2 (coordinator-verified)

Per-binding held-back delivery in Rust and Go; at_most_once never redelivered; served 501 only for the current command's events; bridge test served_correction_pass2_bridge.rs (3). Coordinator rerun: cargo test --locked -p ess-gen -p ess-synth 583 passed 0 failed over 86 lanes; the four pass-2 targets 4+1+3+3; clippy -D warnings exit 0; xtask generate --check up to date.
