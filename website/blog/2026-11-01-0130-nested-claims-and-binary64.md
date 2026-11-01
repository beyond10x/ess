---
title: "0.79 — Nested-member claims, Binary64 in conformance suites, and typed authored inputs"
description: >
  0.79.0 adds scenario format ess-scenario/6 and suite pair ess-conformance/62 and /63: an authored
  claim may name a nested member by its path, an authored literal input is held to its type, and a
  model that reaches a finite Binary64 runs a conformance suite.
slug: nested-claims-and-binary64
tags: [release, ess]
date: 2026-11-01T01:30:00+01:00
release_tag: "0.79.0"
---

0.79.0 adds no source format: a specification keeps its meaning, its IR bytes and its compiled
digest. It adds scenario format `ess-scenario/6` and suite pair `ess-conformance/62` (ordinary) and
`/63` (coverage). The [changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists
every change; this note covers what the new formats carry and the fixes that came with them.

{/* truncate */}

## Nested-member claims

- From `ess-scenario/6` a claim in an act's `response:`, an event's `payload:`, an error's
  `fields:` or a row may name one member of a nested value by its dotted path,
  `checkpoint.checkpoint_id: cp-1`, under one rule on all four surfaces: a field is claimed whole or
  by its paths, not both.
- A response path is carried by `ess-conformance/62` and `/63`; event, error and row paths compile
  to dotted keys older suites already read, so those suites keep their format.
- A partial nested literal in a `response:` is refused as `ESS-AUTHOR-014`, naming each member it
  leaves out at its path.

## Binary64 through a conformance suite

- A model that reaches a `Binary64` in an entity's fields, a command's input or response, an
  event's or error's fields, or a view now synthesizes, authors and runs a suite at
  `ess-conformance/62` (coverage `/63`).
- Rust, Go and TypeScript compare it by its nearest binary64 under IEEE equality (`-0` equals `0`);
  a token with no finite image, such as `1e400`, is refused by name. An authored `Binary64` literal
  of magnitude 2^53 or more is refused as `ESS-AUTHOR-015`, because view and event assertions
  compare an integer token exactly.
- A `Binary64` reached through an entity identity, a view parameter, an actor attribute, a binding
  or a conversion is still refused (`ESS-AUTHOR-036`).

## Authored inputs meet their type

- From `ess-scenario/6` an act's literal input that its declared type refuses, at any depth and
  beside an `$instance` reference, is refused as `ESS-AUTHOR-015` at its member path.
- A refusal guard that only out-of-type inputs meet is refused as `ESS-SYNTH-003`, naming the field,
  the type and the constraint.

## Synthesis fixes

- Both branches of a `!=` or `==` guard that compares an input with its own `example:` are written,
  for text, `Uuid`, `Timestamp`, `Duration` and `Bytes`, and for a struct member a structured example
  reaches.
- An input-guarded refusal beside exhaustive held-state branches, and the byte-length children of an
  `any` over alphabet-bounded texts, are witnessed where they were refused.
- An accepting branch read after `unknown_instance:` is arranged with its row stored first; guards
  on related identities, parent rows and cascades get the rows they read.
- An authored scenario that repeats a key in a mapping is refused as `ESS-AUTHOR-047`.
