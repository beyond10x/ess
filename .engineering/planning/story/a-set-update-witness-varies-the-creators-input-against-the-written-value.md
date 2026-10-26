---
format: aep.planning-md/3
id: story:a-set-update-witness-varies-the-creators-input-against-the-written-value
kind: story
status: active
title: A set update witness varies the creator's input against the written value
summary: 'Adopter batch 2c M: creator and set update both write input.flag, synthesis sends true to both, no row changes, refused'
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T21:52:54Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-26T21:52:54Z", actor: "human:timo", revision: 3}
---
## Problem

Adopter review batch 2c, item M (repros `r3a-work/t1_true` and `r3a-work/t2_creator_input`, ess/25; reproduced on 0.69.2 and on the 0.70.0 integration head). The creator and the set update both write `flag` from `input.flag`. Synthesis sends `true` to both and never tries `false` for the creator, so no selected row would change, and `synced` is refused as ESS-SYNTH-001 "no three rows the filter selects that the arranging commands can produce". The value choice for the set update's rows-change witness does not vary the creator's input against the update's value.

## Acceptance

- `t1_true` and `t2_creator_input` synthesize `synced` with no refusal: the arranged rows hold a value the update changes, and the suite passes the interpreter.
- A model whose creator writes a literal keeps its suite bytes.
