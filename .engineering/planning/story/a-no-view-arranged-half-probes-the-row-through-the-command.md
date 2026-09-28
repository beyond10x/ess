---
format: aep.planning-md/3
id: story:a-no-view-arranged-half-probes-the-row-through-the-command
kind: story
status: draft
title: A no-view arranged half probes the row through the command
relations:
- serves: vision:O2
revision: 1
---
# Story: a no-view arranged half probes the row through the command

## Outcome

Where no identity view shows an arranged row, the arranged half of an input-refusal scenario still detects a target that changes the record on refusal, by following the refusal with a probe the model decides from the row (for example the command's own valid send, whose outcome depends on the row's state).

## Why

Adversary pass 2 on the 0.41 arrangement unit (`review-result:adversary-a-arrangement-pass-2`): with no identity view, the arranged half requires only the error and no event (`crates/verify/ess-conformance/src/synthesize/existence.rs:336`), so a target that silently moves the record on refusal passes; the next valid send would reveal it. Reached only by a built model; nothing in the repository.

## Acceptance

- `issue_209_no_view_arranged_half_catches_a_target_moving_the_record_on_refusal` in `tests/adversary_arrangement_pass2.rs` is flipped from asserting today's pass to asserting failure.

## Scope

- `crates/verify/ess-conformance/src/synthesize/existence.rs` — cited
