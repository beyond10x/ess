---
format: aep.planning-md/1
id: story:early-stop-assertion
kind: story
status: implemented
title: An authored scenario can claim that a consumer halted an ordered scan
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/fixtures/go-billing/target.go
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/report.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
revision: 5
---
## What is missing

`ess-scenario/1` can say what a view holds, in what order, and how many rows it holds. It cannot say
that a consumer *stopped reading*.

The six assertion forms are all predicates over the rows a read returned — `contains`, `excludes`,
`counts`, `ranked`, `at`, `satisfies`. Two implementations that return the same rows are
indistinguishable under every one of them, and "the producer halted" is exactly the fact that
separates them. `authored`'s own module documentation says so, under *What is deliberately not
here*: *"No bound here says an ordered scan stopped … a prefix read in silence is still a prefix.
That is a different feature."*

A migration of forty-five ACD slotmatcher cases into the format hit it twice and refused to fake it.
`ScanOrdered_EarlyTermination` for the ordered set and for the queue — the Go cases at
`internal/slotmatcher/conformance/slotset_tests.go:182-196` and `queue_tests.go:252-266` — each add
three elements, return "stop" from the scan after two, and assert that the scan visited two. Written
as a prefix assertion they would compile, be green against both implementations for reasons
unrelated to the defect, and look like coverage that does not exist.

The defect they would otherwise hide is measured. `acd-rs` cannot early-stop at all:
`ScoredHeapQueue` has no ordered iterator, and `OrderedSlotSet` has a `for_each` that returns
nothing beside a materialising `ordered_keys`
(`acd-rs/crates/acd-slotmatcher/src/slot_set.rs:98,116`). Go can, and pins it.

## What a target has to be able to do

The claim is about what an implementation *did not do*: it stopped pulling. That is observable only
if the target can report it, and the target is the only party that can — the runner sees the rows
and the rows are the same either way.

So the target is asked for a **bounded ordered read**: read this view in its declared order, hand a
consumer at most N rows, and report two facts about the read itself — how many rows the ordered
source **produced**, and whether the read ended because the consumer said stop rather than because
the source ran out. Both are observations about the target's own control flow, never a verdict; the
runner decides, and it requires both `produced == N` and `halted`.

`produced` alone is not enough, and the second field is why: a source holding exactly N rows
produces N and ends because it is empty, which is not a halt. `halted` alone is not enough either: a
target that materialises the whole source and then stops a loop over the materialised copy has
halted a loop and not a scan, and its `produced` is the row count.

An adopter can supply both without instrumentation, because the callback shape is the ordinary one:
count every invocation, return "stop" at N, and record which branch ended the iteration.

A target that cannot answer says so. `scan_view` has a default body answering `Unsupported`, so every
target written against the earlier interface still compiles; the emitted Go runner asks for an
optional interface for the same reason. The scenario is then reported `unsupported`, and §28 makes
the run fail. There is no path on which a scan nobody stopped passes.

## Acceptance

- An assertion states `halts_after: <n>` as one of the mutually exclusive claims about a view, and
  it compiles to a step of its own rather than to a `ViewExpectation` — no predicate over rows can
  decide it.
- A halt claimed after no rows at all is refused, with a stable `ESS-AUTHOR-nnn` code and a test.
- A halt claimed of a view that declares no order is refused, by the code that already refuses a
  position asserted of an unordered view.
- The Rust runner and the emitted Go runner both execute the steps; a target that reports no scan is
  `unsupported` and the run fails; a target that produces the whole source is `failed`.
- The sixty-seven scenarios in `sbf/specs/authored/` still compile with zero refusals.
