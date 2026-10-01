---
format: aep.planning-md/3
id: review-result:adversary-gaps-278-pass-2
kind: review-result
status: active
title: Adversary pass 2, downstream gaps unit feature-request-278
relations:
- reviews: story:feature-request-278
revision: 1
---
unit: story:feature-request-278, tree gaps-278
verdict: INFEASIBLE, red 4 (refusal text only; no synthesized scenario wrong)

- admit_alike called a truthy text guard alike to `!= ""`.
- equivalent spellings over an Optional input still rendered.
- an earlier twin the search skipped was named as the cause.
- several causes were named where one takes the input.

Correction 2 (coordinator-verified): truthiness over non-Boolean never alike; `in [x]` normalised and finite_alike over Boolean, enum and Optional inputs; refuted twins left out; one cause named.
Coordinator rerun in gaps-278: pass1 8/8, pass2 4/4, mixed_guard_overlap 5/5, related_guard_state 4/4, adversary_229_pass2 6/6. Unit gate 2023 passed, 0 failed; generate --check current.
