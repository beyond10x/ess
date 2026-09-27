---
format: aep.planning-md/2
id: review-result:concurrent-history-parallel-safety-round-1
kind: review-result
status: active
title: 'Parallel-safety critic, round 1: concurrent history conformance'
relations:
- reviews: story:declared-fault-injection
- reviews: story:recorded-history-validation
- reviews: story:linearizability-checker-over-the-interpreter
- reviews: story:session-and-eventual-view-checks
- reviews: story:concurrent-history-format
- reviews: story:concurrent-explorer-runner
- reviews: story:concurrent-history-lanes
revision: 1
---
**needs-revision**

story:declared-fault-injection — both this and story:session-and-eventual-view-checks add a new `Fault` enum row inside the single `faults! { ... }` macro block of `crates/verify/ess-conformance/src/faulty.rs` (cited, both bodies' Scope: `DoubleApplyOnRedelivery`/`RetryCreatesSecondEntity` vs `StaleReadUnderReadYourWrites`), and no dependency edge or transitive path orders the pair (`aep plan artifact graph`: story:declared-fault-injection depends only on story:concurrent-explorer-runner and story:external-mutation-explorer-and-toolchain; story:session-and-eventual-view-checks depends only on story:linearizability-checker-over-the-interpreter; neither reaches the other) — .engineering/planning/story/declared-fault-injection.md:39

What I read: all 7 story artifacts via `aep plan artifact show` (Scope + full body), `aep plan artifact graph` filtered to the epic and its story edges, and source-tree confirmation via `crates/verify/ess-conformance/src/faulty.rs` (single `faults! { ... }` macro, lines 206–309, generating the `Fault` enum, `ALL`, and every accessor from one declaration list), `crates/verify/ess-conformance/tests/faults.rs` (a hardcoded `allowance: &[(Fault, usize)]` array at lines 330–342 that every new fault must also extend), and `crates/edge/ess-cli/src/main.rs` (the single `ConformCommand` enum, lines 440–605+, that story:linearizability-checker-over-the-interpreter's `check-history` subcommand and story:concurrent-history-lanes' `web` rendering both extend — this second pair is already correctly ordered by a direct `depends_on` edge, so it is not a finding).

I checked every other pairing of the seven for a shared surface: story:concurrent-history-format's format module/`schemas/`/Go+TS writers against story:concurrent-explorer-runner's `explore.go`/`explore.ts`, story:concurrent-explorer-runner against story:declared-fault-injection (both name `src/go/explore.go`, `src/ts/explore.ts` literally), and story:recorded-history-validation's adapter "beside the history format" and its `check-history` input path against story:concurrent-history-format and story:linearizability-checker-over-the-interpreter. Every one of those is already covered by a direct or transitive `depends_on` edge in the drafted graph, so none is a live parallel-safety defect; story:declared-fault-injection / story:session-and-eventual-view-checks is the one pair sharing a cited file with no edge in either direction and no path through a common ancestor that would serialize them.

What I could not establish precisely: story:concurrent-history-format's exact "Go and TS writers" file names (inferred — not the same files pre-existing in the tree as `explore.go`/`explore.ts`, but not confirmed which new files); story:recorded-history-validation's exact "adapter module" path (inferred, no file yet exists). Neither changes the verdict: both pairs are already ordered by direct or transitive `depends_on` edges regardless of the exact file.

```findings
- file: .engineering/planning/story/declared-fault-injection.md
  line: 39
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: both this and story:session-and-eventual-view-checks add a new `Fault` enum row inside the single `faults! { ... }` macro block of `crates/verify/ess-conformance/src/faulty.rs` (cited, both bodies' Scope), and no dependency edge or transitive path orders the pair (`aep plan artifact graph`)
```
