---
title: "0.67 — Guards that decide absent values, and external branches narrowed to some rows"
description: >
  0.67.0 adds source format `ess/29`: a guard may say what an absent `Optional` compares as, and an
  `external:` branch may apply only to the rows a subject guard selects.
slug: absent-values-and-narrowed-externals
tags: [release, ess]
date: 2026-10-21T12:00:00+02:00
release_tag: "0.67.0"
---

0.67.0 is a feature release. It adds the source format `ess/29` and the suite formats
`ess-conformance/56` and `/57`. A model below `ess/29` keeps its meaning, its IR bytes and its
compiled digest. The [changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists
every change since 0.63.0.

{/* truncate */}

## A guard says what an absent value compares as

A guard over an `Optional` fact used to read an absent value as unknown, so the model could not
decide that record. From `ess/29` the guard says what absence compares as:
`coalesce(is_error, true) == false`. Synthesis sends the absent value first, so the branch it
selects is witnessed ([#620](https://github.com/beyond10x/ess/issues/620)).

## An external branch narrowed to some rows

From `ess/29` an `external:` branch may carry `when_subject: {predicate: …}`: the provider is
asked, and the branch taken, only on a row the predicate selects. The same format refuses a
guarded branch declared below one that a later precedence step answers, naming the order to
declare them in ([#554](https://github.com/beyond10x/ess/issues/554)).

## Conformance and synthesis

- A one-time read cell names its reader and reads the view as that actor. Suites holding such a
  cell are written at `ess-conformance/56` (`/57` with coverage); the Rust, Go and TypeScript
  runners read them ([#614](https://github.com/beyond10x/ess/issues/614)).
- Synthesis witnesses more ranked heads and connective children, and a command reading many
  row-set selectors synthesizes in seconds
  ([#618](https://github.com/beyond10x/ess/issues/618),
  [#621](https://github.com/beyond10x/ess/issues/621),
  [#622](https://github.com/beyond10x/ess/issues/622),
  [#624](https://github.com/beyond10x/ess/issues/624)).
