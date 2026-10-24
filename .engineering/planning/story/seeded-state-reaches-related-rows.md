---
format: aep.planning-md/3
id: story:seeded-state-reaches-related-rows
kind: story
status: active
title: Setup reaches related rows a downstream specification's commands read
summary: A downstream specification has 91 unscenarioed scenarios; 62 of 65 ESS-SYNTH-004 refusals chain from one RenderPatch case that needs a Compiled Patch row; reproduced on 0.67.0 and 0.68.0.
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
- informed_by: story:an-input-of-list-type-may-carry-an-example
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T21:27:38Z", actor: "human:timo", revision: 4, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-22T21:27:38Z", actor: "human:timo", revision: 5, correlation: "ess-0690"}
---
## Need

A downstream specification (public repository beyond10x/patchback, commit 69e87fd) has 91 unscenarioed scenarios that setup cannot arrange (measured on 0.67.0 by the downstream session with `ess specify validate --format json` and `ess verify conform synthesize`; outside 0, unanswered 28). By code:

| code | count |
|---|---|
| ESS-SYNTH-004 (the route runs through a branch no input reaches) | 65 |
| ESS-SYNTH-003 (no candidate satisfies the branch) | 19 |
| ESS-SYNTH-008 (strategy arrange_related_row, family arranges none) | 5 |
| ESS-SYNTH-001 (no witness) | 2 |

The 65 ESS-SYNTH-004 refusals are one chain, read from the refusal texts: `audio.RenderPatch/rendered-limited` (14 cascade refusals) is the route to an `audio.Render` instance that `audio.ImportWav/imported-downmixed` needs (13), whose output `audio.AudioClip` is needed by `fit.StartFit/started-seeded` (30), whose `fit.FitRun` is needed by `fit.ProposeCandidate/proposed` (5). Those four roots are 62 of the 65. The fifth root, `analysis.ComputeNormalizationStats/computed` (3), is separate and was not traced.

Root case: `RenderPatch/rendered` and `rendered-limited` need a related `patchback.patch.Patch` row in state `Compiled`; the `patch-not-compiled` branch is `when_related: {via: input.patch_id, predicate: state != Compiled}`. Synthesis tries 1 arrangement and reports "no candidate of the 1 tried satisfies ... a present related row on which no `when_related` branch claims the input" (ESS-SYNTH-003). The authored scenario `scenarios/subtractive-voice-render-renders.yaml` reaches Compiled through LoadPatch, ValidatePatch and CompilePatch and passes on the interpreted target, so the row is reachable by hand.

The relayed word "seeds" is read here as setup arranging a related row (the downstream repro files are about exactly that; not confirmed in words by them). The first draft of this story guessed an overlap with the related-selector drafts; the traced root case is a single `when_related` state predicate, so that overlap is not supported by the evidence (see Reproduction).

## Reproduction

Reproduced on 0.68.0 with the ess binary of that release, on a copy of the downstream specification (its `ess-inputs.yaml` pin edited from 0.67.0 to 0.68.0, because the dispatcher otherwise delegates to the pinned 0.67.0; the first attempt did exactly that and was discarded). The copy carries the `nodes` view field of the Decimal story (`story:a-view-may-publish-a-decimal`), so the figures include its 12 extra refusals.

| | unscenarioed | unanswered | SYNTH-004 | SYNTH-003 | SYNTH-008 | SYNTH-001 |
|---|---|---|---|---|---|---|
| 0.67.0, commit 69e87fd (downstream measure) | 91 | 28 | 65 | 19 | 5 | 2 |
| 0.67.0, with `nodes` in the view | 103 | 12 | 65 | 19 | 5 | 14 |
| 0.68.0, with `nodes` in the view | 103 | 12 | 65 | 19 | 5 | 14 |

On 0.68.0 the cascade roots are the same: StartFit/started-seeded 30, RenderPatch/rendered-limited 14, ImportWav/imported-downmixed 13, ProposeCandidate/proposed 5, ComputeNormalizationStats/computed 3. Nothing changed between 0.67.0 and 0.68.0 for this gap. The run without `nodes` was not repeated on 0.68.0.

Cause of the root case, verified at the witness: the synthesized `LoadPatch` execute step carries `nodes: []` and `edges` holding one edge between node 1635481 and itself (from the written suite, 0.68.0 run). An edge to a node the patch does not declare is the `unknown-node` input of `ValidatePatch`, so the walk through LoadPatch, ValidatePatch and CompilePatch cannot end in `Compiled`. The `nodes: []` comes from the list witness rule (see `story:an-input-of-list-type-may-carry-an-example`). That `ValidatePatch` then answers `unknown-node` for this input is inferred from the downstream outcome name; not run.

## Where it lives

- `crates/verify/ess-conformance/src/synthesize.rs:1157` (`unsatisfied`, which yields ESS-SYNTH-003) and `:4351` (`unclaimed_external`), as in the first pass (inferred for this report; not re-traced).
- `crates/verify/ess-conformance/src/witness.rs:993` and `:5355-5372`: a `List` input without an expansion or copy is witnessed as the empty sequence (traced); this is where `nodes: []` comes from.

## Acceptance (provisional)

- Once a list input can carry a witness (`story:an-input-of-list-type-may-carry-an-example`), the downstream `RenderPatch/rendered` and `rendered-limited` scenarios are synthesized with a related Patch row in state Compiled, or refused with a reason that names what is missing.
- The 62 cascade refusals under that chain drop with it; the count is measured by the downstream specification, not promised here.
- The remaining 19 ESS-SYNTH-003, 5 ESS-SYNTH-008, 2 ESS-SYNTH-001 and the ComputeNormalizationStats root are grouped by cause afterwards; each is folded into `story:a-command-with-two-related-selectors-arranges-both`, `story:a-related-selector-compares-a-row-field-with-a-subject-field` or `story:a-copied-related-identity-is-witnessed` where it fits, or gets its own story.
