---
format: aep.planning-md/2
id: story:count-guards-above-one-are-synthesized
kind: story
status: draft
title: A count guard above one gets synthesized scenarios
relations:
- serves: vision:O2
revision: 1
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
