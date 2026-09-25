---
title: "0.28–0.30 — the history a subject keeps, and what a retry returns"
description: >
  Source ess/6 models enum subject history and input eligibility beside an independently arranged
  failure. Source ess/7 declares that a retry returns the original command's result. Conformance
  suites compare actual subject values and actual retained responses. 0.30.0 shipped the agent
  plugin from this repository for one release.
slug: what-a-retry-returns
tags: [release, ess]
date: 2026-09-25T12:00:00+02:00
release_tag: "0.30.0"
release_commit: 67058c4a5964c82099865858f32eac9c0f27a472
---

Three releases. Two of them move a claim a conformance suite used to make about a subject's shape
into a comparison of the values an implementation actually produced, and each costs a source
format version — `ess/6` and `ess/7`, both described in
[the version history](/ess/docs/reference/spec-versions).

{/* truncate */}

## The history a subject keeps

`ess/6` (0.28.0) models bounded enum subject history with explicit silent preservation, and input
eligibility beside an independently arranged external cause. Conformance synthesis observes
history separately from lifecycle, so two histories that end in the same state stay distinct.

Suite formats 10 and 11 add no-error assertions and actual subject snapshots. The Rust, Go and
TypeScript runners **reject a missing, duplicated or changed subject** instead of accepting a row
that merely has the right shape. Documents in older formats keep them and refuse the new
vocabulary.

## What a retry returns

`ess/7` (0.29.0) declares command-local `replays`: a retry returns the originating success's typed
result and preserves its original subject. An effect-free named error can cover the remaining
finite subject states.

Suites 12 and 13 capture the original result, input, actor and identity before an unforced retry,
and Rust and generated Go **compare the exact typed response** and a fresh complete observation of
the subject. TypeScript and browser readers refuse these suites before any callback runs, and
Decimal and Binary64 response positions remain unsupported. `ess-diff/6` records replay origins.

The generated witness covers an immediate retry. Persistence, restart and retries against a later
head remain acceptance the adopter owns.

## The plugin, for one release

0.30.0 shipped the ESS agent plugin from this repository at the binary's version, with `ess skill`
printing its skills. The next release moves it to `beyond10x/agentplugins` beside every other
plugin and removes `ess skill`.
