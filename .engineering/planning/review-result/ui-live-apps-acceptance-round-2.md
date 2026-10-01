---
format: aep.planning-md/3
id: review-result:ui-live-apps-acceptance-round-2
kind: review-result
status: active
title: Plan critic acceptance, round 2, epic:ui-live-apps
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

story:served-store-and-entry — the served-notes fixture names no actor granted AddNote, yet the entry-point tests expect AddNote 202 under --callers actor-header and 403 not granted without it; a command no declared actor may invoke is refused to every caller — ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/served-store-and-entry.md:109-118 against website/docs/guides/synthesize.md:278-279

Round-1 findings 1-7: all fixed (critic's check). Not established: whether the gatepass servers produce 501 committed/not answers for answers.json.

```findings
[
  {"file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/served-store-and-entry.md", "line": 110, "category": "acceptance", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "the served-notes fixture names no actor granted AddNote, yet the entry-point tests expect AddNote 202 under --callers actor-header and 403 not granted without it; a command no declared actor may invoke is refused to every caller (synthesize.md:278-279), so the 202 cannot pass until the body names the fixture's granted actor and the actor the tests send"}
]
```
