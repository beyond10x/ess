---
format: aep.planning-md/3
id: review-result:ui-live-apps-parallel-round-2
kind: review-result
status: active
title: Plan critic parallel safety, round 2, epic:ui-live-apps
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

story:related-via-stored-reference — edits ess-synth/tests/declared_behaviour.rs like story:go-generated-behaviour with no ordering chain; the epic says the Go and ess/21 chains share no file — related-via-stored-reference.md:46 (go-generated-behaviour.md:59-60; epic :51)
story:related-via-optional-input — cites ess-gen/src/openapi.rs:1155 but leaves it untyped; story:served-view-params types the same file; no edge, no mention — related-via-optional-input.md:69
story:served-store-and-entry — --static needs {go,rust}/http.rs (Serve builds its own handler), not in its scope; story:served-view-params types both with no edge between them; served-view-params leaves {go,rust}/port.rs untyped — served-store-and-entry.md:58,100,130

Round-1 findings 1-7: all fixed (critic's check).

```findings
[
  {"file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-via-stored-reference.md", "line": 46, "category": "parallel-safety", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "edits crates/generate/ess-synth/tests/declared_behaviour.rs (cited in Acceptance, untyped) like story:go-generated-behaviour (cited, typed); no depends_on chain links them, neither body names the other, and epic line 51 says the Go and ess/21 chains share no file; add an ordering edge recording the file or move the test out of it"},
  {"file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-via-optional-input.md", "line": 69, "category": "parallel-safety", "severity": "warning", "verdict": "needs-revision", "origin": "pre-existing", "message": "cites ess-gen/src/openapi.rs:1155 (OpenAPI 'when present' wording) but leaves it out of the typed scope; story:served-view-params types the same file (comment at 560-564); no edge and no mention on either side; add an edge, type the file, or say so in Sequencing"},
  {"file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/served-store-and-entry.md", "line": 100, "category": "parallel-safety", "severity": "warning", "verdict": "needs-revision", "origin": "pre-existing", "message": "inferred: --static for unanswered paths needs ess-synth go/http.rs (Serve builds its own handler, dispatch unexported) and rust/http.rs serve_function, neither in this Scope; story:served-view-params types both http.rs files; both depend only on go-generated-behaviour with no edge between them; served-view-params also leaves {go,rust}/port.rs untyped"}
]
```
