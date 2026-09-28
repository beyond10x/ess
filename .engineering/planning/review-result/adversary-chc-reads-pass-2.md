---
format: aep.planning-md/3
id: review-result:adversary-chc-reads-pass-2
kind: review-result
status: active
title: Adversary pass 2, concurrent-history wave 5 unit reads
relations:
- reviews: story:explorers-record-view-reads
revision: 1
---
# Adversary pass 2 — story:explorers-record-view-reads

Dispatched as `aep:adversary` after correction round 1, 2026-09-28. Header and findings verbatim.

unit: story:explorers-record-view-reads, uncommitted working tree on e1b9468159 (plus the coordinator's merge fix)
verdict: NEEDS-CHANGE
cases: executed 806→809, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (scratch log); build output went to the assigned build dir
needs-coordinator: none

Cases (crates/edge/ess-cli/tests/explore_concurrent_reads_adversary_2.rs): more phantoms than lost
creations are still a violation (RED: 47 of 48 hidden); a stale read beside the row of a lost
creation is not hidden (RED: 2 seeds); the rule writes equal bytes in Go and TypeScript and again
(green).

Suite: `cargo test -p ess-cli --locked --no-fail-fast` EXIT=101, 807 passed, 2 failed (the two new),
4 ignored.

Coordinator routing: all `introduced` → correction round 2. Decided: withhold at most one unnamed row
per eligible unanswered creation; named rows are always written; a two-entity fixture covers the
`creates == source` clause.

```findings
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 2210
  category: property
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "One unanswered creation of the view's entity withholds every read showing any number of unnamed rows, so invoices nobody created go unjudged: with one rejected CreateInvoice answer lost, double-apply violations fall from 144 to 62 over 200 seeds and 47 of 48 seeds whose read showed two or more such invoices are not violations, in Go and TypeScript alike (src/ts/explore.ts:1954)."
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 2441
  category: property
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A read showing a lost creation's row is written without any rows, so the stale part carried by its named rows is withheld too: restoring only the named rows makes 2 of 32 withheld stale-read seeds violations the written history does not show, while the same restoration on the correct target gives none (src/ts/explore.ts:2202)."
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 2217
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "No fixture specification has two creatable entities, so no case distinguishes the rule's creates == source clause and a mutant removing it would keep every test green."
```
