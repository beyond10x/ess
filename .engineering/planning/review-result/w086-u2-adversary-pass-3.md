---
format: aep.planning-md/3
id: review-result:w086-u2-adversary-pass-3
kind: review-result
status: active
title: 'UI-adopter wave 1: adversary pass 3 of unit U2 (toggle state)'
relations:
- reviews: story:a-toggle-shows-the-row-state-it-flips
revision: 1
---
# UI-adopter wave 1: adversary pass 3 of unit U2 (toggle state)

Unit: `story:a-toggle-shows-the-row-state-it-flips`. The fix is commit `a4cef75e13` from fix round 2. The adversary attacked it at integration `efbf3380d0`, in tree `ess-w086-adv-a`.

## Verdict

Needs-revision, with 3 red cases. They are in `adversary_w086_u2_pass3.rs` in `ess-ui-react` and in `ess-ui-tui`.

## Introduced (acceptance)

- React's display evaluator reads no negative literal outside filter mode, so `not (row.level == -1)` draws every row on.
- The terminal hands a `checked:` the filter grammar refuses (`row.id in row.owners`) to `expr::eval`, which reads no `in`, `and` or `or`, so it is drawn off.

## Also found

- The terminal judges truthiness with `expr::truthy`, which differs from `filter::truthy` and from JavaScript. This is infeasible as filed, but it lives in the same function.
- The `toggle_state_unknown` message says "from a known root" but also fires on `actor` and `draft`.

## Decision

Everything goes back to U2 as fix round 3. The case files move into the unit.

```findings
[
{"file":"crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl","line":62,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the display evaluator reads no negative literal outside filter mode, so a toggle state `row.level == -1` from `not (row.level == -1)` draws every row on (ess-ui-react adversary_w086_u2_pass3)"},
{"file":"crates/ui/ess-ui-tui/src/app.rs","line":1660,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a checked: the filter grammar refuses (`row.id in row.owners`) falls back to expr::eval, which reads no in/and/or, so the terminal draws it off (ess-ui-tui adversary_w086_u2_pass3)"},
{"file":"crates/ui/ess-ui-tui/src/app.rs","line":1652,"category":"judgement","severity":"warning","verdict":"INFEASIBLE","origin":"introduced","message":"toggle_on judges truthiness with expr::truthy, which differs from filter::truthy and from JavaScript truthiness"},
{"file":"crates/ui/ess-ui-check/src/rules.rs","line":1083,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"toggle_state_unknown says a path must start from a known root but also fires on actor/draft, which the renderers read as roots"}
]
```
