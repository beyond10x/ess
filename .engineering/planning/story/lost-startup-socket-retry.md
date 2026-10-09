---
format: aep.planning-md/3
id: story:lost-startup-socket-retry
kind: story
status: implemented
title: Retry a lost startup socket while the browser remains alive
relations:
- serves: vision:O2
- informed_by: review-result:adversary-wave24-unit1-pass-2
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/browser_startup_slow_serve_boundary.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T11:44:46Z", actor: "human:timo", revision: 3, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T11:44:46Z", actor: "human:timo", revision: 4, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "active", to: "implemented", at: "2026-10-08T09:55:06Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

A browser startup retries a transient lost upgrade socket while its child remains alive and its startup deadline has not elapsed, consistently with refused TCP connections and a boot-time404. A genuine protocol defect after readiness remains distinguishable.

## Existing evidence

This is the untracked G3 defect already preserved as an ignored regression in browser_startup_slow_serve_boundary.rs:194, not a new product feature. Fresh execution at482609 with --ignored failed exactly: one connection attempt before giving up although the child remained alive and the2second budget remained. The complete ignored lane was0passed4failed, and the current normal lane2passed4ignored. Logs are server target/backlog-input/browser-startup-existing-defects-red.log and browser-startup-reconciliation.log.

## Acceptance

- a_socket_lost_while_the_child_is_alive_is_retried_like_every_other_state_on_the_way_up executes without ignore and passes against its deterministic stand-in.
- The retry remains bounded by the original absolute deadline and emits measured startup/stderr evidence when readiness is never established.
- Child exit, malformed successful-upgrade/protocol response, successful early/late readiness, and session timeout controls preserve their existing distinctions.

## Decision

Accept the harness defect repair together with the existing answered-HTTP and startup-lock stories. No production command semantics or authored format change.

## Scope

Cited: crates/edge/ess-cli/tests/support/browser.rs::reach_bidi (upgrade error branch); crates/edge/ess-cli/tests/browser_startup_slow_serve_boundary.rs::a_socket_lost_while_the_child_is_alive_is_retried_like_every_other_state_on_the_way_up. Review related startup classification assertions before changing them.
