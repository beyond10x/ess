---
format: aep.planning-md/3
id: story:a-node-named-tone-by-is-not-a-tone
kind: story
status: draft
title: A page, state or widget named tone_by is not read as a tone
summary: The loader reads any key spelled tone_by as the tone construct (UI-adopter wave 1 U1 fix-round note, pre-existing, backlog)
tags:
- backlog
- from-finding
- review-finding
relations:
- informed_by: review-result:w086-u1-adversary-pass-1
revision: 1
---
## Need

The loader treats every mapping key spelled `tone_by` under `shells`, `pages` or `widgets` as the
tone construct (`crates/ui/ess-ui/src/expand.rs:1465`, `Tones::value`), and `"tone_by"` sits in
`OPAQUE` (`expand.rs:43`), so local shorthands and widget expansion skip anything under such a key.
A page, page state or widget named `tone_by` would then fail to load ("has neither", `expand.rs:1495`)
or lose its expansion. This is the same flaw the value-labels unit fixed for `labels` in fix round 1
(review-result:w086-u1-adversary-pass-1); found by reading the base at 83391ebaeb, not yet run.

## Acceptance

- A document whose page, page state or widget is named `tone_by`, and which uses no tone construct,
  loads and expands as the same document with another name does.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
