---
format: aep.planning-md/3
id: story:a-state-only-a-set-move-reaches-is-named-in-the-refusal
kind: story
status: draft
title: A state only a set move reaches is named in the refusal
summary: 'Adopter batch 2c L: the documented set-move limitation is refused with a no-three-rows message and filter help that do not name it'
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Adopter review batch 2c, item L (repro `repro-r3a-tel1`, ess/25; reproduced on 0.69.2 and on the 0.70.0 integration head). `Resume/resumed` moves every `Child` where `key == input.key` from `Parked` to `Held`; `Parked` is reached only through `Park/parked`, itself a filtered set move. Synthesis refuses `resumed` as ESS-SYNTH-001 "no three rows the filter selects that the arranging commands can produce", with help about writing filtered fields from an input. The refusal is the documented limitation (`docs/design/set-effects-over-filtered-instances.md:63`: a state reachable only through a set move cannot be arranged for another scenario), but the message names neither the state nor the set move, and its help points at the filter, which is already written from an input.

## Acceptance

- The refusal of `repro-r3a-tel1` names `Parked` as a state only the set move `demo.z.Park/parked` reaches and says such a state is not arranged; its help names the remedy (a single-row transition into the state, or an authored scenario).
- The refusal keeps its code and its occurrence count; other "no three rows" refusals keep their text.
