---
format: aep.planning-md/2
id: review-result:concurrent-history-parallel-safety-round-3
kind: review-result
status: active
title: Parallel-safety critic, round 3 (wave-1 revisions)
relations:
- reviews: story:concurrent-history-format
- reviews: story:session-and-eventual-view-checks
- reviews: epic:concurrent-history-conformance
- reviews: story:declared-fault-injection
- reviews: story:concurrent-explorer-runner
revision: 1
---
approve

I read the five revised artifacts (`epic:concurrent-history-conformance`, `story:declared-fault-injection`, `story:concurrent-history-format`, `story:concurrent-explorer-runner`, `story:session-and-eventual-view-checks`) via `aep plan artifact show` in `the integration worktree` (commit `d6a6d7655`), plus `.engineering/waves/concurrent-history-wave-1.md` for the planned sequencing, and cross-checked citations against the tree (`ls`/`grep` on `crates/verify/ess-conformance/src/{lib.rs,go,ts}`, `tests/`).

All four stories carry an unusually complete `## Scope` section (dated 2026-09-27, `aep:story-scoper` against `472d35fbe`) that already does this lane's job: every surface is marked cited/inferred, and each ends with an explicit `Would collide with:` line naming the colliding sibling and the shared file(s).

Verified pairs and their sequencing:
- `story:declared-fault-injection` × `story:concurrent-explorer-runner` — share `src/go/explore.go`, `src/ts/explore.ts` — named in DFI's scope, and DFI's own `depends_on story:concurrent-explorer-runner` sequences them (`story:declared-fault-injection` relations).
- `story:declared-fault-injection` × `story:session-and-eventual-view-checks` — share `src/faulty.rs` (and, unenumerated but covered by the same edge, `tests/faults.rs`, `src/reference.rs`) — named, and DFI's `depends_on story:session-and-eventual-view-checks` sequences them.
- `story:concurrent-history-format` × `story:concurrent-explorer-runner` — CHF explicitly hands the Go/TS writer files to CER (`## Not covered`/scope "Not here" line) and CER's `depends_on story:concurrent-history-format` sequences them; confirmed no file overlap remains (CHF: `history.rs`, `lib.rs`, `schemas/`; CER: `go/explore.go`, `ts/explore.ts`, `go/mod.rs`, `ts/mod.rs`, fixtures).
- `story:concurrent-history-format` × (out-of-set) `story:interpreted-command-execution` — both add one `pub mod` line to `crates/verify/ess-conformance/src/lib.rs`; named in CHF's own scope ("anything adding a module to `lib.rs`") and again in the wave-1 table with a `git merge-tree` check before either merges. Verified against the live `lib.rs` (39 alphabetic `mod` lines already there).
- `story:session-and-eventual-view-checks` × `story:concurrent-explorer-runner` — no shared file (SEVC: `faulty.rs`, `tests/faults.rs`, `reference.rs`, `view.rs`; CER: explorer/fixture files); consistent with no dependency edge between them and the wave page batching them together in wave 3.

No unnamed collision, no surface left unassessed, and no surface named so widely it forbids everything (each story narrows from a stated "primary surface" directory down to specific cited files).

What I could not establish, and left out of the verdict: the epic's body prose ("Depends on existing work") still names the archived `#156` story rather than its three successors, even though `relations` was correctly repointed — a real defect, but it is a scope/consistency issue, not a concurrency one, so it belongs to `plan-critic-scope`, not here. Likewise CHF's generic "tests/ (a new history test)" note is inferred and unnamed by filename — I found no matching filename in the other three stories' scopes (DFI's `tests/explore.rs`/`tests/faults.rs`, CER's `tests/explore.rs`), so I don't read it as an actual collision, only as a residual unknown CHF's own scope already flags at "medium" confidence.

```findings
[]
```
