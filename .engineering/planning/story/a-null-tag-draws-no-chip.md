---
format: aep.planning-md/3
id: story:a-null-tag-draws-no-chip
kind: story
status: implemented
title: A null entry in a tags field draws no chip
summary: 'Runtime fix taken upstream: tagText drew "null" for a null list entry'
tags:
- queue-0.90.0
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:18:59Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-06T22:18:59Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-07T04:10:57Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Need

A runtime fix that a downstream adopter carries in its copy of the generated React runtime;
0.90.0 takes it upstream. A field `as: tags` whose list holds `null` draws a chip reading "null":
`tagText` in `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl` turns every entry into
text with `String(tag)`. A `null` entry carries no tag.

## Acceptance

- A record or column field `as: tags` over `["a", null, "b"]` draws two chips, `a` and `b`, in
  the React project and in the terminal.
- A list that holds only `null` draws no chip and nothing else.
- An object entry still draws its `label`. Any other value still draws its text.

## Scope

- inferred: `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl` (`tagText`, the `tags`
  display case), `crates/ui/ess-ui-tui/src/` (the tags display), the React tests for tags display.
