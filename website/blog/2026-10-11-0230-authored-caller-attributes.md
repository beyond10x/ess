---
title: "0.59 — An authored act states the caller its command is sent as"
description: >
  0.59.0 lets an authored scenario state the caller attributes its command reads, and fixes two
  cases where synthesis refused or mis-chose a scenario.
slug: authored-caller-attributes
tags: [release, ess]
date: 2026-10-11T02:30:00+02:00
release_tag: "0.59.0"
---

0.59.0 is a feature release. It adds the authored scenario format `ess-scenario/5`; specifications,
suites and deltas keep their formats. The
[changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every change since 0.55.0.

{/* truncate */}

## An authored act states its caller

An act in an `ess-scenario/5` document states the caller its command is sent as, under `caller:`.
Each attribute value is a literal of its declared type, or `{$instance: name}` for an instance
arranged with `setup:`. A command that records `{caller: account_id}` now runs from an authored
scenario on the `interpreted` target and the generated Go and TypeScript runners.

- `ess verify conform author` refuses an attribute the act's `actor:` does not declare
  (`ESS-AUTHOR-013`), a required attribute the command reads that the act leaves unstated
  (`ESS-AUTHOR-014`), and an instance captured at run time (`ESS-AUTHOR-015`), naming the attribute
  and the command.
- An act with no `caller:` sends none, and its suite keeps its bytes. `ess-scenario/1` to `/4`
  refuse the key.
- A suite step that reaches a caller attribute nothing supplied is still `unsupported`, now with a
  reason naming the attribute and `caller:`.

## Synthesis

- A command whose branches a stored field of its subject selects (`when_subject:`) gets its
  scenarios even when no view observes that field; synthesis used to refuse each with
  `ESS-SYNTH-001`. A model with an immediate view covering the guarded fields keeps its suite byte
  for byte ([#496](https://github.com/beyond10x/ess/issues/496)).
- A `when_related:` guard that reads a member of a struct input is decided against the key the
  subject was created under, where it used to choose a row against a key the search made up. A
  struct sent as a literal keeps its scenarios byte for byte ([#521](https://github.com/beyond10x/ess/issues/521)).
