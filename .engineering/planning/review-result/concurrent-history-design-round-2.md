---
format: aep.planning-md/3
id: review-result:concurrent-history-design-round-2
kind: review-result
status: active
title: 'Design critic, round 2: concurrent history conformance'
relations:
- reviews: story:session-and-eventual-view-checks
- reviews: story:concurrent-history-format
- reviews: story:declared-fault-injection
- reviews: story:recorded-history-validation
- reviews: story:go-explorer-concurrent-porcupine
- reviews: story:concurrent-explorer-runner
- reviews: story:linearizability-checker-over-the-interpreter
- reviews: story:concurrent-history-lanes
revision: 1
---
needs-revision

- story:concurrent-explorer-runner — its outcome re-describes the same feature story:go-explorer-concurrent-porcupine already delivers to the same file ("gains a concurrent mode: a sequential prefix, then 2–4 ... clients issuing commands at once" against `src/go/explore.go`) without saying whether it replaces that story's Go-native Porcupine check or the two coexist, and the store's own `supersedes` relation goes unused to say which — `.engineering/planning/story/go-explorer-concurrent-porcupine.md:19-20` vs `.engineering/planning/story/concurrent-explorer-runner.md:23-24`, relation vocabulary from `aep plan artifact relations`.

What I read: all 8 in-set stories and the epic via `aep plan artifact show`, plus the 3 external-dependency stories (`story:interpreted-command-execution`, `story:outcome-shapes-beyond-ess-14`, `story:external-mutation-explorer-and-toolchain`); `review-result:concurrent-history-design-round-1` for the carried-over finding (confirmed fixed: `story:concurrent-history-lanes` now carries `depends_on story:concurrent-explorer-runner`); `aep plan artifact relations` for edge semantics; `aep plan artifact graph` (full, 1019+ edges, grepped for every id in and adjacent to this set, including the new story's neighbours) — walked the `depends_on` chain from every story and found no cycle: `{concurrent-history-format, go-explorer-concurrent-porcupine}` → `linearizability-checker-over-the-interpreter` → `{concurrent-explorer-runner, session-and-eventual-view-checks}` → `{concurrent-history-lanes, declared-fault-injection, recorded-history-validation}`, with multiple parallel branches, not a serialising queue; `aep plan artifact validate` (backgrounded, completed exit 0) — its 18-no-scope / 91-no-findings-block / 45-no-outcome items are all pre-existing, store-wide, and touch none of this set's 11 artifacts, so none is my finding. Also read `review-result:concurrent-history-parallel-safety-round-2` and `review-result:concurrent-history-acceptance-round-2` (both `approve`) to confirm the file-collision on `src/go/explore.go` is already ordered in their lane and neither treated the outcome-duplication as their concern.

What I could not establish: round-1's open unease (whether `linearizability-checker-over-the-interpreter`'s handling of `external:` branches needs an edge to `story:external-mutation-explorer-and-toolchain`) is unchanged this round — still plausible but not confirmable from the bodies, so I repeat it as unease, not a finding. Out of my lane: the epic's own "Outcome" and "Prior art adapted" table (`.engineering/planning/epic/concurrent-history-conformance.md:33-52`) was not updated to mention the new `story:go-explorer-concurrent-porcupine` at all — that is a completeness/coverage question for `plan-critic-scope`, not shape, and does not set my verdict.

```findings
- file: .engineering/planning/story/concurrent-explorer-runner.md
  line: 23
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the outcome re-describes the same feature story:go-explorer-concurrent-porcupine already delivers to the same file (\"gains a concurrent mode: a sequential prefix, then 2-4 ... clients issuing commands at once\" against src/go/explore.go) without saying whether it replaces that story's Go-native Porcupine check or the two coexist, and the store's own supersedes relation goes unused to say which"
```
