---
title: "0.40 — generated runtimes run every suite, and reader-side composition"
description: >
  0.40.0 runs suites `ess-conformance/22` to `/27` in the generated Go and TypeScript runtimes
  with the Rust runner's verdicts, and `ess-composition/3` admits the widenings a consumer reads.
slug: runtime-parity
tags: [release, ess]
date: 2026-09-29T12:00:00+02:00
release_tag: "0.40.0"
release_commit: eea1558d77cf6542c030cd444cd8289228530260
---

0.40.0 adds the composition format `ess-composition/3`, described in
[the version history](/ess/docs/reference/spec-versions).

{/* truncate */}

## One verdict in every runtime

The generated Go and TypeScript runtimes run suites `ess-conformance/22` to `/27`, which 0.38.0
refused, and reach the Rust runner's verdict on each scenario.

## Reader-side composition

A `conformances` entry in `ess-composition/3` may carry `reader: true`: the consumer's type reads
every value the provider's type admits, so a widening the consumer can read is admitted rather than
refused.

## Names

A duplicate name is refused once, where it is declared, and the published schema states the
characters each kind of name may use.
