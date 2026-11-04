---
format: aep.planning-md/3
id: review-result:w086-u3-adversary-pass-1
kind: review-result
status: active
title: 'UI-adopter wave 1: adversary pass 1 of unit U3 (page size)'
relations:
- reviews: story:a-server-paged-collection-has-a-page-size-control
revision: 1
---
# UI-adopter wave 1: adversary pass 1 of unit U3 (page size)

Unit: `story:a-server-paged-collection-has-a-page-size-control`, unit commit `05114f9e2f`, attacked at integration `27fb9d4b22` in tree `ess-w086-adv-a`.

- Verdict: needs-revision. Cases executed 123 -> 127, 4 red (`crates/ui/ess-ui-react/tests/adversary_w086_u3_pass1.rs`, `crates/ui/ess-ui-check/tests/adversary_w086_u3_pass1.rs`).
- Origin: 4 introduced, routed back to the unit as fix round 1 with the coordinator's decisions: one write for both paths under one state, the select offers a current size missing from `sizes`, the page count only from a size of at least 1, a new error check `pager_outside_collection`.
- One pre-existing note: the fixture adapter pages by the param names `page` and `size`; filed as a draft story tagged from-finding.
- Attacked, not broken: `pager_params` and `pager_sizes` refusals, four mutants of the Next guard and page count, the default sizes drift check, the legacy footer, URL-store writes, the reference page.

```findings
[
{"file":"crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl","line":297,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"choosing a size when page and size bind sub-paths of one state loses the size write, because two scope.set calls each rebuild the state from the stale render values (adversary_choosing_a_size_keeps_the_size_when_both_paths_share_one_state)"},
{"file":"crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl","line":294,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a size state whose value is not among sizes gives a select with no matching option, so a browser shows the first size instead of the current one and choosing it fires no change (adversary_the_select_offers_the_size_the_read_sends)"},
{"file":"crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl","line":275,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"an unset size state falls back to sizes[0] for the page count, so Next is disabled on a page that is not the last (adversary_an_unset_size_does_not_disable_next)"},
{"file":"crates/ui/ess-ui-check/src/rules.rs","line":166,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the check accepts a pager on any composite's read, but only the collection renders one, so a declared pager on a record, board, choice or action loads shows no footer and nothing reports it (adversary_a_pager_no_renderer_shows_is_reported)"},
{"file":"crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl","line":141,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"out of scope: the fixture adapter pages by the param names page and size, so with the default fixture adapter a pager naming other params changes the page number but not the rows"}
]
```
