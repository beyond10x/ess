---
format: aep.planning-md/3
id: story:an-absent-read-inside-a-struct-payload-field-is-claimed-absent
kind: story
status: draft
title: An absent read inside a struct payload field is claimed absent
summary: A struct event field's member read absent is claimed as an exact nested null, so the interpreted target fails its own suite (turn 1 lane-1 adversary, origin undecided, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:zi-w1-lane1-adv-1
revision: 1
---
## Need

Turn 1 made synthesis claim a top-level event payload field that the arrangement reads as absent
through the field's leaf, in either spelling, instead of as an exact `null`
(story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null, beyond10x/ess#765).
Its adversary (review-result:zi-w1-lane1-adv-1) found that a member of a struct payload field is
still claimed as an exact nested `null`. With
`holder: {owner: {subject: owner}, agent: {subject: agent}}` and `agent` held absent, the suite
expects `holder = {"agent":null,"owner":"owner"}`, and the interpreted target, which leaves the
member out, fails its own synthesized suite with `ESS-CF-PAYLOAD` (observed
`holder = {"owner":"owner"}`). The struct branch of `determined_fields`
(`crates/verify/ess-conformance/src/synthesize.rs`) builds the nested value. Whether this predates
the turn was not run, so the origin is undecided.

The red case is kept in the turn scratch as `lane1-backlog-red-case.rs`.

## Acceptance

- An absent read inside a struct payload field is claimed absent in whichever spelling the
  member's presence admits, not as an exact nested `null`.
- The interpreted target passes every synthesized scenario of the nested model above.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs` (cited by the review: `determined_fields`)
- `crates/verify/ess-conformance/src/response.rs` (inferred: the payload shape leaves)
