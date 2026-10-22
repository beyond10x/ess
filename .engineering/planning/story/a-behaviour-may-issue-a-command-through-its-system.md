---
format: aep.planning-md/3
id: story:a-behaviour-may-issue-a-command-through-its-system
kind: story
status: draft
title: A generated behaviour may issue a command its outcome declares, through the system
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Need

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22). A command's behaviour in the adopter's server has to issue another command (a send that starts a task, a cancel that notifies), and the generated behaviour trait offers no route to the system that owns the other component. The adopter works around it outside the generated code.

## Reproduction

Not reproduced by a run; read from generated 0.68.0 output of a minimal spec (`--target rust --layout crate`):

- The behaviour obligation is `fn fetch_job(&mut self, input: super::FetchJob) -> Result<super::FetchJobOutcome, crate::obligation::UnmetObligation>`: no system handle, no context through which a command is issued.
- The generated `System` holds each component as a public field (`pub jobs: crate::ports::jobs::Jobs<JobsBehaviors>`) and moves events by `pump`, so a command enters only from outside, through a component's port.

Repro files requested from the adopter: the server code that issues a command from inside a behaviour (the adopter's `crates/a2a-server/src/ports.rs`, around line 320), and the spec outcome it implements.

## Where it lives

- The obligation trait's method: `crates/generate/ess-synth/src/rust/obligation.rs:216` (its doc string, traced to the generated trait).
- The generated `System`: `crates/generate/ess-synth/src/rust/system.rs:840` (`system_struct`), traced; Go has the same shape in `crates/generate/ess-synth/src/go/system.rs`.
- `story:system-issued-command-is-awaited-not-driven` (implemented) covers how conformance awaits a command the system issues; it does not give a behaviour a way to issue one.

## Acceptance (provisional)

- A behaviour obligation can issue a command the specification says its outcome issues, through a typed handle the generated code passes in, and the command enters the same system and log.
- A command the specification does not say the outcome issues is not reachable from that handle.
- Conformance's existing await of a system-issued command passes against a generated implementation that uses the handle.

## Resolution

Covered by the existing route. Answered from ESS sources on 2026-10-22 against the adopter's gap-9 evidence (the server runs agent steps such as StartWorking, UpdateArtifact and CompleteTask on a second `Generated::new(ports)` outside `System`, so their events never reach the system log).

- **The route.** The implementation's own loop (the adapter or scheduler, not a behaviour) issues each step as its own command through the owning component's port on the generated `System`, `system.<component>.<command>(input)`, then calls `system.pump()`. Each step's events land on the system log and reach every reacting binding. The generated `System` has no `execute`. Its component fields are public "because commands enter the system through a component's own port", emitted at `crates/generate/ess-synth/src/rust/system.rs:896-902`. The pump is described at `:8-34`.
- **What ess/27 adds (0.63.0).** Declare the actor that grants the agent-side commands `system: true`. The commands become system-issued, and conformance awaits them (`await_outcome`, `expect_not_issued`) rather than driving them. See `CHANGELOG.md:335-346` and `docs/design/system-issued-commands.md`. The design keeps generated servers unchanged "by decision: the owning component still accepts the command, because that is where the system's own scheduler sends it" (`docs/design/system-issued-commands.md:382`).
- **The pattern in a conformance case.** `crates/verify/ess-conformance/tests/system_issued_command.rs:230-296`: the target never accepts the system's step through `execute_command` (`:247-251`). It issues the step itself and answers the await with the input it chose (`:258-296`).
- **Not supported, and not needed for this report.** Issuing a command from inside a behaviour or a port. A behaviour obligation receives only its input over the storage and context ports (`crates/generate/ess-synth/src/rust/obligation.rs:216`) and holds no handle to `System`. No ESS design asks for one: an outcome causing another command is a reaction (a binding the pump delivers) or a system-issued command from the implementation's loop.

So this is a usage gap on the adopter's side. Kept as a draft and not moved. If the adopter still needs a behaviour to issue a command after moving to the route above, reopen it with that case.
