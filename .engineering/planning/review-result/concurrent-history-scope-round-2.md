---
format: aep.planning-md/2
id: review-result:concurrent-history-scope-round-2
kind: review-result
status: active
title: 'Scope critic, round 2: concurrent history conformance'
relations:
- reviews: story:linearizability-checker-over-the-interpreter
- reviews: story:recorded-history-validation
- reviews: story:declared-fault-injection
- reviews: story:concurrent-history-lanes
- reviews: story:go-explorer-concurrent-porcupine
- reviews: story:concurrent-history-format
- reviews: story:concurrent-explorer-runner
- reviews: story:session-and-eventual-view-checks
revision: 1
---
approve or needs-revision verdict below, per the plan-critic-scope rubric.

**needs-revision**

story:go-explorer-concurrent-porcupine — pins the emitted Go package to a production dependency on Porcupine v1.3.1 (verified through the adopter's `go.sum`) and checks concurrency against "the explorer's existing IR model" rather than "the specification's own model" the epic names (the Rust interpreter, per the epic's own dependency list); the epic's prior-art table assigns Porcupine (Go) only to `story:linearizability-checker-over-the-interpreter` and `story:concurrent-history-lanes`, and nothing else in the epic promises a production-facing Go/Porcupine dependency for adopters — .engineering/planning/story/go-explorer-concurrent-porcupine.md:19-30, .engineering/planning/epic/concurrent-history-conformance.md:35-36,46,56-57

story:concurrent-explorer-runner — restates, on the Go side, the same outcome story:go-explorer-concurrent-porcupine already claims ("the explorer gains a concurrent mode: a sequential prefix, then 2–4 [goroutine] clients issuing commands at once") without saying whether it builds on, replaces, or runs alongside that story's already-added concurrent mode in the same file (`src/go/explore.go`), so both will be markable done for what reads as one mechanism — .engineering/planning/story/concurrent-explorer-runner.md:23-24, .engineering/planning/story/go-explorer-concurrent-porcupine.md:19-20

What I read: the parent (`aep plan artifact show epic:concurrent-history-conformance`, revision 2), all 8 drafted stories including the new `story:go-explorer-concurrent-porcupine` (`aep plan artifact show story:<id>` for each), `review-result:concurrent-history-scope-round-1` (my own prior round), `aep plan artifact graph` (confirmed exactly 8 `decomposes` edges onto the epic, none elsewhere), `aep plan artifact relations` and `aep plan artifact kinds`, `aep plan artifact history` on the epic and on the four artifacts round-1 touched, plus the raw files under `.engineering/planning/epic/` and `.engineering/planning/story/` for line-accurate citations.

Extracted the same 15 promise-type statements from the parent as round 1 (5 Outcome sentences, 1 Specification claim, 3 named dependencies, 6 prior-art-table assignments) plus 3 named exclusions. Both round-1 findings are resolved and not reintroduced: the epic's Outcome (lines 37-40, revision 2) now explicitly carves `story:concurrent-history-lanes` and `story:recorded-history-validation` out of the "no earlier check catches" requirement, and `story:declared-fault-injection` (revision 2) replaced the ungrounded "target restart" fault with an explicit "Not covered" line naming the same reason round 1 gave. All 15 promises still trace to exactly one item, and all 3 exclusions are still honored. The new story does not fill a gap — it is additive, and its outcome is what the two findings above concern.

What I could not establish: whether the drafter intends `story:go-explorer-concurrent-porcupine`'s concurrent mode to be superseded/refactored once `story:concurrent-explorer-runner` ships, or to persist as a permanent parallel Go-only check — no artifact says which, so I could not tell whether this is a genuine second finding or one defect described twice; whether the two stories' shared scope on `src/go/explore.go` is a live ordering hazard is plan-critic-parallel-safety's lane, not mine, and I note but do not score it; whether either story's acceptance is checkable is plan-critic-acceptance's lane.

```findings
- file: .engineering/planning/story/go-explorer-concurrent-porcupine.md
  line: 19
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: the outcome commits the emitted Go package to a production dependency on Porcupine v1.3.1 and checks against the explorer's own IR model rather than the specification's own model (the Rust interpreter) the epic names, and this is not traceable to any sentence in the parent, whose prior-art table assigns Porcupine (Go) only to story:linearizability-checker-over-the-interpreter and story:concurrent-history-lanes
- file: .engineering/planning/story/concurrent-explorer-runner.md
  line: 23
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: the outcome restates, on the Go side, the same "gains a concurrent mode" claim story:go-explorer-concurrent-porcupine already makes for the same file, with neither body saying whether one builds on, replaces, or runs alongside the other
```

Relevant paths: `.engineering/planning/epic/concurrent-history-conformance.md`, `.engineering/planning/story/go-explorer-concurrent-porcupine.md`, `.engineering/planning/story/concurrent-explorer-runner.md`, `.engineering/planning/story/concurrent-history-format.md`, `.engineering/planning/story/declared-fault-injection.md`, `.engineering/planning/review-result/concurrent-history-scope-round-1.md`.
