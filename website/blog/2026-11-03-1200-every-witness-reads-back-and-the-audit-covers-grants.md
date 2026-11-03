---
title: "0.85 — Every witness reads back its effect, and the mutation audit covers grants"
description: >
  0.85.0 adds format ess/34, in which a compensating refusal may delete its row and a named refusal
  requires the row it addressed unchanged. The mutation audit gains two grant classes,
  lists the sites it could not mutate and runs on the scenario clock. Go synthesis, the clap
  target and the generated explorers fix six defects.
slug: every-witness-reads-back-and-the-audit-covers-grants
tags: [release, ess]
date: 2026-11-03T12:00:00+01:00
release_tag: "0.85.0"
---

0.85.0 adds one source format, `ess/34`, and moves the mutation report to `ess-mutation-report/8`
and the manifest to `ess-mutation-manifest/7` where they list something new. A model with nothing
new keeps its bytes. The [changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md)
lists every change; this note covers each of them.

{/* truncate */}

## Every witness reads back its effect

- From `ess/34` a compensating refusal (`compensates: true` beside `external:`) may delete its
  addressed row with `deletes:` and `instance:`. Its scenario requires the error, no event and the
  row absent from every immediate view.
- From `ess/34` a named `when:` or `when_related:` refusal of a command addressing an existing row
  snapshots that row completely and requires it unchanged. Below it, suites keep their bytes.
- At every format, a further witness of a deleting branch requires the deleted row absent, and a
  guard's boundary row for a `deletes:` branch is sent for a row of its own.
- A non-refusing `unknown_instance:` branch must store no row with the identity it was sent.
- A view row field set from an `Optional<T>` input through its declared conversion is asserted at
  the value sent.

## The mutation audit covers grants and lists its gaps

- Two new classes audit actor grants: `grant-drop` removes one command from one actor's `may:`,
  `grant-add` gives an actor one served command it lacks
  ([#710](https://github.com/beyond10x/ess/issues/710)). Where either is selected, the baseline
  and every mutant's suite keep their `<command>/grant/denied` scenarios.
- Each site a selected class excluded or could not mutate is listed with its reason under
  `unmutated_sites`, in the text, the report and the manifest
  ([#711](https://github.com/beyond10x/ess/issues/711)). The summary says how many of the sites
  considered were scored. `--fail-on-unmutated` makes such a site block exit 0; the default exit
  status is unchanged.
- `error-swap` mutates a refusal that carries its error's payload, re-keying the entry to the
  error swapped in ([#709](https://github.com/beyond10x/ess/issues/709)).
- `--target interpreted` runs synthesized scenarios on the scenario clock `run` uses, so a model
  with a `now` guard is scored.

## Go synthesis and the generated explorers

- Go synthesis copies a required event field into an `Optional` command input as a pointer, and
  writes a served component named `server` or `wire` under a repaired file name instead of
  panicking.
- `--target clap` generates one tree with every group of the components that share a
  `cli.binary`; validation refuses a group, command or view two of them both declare.
- The generated explorers agree with the interpreter on an `external:` branch for a stored row,
  fail an answer that leaves out a required response member, and explore only the scoped
  component's commands and views.
