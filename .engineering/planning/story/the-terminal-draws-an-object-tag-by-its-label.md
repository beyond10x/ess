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
scope:
- confidence: cited
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/tests/object_tags.rs
revision: 8
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

## Decisions

- An object entry without `label` is drawn as React draws it, its compact JSON (`tagText`:
  `JSON.stringify`), with keys in document order: the story brings the terminal to React's
  rendering, so the React template stays as it is.

## Acceptance

Tests run with `cargo test -p ess-ui-tui --locked --test object_tags`, over the reproduction's
collection.

- `an_object_tag_is_drawn_by_its_label`: the row `i-1` with tags
  `[{label: Alpha, id: 1}, {label: Beta, id: 2}]` draws `#Alpha #Beta`.
- `an_object_tag_without_a_label_is_drawn_as_react_draws_it`: a row with tags `[{id: 3, kind: x}]`
  draws `#{"id":3,"kind":"x"}`, the text `tagText` returns for that entry.
- `string_and_null_tags_are_unchanged`: the row `i-2` with tags `[plain]` draws `#plain`, and a row
  with tags `[plain, null]` draws `#plain`.
- `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl` is unchanged.

## Scope

- cited: `crates/ui/ess-ui-tui/src/view.rs` (the `tags` arm of the value formatter :1588)
- inferred: `crates/ui/ess-ui-tui/tests/object_tags.rs` (new)

confidence: high

Shared surfaces: proposes its `CHANGELOG.md` sentence; moves no byte-pin row.
