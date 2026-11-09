---
format: aep.planning-md/3
id: story:the-terminal-draws-an-object-tag-by-its-label
kind: story
status: draft
title: The terminal draws an object tag by its label, as React does
summary: 'Terminal draws an object tag as key: value, React draws its label (0.90.0 U3 implementor; from-finding)'
tags:
- backlog
- defect
- from-finding
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

Found by the U3 implementor of the 0.90.0 wave. Pre-existing, outside that unit's acceptance, and
reproduced on 0.92.0.

A field `as: tags` whose entries are objects draws their `label` in the React project (`tagText` in
`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl`). The terminal draws such an entry as
`key: value` text instead (`crates/ui/ess-ui-tui/src/view.rs`, the `Some("tags")` arm near `:1588`,
which formats each entry with `display`). The two renderers disagree on one document.

Reproduction (0.92.0): a collection `{reads: {view: items.All}, columns: [name, {field: tags, as: tags}]}`
over rows `{id: i-1, name: Anvil, tags: [{label: Alpha, id: 1}, {label: Beta, id: 2}]}` and
`{id: i-2, name: Bolt, tags: [plain]}`.

```
$ ess ui test --path ui.yaml t.yaml
failed  an object tag is drawn by its label (step 2)
        expected "#Alpha #Beta" at pages/items/sections/list/rows/i-1; it shows:
        Anvil  #label: Alpha, id: 1 #label: Beta, id: 2
passed  a string tag is drawn as before
```

## Acceptance

- A record or column field `as: tags` over `[{label: "Alpha", id: 1}, {label: "Beta", id: 2}]`
  draws the chips `#Alpha #Beta` in the terminal, as React draws `Alpha` and `Beta`.
- An object entry without `label` draws the same text in both renderers (today React writes
  `JSON.stringify` of it and the terminal `key: value`; the fix picks one and applies it to both).
- Plain string entries and null entries are unchanged.

## Scope

- cited: `crates/ui/ess-ui-tui/src/view.rs` (the `tags` arm of the value formatter)
- cited: `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl` (`tagText`, only for the label-less case)
- inferred: `crates/ui/ess-ui-tui/tests/` or `crates/ui/ess-ui-test/tests/` (the terminal value-format case)
- inferred: `CHANGELOG.md`

confidence: high
