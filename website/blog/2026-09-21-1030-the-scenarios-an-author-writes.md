---
title: "0.16–0.18 — the scenarios an author writes, and a browser that plays them"
description: >
  A specification can carry authored scenarios, not only the ones it obliges: modelled instances,
  command timelines, expectations, a claimed length of time, and a consumer that halted an ordered
  scan. `ess verify conform web` walks them in a browser without executing an implementation. Also:
  why there is no 0.17.0 release.
slug: the-scenarios-an-author-writes
tags: [release, ess, conformance]
date: 2026-09-21T10:30:00+02:00
release_tag: "0.18.0"
release_commit: 43c0b79fddfab0af72b7a9dd81328699db8d6db1
---

Synthesis writes the scenarios a specification *obliges*. These releases let an author write the
ones it merely *permits*, and give three of them claims the format previously could not carry.

{/* truncate */}

## Authored scenarios

`ess-scenario/1` lets an author declare modelled instances, command timelines and expectations.
They compile into `ess-conformance/2` with **identities distinct from synthesized suites**, so a
target's report says which kind of claim it answered. Twenty-seven refusals check the names in an
authored scenario against the model, because a scenario naming a command that does not exist is a
test of nothing that passes.

## A claim about time

`mark:` and `elapsed:` add `not_before`, `within` and `quiet` bounds. The suite states a duration and
its anchor; the target reports an observed or advanced clock. A target with no clock support reports
`unsupported` rather than passing — the distinction the whole conformance model rests on. These
claims select `ess-conformance/3`.

## A claim about stopping

The six assertion forms are all predicates over the rows a read returned — `contains`, `excludes`,
`counts`, `ranked`, `at`, `satisfies` — and **two implementations that return the same rows are
indistinguishable under every one of them**. "The producer stopped" is exactly the fact that
separates them, and the format could not say it.

`halts_after: <n>` requires a consumer to stop an ordered scan after n rows. The target reports rows
produced and whether the consumer ended the read; a target without incremental reads reports
`unsupported`. This adds a claim to `ess-conformance/4`.

A migration of forty-five ACD slotmatcher cases hit that wall twice before the key existed.

## A browser that plays a specification

`ess verify conform web` emits a page that walks authored scenarios, showing actors, state, selected
views and declared consequences. It replays **model-determined behaviour without executing an
implementation**, so an author can inspect a specification before filling a single obligation.

That is a different use for a conformance suite than checking a target with one. It is the first
form in which a specification answers back to the person writing it.

## There is no 0.17.0 release

The `0.17.0` tag exists and its changelog section describes the halt assertion. Neither is the whole
story, and the section now says so at the top.

`0.17.0` was cut from a branch off 0.16.0 holding the scenario player, while `halts_after` was on
`main`. **Neither commit is an ancestor of the other**, and `halts_after` appears zero times at the
tag. Everything written under 0.17.0 first ships in **0.18.0**, which is the first release holding
both.

A binary calling itself 0.17.0 has one of the two features, and which one depends on where it was
built — check it for `halts_after` rather than trusting the number. The tag stays public because
deleting a tag that has been public for weeks breaks whatever pins it, to tidy a report.
