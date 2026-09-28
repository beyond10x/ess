---
format: aep.planning-md/3
id: review-result:concurrent-history-scope-round-3
kind: review-result
status: active
title: Scope critic, round 3 (wave-1 revisions)
relations:
- reviews: story:declared-fault-injection
- reviews: story:session-and-eventual-view-checks
- reviews: epic:concurrent-history-conformance
- reviews: story:concurrent-history-format
- reviews: story:concurrent-explorer-runner
revision: 1
---
approve

What I read: the parent epic's whole body (`aep plan artifact show epic:concurrent-history-conformance`) before any item; all 7 decomposing stories' whole bodies (`aep plan artifact show story:{declared-fault-injection,concurrent-history-format,concurrent-explorer-runner,session-and-eventual-view-checks,linearizability-checker-over-the-interpreter,concurrent-history-lanes,recorded-history-validation}`); `aep plan artifact graph` grepped for the epic/story/successor ids; `aep plan artifact relations` and `aep plan artifact kinds`; both prior panel rounds (`review-result:concurrent-history-scope-round-1`, `-round-2`); `aep plan artifact history` on the epic and on `story:declared-fault-injection`; and `git diff 6223ff9da d6a6d7655` on the epic file, all 7 decomposing stories, the three successor stories (`explorer-takes-external-branches`, `mutate-drives-an-external-target`, `ess-manages-its-toolchain`) and the archived `story:external-mutation-explorer-and-toolchain`, plus `git log --oneline --graph 6223ff9da..d6a6d7655` to confirm the three successors landed through genuine merged implementation commits (`73ffed6d7`, `72a58ac8c`, `212480862`), not fabricated dependencies.

Extracted the same 15 promise-type statements from the parent as both prior rounds (5 Outcome sentences, 1 Specification claim, 3 named dependencies, 6 prior-art-table assignments) plus 3 named exclusions — unchanged, because the diff shows the epic's prose body is byte-identical across this round; only its `relations:` block changed. All 15 still trace to exactly one item. The dependency repoint replaces one `depends_on` to the archived `story:external-mutation-explorer-and-toolchain` with three `depends_on` edges to its already-`implemented` successors, verified real via the merge log above; the other two named dependencies (`story:interpreted-command-execution`, `story:outcome-shapes-beyond-ess-14`) are untouched. All 3 exclusions (agentplugins catalogue row, adopter-runtime deterministic simulation, TLA+/Alloy/Quint export) remain honored — none of the four touched artifacts (epic, `declared-fault-injection`, `concurrent-history-format`, `concurrent-explorer-runner`, `session-and-eventual-view-checks`) reach into them.

No gap: the epic's dependency prerequisites and its Outcome/Specification/Prior-art promises are unchanged and previously confirmed fully traced. No reach: the new machine-generated Scope sections describe implementation surface, not new outcome claims; the Go/TS equal-bytes acceptance line still claims its outcome exactly once (moved from `concurrent-history-format` to `concurrent-explorer-runner`, which the story's own text says "owns both writers"); the added model-drift line and the `StaleReadUnderReadYourWrites`/`StaleReadYourWrites` separation line operationalize outcomes the stories already claimed rather than adding new ones. No duplicate claim, no silent narrowing.

What I could not establish: (1) the epic's own "Depends on existing work" prose still names the archived `story:external-mutation-explorer-and-toolchain` (#156) by title and does not mention its three successors, even though `relations:` was repointed (`.engineering/planning/epic/concurrent-history-conformance.md`) — this is a self-consistency question in the parent's own body, not a promise a drafted item fails to claim or a claim nothing asked for, so it does not fit any of my four defect categories; it belongs to `plan-critic-design`'s lane, noted but not scored. (2) `aep plan artifact history` on the epic and on `story:declared-fault-injection` logs only the additions of the new `depends_on` edges, with no visible removal event for the edge to the archived story, though the raw file and `show`/`graph` confirm it is gone — I could not establish whether the journal records removals under an event type I did not query or resolves them on display; a store-journal integrity question outside my read verbs, and it changes no promise-coverage answer. (3) whether the Scope sections' "Would collide with" notes (e.g., `declared-fault-injection` vs. `concurrent-explorer-runner` and `session-and-eventual-view-checks`, both already `depends_on` and thus sequenced) describe a live ordering hazard is `plan-critic-parallel-safety`'s lane, not mine.

```findings
[]
```
