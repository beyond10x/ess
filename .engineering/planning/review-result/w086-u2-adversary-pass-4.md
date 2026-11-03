---
format: aep.planning-md/3
id: review-result:w086-u2-adversary-pass-4
kind: review-result
status: active
title: 'UI-adopter wave 1: adversary pass 4 of unit U2 (toggle state)'
relations:
- reviews: story:a-toggle-shows-the-row-state-it-flips
revision: 1
---
# UI-adopter wave 1: adversary pass 4 of unit U2 (toggle state)

Unit: `story:a-toggle-shows-the-row-state-it-flips`. Pass 4 attacked fix commit `6d819f3177` (fix round 3) in tree `ess-w086-u2`.

- Verdict: approve, with 137 cases and 3 red. The red cases are hand-built `checked:` expressions that no downstream adopter was shown to write. Both findings are INFEASIBLE notes, and the coordinator's call is approve.
- F1: `checked_unreadable` is silent on a `checked:` that reads no path (`not true`, `1 == 2`, `not -1`). React returns such text as truthy literal text (`expr.ts.tmpl:380`), while the terminal evaluates it (`rules.rs:1264`). Filed as `story:a-checked-that-reads-no-path-draws-alike`.
- F2: React's tokenizer splits on ASCII whitespace only (`expr.ts.tmpl:48`), so a no-break space makes `evaluate` return text, and the filter grammar reads it. Filed as `story:both-renderers-tokenize-whitespace-alike`.
- The 3 red cases are copied into those drafts and not committed; the files were removed from the unit tree.
- Attacked, not broken:
  - negative literals in every position;
  - `checked_unreadable` on unreadable texts and on roots that are not admitted;
  - `filter::truthy` against React on empty list, empty map, `"false"`, `0`, null and a missing field.

```findings
[
{"file":"crates/ui/ess-ui-check/src/rules.rs","line":1264,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"checked_unreadable is silent on a checked: that reads no path (not true, 1 == 2, not -1); React's evaluate returns it as truthy literal text (expr.ts.tmpl:380) while the terminal evaluates it, so the marks differ"},
{"file":"crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl","line":48,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"React's tokenizer splits on ASCII whitespace only, so a no-break space makes evaluate return text while the filter grammar reads the expression; the check is silent"}
]
```
