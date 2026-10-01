---
title: "0.50 — Aggregates over rows a when_related guard admits"
description: >
  0.50.0 is one synthesis fix: an aggregate view whose creating command has a when_related guard
  gets its aggregate scenario instead of being refused with ESS-SYNTH-017.
slug: aggregates-over-related-guards
tags: [release, ess]
date: 2026-10-01T22:00:00+02:00
release_tag: "0.50.0"
---

0.50.0 is a fix release. Specifications, suites and deltas keep their formats.

{/* truncate */}

## Aggregates over a `when_related`-guarded creator

Until now, an aggregate view over rows created by a command with a `when_related` guard was
refused with ESS-SYNTH-017: synthesis could not create the rows the view counts, because each
needed a related row the guard would accept. From 0.50.0 the view gets its `<view>/aggregate`
scenario:

- each row's related row is arranged first, and rows given one value of the guard's input share
  it;
- every input a guard predicate compares with the related row's owner link names an owner that
  is arranged too;
- a group key read from a related row's owner link holds one owner per value, so a target that
  groups under the wrong owner fails.

Where a row still cannot be arranged, the refusal names the reason (beyond10x/ess#272).
