---
title: "0.39 — concurrent histories and direct library returns"
description: >
  0.39.0 checks a run of several clients against the specification's own model, records such runs
  from the Go and TypeScript explorers and from production logs, and adds typed direct returns.
slug: concurrent-histories
tags: [release, ess]
date: 2026-09-28T21:00:00+02:00
release_tag: "0.39.0"
release_commit: 9966add7174bb230e46f9c075245b633680bde43
---

0.39.0 adds the history format `ess-history/1` and the source format `ess/17`, described in
[the version history](/ess/docs/reference/spec-versions).

{/* truncate */}

## Several clients at once

Every earlier check drives a target with one client and one call at a time. `ess verify conform
check-history` reads a run of several clients, each call with its invoke and return instants, and
searches for an order the specification's own model accepts. Each view read is held to the
consistency the view declares.

## Where histories come from

The generated Go and TypeScript explorers record concurrent histories against a target and can
inject the faults a specification declares. `ess verify conform import-history` reads a recorded
production log through an `ess-history-adapter/1` document. `ess verify conform web --history`
draws a failing history as client lanes.

## Direct returns

`returns: true` in `ess/17` declares that a command answers with a typed value. Authored scenarios
assert literal responses, and the Rust runner checks the actual return values.
