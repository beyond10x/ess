---
title: "0.37 — outcome shapes, guards over the input, and new value types"
description: >
  0.37.0 adds outcome shapes, subject guards that read the input, case-insensitive text, aggregates
  over optional fields, new value types and a pinned toolchain.
slug: outcome-shapes
tags: [release, ess]
date: 2026-09-28T12:00:00+02:00
release_tag: "0.37.0"
release_commit: 472d35fbe3109ad47f89a65df44b60c1f14acd55
---

0.37.0 adds the source format `ess/15`, described in
[the version history](/ess/docs/reference/spec-versions).

{/* truncate */}

## Outcome shapes

An outcome can delete its subject (`deletes:`), create into a declared state (`into:`), answer an
identity no record carries (`unknown_instance:`), or accept a request without a subject
(`accepts: nothing`). System `preconditions:` run before every scenario.

## Guards and text

A subject guard may compare a stored field with the input, `recording_id != input.recording_id`.
`equals_ignore_case` and `in_ignore_case` compare text under ASCII case folding. Aggregates may skip
absent values of optional fields, and an optional group key makes absence its own group.

## Values and toolchain

`Json`, `prefix:` on text newtypes and field `presence:` describe payloads more exactly. An exact
`requires: ess X.Y.Z` pin makes any `ess` run that release from a verified cache.
