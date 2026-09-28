---
format: aep.planning-md/3
id: epic:concurrent-history-conformance
kind: epic
status: draft
title: An implementation is held to the specification under concurrent clients and declared faults
summary: Concurrent histories against the adopter's target, checked against the IR model at each view's declared consistency, with declared faults injected.
owner: ess
relations:
- depends_on: story:external-mutation-explorer-and-toolchain
- depends_on: story:interpreted-command-execution
- serves: vision:O2
- informed_by: epic:model-driven-interpretation
- depends_on: story:outcome-shapes-beyond-ess-14
revision: 1
---
# Epic: concurrent history conformance

## Problem

Every ESS check that executes an adopter's implementation drives it with one client, one command
at a time, and no injected fault:

- the synthesized suite runs scenarios step by step;
- the TypeScript explorer awaits each command before the next (`src/ts/explore.ts:717` on
  `be44a3365`);
- `ess verify conform mutate` runs only against built-in targets (#153).

Lost updates, events applied twice on redelivery, a retry that creates a second entity, a stale read
under a declared `read_your_writes` or `Current` view, and two transitions out of one state that both
succeed all pass every single-client suite.

## Outcome

`ess` runs 2–4 concurrent clients against a target and records the history. It checks that history
against the specification's own model, at the consistency level each view declares. It injects only
faults the specification already declares. A shrunk failing history is shown as client lanes. Each
check that adds a bug class has a planted fault that it catches and no earlier check catches: a
`faulty.rs` row for the Rust targets, a mutant in the explore fixture target for the Go and TS
explorers. `story:concurrent-history-lanes` (rendering) and `story:recorded-history-validation` (a new
history source) add no bug class; each reuses an existing fault row as its control.

## One checker, one model, no new dependency

Decided 2026-09-27. The model (the Rust interpreter), the linearizability and consistency checker,
and shrinking exist once, in Rust, in the `ess` binary. The Go and TypeScript explorers are thin
clients: they drive the adopter's target concurrently, write `ess-history/1`, and run
`ess verify conform check-history`. Adopters already install `ess` to generate those packages.

No crate, Go module, npm package or external tool is added to `ess` or to any emitted package.
The sources below are read as algorithm references only; no code is copied from them. An earlier
draft put a separate checker in the Go package (`story:go-explorer-concurrent-porcupine`, archived
and superseded by `story:concurrent-explorer-runner`). That would have meant three copies of the
checker and a Go-only head start; it was dropped for one implementation.

The history format itself is specified in ESS (`models/concurrent-history/`), so its Go,
TypeScript and Rust types come from one source. The checker is not specified in ESS: it is a
search algorithm, not behaviour a caller observes, and ESS does not synthesize behaviour
(`docs/design/linker-never-chooses.md`).

Design record with the research, the effort/gain ranking and these decisions:
`docs/design/concurrent-history-conformance.md`.

## Prior art read (web research 2026-09-27)

| source | what is taken | where |
|---|---|---|
| Porcupine, Go, MIT, v1.3.1 (2026-09-21), `NondeterministicModel`, `Ok/Illegal/Unknown`, HTML visualizer — https://github.com/anishathalye/porcupine | the algorithm: WGL search with P-compositionality, nondeterministic step, timeout → `Unknown`, and the lane view | `story:linearizability-checker-over-the-interpreter`, `story:concurrent-history-lanes` |
| porcupine-rs 0.3.0, MIT, 232 downloads (crates.io, 2026-09-27); no nondeterministic models (its lib.rs page) | not used; it has no nondeterministic models, and ESS outcomes are nondeterministic (external branches, generated identities, eventual views) | `story:linearizability-checker-over-the-interpreter` |
| QuickCheck/PULSE parallel state-machine testing; `proptest-state-machine` is sequential only (docs.rs) | sequential prefix + parallel suffix, shrink to a minimal history | `story:concurrent-explorer-runner` |
| S2 linearizability testing with turmoil + Porcupine — https://s2.dev/blog/linearizability | an indeterminate (timed-out) operation gets return time = after every other operation | `story:concurrent-history-format`, `story:declared-fault-injection` |
| Jepsen Elle, Clojure — https://github.com/jepsen-io/elle | session anomaly vocabulary; JVM only, not a dependency | `story:session-and-eventual-view-checks` |
| AWS P + PObserve; TLA+ trace validation (SEFM 2024, arXiv 2404.16075) | checking recorded production logs against the spec | `story:recorded-history-validation` |
| Stateright 0.31.0; quint-connect 0.1.2 | not adopted: they model-check an actor model or replay traces from a separate spec language, and ESS already has its model in the IR | — |

## Depends on existing work

- `story:interpreted-command-execution` (`epic:model-driven-interpretation`): the Rust interpreter
  is the sequential model the checker searches against.
- `story:outcome-shapes-beyond-ess-14` (#152): the ambient precondition / explorer seed.
- `story:external-mutation-explorer-and-toolchain` (#156): external branches as explorer choices.

## Specification

`models/concurrent-history/` declares `concurrent.history.History` and
`concurrent.history.Operation` (`ess specify validate --path models/concurrent-history`:
`concurrent v1 — 2 file(s), valid`).

## Not covered

- The `ess:hardening` skill catalogue in `beyond10x/agentplugins` gains a technique row once the
  runner ships; that is in another repository's store.
- Deterministic simulation of the adopter's runtime (turmoil, madsim, Antithesis): ESS does not own
  the adopter's scheduler.
- TLA+/Alloy/Quint export.
