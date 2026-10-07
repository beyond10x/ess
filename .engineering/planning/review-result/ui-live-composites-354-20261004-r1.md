---
format: aep.planning-md/3
id: review-result:ui-live-composites-354-20261004-r1
kind: review-result
status: active
title: 'UI live composites adversary pass 1: terminal re-read and shared-read effects'
relations:
- reviews: story:feature-request-354
revision: 1
---
unit: W2-7 #354 live composites in tabs and headers, pass 1
verdict: NEEDS-CHANGE
cases: executed 597→606, red 11 (6 adversary + 5 failing only because esbuild is not installed)
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: review scratch logs; build dir cleaned
needs-coordinator: no

Publication copy of the only adversary pass on the #354 unit (worktree `<worktrees>/ess/ess-w2-354-ui-live-composites-20261004`, base `5064707da`, uncommitted diff of 13 files). The reviewer added `crates/ui/ess-ui-tui/tests/adversary_live_composites_pass1.rs`, `crates/ui/ess-ui-react/tests/adversary_live_composites_pass1.rs`, `crates/ui/ess-ui-check/tests/adversary_live_composites_pass1.rs` and `crates/ui/ess-ui/tests/adversary_node_live_pass1.rs`; no production file edited. The two React cases pass where the terminal ones fail, so the renderer disagreement is measured.

F1 (blocker) `crates/ui/ess-ui-tui/src/app.rs:977`: the terminal keys a re-shown live node by path only; every row's expand shares `…/expand` (`:1240`), so an expand shown again for a row whose read is cached is never re-read and shows rows that missed events played while hidden. React re-reads it.

F2 (blocker) `app.rs:1628`: `nested_targets` drops a nested node whose read a section already takes, so the node's own live effect, match and only_if are ignored, and a paged-away section holds the event as "+1 new" so a header metric never counts it. React applies each reader's live. The implementor's own fixture shares `objectives.Evidence` between the header metric and a tab collection.

F3 (warning) `crates/ui/ess-ui/src/model.rs:961` (with `:1068`): `live` on a choice that has `reads` is refused with the false message that the node reads nothing (`Composite::reads` returns None for every choice); terminal form-field choices are also collected with `node_reads`, not `one_node_reads` (`app.rs:1254`, `:1261`).

F4 (warning) `crates/ui/ess-ui-check/src/model.rs:380` (`rules.rs:659`): `--model` checks `header.title_from.field` against nothing, so a misspelt or wire-renamed field silently pins the literal title.

Attacked and not broken: no double application; inactive tab applies no event and re-reads when shown in both renderers; `header.live` is never data delivery; loader refusals hold; bound poll paths match document node paths; title_from with empty, null or missing field agrees across renderers.

```findings
[
  {"file": "crates/ui/ess-ui-tui/src/app.rs", "line": 977, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "The terminal keys a re-shown live node by path only, so an expand shown again for a row whose read is cached is never re-read and shows rows that missed the events played while it was hidden; React re-reads it."},
  {"file": "crates/ui/ess-ui-tui/src/app.rs", "line": 1628, "category": "concurrency", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "nested_targets drops a nested node whose read a section already takes, so the node's own live effect is ignored and a paged-away section holds the event from it; React applies each reader's live."},
  {"file": "crates/ui/ess-ui/src/model.rs", "line": 961, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "live on a choice with its own reads is refused with the false message that the node reads nothing, contradicting the schema note that a member with reads takes live."},
  {"file": "crates/ui/ess-ui-check/src/model.rs", "line": 380, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "check --model holds every other row-field name to the view under row_fields but not header.title_from.field, so a misspelt or wire-renamed field silently pins the literal title."}
]
```
