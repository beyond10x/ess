---
format: aep.planning-md/2
id: review-result:concurrent-history-design-round-1
kind: review-result
status: active
title: 'Design critic, round 1: concurrent history conformance'
relations:
- reviews: story:session-and-eventual-view-checks
- reviews: story:concurrent-history-format
- reviews: story:concurrent-explorer-runner
- reviews: story:recorded-history-validation
- reviews: story:declared-fault-injection
- reviews: story:concurrent-history-lanes
- reviews: story:linearizability-checker-over-the-interpreter
revision: 1
---
needs-revision

story:concurrent-history-lanes — its acceptance is stated only against "the shrunk `LostUpdate` history," an artifact whose existence is a product of story:concurrent-explorer-runner's shrink behaviour (and named as such in the epic's own outcome), yet the story records no `depends_on` edge to it — `.engineering/planning/story/concurrent-history-lanes.md:24` and `.engineering/planning/story/concurrent-explorer-runner.md:24-25`, with the epic's own coupling of the two at `.engineering/planning/epic/concurrent-history-conformance.md:37`.

```findings
- file: .engineering/planning/story/concurrent-history-lanes.md
  line: 24
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: the acceptance ("the shrunk `LostUpdate` history") names an artifact that only exists once story:concurrent-explorer-runner's shrink behaviour ships, but this story's relations declare depends_on only on story:linearizability-checker-over-the-interpreter, not on story:concurrent-explorer-runner
```

What I read: all 10 named artifacts (`epic:concurrent-history-conformance`, the 7 draft stories, and the 3 existing dependency stories) via `aep plan artifact show <id>`; `aep plan artifact relations` for edge semantics; `aep plan artifact graph` filtered to the set's ids and their neighbours (1019 edges total in the store, all edges touching this set inspected, none walked outside it turned up a second cycle candidate); `aep plan artifact validate` (background run, completed) — its output is entirely pre-existing review-outcome hygiene items unrelated to this set, nothing about these ten artifacts.

Walked the `depends_on` chain from every story: `concurrent-history-format` (no in-set deps) → `linearizability-checker-over-the-interpreter` → three parallel branches (`concurrent-explorer-runner`, `session-and-eventual-view-checks`, `concurrent-history-lanes`) → `declared-fault-injection` and `recorded-history-validation` converge from those branches. No cycle. Not a serialising chain — three items can proceed in parallel once the checker lands. No split abstraction found: each story's acceptance is independently demonstrable (`concurrent-history-format`'s byte-equality and refusal tests do not need the checker; `linearizability-checker-over-the-interpreter`'s LostUpdate/differential/exit-code tests do not need session/eventual or explorer work). The one file-sharing pair I checked for a split abstraction (`concurrent-explorer-runner` and `declared-fault-injection`, both touching `src/go/explore.go`/`src/ts/explore.ts`) already carries the ordering edge and reads as a legitimate shared-surface sequencing, out of my lane per the rubric's own example.

What I could not establish: whether `story:linearizability-checker-over-the-interpreter`'s handling of "an `external:` branch" as a nondeterministic-step source needs `story:external-mutation-explorer-and-toolchain` (no edge records it) — plausibly covered transitively through the interpreter's own outcome-shape derivation in `story:interpreted-command-execution`, but I could not confirm from the bodies alone; flagging as unease, not a finding.
