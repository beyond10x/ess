---
format: aep.planning-md/3
id: review-result:ui-live-apps-design-round-2
kind: review-result
status: active
title: Plan critic design, round 2, epic:ui-live-apps
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

story:served-store-and-entry — the Context port (assigns a created row's identity) stays owed and nothing in the set generates it, so AddNote cannot run with no hand-written code — served-store-and-entry.md:89-103 against go-generated-behaviour.md:55, generated/go/gatepass/PLAN.md:53
story:related-guard-behaviour — generates the stored-reference via following the interpreter, but related-via-optional-input decides the interpreter keeps declining a subject via (execute.rs:524-528) and determined.rs:3-4 needs interpreter semantics — related-guard-behaviour.md:52,72 against related-via-optional-input.md:57
story:served-view-params — Decisions still says param-view queries stay obligations "until story:go-generated-behaviour … say otherwise", contradicting the epic exclusion — served-view-params.md:36 against :48

Round-1 findings 1-8 and 10: fixed; 9: partly (critic's check). Graph acyclic over 30 edges.

```findings
[
  {"file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/served-store-and-entry.md", "line": 89, "category": "design", "severity": "blocker", "verdict": "needs-revision", "origin": "introduced", "message": "the store and entry point cover only the storage port; the Context port that assigns a created row's identity stays owed (go-generated-behaviour.md:55) and nothing in the set generates it, so AddNote (a creates: with an assigned identity, determined.rs:250-263) cannot run with no hand-written code; generate an in-memory Context beside the store or name the story that does"},
  {"file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-guard-behaviour.md", "line": 72, "category": "design", "severity": "blocker", "verdict": "needs-revision", "origin": "introduced", "message": "it generates the stored-reference via following the interpreter, but related-via-optional-input:57 decides the interpreter keeps declining a subject via (execute.rs:524-528) and determined.rs:3-4 counts a behaviour as generated only if it is expressible with the interpreter's semantics; move interpreter execution of a subject via into related-via-stored-reference, or record here that determined.rs's rule is amended for this via"},
  {"file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/served-view-params.md", "line": 36, "category": "design", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "round-1 finding 9 is partly fixed (the epic excludes param-view queries) but Decisions still says they stay obligations until story:go-generated-behaviour and the Rust query generator say otherwise, contradicting its own Sequencing, the epic exclusion and go-generated-behaviour:55; cut it to 'stay obligations in every target'"}
]
```
