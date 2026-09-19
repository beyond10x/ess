---
format: aep.planning-md/1
id: story:cross-runtime-verdict-equivalence
kind: story
status: draft
title: One corpus holds every emitted runtime to the same verdicts
summary: Two emitted runtimes reaching different verdicts on one suite is undetectable today; the primitive corpus covers the bottom layer only
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/specify/ess-primitives/tests/vectors
- confidence: cited
  path: crates/verify/ess-conformance
revision: 2
---
## What this is

Nothing checks that two emitted conformance runtimes reach the same verdict on the same suite.

Today there are two: the Rust reference and `--target go`. `story:java-conformance-target` adds a
third and `story:typescript-conformance-target` a fourth. Each one re-implements the same five things
in a different language — the predicate evaluator, the response decoder, the reading accessor, the
scenario runner and the report writer — and the only comparison between any two of them covers the
bottom layer of the first.

## What already exists, and exactly how far it reaches

`crates/specify/ess-primitives/tests/vectors/primitive-semantics.json` is the normative primitive
corpus. `docs/design/review-primitive-semantics.md` states the rule as *one grammar, three
implementations, one corpus*, and `crates/verify/ess-conformance/tests/primitive_corpus.rs` answers
it from three sides: Rust's `Holds::admits`, the Go runtime's `primitive`, and the browser adapter's
`primitiveAdmits`. `tests/primitive_divergence.rs` carries the adversarial vectors the corpus does
not.

That is the right mechanism. It covers **primitive admission** — whether a value is of a type — and
nothing above it:

| layer | Rust | Go | compared across languages |
| --- | --- | --- | --- |
| primitive admission | `Holds::admits` | `primitive` | **yes**, `primitive-semantics.json` |
| predicate evaluation | `src/decision.rs`, 371 lines, and `src/accessor.rs` | `src/go/predicate.go`, 693 lines | no |
| response decoding | — | `src/go/response.go`, 374 lines | no |
| reading accessor and coordinate | `src/reading.rs` | `src/go/reading.go` 217 + `crates/specify/ess-domain/src/reading/coordinate.go` 186 | no |
| scenario ordering, barriers, skips | `src/runner.rs`, 3 037 lines | `src/go/runtime.go`, 6 741 lines | no |
| report counts and status | `src/report.rs`, 784 lines | inside `runtime.go` | no |

`crates/edge/ess-cli/tests/go_conformance.rs` holds the Go runtime to golden expectations recorded
from itself. That catches a regression in Go. It cannot catch Go and Rust having always disagreed.

Measured 2026-09-19.

## Why it gets worse with every target added

`story:java-conformance-target` gives the reason to emit a runner rather than let an adopter write
one: *two hand-written runners that agree only on the day the second was written*. Emitting from one
crate removes the duplication of **authorship**. It does not remove the duplication of
**semantics** — `src/go/runtime.go` is hand-written Go shipped as a static asset, and a TypeScript or
Java one will be hand-written too. Four runtimes are six pairs, and today zero pairs are checked
above the primitive layer.

The failure mode is the one worth designing against. A suite that passes in one language and fails in
another is indistinguishable, from where the adopter sits, from a defect in the system under test.
They debug their own code. The divergence is found by whoever happens to run two runtimes against one
implementation, which is nobody.

## What this asks for

1. **A normative runtime corpus** — language-neutral, versioned with the suite format, covering every
   construct a suite document can carry: each predicate form, each outcome kind, `ErrUnsupported`,
   the `elapsed:` windows, at-least-once redelivery, the skipped/unsupported/failed distinction, and
   the count arithmetic that turns those into a status.
2. **A replay target in every emitted runtime** — a target that answers each method from a recorded
   transcript rather than from a running system. This is the part that makes the comparison possible
   at all: without it, comparing two runtimes needs two real implementations, and a disagreement is
   attributable to either. With it, the runtime is the only variable.
3. **A gate** — for each runtime, run the corpus through its replay target and compare the emitted
   report against the normative expectation, field by field. A new `--target <lang>` does not merge
   until it passes.
4. **The Rust reference is the oracle.** Expected reports are generated from it, and a disagreement
   is a finding against the new runtime until somebody shows otherwise.

## Acceptance

- A corpus document exists, validates, and is read by every runtime's replay target from one file —
  the way `primitive-semantics.json` is read by three implementations from one file.
- The Rust reference and the Go runtime both pass the gate over it, and the gate runs in `task check`.
- A deliberately introduced divergence in `src/go/predicate.go` — one operator's semantics changed —
  fails the gate and names the vector it failed on. A divergence in the count arithmetic fails it too.
- Corpus coverage is measured and reported: a predicate form or a report field no vector exercises is
  named as uncovered rather than passing silently.
- `story:typescript-conformance-target` and `story:java-conformance-target` each cite this gate in
  their own acceptance, and neither merges a runtime that has not passed it.

## What this is not

- Not a replacement for the primitive corpus. That corpus stays where it is and becomes the bottom
  layer of this one.
- Not a second runner. The corpus compares runtimes against a recorded transcript; it never executes
  a specification against a real system.
- Not `ess-conformance-replay/1` (`src/web_replay.rs`), which is the paired browser projection and
  answers a different question.
