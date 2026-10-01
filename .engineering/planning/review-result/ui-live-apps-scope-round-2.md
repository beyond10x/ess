---
format: aep.planning-md/3
id: review-result:ui-live-apps-scope-round-2
kind: review-result
status: active
title: Plan critic scope, round 2, epic:ui-live-apps
relations:
- reviews: story:ui-react-plain
- reviews: story:ui-binding-contract
- reviews: story:ui-react-live-binding
- reviews: story:ui-tui-live-binding
- reviews: story:ui-tui-app-generator
- reviews: story:served-view-params
- reviews: story:go-generated-behaviour
- reviews: story:served-store-and-entry
- reviews: story:related-guard-behaviour
- reviews: story:related-via-optional-input
- reviews: story:related-via-stored-reference
revision: 1
---
needs-revision

ui-binding-contract — the epic says server paging is "refused by name or answered by an idiom (story:ui-binding-contract)" but this story's refusals cover only an unserved name, a non-scalar param and server-held state, with no test refusing a paging: server/cursor read, and its ViewRoute carries paging: Option<PagingParams> — ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-binding-contract.md:66

Round-1 findings 1-5: all fixed (critic's check).

```findings
[
  {"file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-binding-contract.md", "line": 66, "category": "scope", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "the epic says server paging is refused by name or answered by an idiom (story:ui-binding-contract) but this story's refusals cover only an unserved name, a non-scalar param and server-held state, no acceptance test refuses a paging: server/cursor read, and its ViewRoute carries paging: Option<PagingParams>, modelling the paging the epic excludes; add a by-name refusal and its test and drop the field, or record the narrowing"}
]
```
