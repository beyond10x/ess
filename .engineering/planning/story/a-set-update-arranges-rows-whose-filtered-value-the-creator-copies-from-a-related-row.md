---
format: aep.planning-md/3
id: story:a-set-update-arranges-rows-whose-filtered-value-the-creator-copies-from-a-related-row
kind: story
status: draft
title: A set update arranges rows whose value the creator copies from a related row
summary: 'Adopter batch 2c K: a creator copying flag from the related row leaves SyncFlag/synced refused as ESS-SYNTH-001 no three rows'
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Adopter review batch 2c, item K (repro `repro-r3a-r1`, ess/25; reproduced on 0.69.2 and on the 0.70.0 integration head). `AddChild` creates a `Child` with `flag: {related: {via: input.parent_id, field: flag}}`; `SyncFlag/synced` updates every `Child` where `parent_id == input.parent_id`, writing `flag: true`. Synthesis refuses `synced` as ESS-SYNTH-001: "`synced.instances` is `demo.z.Child`, which has no three rows the filter selects that the arranging commands can produce" (2 scenarios, 1 refusal). A literal `flag: false` gives 3/0; a copied value on a field the update does not write still refuses. The fixes of beyond10x/ess#682 and 0.65.0 cover the creator's `when_related` refusal, not a value the creator copies from the related row. The help ("write each field the filter reads from an input or a caller attribute") does not apply: the filter reads `parent_id`, an input.

## Acceptance

- `repro-r3a-r1` and `repro-r3a` synthesize `SyncFlag/synced` with no refusal, and the suite passes the interpreter.
- A model whose creator writes the filtered fields from inputs keeps its suite bytes.
