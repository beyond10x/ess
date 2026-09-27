---
format: aep.planning-md/2
id: review-result:concurrent-history-acceptance-round-2
kind: review-result
status: active
title: 'Acceptance critic, round 2: concurrent history conformance'
relations:
- reviews: story:session-and-eventual-view-checks
- reviews: story:linearizability-checker-over-the-interpreter
- reviews: story:concurrent-explorer-runner
- reviews: story:go-explorer-concurrent-porcupine
- reviews: story:concurrent-history-format
- reviews: story:recorded-history-validation
- reviews: story:declared-fault-injection
- reviews: story:concurrent-history-lanes
revision: 1
---
# Acceptance critic — round 2

approve

**What I read:** all 8 artifacts in full via `aep plan artifact show <id>` — story:go-explorer-concurrent-porcupine, story:concurrent-history-format, story:linearizability-checker-over-the-interpreter, story:concurrent-explorer-runner, story:session-and-eventual-view-checks, story:declared-fault-injection, story:concurrent-history-lanes, story:recorded-history-validation — plus `aep plan artifact show review-result:concurrent-history-acceptance-round-1`, `aep plan artifact kinds`, `aep plan artifact lifecycle story`, and `aep plan artifact show epic:concurrent-history-conformance` for context. Cross-checked tree paths/symbols named in acceptance and scope text: `tests/fixtures/explore_target.go`, `crates/verify/ess-conformance/tests/explore.rs`, `crates/verify/ess-conformance/src/go/{explore.go,mod.rs}`, `crates/verify/ess-conformance/src/ts/explore.ts`, `crates/verify/ess-conformance/src/faulty.rs`, `crates/verify/ess-conformance/src/web.rs`, `models/concurrent-history/` — all exist as named.

All three round-1 findings are fixed as claimed: story:concurrent-history-format (rev 2) now carries a separate bullet for "no return instant" and for "return precedes invoke"; story:concurrent-explorer-runner (rev 2) now separates the search bullet (200 seeds find a violation) from the shrink bullet; story:declared-fault-injection (rev 2) now gives `DoubleApplyOnRedelivery` and `RetryCreatesSecondEntity` their own bullets. No new bundling defect in any of the three.

The new story:go-explorer-concurrent-porcupine's four acceptance bullets are each single, observable and checkable against a named path (`tests/fixtures/explore_target.go`, `tests/explore.rs`, the emitted `go.mod`), and follow the epic's existing idiom for compound-but-coupled facts (e.g. "0 Illegal and 0 Unknown" mirrors the unflagged "0 violations and 0 Unknown" already present and unflagged in story:concurrent-explorer-runner) — I did not flag these as new bundling, for consistency with round 1's treatment of the same idiom elsewhere in this epic.

**What I could not establish:** two borderline compound bullets I chose not to flag, noting them for a second reader: (1) story:concurrent-explorer-runner's shrink bullet ("holds at most 2 clients and 4 operations, and the checker still rejects it") states a size bound and a soundness property together — I read these as one coupled claim about shrink correctness rather than two independently-landable capabilities, since an unsound shrink isn't meaningfully "the shrunk history for that violation" at all; (2) story:go-explorer-concurrent-porcupine's third bullet ("writes a Porcupine HTML file, and tests/explore.rs asserts it exists and names the mutant's two conflicting operations") describes one artifact's existence and content together, matching the pattern already accepted in story:concurrent-history-lanes ("names the two conflicting operations and the state..."). Scope/coupling/parallel-safety concerns in these same bodies are out of my lane and not represented above.

```findings
[]
```
