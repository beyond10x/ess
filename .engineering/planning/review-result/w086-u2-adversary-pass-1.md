---
format: aep.planning-md/3
id: review-result:w086-u2-adversary-pass-1
kind: review-result
status: active
title: 'UI-adopter wave 1: adversary pass 1 of unit U2 (toggle state)'
relations:
- reviews: story:a-toggle-shows-the-row-state-it-flips
revision: 1
---
# UI-adopter wave 1: adversary pass 1 of unit U2 (toggle state)

Unit: `story:a-toggle-shows-the-row-state-it-flips`, unit commit `b8b53b81da`, attacked at integration `d29dd12cc3` in tree `ess-w086-adv-a`.

- Verdict: needs-revision. 9 adversary cases in four crates, 4 red (`adversary_w086_u2_pass1.rs` in `ess-ui`, `ess-ui-check`, `ess-ui-tui`; the React case is green).
- Origin: 3 introduced, routed back to the unit as fix round 1 with the coordinator's decisions: drop the `loads` exemption of `toggle_state_unknown`; derive the state from the parsed negation, not from a text prefix, so `not(x)`, `(not x)` and `not (x)` name `x` and both renderers read it.
- One pre-existing note (terminal header toggles show no mark; `checked:` and bind row paths are not checked against the view's fields): filed as a draft story tagged from-finding.
- Attacked, not broken: two `not` entries, `not a and b`, `not` over literal text, extra spaces, double negation, `checked:` winning in both renderers, the `checked_needs_toggle` path, `sets:` toggles unchanged, the terminal hint offsets, toggles in widgets, account-menu entries, column switches.

```findings
[
{"file":"crates/ui/ess-ui-check/src/rules.rs","line":1160,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"toggle_state_unknown exempts a does: toggle with loads, though no renderer reads loads for a toggle's state, so such a toggle renders off unreported (red: a_command_toggle_with_loads_and_no_known_state_warns)"},
{"file":"crates/ui/ess-ui/src/toggle.rs","line":82,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"for `not (row.enabled)` the library hands both renderers the raw text `(row.enabled)`, which React evaluates to the row value and the terminal (expr.rs:65) cannot, so the two disagree with no warning (red: a_parenthesised_not_bind_shows_the_row_state)"},
{"file":"crates/ui/ess-ui/src/toggle.rs","line":83,"category":"boundary","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"`not(row.enabled)` and `(not row.enabled)` parse as one negation of a path but the text-prefix check names no state, so the toggle renders off and the warning asks for the form already written (red: a_not_bind_written_without_a_space_names_its_state, a_parenthesised_not_bind_names_its_state)"},
{"file":"crates/ui/ess-ui-tui/src/view.rs","line":425,"category":"judgement","severity":"note","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"out of scope: terminal header toggles show no on/off mark, and checked:/bind row paths are not checked against the view's fields"}
]
```
