---
format: aep.planning-md/3
id: story:a-no-view-held-state-refusal-probes-the-record
kind: story
status: draft
title: A no-view held-state refusal probes the record through a command
relations:
- serves: vision:O2
revision: 1
---
## Finding

Out of scope in adversary pass 1 of wave D357 U3 (`review-result:adversary-d357-u3-pass-1`): the no-view fallback that U3 replaced in `arranged_refusal` with a probe through the addressing command is still in `held_state_refusals` (`crates/verify/ess-conformance/src/synthesize/existence.rs`, near :613-618). A held-state refusal on a model with no identity view requires only the error and no event, so a target that changes the record on refusal and publishes nothing passes it.

## Acceptance

- On a model with no identity view, a held-state refusal scenario probes the record after the refused send through a branch that runs from the held state, as the arranged input-refusal half does, and a target that moves the record on refusal fails it; the correct target passes.
- Where no such branch has a valid send, the scenario is withdrawn and refused as `ESS-SYNTH-001` naming the branches tried.
- A model with an identity view, and every committed model, keeps its suite bytes.
