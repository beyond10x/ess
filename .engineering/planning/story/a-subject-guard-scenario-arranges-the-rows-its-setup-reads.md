---
format: aep.planning-md/3
id: story:a-subject-guard-scenario-arranges-the-rows-its-setup-reads
kind: story
status: draft
title: A subject-guard scenario arranges the rows its setup reads
summary: Synthesized when_subject scenario fails against the reference implementation (#554 triage)
refs:
- provider: github
  reference: beyond10x/ess#554
relations:
- serves: vision:O2
- decomposes: epic:generic-guard-precedence
revision: 1
---
## Problem

Found while triaging beyond10x/ess#554 on 0.74.0 (`ess/32`, suite `ess-conformance/34`). A command
whose outcomes are, in order, `unknown_instance: true`, a `when_subject:` predicate over a stored
Boolean, `when_related: {via: input.<id>, exists: false}`, a `when_related:` predicate on that row,
and an updating branch that sets the Boolean, synthesizes an `<command>/outcome/<subject-branch>`
scenario that fails against the built-in reference implementation:

- the scenario reaches the subject state by running the command's own updating branch, with a fresh
  identity for the related input and no arranged related row;
- that setup run answers the `exists: false` branch instead, so the subject is never changed;
- the scenario then expects the `when_subject:` branch and observes the related-row branch
  (`ESS-CF-OUTCOME`: expected `already-sealed`, observed `no-review`; `ESS-CF-VIEW`: expected
  `sealed = true`, observed `false`).

A suite ESS synthesizes must pass against ESS's own reference implementation.

## Acceptance

`ess conform synthesize` followed by `ess conform run` over the minimal specification reports the
`<subject-branch>` scenario `passed`: the setup that drives the subject into the guarded state first
arranges every row the updating branch's guards read, so the setup run takes the updating branch.
Pinned by a conformance test named
`a_subject_guard_scenario_arranges_the_related_row_its_setup_reads`.

## Evidence

Repro (minimal specification, synthesized suite and run report) outside the repository; the
specification is restated in the test fixture when the story is implemented.
