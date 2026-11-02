---
format: aep.planning-md/3
id: story:an-admitted-timestamp-is-an-instant-the-server-can-order
kind: story
status: draft
title: An admitted Timestamp is an instant the generated server can order
summary: Leap second and long fractions are admitted but unorderable (W1 GEN1 review, backlog)
tags:
- backlog
- review-finding
relations:
- informed_by: review-result:adversary-w1-gen1-pass-1
revision: 1
---
## Need

The generated decoder admits a `Timestamp` with a leap second (`23:59:60`) or a fraction past 9 digits,
as `format: date-time` does, but the tree's `rfc3339-instants` vectors and the generated `instant_of`/`instant`
grammar refuse both, so a stored value is Unknown in every Timestamp guard and byte-ordered in views.
Before W1 any text was stored; W1 narrowed the gap. Found by review-result:adversary-w1-gen1-pass-1
(red cases `the_timestamp_decoder_answers_the_trees_rfc3339_instant_vectors` and
`every_timestamp_the_decoder_admits_is_an_instant_the_generated_server_can_order`, left out of the committed file).

## Acceptance

- One RFC 3339 grammar is chosen for leap seconds and long fractions; the decoder, the instant grammar and
  the served schema agree on it, and every admitted `Timestamp` is an instant the server can order.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.
