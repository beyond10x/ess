---
format: aep.planning-md/3
id: review-result:w086-u1-adversary-pass-1
kind: review-result
status: active
title: 'UI-adopter wave 1: adversary pass 1 of unit U1 (value labels)'
relations:
- reviews: story:a-value-shows-its-declared-label-beside-its-tone
revision: 1
---
# UI-adopter wave 1: adversary pass 1 of unit U1 (value labels)

Unit: `story:a-value-shows-its-declared-label-beside-its-tone`, unit commit `fe9b8046d8`, attacked at integration `27fb9d4b22` in tree `ess-w086-adv-a`.

- Verdict: needs-revision. Cases executed 29 -> 33, 3 red (`crates/ui/ess-ui/tests/adversary_w086_u1_pass1.rs`: a page, a page state and a widget named `labels` no longer load; a control with the page named `runs` loads).
- Origin: introduced (the base `expand.rs` at `83391ebaeb` has no `labels` handling).
- Attacked, not broken: boolean and number keys, a key written twice, widget arguments, `label_map_refs`/`label_map_unused` paths, React key quoting, prototype-named values, secret and tags fields, choice-label order (React and terminal agree), terminal records and tabs, badge tone and class from the raw value.
- Routed back to the unit as fix round 1; the adversary file is committed with the fix.

```findings
[
{"file":"crates/ui/ess-ui/src/expand.rs","line":1619,"category":"contract-drift","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"resolve_labels rewrites every mapping key spelled `labels` under shells/pages/widgets, and OPAQUE now skips it, so a document that names a page, page state or widget `labels` (without using the new construct) no longer loads (a_page_named_labels_loads, a_page_state_named_labels_loads, a_widget_named_labels_loads_and_expands)."}
]
```
