---
format: aep.planning-md/3
id: story:the-date-time-agreement-oracle-refuses-non-digits
kind: story
status: draft
title: The date-time agreement oracle refuses non-digits at digit positions
summary: jsonschema 0.52.1 takes + - . for digits in date-time (W1 GEN1 review, pre-existing, backlog)
tags:
- backlog
- review-finding
relations:
- informed_by: review-result:security-w1-gen1-pass-1
revision: 1
---
## Need

The agreement test `crates/generate/ess-synth/tests/json_body_closure.rs:507` uses jsonschema 0.52.1 as its
oracle; that version takes `+`, `-` and `.` for digits in `date-time` (153 of 60,053 seeded strings, e.g.
`202+-10-29T12:00:00Z`), so a future body could pass the agreement check while RFC 3339 refuses it.
Found by review-result:security-w1-gen1-pass-1.

## Acceptance

- The agreement test's `date-time` oracle refuses a non-digit at a digit position, by a validator version
  that does or by a check beside it, and a seeded case shows it.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.
