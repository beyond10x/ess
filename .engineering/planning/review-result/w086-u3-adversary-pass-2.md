---
format: aep.planning-md/3
id: review-result:w086-u3-adversary-pass-2
kind: review-result
status: active
title: 'UI-adopter wave 1: adversary pass 2 of unit U3 (page size)'
relations:
- reviews: story:a-server-paged-collection-has-a-page-size-control
revision: 1
---
# UI-adopter wave 1: adversary pass 2 of unit U3 (page size)

Unit: `story:a-server-paged-collection-has-a-page-size-control`, fix commit `2f504a8c57` (fix round 1), attacked at integration `6b1798614c` in tree `ess-w086-adv-a`.

- Verdict: needs-revision. Cases executed 113 -> 124, 3 red (`adversary_w086_u3_pass2.rs` in `ess-ui-react` and `ess-ui-check`); all four pass-1 cases green.
- Introduced: a held size of 0 is labelled "Default" while the read sends it. Routed back as fix round 2: "Default" only while the state holds nothing.
- Introduced, out of the Acceptance: `pager_params` accepts page and size on one state path, or one inside the other. The coordinator takes it into fix round 2 under the existing `pager_params` check, since the unit introduced it and the red cases exist.
- Attacked, not broken: two URL states, different depths of one URL state, the "Default" round trip with Back, numeric strings and fractions, the legacy footer on a nested path, pagers inside widget uses, `pager_outside_collection` over all ten non-collection reads.

```findings
[
{"file":"crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl","line":312,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"A size of 0 (or any held value pageSize rejects) is labelled Default while the read sends it, contradicting the doc comment at :262 and the Acceptance's showing the current size; red case adversary_a_zero_size_in_the_url_is_not_shown_as_the_default."},
{"file":"crates/ui/ess-ui-check/src/rules.rs","line":209,"category":"judgement","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"Out of scope: pager_params accepts page and size bound to one state path, or a size path inside the page path, where the page write replaces the chosen size; red cases adversary_page_and_size_bound_to_one_state_path_is_refused and adversary_a_size_path_inside_the_page_path_is_refused, and nothing found that reaches it."}
]
```
