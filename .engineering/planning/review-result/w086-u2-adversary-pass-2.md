---
format: aep.planning-md/3
id: review-result:w086-u2-adversary-pass-2
kind: review-result
status: active
title: 'UI-adopter wave 1: adversary pass 2 of unit U2 (toggle state)'
relations:
- reviews: story:a-toggle-shows-the-row-state-it-flips
revision: 1
---
# UI-adopter wave 1: adversary pass 2 of unit U2 (toggle state)

Unit: `story:a-toggle-shows-the-row-state-it-flips`, fix commit `dd38413e71` (fix round 1), attacked at integration `f8b12559cf` in tree `ess-w086-adv-a`.

- Verdict: needs-revision. Cases executed 138 -> 151, 8 red (`adversary_w086_u2_pass2.rs` in `ess-ui`, `ess-ui-check`, `ess-ui-tui`, `ess-ui-react`); all pass-1 cases green.
- Origin: 3 introduced by fix round 1, routed back as fix round 2 with the coordinator's decisions: every string literal written quoted; a state only when every operand path starts at a known expression root; the terminal evaluates the state with the filter grammar it is written in. The Action `loads` schema note that claimed a toggle reads its value from `loads` is reworded in the same round.
- Attacked, not broken: `and`/`or` precedence under `not`, `not` inside `and`, `in` lists with quoted spaces and numbers, `checked:` winning over `sets` and a flipped bind, the `loads` removal against every example document (no new warning), React evaluating a canonical `and` state.

```findings
[
{"file":"crates/ui/ess-ui/src/toggle.rs","line":151,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"literal() writes a quoted string bare when the filter grammar reads it back, but a bare root name such as draft is a path in the display grammar the renderers use, so not (row.stage == 'draft') names row.stage == draft, React draws the draft row off and the terminal draws it on"},
{"file":"crates/ui/ess-ui/src/toggle.rs","line":85,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"inner.paths() accepts any dotted word as a path, so not item.enabled, literal text to both renderers, names the state item.enabled, every row draws on, and toggle_state_unknown does not fire"},
{"file":"crates/ui/ess-ui-tui/src/app.rs","line":1645,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"toggle_on evaluates the library's state with expr::eval, which reads no and/or/in, so states the library names from not (a and b), not (a or b) and not (x in [..]) draw off in the terminal while React draws them on, with no warning"}
]
```
