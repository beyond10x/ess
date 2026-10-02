---
format: aep.planning-md/3
id: story:count-guards-above-one-are-synthesized
kind: story
status: implemented
title: A count guard above one gets synthesized scenarios
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/stored_field_guards_adversary.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T12:40:19Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T12:40:20Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "active", to: "implemented", at: "2026-10-02T12:40:21Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

A guard such as `tags.count > 1` (over command input or over a stored field) gets synthesized
scenarios for both branches.

## Why

Found during the 2026-09-25 wave. Unit A1 (#94) synthesizes list guards from one-element lists, so
`.count >= 1` and `exists`/`forall` work, but `.count > 1` refuses: over input with `ESS-SYNTH-003`
("no candidate of the 2 tried satisfies labels.count > 1"), over a stored list with `ESS-SYNTH-001`
and a hint that tells the author to drop the field (review-result:c-adversary-pass-1, finding 3;
unit A1's own report). Cases in `crates/verify/ess-conformance/tests/stored_field_guards_adversary.rs`
pin today's refusal.

## Acceptance

- The witness builds a list of `N + 1` (and `N`) elements for a `.count` comparison against literal
  `N`, bounded, for input and stored lists.
- The re-pinned adversary cases flip to asserting both scenarios.
- The misleading "drop it from the command's input" hint is not printed for a count guard.

## Released acceptance verified

Independent source/evidence audit by server_corrections finds all three acceptance items satisfied in release0.51.0. witness.rs:1906-1982 builds a bounded count ladder. stored_field_guards_adversary.rs:282 and :432 require both input/stored branches and two-element witnesses; :476 requires ESS-SYNTH-018 without the misleading drop-input hint. These relevant source/test files are unchanged from0.51.0.

The retained f863ee full package execution passed all14 cases in this adversary binary (target/backlog-input/group-packages-f863ee.log:3456-3474, completed2026-10-02T11:46:44Z). Audit own new execution count0; evidence comes from that actual earlier run. This closes stale draft state. Separate upper-count boundary requests are not part of this story's three acceptance items.
