---
format: aep.planning-md/3
id: review-result:adversary-gaps-272-pass-2
kind: review-result
status: active
title: Adversary pass 2, downstream gaps unit feature-request-272
relations:
- reviews: story:feature-request-272
revision: 1
---
unit: story:feature-request-272, tree gaps-272
verdict: NEEDS-CHANGE, red 3 (introduced)

- warning: a row owned through another input than the guard compares was refused although witnessed.
- warning: two inputs compared with one owner link were refused although witnessed.
- warning: every row drove a fresh related row, so a count-groups-as-1 target passed.
- note: one shared owner let a count scoped to one owner pass the ungrouped count.

Correction 2 (coordinator-verified): separate study owner for the compared input; one GuardLink with several inputs; related rows shared per via value; rows spread over two owners where no compared input is read.
Coordinator rerun in gaps-272: adversary_272 pass1 5/5, pass2 5/5, aggregate_related_guard 5/5, aggregate_related_key 4/4, adversary_257_pass2 11/11, adversary_270_pass2 9/9, adversary_271_pass2 7/7; the requester's three models synthesize with 0 refusals (0.49.0: 1 each).
