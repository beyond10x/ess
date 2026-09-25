---
format: aep.planning-md/2
id: epic:model-driven-interpretation
kind: epic
status: active
title: A specification can be executed without being implemented
summary: A model-driven ConformanceTarget that reads the IR, so any specification runs before anybody fills an obligation.
owner: ess
relations:
- serves: vision:O2
revision: 4
---
# Epic: model-driven interpretation

## Outcome

A specification can be executed without anybody hand-writing an implementation of it.
`ess conform run --target interpreted` runs a committed suite against an interpreter that derives
behaviour from the compiled IR, so a document that has never run becomes falsifiable the moment it
compiles.

Today the toolchain validates, compiles, projects and synthesises, and executes only what two people
hand-wrote: `Billing` over `examples/billing/` and `Oracle` over `examples/oracle-fixture/`
(`crates/verify/ess-conformance/src/reference.rs`). A specification outside those two has no target,
so every claim anybody makes about it is a claim about a document.

Anybody would know it had happened when a specification nobody has implemented reports scenario
results, and a wrong specification is caught by a red scenario rather than by a reviewer.

## Why Now

Obligations are filled slowly and specifications are written quickly, so the gap between the two is
what grows. An interpreter does not close it; it makes it survivable, because the document becomes
falsifiable at authoring time instead of at implementation time. Its own correctness is checkable
today and gets no cheaper later: two hand-written targets and their committed suites already exist
to disagree with it.

## Scope

- A `ConformanceTarget` implementation that reads the compiled IR and derives every model-determined
  fact: which outcome a command may take, the field writes it makes, the transition it takes and
  from which states, the events it emits with their payload mappings, whether a `wrong_state` branch
  refuses or accepts, what `invariants:` require at rest, a view's rows, and which binding reacts to
  which event with what mapping, `delivery` and `on_failure`.
- The scenario-supplied boundary for what the model does not determine: an `external:` outcome
  through `configure_external_outcome`, time through `mark_instant` and `observe_elapsed`, identities
  the interpreter mints and a scenario binds by name with `capture:`.
- `--target interpreted` in the same argument slot that already names `billing` and `oracle`.
- The trust gate: both committed suites, and the `faulty.rs` fault matrix.

## Out of Scope

- **Behavioural synthesis.** Nothing is emitted. Interpreting a document at test time and emitting an
  implementation are different acts, and only the second leaves something somebody could deploy.
- **Becoming a realization.** Nothing links the interpreter and the linker may not select it. If that
  ever changes, gap register D-2 applies to it and its design page is void.
- **Filling obligations.** This makes filling them postponable, not unnecessary. A specification that
  runs only under an interpreter has still never been implemented.
- **Changing an existing target, suite, emitter or generated tree.**

## Risks

- **Immediacy.** A single in-memory map is tempted into serving `consistency: eventual` immediately.
  `reference.rs` lags it by `Billing::DEFAULT_LAG` further reads on purpose — a suite that never
  waits never tests the word. Shown early by an eventual-view scenario passing without a wait.
- **Swallowed obligations.** The interpreter has no unfilled ports, so it is tempted to drop the
  distinction between an unmet obligation and a delivery failure. Emitted pumps keep that line
  (`crates/generate/ess-synth/src/rust/system.rs:23-27`); escalating an unmet obligation publishes a
  domain event for a defect no provider caused. Shown early by a scenario that scripts an external
  outcome, gets no answer, and reports a delivery failure.
- **Invented refusals.** A target that answers `wrong-state` whenever it is inconvenient turns the
  suite green by making every branch reachable. Shown early by the fault matrix losing
  discrimination.
- **A missing constraint's home.** Wave 6 cites `docs/plan/gap-register.md`, which does not exist at
  `11fc669`; D-2 survives only as a sentence quoted in a plan document and in the design page. A
  constraint whose home is missing is one the next reader re-litigates.
