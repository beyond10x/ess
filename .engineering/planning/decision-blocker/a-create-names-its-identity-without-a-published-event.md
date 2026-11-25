---
format: aep.planning-md/3
id: decision-blocker:a-create-names-its-identity-without-a-published-event
kind: decision-blocker
status: open
title: 'Where a created identity is read from when a component publishes nothing: EV-D1'
summary: A request to change the creating outcome instance rule (ESS-COMMAND-001); the owner decides, the fail-closed option is A
tags:
- architecture
- gap-reports-2026-11-24
relations:
- blocks: story:a-component-that-publishes-nothing-keeps-the-events-its-commands-emit-unpublished
revision: 1
---
EV-D1 is the owner's: a request to change a validation rule that every released specification satisfies (`ESS-COMMAND-001`, the creating outcome's `instance:` names a field of an emitted event). The design text leaves the source of a created identity open only in the sense that the crate documentation gives a reason and no design page decides it (`crates/specify/ess-domain/src/command.rs:130`, `docs/design/outcome-shapes.md:248-262`), so it is written as a decision, not taken.

## Why this is open

`story:a-component-that-publishes-nothing-keeps-the-events-its-commands-emit-unpublished` documents what is already expressible (leave the event out of `publishes:`). A downstream application also asks that a create may return its identity without an event. Today `Effect::Creates` reads the identity from an emitted event only (`command.rs:1096-1103`), the compiler resolves it to `ResolvedInstance::Observed { event, field }` (`crates/specify/ess-compiler/src/resolve.rs:4239-4255`), and every generated target and every conformance scenario reads a created identity from that event.

## EV-D1: where a created identity is read from

- A. The rule stays: a creating outcome names a field of an event it emits, and a component that publishes nothing leaves the event out of `publishes:`. ESS says so on the reference page and pins it by a test. Cost: a suite for such a component reads the identity through `observe_events`, which a target that publishes nothing answers `unsupported`, and the run fails that scenario as a target failure (`docs/design/conformance-target-process.md:17`, `:274`).
- B. A creating outcome may name a field of the command's response instead (`instance:` on a response member), with no event. Cost: a new surface for `instance:` and a new `ResolvedInstance` variant; a new `format: ess/N` and a second spelling of the rule; synthesis, the interpreted target, the Rust and Go behaviours and the conformance runners all read the identity from either source; a model that names both is a conflict to define.
- C. An event may be declared internal (a flag on the event, or a list on the component), still emitted and still the source of the identity, never delivered and never observed by a suite. Cost: a new kind of declaration beside `publishes:` that A already states by omission; the suite then skips the event step for it, so the scenario reads the identity from nowhere unless B is also taken.

Fail-closed: A, the rule stays and nothing is relaxed. This changes what a released reader accepts (B, C) and the shape of the specification (a new kind of declaration), so the owner decides; there is no default beyond the fail-closed one.
