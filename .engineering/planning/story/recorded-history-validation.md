---
format: aep.planning-md/3
id: story:recorded-history-validation
kind: story
status: draft
title: A recorded production history is validated against the specification
owner: ess
tags:
- later-milestone
relations:
- depends_on: story:session-and-eventual-view-checks
- decomposes: epic:concurrent-history-conformance
- depends_on: story:linearizability-checker-over-the-interpreter
- serves: vision:O2
revision: 1
---
# Story: a recorded production history is validated against the specification (later milestone)

## Outcome

An operator converts recorded command/response logs, such as an Eventlog stream, into
`ess-history/1` through a declared adapter. `check-history` then judges the recorded history. A
field the log does not carry is reported as a coverage gap and never guessed. Precedents: AWS P +
PObserve (post-hoc validation of structured service logs) and TLA+ trace validation (SEFM 2024,
arXiv 2404.16075).

## Acceptance

- A recorded history from the billing target with `LostUpdate` active is a violation. The same run
  without the fault passes.
- A log missing return instants is refused, with the missing field named per operation.

## Scope

An adapter module beside the history format, and the `check-history` input path.
