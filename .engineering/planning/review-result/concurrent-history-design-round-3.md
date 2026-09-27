---
format: aep.planning-md/2
id: review-result:concurrent-history-design-round-3
kind: review-result
status: active
title: Design critic, round 3 (wave-1 revisions)
relations:
- reviews: story:declared-fault-injection
- reviews: story:session-and-eventual-view-checks
- reviews: story:concurrent-explorer-runner
- reviews: epic:concurrent-history-conformance
- reviews: story:concurrent-history-format
revision: 1
---
needs-revision

epic:concurrent-history-conformance — its "Depends on existing work" section still names the archived `story:external-mutation-explorer-and-toolchain` (#156) as the reason for the "external branches" dependency, and gives no reason at all for the two other successor edges the repoint added, `story:mutate-drives-an-external-target` and `story:ess-manages-its-toolchain` — `.engineering/planning/epic/concurrent-history-conformance.md:83` vs the `relations` block at `:10-16`.

story:declared-fault-injection — its depends_on edges to `story:mutate-drives-an-external-target` and `story:ess-manages-its-toolchain` (added by the same repoint) name no design reason anywhere; the design record's own plan row for this story accounts for its dependencies as only "the runner, the session checks" plus the archived story's external-branches slice, not mutation-via-own-runner or toolchain pinning — `docs/design/concurrent-history-conformance.md:106` vs the story's `relations` block.

What I read: the 5 named artifacts (`epic:concurrent-history-conformance`, `story:declared-fault-injection`, `story:concurrent-history-format`, `story:concurrent-explorer-runner`, `story:session-and-eventual-view-checks`) via `aep plan artifact show`, plus 4 context artifacts the repoint touches (`story:explorer-takes-external-branches`, `story:mutate-drives-an-external-target`, `story:ess-manages-its-toolchain`, `story:linearizability-checker-over-the-interpreter`) and the two prior design rounds (`review-result:concurrent-history-design-round-1`, `-round-2`) to confirm their findings are already fixed (both are: the missing `concurrent-history-lanes`→`concurrent-explorer-runner` edge, and the missing `supersedes` on `concurrent-explorer-runner`). `aep plan artifact relations` for edge semantics; `aep plan artifact graph` (full store, grepped for every id in and adjacent to this set — walked ~20 `depends_on` edges, including outward to the three now-implemented successor stories, the archived predecessor, and `epic:retrofit-findings-20260927`, which those three decompose into) — no cycle. Not a serialising chain: `concurrent-history-format` → `linearizability-checker-over-the-interpreter` → parallel `{concurrent-explorer-runner, session-and-eventual-view-checks}` → converging `declared-fault-injection`/`recorded-history-validation`/`concurrent-history-lanes`, unchanged by this round's edits. No split abstraction: the Go/TS-equal-bytes acceptance line moved to `concurrent-explorer-runner`, which is the story that already owns both writers via its existing `depends_on story:concurrent-history-format` edge — this closes a seam rather than opening one. Also read `docs/design/concurrent-history-conformance.md` (the epic's own cited design record) and `aep plan artifact validate` (full run, 160 lines; nothing it reports touches any of these 5 artifacts).

What I could not establish: whether `story:linearizability-checker-over-the-interpreter`'s "external: branch" nondeterministic-step source needs an edge to `story:explorer-takes-external-branches` — round 1's unease about this, previously against the now-archived predecessor, is unchanged by this round (that story was not revised here) and I could not confirm it from the bodies alone. Out of my lane: whether the depends_on repoint should have picked one successor per story rather than all three uniformly is a scope question (does the set still cover what it was drafted from), not a shape question, and does not set my verdict.

```findings
- file: .engineering/planning/epic/concurrent-history-conformance.md
  line: 83
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the "Depends on existing work" section still names the archived story:external-mutation-explorer-and-toolchain instead of the three successors the relations block was just repointed to, and states no reason for two of the five depends_on edges (story:mutate-drives-an-external-target, story:ess-manages-its-toolchain)
- file: .engineering/planning/story/declared-fault-injection.md
  line: 8
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: depends_on edges to story:mutate-drives-an-external-target and story:ess-manages-its-toolchain name no design reason anywhere; the design record's plan table accounts for this story's dependencies as only "the runner, the session checks" plus the archived story's external-branches slice, not mutation-via-own-runner or toolchain pinning
```
