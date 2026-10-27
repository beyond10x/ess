---
format: aep.planning-md/3
id: story:a-binding-destination-arrangement-keeps-the-trigger-on-its-branch
kind: story
status: draft
title: A binding destination arrangement keeps the trigger on its branch
summary: arranged_destination re-checks the trigger branch after arranging, as related_decided does since 0.72.0
tags:
- ess-0.73.0
relations:
- informed_by: review-result:ess-0720-wave-1-adversary-pass-1
revision: 2
---
## Problem

When a binding's destination row is arranged (`arranged_destination` in
`synthesize/binding_effects.rs`), the arrangement can change a value the trigger sends. The only
check made is whether the invoked command reads a stored field. The trigger's branch is not
re-checked afterwards. This is the kind of gap the 0.72.0 adversary pass found and fixed in
`related_decided`, `related_missing` and the set-guard arrangement
(review-result:ess-0720-wave-1-adversary-pass-1). The adversary found this site by reading the code
and has not run a failing case on it.

## Acceptance

- A conformance case named `a_binding_destination_arrangement_keeps_the_trigger_on_its_branch`.
  Its model is one where arranging the destination row would move the trigger off its branch. On it,
  synthesis either keeps the trigger on its branch, so that every binding scenario passes
  `--target interpreted`, or refuses with ESS-SYNTH-010 naming the trigger's guard.
- No model in the suite-bytes tables moves.
