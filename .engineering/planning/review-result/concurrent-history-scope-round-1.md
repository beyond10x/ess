---
format: aep.planning-md/2
id: review-result:concurrent-history-scope-round-1
kind: review-result
status: active
title: 'Scope critic, round 1: concurrent history conformance'
relations:
- reviews: story:concurrent-history-format
- reviews: story:concurrent-history-lanes
- reviews: story:session-and-eventual-view-checks
- reviews: story:concurrent-explorer-runner
- reviews: story:linearizability-checker-over-the-interpreter
- reviews: story:recorded-history-validation
- reviews: story:declared-fault-injection
revision: 1
---
approve or needs-revision verdict below, per the plan-critic-scope rubric.

**needs-revision**

story:recorded-history-validation — the epic requires "Each check has a planted fault in `faulty.rs` that it catches and that no earlier check catches," but this story's only fault-linked acceptance reuses `LostUpdate`, already caught by story:linearizability-checker-over-the-interpreter, with nothing said about dropping the "no earlier check catches" requirement for this story — `.engineering/planning/epic/concurrent-history-conformance.md:37-38` (quoted), `.engineering/planning/story/recorded-history-validation.md:29`

story:declared-fault-injection — the "target restart between operations, where the target declares it supports one" fault is not traceable to any sentence in the parent: the epic scopes injected faults to what "the specification already declares," and the other three bullets are each grounded in existing spec vocabulary (`delivery: at_least_once`, `replays`, `external:` branches — confirmed present in `examples/billing/components.yaml:62`, fixtures under `crates/verify/ess-conformance/tests/fixtures/retained-replay.yaml`, `examples/billing/domains/email.yaml:45`), but no restart-capability declaration exists in any specification file, realization, or the `Target` trait (`crates/verify/ess-conformance/src/target.rs`) — `.engineering/planning/story/declared-fault-injection.md:24`, `.engineering/planning/epic/concurrent-history-conformance.md:36-37`

What I read: the parent (`aep plan artifact show epic:concurrent-history-conformance`), all 7 drafted stories (`aep plan artifact show story:concurrent-history-format|linearizability-checker-over-the-interpreter|concurrent-explorer-runner|session-and-eventual-view-checks|declared-fault-injection|concurrent-history-lanes|recorded-history-validation`), `aep plan artifact graph` (grepped for the epic/story ids, confirming exactly these 7 `decomposes` edges and no other artifact claiming part of the epic), `aep plan artifact relations` (confirmed `decomposes` is the drafted-from edge), plus grep over `models/concurrent-history/`, `examples/billing/`, and `crates/verify/ess-conformance/src/target.rs` to check the spec-vocabulary grounding of the fault-injection bullets and the epic's `consistency:` levels (only `Current` (unmarked), `read_your_writes`, `eventual` exist anywhere in the tree).

Extracted 15 promise-type statements from the parent (5 Outcome sentences, 1 Specification claim, 3 named dependencies, 6 prior-art-table assignments) plus 3 named exclusions. 14 traced cleanly to exactly one item; 1 (the "no earlier check catches" sentence) traced only partially, as above. All 3 exclusions (agentplugins catalogue row, adopter-runtime deterministic simulation, TLA+/Alloy/Quint export) are honored — nothing in the 7 stories reaches into them.

What I could not establish: whether "target restart" is meant to introduce new specification vocabulary the epic simply didn't spell out (undecided — no evidence either way in the store); whether the overlapping `Scope` mentions of `src/go/explore.go`/`src/ts/explore.ts` in both story:concurrent-history-format and story:concurrent-explorer-runner represent a real ordering hazard is outside my lane (plan-critic-parallel-safety's), noted but not scored here; whether the declared-fault-injection report-count acceptance is checkable is plan-critic-acceptance's lane, not mine.

```findings
- file: .engineering/planning/epic/concurrent-history-conformance.md
  line: 37
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: the epic requires every check to have a planted fault in faulty.rs that no earlier check catches, but story:recorded-history-validation's only fault-linked acceptance reuses LostUpdate, already caught by story:linearizability-checker-over-the-interpreter
- file: .engineering/planning/story/declared-fault-injection.md
  line: 24
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the "target restart between operations, where the target declares it supports one" fault is not traceable to any sentence in the parent and no restart-capability declaration exists anywhere in the specification, realizations, or the Target trait, unlike the story's other three fault types which are all grounded in existing declared spec vocabulary
```
