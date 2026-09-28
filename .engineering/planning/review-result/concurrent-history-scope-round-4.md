---
format: aep.planning-md/3
id: review-result:concurrent-history-scope-round-4
kind: review-result
status: active
title: Scope critic, round 4 (wave-1 revisions)
relations:
- reviews: story:declared-fault-injection
- reviews: story:concurrent-history-format
- reviews: story:concurrent-explorer-runner
- reviews: story:session-and-eventual-view-checks
- reviews: epic:concurrent-history-conformance
revision: 1
---
approve

What I read: parent epic whole body first (`aep plan artifact show epic:concurrent-history-conformance`), before any item, and wrote down its promise list; then all 5 named items' whole bodies (`aep plan artifact show story:{declared-fault-injection,concurrent-history-format,concurrent-explorer-runner,session-and-eventual-view-checks}`); `aep plan artifact graph` grepped for the epic, the five items, their dependency targets and the three retrofit successors; `aep plan artifact relations`/`kinds`; the prior round `review-result:concurrent-history-{acceptance,design,scope,parallel-safety}-round-3`; `git diff` on the epic, `session-and-eventual-view-checks`, `concurrent-history-format`, `declared-fault-injection` and `interpreted-command-execution` (all five files the working tree shows modified) against `6223ff9da`/`d6a6d7655`; and `git log --oneline` on the two named files to confirm no other revision landed unremarked.

Extracted the same 15 promise-type statements from the parent as round 3 (5 Outcome sentences, 1 Specification claim, 3 named dependencies, 6 prior-art-table assignments) plus 3 named exclusions. All 15 still trace to exactly one item, and all 3 exclusions are still honored — the parent's prose body is unchanged except the "Depends on existing work" paragraph, and that paragraph still names the same 3 dependencies the `relations:` block carries (`story:interpreted-command-execution`, `story:outcome-shapes-beyond-ess-14`, `story:explorer-takes-external-branches`), just with new sentences explaining why the other two retrofit successors carry no edge.

Both round-4 revisions are non-scope moves: (1) the epic's `depends_on` edges to `story:mutate-drives-an-external-target` and `story:ess-manages-its-toolchain` are removed and the epic's own prose now says why — this closes the self-inconsistency `plan-critic-scope` round 3 flagged as out-of-lane and `plan-critic-design`/`plan-critic-parallel-safety` round 3 flagged as a design defect (needs-revision on exactly this point); removing the edges drops no promise, since neither successor's capability appears in any Outcome/Acceptance sentence of the epic or `story:declared-fault-injection`. (2) `story:session-and-eventual-view-checks`'s bundled acceptance bullet is split into two bullets carrying the identical two claims (new row caught only by the history check; `StaleReadYourWrites` stays a separate row caught by the single-client suite) — same content, no promise added, none dropped, none duplicated. Neither revision creates reach beyond the parent, a duplicate claim, or a silent narrowing.

What I could not establish: none — both revisions are scoped exactly to what round 3's acceptance and design/parallel-safety findings named, and I confirmed each against the diff rather than the prose alone.

```findings
[]
```
