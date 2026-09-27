---
title: "0.34–0.36 — predicates, stored-field guards, aggregates, and values read from the row"
description: >
  0.34.0 adds string predicate operators, guards over the subject's stored fields and aggregate
  views. 0.35.0 resolves typed fixture values before a scenario starts. 0.36.0 lets a payload or
  sets value read the subject, increment a stored number, fall back to a minted value and fill a
  struct field by field.
slug: values-read-from-the-row
tags: [release, ess]
date: 2026-09-27T18:00:00+02:00
release_tag: "0.36.0"
release_commit: be44a3365eb273cb3d447b74cd5b0e75181d284e
---

Three releases, each adding a source format version described in
[the version history](/ess/docs/reference/spec-versions).

{/* truncate */}

## What a guard can read

0.34.0 adds `starts_with`, `ends_with` and `contains` over text, and `when_subject: {predicate: …}`,
which guards an outcome by the addressed entity's stored fields. Aggregate views count, sum and
average rows of one entity, grouped or not. Synthesis witnesses both sides of each new guard.

## Values a scenario cannot invent

0.35.0 lets a command declare `fixture_inputs:`: a deployed resource's identity, which no generator
can choose, is resolved from an independent provider before the scenario starts, and the same value
is compared in the events it causes.

## Values read from the row

0.36.0 adds value expressions under `ess/14`: `{subject: <field>}` reads the addressed entity before
the outcome, `{increment: n}` adds to a stored number, `{input: f, else: {generated: true}}` takes
an optional input or a minted value, and a nested mapping fills a struct field by field. A Decimal
literal is admitted in every format, and the text `subject.note` is refused rather than compiled as
text.
