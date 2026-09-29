---
title: "0.41 — state-scoped refusals, guards over a related row, and a delivery context"
description: >
  0.41.0 adds source format `ess/18`: a refusal scoped to some held states, `state` in a
  `when_subject` predicate, a guard over the row of another entity, and a binding that reads the
  context its event was delivered with.
slug: state-scoped-refusals
tags: [release, ess]
date: 2026-09-29T18:00:00+02:00
release_tag: "0.41.0"
release_commit: a4aa8b6154bdd46e2cdbfd5e57f09da874808e0c
---

0.41.0 adds the source format `ess/18`, described in
[the version history](/ess/docs/reference/spec-versions).

{/* truncate */}

## Refusals by held state

`when_subject_state:` may list several states, and a refusal may carry it without naming a
subject, so a command can answer differently in states its moves do not start from. A
`when_subject` predicate may read `state` beside the stored fields.

## A guard over another entity

`when_related:` selects a branch by the row of another entity whose identity the input carries:
`exists: false` when there is none, or a predicate over its stored fields.

## Delivery context

An event binding may declare the context its channel delivers with the event and read it as
`context.<field>`; conformance delivers such an event itself.

## Synthesis and mutation

Map inputs, view filters over identities and links, every creating command, and the record an
input refusal needs are now witnessed. The mutation audit scores gained refusals and skipped
baselines apart from survivors.
