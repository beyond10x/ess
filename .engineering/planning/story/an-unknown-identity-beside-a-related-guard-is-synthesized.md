---
format: aep.planning-md/3
id: story:an-unknown-identity-beside-a-related-guard-is-synthesized
kind: story
status: draft
title: An unknown identity beside a related guard is synthesized
summary: ESS-SYNTH-008 refuses an unknown_instance outcome that reads no related row (#554 triage)
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
whose outcomes are `unknown_instance: true`, then `when_related: {via: input.<id>, exists: false}`,
then an updating branch, gets no `<command>/outcome/unknown-<entity>` scenario:

```text
refusal[ESS-SYNTH-008]: outcome <command>/unknown-incident has no scenario `<command>/outcome/unknown-incident`
  its command's strategy is `arrange_related_row`: ... and this scenario family arranges none
```

The unknown-identity branch is selected by the absence of the addressed subject, not by the related
row, and is the first branch: a scenario that sends an identity no row carries reaches it whatever
the related input names. The same shape with a `when_subject:` branch added (a sibling command in the
same repro) synthesizes its `unknown-incident` scenario and passes, so the refusal comes from the
command-wide `arrange_related_row` strategy being applied to a branch that does not read the related
row.

## Acceptance

`ess conform synthesize` over the minimal specification reports no ESS-SYNTH-008 for the
unknown-identity outcome and emits `<command>/outcome/unknown-<entity>`, which `ess conform run`
reports `passed` against the reference implementation. Pinned by a conformance test named
`an_unknown_identity_beside_a_related_guard_is_synthesized`.

## Evidence

Repro (minimal specification and synthesis output) outside the repository; the specification is
restated in the test fixture when the story is implemented.
