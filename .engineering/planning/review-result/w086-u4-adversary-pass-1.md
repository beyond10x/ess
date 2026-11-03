---
format: aep.planning-md/3
id: review-result:w086-u4-adversary-pass-1
kind: review-result
status: active
title: 'UI-adopter wave 1: adversary pass 1 of unit U4 (load error path)'
relations:
- reviews: story:a-refused-node-is-reported-at-its-own-path
revision: 1
---
# UI-adopter wave 1: adversary pass 1 of unit U4 (load error path)

Unit: `story:a-refused-node-is-reported-at-its-own-path`, unit commit `c75123668d`, attacked at integration `f8b12559cf` in tree `ess-w086-adv-a`.

- Verdict: needs-revision. Cases executed 165 -> 172, 4 red (`adversary_w086_u4_pass1.rs` in `ess-ui`, `ess-ui-check`, `ess-ui-react`).
- Introduced: the `states` arms of `locate` look only at the last key (blocker); `export_selection` admits `bulk: true` though the runtime scopes `selection` only around `bulk_actions`; two notes (the untagged `ExportParams` error names no shape; an empty title draws an empty terminal line).
- Pre-existing: entries of `label_maps`, `tone_maps` and `themes` are still read as container keys. `label_maps` is new in this release, so the coordinator takes it into the fix round.
- All five routed back as fix round 1: `states` arms only where the model puts section states; only `bulk_actions` counts as bulk; named entries of those document maps; a message naming the expected shapes; an empty title is no title in the terminal.
- Attacked, not broken: refusals at `states/empty`, `failed`, `forbidden`, `stale` (unknown key, missing `message`, wrongly typed `title`), a refused key in the empty state's `action`, nested export params refused at the bulk action, `params: {}`, `export_selection` on header, empty-state and record actions.

```findings
[
{"file":"crates/ui/ess-ui/src/locate.rs","line":131,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the new states arms classify any map under a key named `states` as SectionStates, so a valid tone map or state record field named `states` is refused first and hides the real refusal (adversary_w086_u4_pass1: a_tone_map_named_states_hides_no_later_refusal, a_state_type_field_named_states_hides_no_later_refusal)"},
{"file":"crates/ui/ess-ui-check/src/rules.rs","line":51,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"export_selection admits selection.<field> on any `bulk: true` action and says so in its message, but the React runtime scopes `selection` only around bulk_actions, so such an export reads `{}` (ess-ui-react adversary_w086_u4_pass1: a_selection_param_the_check_admits_reaches_the_export)"},
{"file":"crates/ui/ess-ui/src/locate.rs","line":149,"category":"acceptance","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"a map entry the author names in a map that NodePath.syntax.containers does not list (label_maps, tone_maps, themes) is still read as a container key, so a label map named `action` is refused as an Action and hides the real refusal (out_of_scope_a_label_map_named_action_hides_no_later_refusal)"},
{"file":"crates/ui/ess-ui/src/model.rs","line":2457,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the untagged ExportParams turns a wrongly shaped params value into a generic untagged-enum error naming neither the key nor the expected shape"},
{"file":"crates/ui/ess-ui-tui/src/view.rs","line":562,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"an empty-string title draws an empty bold line in the terminal while React draws no heading"}
]
```
