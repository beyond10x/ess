---
format: aep.planning-md/3
id: story:a-present-optional-element-is-appended
kind: story
status: draft
title: A suite appends a present Optional element and expects its value
tags:
- ess-d368-close
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by both review passes of wave D368 U1 (`review-result:adversary-d368-u1-pass-1`, `review-result:security-d368-u1-pass-1`), out of that story's Acceptance: the suite for `{append: input.<field>}` over a `List<Optional<T>>` never sends a present element, so a target that always appends `null` passes (`synthesize.rs`, `appended_at`).

## Acceptance

- Where a command appends an `Optional<T>` input element, synthesis sends one scenario with a present element and expects that value in the view row, beside the absent and `null` cases.
- A target that always appends `null` fails that scenario; the interpreted target and generated Rust and Go pass it.
- Committed models whose suites move are named.
